//! `--run_under` and `--script_path`: what `run` reads besides the build
//! flags (buildfiji-ef2). Extraction follows [`crate::test_flags::extract`]'s
//! shape, and only `run` has them: `build` and `test` leave the tokens
//! alone, so `clap_flags::validate` still refuses them there.

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `run`.
pub const IMPLEMENTED: &[&str] = &["run_under", "script_path"];

/// What `--run_under` names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunUnder {
    /// A command line, run by `/bin/bash -c` with the program and its
    /// arguments appended: `valgrind --quiet`.
    Command(String),
    /// An executable target and its options: `//pkg:tool --verbose`. The
    /// target is built with the one being run.
    Target { label: String, options: Vec<String> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunFlags {
    pub run_under: Option<RunUnder>,
    /// Write a script that runs the target there, and do not run it.
    pub script_path: Option<PathBuf>,
}

/// Pull [`RunFlags`] out of `args` for `command`, returning the rest in order.
pub fn extract(args: &[String], command: &str) -> Result<(RunFlags, Vec<String>), String> {
    let mut flags = RunFlags::default();
    if command != "run" {
        return Ok((flags, args.to_vec()));
    }
    let registry = FlagRegistry::global();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            rest.push(arg.clone());
            continue;
        }
        let Ok(m) = registry.resolve(arg, command) else {
            rest.push(arg.clone());
            continue;
        };
        let name = m.flag.name;
        if !IMPLEMENTED.contains(&name) {
            rest.push(arg.clone());
            continue;
        }
        let Some(value) = m.value.map(str::to_string).or_else(|| iter.next().cloned()) else {
            rest.push(arg.clone());
            continue;
        };
        match name {
            "run_under" => {
                flags.run_under = Some(
                    parse_run_under(&value)
                        .map_err(|e| format!("While parsing option --run_under={value}: {e}"))?,
                );
            }
            _ => flags.script_path = Some(PathBuf::from(value)),
        }
    }
    Ok((flags, rest))
}

/// A `--run_under` value as Bazel reads it: a first word starting `//` or
/// `@` is a target, anything else a command line, kept as written.
fn parse_run_under(value: &str) -> Result<RunUnder, String> {
    let words = tokenize(value).map_err(|e| format!("Not a valid command prefix {e}"))?;
    let Some(first) = words.first() else {
        return Err("Empty command".to_owned());
    };
    if first.starts_with("//") || first.starts_with('@') {
        Ok(RunUnder::Target {
            label: first.clone(),
            options: words[1..].to_vec(),
        })
    } else {
        Ok(RunUnder::Command(value.to_owned()))
    }
}

/// The words of a shell command line: whitespace separates, quotes and
/// backslashes protect, as in Bazel's `ShellUtils.tokenize`.
fn tokenize(text: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut quote: Option<char> = None;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some('"'), '\\') => match chars.next() {
                Some(n) if n == '"' || n == '\\' => word.push(n),
                Some(n) => {
                    word.push('\\');
                    word.push(n);
                }
                None => return Err("unterminated quotation".to_owned()),
            },
            (Some(_), c) => word.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                in_word = true;
            }
            (None, '\\') => match chars.next() {
                Some(n) => {
                    word.push(n);
                    in_word = true;
                }
                None => return Err("backslash at end of string".to_owned()),
            },
            (None, c) if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut word));
                    in_word = false;
                }
            }
            (None, c) => {
                word.push(c);
                in_word = true;
            }
        }
    }
    if quote.is_some() {
        return Err("unterminated quotation".to_owned());
    }
    if in_word {
        words.push(word);
    }
    Ok(words)
}

/// A word as the shell reads it back: left as it is if it is made of safe
/// characters, else in single quotes. Bazel's `ShellEscaper`.
pub fn shell_escape(word: &str) -> String {
    let safe = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    if safe {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn only_run_reads_them() {
        let (flags, rest) = extract(&args(&["--run_under=x", "//a"]), "build").unwrap();
        assert_eq!(flags, RunFlags::default());
        assert_eq!(rest, args(&["--run_under=x", "//a"]));
    }

    #[test]
    fn a_command_is_kept_as_written_and_a_label_takes_options() {
        let (flags, rest) = extract(
            &args(&["--run_under", "strace  -c", "--script_path=/s.sh", "//a"]),
            "run",
        )
        .unwrap();
        assert_eq!(
            flags.run_under,
            Some(RunUnder::Command("strace  -c".into()))
        );
        assert_eq!(flags.script_path, Some(PathBuf::from("/s.sh")));
        assert_eq!(rest, args(&["//a"]));
        let (flags, _) = extract(&args(&["--run_under=//p:t --opt 'a b'"]), "run").unwrap();
        assert_eq!(
            flags.run_under,
            Some(RunUnder::Target {
                label: "//p:t".into(),
                options: args(&["--opt", "a b"]),
            })
        );
    }

    #[test]
    fn an_empty_command_is_refused_as_bazel_refuses_it() {
        assert_eq!(
            extract(&args(&["--run_under="]), "run").unwrap_err(),
            "While parsing option --run_under=: Empty command"
        );
    }

    #[test]
    fn escaping_leaves_safe_words_and_quotes_the_rest() {
        assert_eq!(shell_escape("a/b-c.d"), "a/b-c.d");
        assert_eq!(shell_escape("b c"), "'b c'");
        assert_eq!(shell_escape("d\"e"), "'d\"e'");
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
        assert_eq!(shell_escape(""), "''");
    }
}
