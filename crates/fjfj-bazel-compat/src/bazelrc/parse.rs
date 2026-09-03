//! Lexing and parsing of a single `.bazelrc` file's text into an [`RcFile`].
//!
//! `.bazelrc` is not a standard config format: line-continuation via a
//! trailing `\`, then each logical line is either `import`/`try-import`, a
//! full-line `#` comment, or `<command>[:<config>] <flag>...` with
//! POSIX-shell-style word splitting (quotes and backslash escapes; no
//! variable expansion or globbing). Splicing continuations and telling
//! directives apart is plain Bazel semantics done by hand; the interesting
//! grammar — shell word splitting — is a Chumsky parser. See
//! `docs/design/cli-compat.md`.

use std::ops::Range;

use chumsky::prelude::*;

use super::ast::{Directive, Line, RcFile};

/// A parse error at a byte span in the original (unspliced) source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line_no: usize,
    pub span: Range<usize>,
    pub message: String,
}

/// Parse `.bazelrc` source text into its directives, in source order.
/// Errors are collected per logical line rather than aborting at the first
/// one, so a caller (e.g. `resolve`) can report every problem in a file at
/// once.
pub fn parse_rc_file(src: &str) -> (RcFile, Vec<ParseError>) {
    let mut lines = Vec::new();
    let mut errors = Vec::new();

    for logical in splice_continuations(src) {
        let trimmed = logical.text.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match line_words_parser()
            .parse(logical.text.as_str())
            .into_result()
        {
            Ok(words) if words.is_empty() => {}
            Ok(words) => match interpret(words) {
                Ok(directive) => lines.push(Line {
                    directive,
                    line_no: logical.line_no,
                    span: logical.span.clone(),
                }),
                Err(message) => errors.push(ParseError {
                    line_no: logical.line_no,
                    span: logical.span.clone(),
                    message,
                }),
            },
            Err(errs) => {
                for e in errs {
                    let r = e.span().into_range();
                    errors.push(ParseError {
                        line_no: logical.line_no,
                        span: logical.span.start + r.start..logical.span.start + r.end,
                        message: e.to_string(),
                    });
                }
            }
        }
    }

    (RcFile { lines }, errors)
}

/// A `.bazelrc` line after joining `\`-continuations, with enough position
/// info to map back into the original source for diagnostics.
struct LogicalLine {
    text: String,
    /// 1-based physical line number the logical line started on.
    line_no: usize,
    /// Byte span in the original source covering every physical line joined
    /// into this logical line, continuation newlines included.
    span: Range<usize>,
}

/// Join `\`-continued physical lines. A trailing `\` immediately before the
/// newline (Windows `\r\n` tolerated) splices the next physical line on,
/// joined by a single space so tokens either side of the break don't fuse.
fn splice_continuations(src: &str) -> Vec<LogicalLine> {
    let mut out = Vec::new();
    let mut physical_line_no = 0usize;
    let mut offset = 0usize;

    let mut cur_text = String::new();
    let mut cur_start_line = 1usize;
    let mut cur_start_offset = 0usize;
    let mut in_continuation = false;

    for raw_line in src.split_inclusive('\n') {
        physical_line_no += 1;
        let line_start = offset;
        offset += raw_line.len();

        let no_newline = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let stripped = no_newline.strip_suffix('\r').unwrap_or(no_newline);

        if !in_continuation {
            cur_start_line = physical_line_no;
            cur_start_offset = line_start;
            cur_text.clear();
        } else {
            cur_text.push(' ');
        }

        if let Some(body) = stripped.strip_suffix('\\') {
            cur_text.push_str(body);
            in_continuation = true;
        } else {
            cur_text.push_str(stripped);
            out.push(LogicalLine {
                text: std::mem::take(&mut cur_text),
                line_no: cur_start_line,
                span: cur_start_offset..offset,
            });
            in_continuation = false;
        }
    }
    // A dangling `\` on the final line (no trailing newline): keep whatever
    // was accumulated rather than silently dropping it.
    if in_continuation {
        out.push(LogicalLine {
            text: cur_text,
            line_no: cur_start_line,
            span: cur_start_offset..offset,
        });
    }

    out
}

/// Shell-word-split a logical line into its directive's words. Trailing
/// `# comment` (outside quotes) is dropped.
fn line_words_parser<'a>() -> impl Parser<'a, &'a str, Vec<String>, extra::Err<Rich<'a, char>>> {
    let ws = one_of(" \t").repeated().at_least(1);
    let comment = just('#').then(any().repeated()).ignored();

    word_parser()
        .separated_by(ws)
        .allow_leading()
        .allow_trailing()
        .collect::<Vec<String>>()
        .then_ignore(comment.or_not())
        .then_ignore(end())
}

/// One shell-style word: a run of unquoted/`'...'`/`"..."` segments,
/// concatenated (so `foo"bar baz"qux` is one word, as in a real shell).
/// No variable expansion, no globbing — `.bazelrc` deliberately has neither.
fn word_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    let is_delim = |c: &char| c.is_whitespace() || *c == '#';

    // Outside quotes: `\` escapes the very next character (including
    // whitespace or `#`), stripping its special meaning.
    let unquoted_char = choice((
        just('\\').ignore_then(any()),
        any().filter(move |c: &char| !is_delim(c) && *c != '\'' && *c != '"' && *c != '\\'),
    ));
    let unquoted = unquoted_char.repeated().at_least(1).collect::<String>();

    // Single quotes: fully literal, no escapes recognised (POSIX rules).
    let single_quoted = any()
        .filter(|c: &char| *c != '\'')
        .repeated()
        .collect::<String>()
        .delimited_by(just('\''), just('\''));

    // Double quotes: `\\` and `\"` are recognised escapes; anything else,
    // including a bare `\`, is passed through literally.
    let double_quoted_char = choice((
        just('\\').ignore_then(one_of("\\\"")),
        any().filter(|c: &char| *c != '"'),
    ));
    let double_quoted = double_quoted_char
        .repeated()
        .collect::<String>()
        .delimited_by(just('"'), just('"'));

    choice((single_quoted, double_quoted, unquoted))
        .repeated()
        .at_least(1)
        .collect::<Vec<String>>()
        .map(|segments| segments.concat())
}

/// Turn a line's words into a [`Directive`], or an error message for a
/// malformed `import`/`try-import`.
fn interpret(mut words: Vec<String>) -> Result<Directive, String> {
    let head = words.remove(0);
    match head.as_str() {
        "import" | "try-import" if words.len() != 1 => Err(format!(
            "'{head}' takes exactly one path argument, got {}",
            words.len()
        )),
        "import" => Ok(Directive::Import {
            path: words.remove(0),
        }),
        "try-import" => Ok(Directive::TryImport {
            path: words.remove(0),
        }),
        _ => {
            let (command, config) = match head.split_once(':') {
                Some((c, cfg)) => (c.to_string(), Some(cfg.to_string())),
                None => (head, None),
            };
            Ok(Directive::CommandFlags {
                command,
                config,
                flags: words,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directives(src: &str) -> Vec<Directive> {
        let (rc, errors) = parse_rc_file(src);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        rc.lines.into_iter().map(|l| l.directive).collect()
    }

    #[test]
    fn skips_blank_and_comment_lines() {
        assert_eq!(directives("\n  \n# just a comment\n"), vec![]);
    }

    #[test]
    fn plain_command_line() {
        assert_eq!(
            directives("build --copt=-O2 --strip=never"),
            vec![Directive::CommandFlags {
                command: "build".into(),
                config: None,
                flags: vec!["--copt=-O2".into(), "--strip=never".into()],
            }]
        );
    }

    #[test]
    fn command_config_line() {
        assert_eq!(
            directives("build:asan --copt=-fsanitize=address"),
            vec![Directive::CommandFlags {
                command: "build".into(),
                config: Some("asan".into()),
                flags: vec!["--copt=-fsanitize=address".into()],
            }]
        );
    }

    #[test]
    fn common_applies_to_every_command() {
        assert_eq!(
            directives("common --color=yes"),
            vec![Directive::CommandFlags {
                command: "common".into(),
                config: None,
                flags: vec!["--color=yes".into()],
            }]
        );
    }

    #[test]
    fn import_and_try_import() {
        assert_eq!(
            directives("import %workspace%/tools/bazel.rc\ntry-import user.bazelrc\n"),
            vec![
                Directive::Import {
                    path: "%workspace%/tools/bazel.rc".into()
                },
                Directive::TryImport {
                    path: "user.bazelrc".into()
                },
            ]
        );
    }

    #[test]
    fn trailing_comment_is_dropped() {
        assert_eq!(
            directives("build --copt=-O2 # optimize"),
            vec![Directive::CommandFlags {
                command: "build".into(),
                config: None,
                flags: vec!["--copt=-O2".into()],
            }]
        );
    }

    #[test]
    fn quoting_and_escapes() {
        assert_eq!(
            directives(r#"build --a='literal $x' --b="esc\"aped" --c=no\ space"#),
            vec![Directive::CommandFlags {
                command: "build".into(),
                config: None,
                flags: vec![
                    "--a=literal $x".into(),
                    r#"--b=esc"aped"#.into(),
                    "--c=no space".into(),
                ],
            }]
        );
    }

    #[test]
    fn line_continuation_joins_with_a_space() {
        assert_eq!(
            directives("build --copt=-O2 \\\n  --strip=never\n"),
            vec![Directive::CommandFlags {
                command: "build".into(),
                config: None,
                flags: vec!["--copt=-O2".into(), "--strip=never".into()],
            }]
        );
    }

    #[test]
    fn bad_import_arity_is_an_error() {
        let (rc, errors) = parse_rc_file("import\n");
        assert!(rc.lines.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("import"));
    }
}
