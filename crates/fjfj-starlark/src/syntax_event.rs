//! A parse or resolution error as Bazel 9.2.0 reports it: one event line,
//! `<file>:<line>:<column>: <message>`, where the `starlark` crate prints a
//! code snippet and its own words.
//!
//! The words are Bazel's: `syntax error at 'newline': expected expression`,
//! `functions may not be defined in BUILD files. ...`, `name 'z' is not
//! defined`. The crate says what it found and what it wanted in one fixed
//! form (`Parse error: unexpected new line, expected expression`), which is
//! read back here; a message in no known form is passed on as it is.

use starlark::ErrorKind;

/// `error` as the event Bazel prints for a file named `file`.
pub fn syntax_event(file: &str, error: &starlark::Error) -> String {
    let (line, column) = place(error);
    format!("{file}:{line}:{column}: {}", message(error))
}

/// The line and column (from 1) of the error; an error at the end of the
/// file is on the line after its last newline.
fn place(error: &starlark::Error) -> (usize, usize) {
    let Some(at) = error.span() else {
        return (1, 1);
    };
    let begin = at.resolve_span().begin;
    let source = at.file.source();
    // A span with nothing in it at the start of a file that has text is how
    // the crate says "the end".
    if at.span.begin().get() == 0 && at.span.end().get() == 0 && !source.is_empty() {
        let lines = source.matches('\n').count();
        return if source.ends_with('\n') {
            (lines + 1, 1)
        } else {
            let last = source.rsplit('\n').next().unwrap_or("");
            (lines + 1, last.chars().count() + 1)
        };
    }
    (begin.line + 1, begin.column + 1)
}

/// Bazel's words for the error.
fn message(error: &starlark::Error) -> String {
    let text = error.without_diagnostic().to_string();
    let text = text.trim();
    match error.kind() {
        ErrorKind::Scope(_) => {
            if let Some(name) = between(text, "Variable `", "` not found") {
                return format!("name '{name}' is not defined");
            }
            text.to_owned()
        }
        _ => parse_message(text),
    }
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let rest = text.strip_prefix(start)?;
    Some(&rest[..rest.find(end)?])
}

fn parse_message(text: &str) -> String {
    if text.contains("`def` is not allowed in this dialect")
        || text.contains("`lambda` is not allowed in this dialect")
    {
        return "functions may not be defined in BUILD files. You may move the function to a .bzl file and load it.".to_owned();
    }
    if text.contains("`for` cannot be used outside `def`") {
        return "`for` statements are not allowed in BUILD files. You may inline the loop, move it to a function definition (in a .bzl file), or as a last resort use a list comprehension.".to_owned();
    }
    if text.contains("`return` cannot be used outside of a `def`") {
        return "return statements must be inside a function".to_owned();
    }
    if text.contains("Python-style generator expressions") {
        return "syntax error at 'for': Starlark does not support Python-style generator expressions".to_owned();
    }
    if text.contains("unfinished string literal") {
        return "unclosed string literal".to_owned();
    }
    if let Some(literal) = between(
        text,
        "Parse error: integer cannot have leading 0, got `",
        "`",
    ) {
        return format!(
            "invalid octal literal: {literal} (use '0o{}')",
            literal.trim_start_matches('0')
        );
    }
    let Some(rest) = text.strip_prefix("Parse error: unexpected ") else {
        return text.to_owned();
    };
    let Some((found, wanted)) = rest.split_once(", expected ") else {
        return text.to_owned();
    };
    if wanted == "new indentation block" {
        return "expected an indented block".to_owned();
    }
    if wanted == "keyword 'else'" {
        return "missing else clause in conditional expression or semicolon before if".to_owned();
    }
    format!(
        "syntax error at '{}': expected {}",
        token(found),
        wanted_token(wanted)
    )
}

/// How Bazel writes the token the parser found.
fn token(found: &str) -> String {
    match found {
        "new line" => "newline".to_owned(),
        "end of file" => "newline".to_owned(),
        other => quoted(other).unwrap_or(other).to_owned(),
    }
}

/// How Bazel writes what the parser wanted.
fn wanted_token(wanted: &str) -> String {
    match wanted {
        "new line" => "newline".to_owned(),
        "expression" => "expression".to_owned(),
        // What can follow a comprehension: the list is Bazel's, quotes and all.
        "']', 'for' or 'if'" | "'}', 'for' or 'if'" => wanted.to_owned(),
        other => quoted(other).unwrap_or(other).to_owned(),
    }
}

/// What is in the single quotes of `integer literal '2'`, `symbol ']'`.
fn quoted(text: &str) -> Option<&str> {
    let start = text.find('\'')? + 1;
    let end = text.rfind('\'')?;
    (end >= start).then(|| &text[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileKind, dialect::parse_checked};

    /// What Bazel 9.2.0 printed for a BUILD file of this text, as probed.
    const PROBED: &[(&str, &str)] = &[
        (
            "x = (\n",
            "BUILD:2:1: syntax error at 'newline': expected expression",
        ),
        (
            "x = [1, 2\n",
            "BUILD:2:1: syntax error at 'newline': expected ]",
        ),
        (
            "x = 1 +\n",
            "BUILD:1:8: syntax error at 'newline': expected expression",
        ),
        (
            "x ==\n",
            "BUILD:1:5: syntax error at 'newline': expected expression",
        ),
        (
            "x = not\n",
            "BUILD:1:8: syntax error at 'newline': expected expression",
        ),
        (
            "x = f(\n",
            "BUILD:2:1: syntax error at 'newline': expected expression",
        ),
        (
            "x = 1 2\n",
            "BUILD:1:7: syntax error at '2': expected newline",
        ),
        (
            "x = [1,\n2\n3]\n",
            "BUILD:3:1: syntax error at '3': expected ]",
        ),
        (
            "x = {1: 2, 3}\n",
            "BUILD:1:13: syntax error at '}': expected :",
        ),
        ("if x:\n", "BUILD:2:1: expected an indented block"),
        ("x = \"abc\n", "BUILD:1:5: unclosed string literal"),
        (
            "x = 08\n",
            "BUILD:1:5: invalid octal literal: 08 (use '0o8')",
        ),
        (
            "return 1\n",
            "BUILD:1:1: return statements must be inside a function",
        ),
        (
            "def f(): pass\n",
            "BUILD:1:1: functions may not be defined in BUILD files. You may move the function to a .bzl file and load it.",
        ),
        (
            "x = lambda: 1\n",
            "BUILD:1:5: functions may not be defined in BUILD files. You may move the function to a .bzl file and load it.",
        ),
        (
            "for x in []: pass\n",
            "BUILD:1:1: `for` statements are not allowed in BUILD files. You may inline the loop, move it to a function definition (in a .bzl file), or as a last resort use a list comprehension.",
        ),
        (
            "x = 1 if 2\n",
            "BUILD:1:5: missing else clause in conditional expression or semicolon before if",
        ),
        ("x = 0xZ\n", "BUILD:1:5: invalid hex literal"),
        ("x = 0x\n", "BUILD:1:5: invalid hex literal"),
        ("x = 0b2\n", "BUILD:1:5: invalid binary literal"),
        ("x = 0o\n", "BUILD:1:5: invalid base-8 integer literal: 0o"),
        (
            "x = 0o9\n",
            "BUILD:1:5: invalid base-8 integer literal: 0o9",
        ),
        (
            "x = [i for i in 1 2]\n",
            "BUILD:1:19: syntax error at '2': expected ']', 'for' or 'if'",
        ),
        (
            "x = {i: i for i in 1 2}\n",
            "BUILD:1:22: syntax error at '2': expected '}', 'for' or 'if'",
        ),
        (
            "x = f(i for i in 1 2)\n",
            "BUILD:1:9: syntax error at 'for': Starlark does not support Python-style generator expressions",
        ),
        ("f(a=1, a=2)\n", "BUILD:1:8: duplicate keyword argument: a"),
        (
            "print(a=1, 2)\n",
            "BUILD:1:12: positional argument may not follow keyword argument",
        ),
    ];

    #[test]
    fn a_syntax_error_is_the_event_bazel_prints() {
        let mut wrong = Vec::new();
        for (src, want) in PROBED {
            match parse_checked("BUILD", src, FileKind::Build) {
                Err(e) => {
                    let got = syntax_event("BUILD", &e);
                    if got != *want {
                        wrong.push(format!("{src:?}\n  want: {want}\n  got:  {got}"));
                    }
                }
                Ok(_) => wrong.push(format!("{src:?} parsed")),
            }
        }
        assert!(
            wrong.is_empty(),
            "{} differ:\n{}",
            wrong.len(),
            wrong.join("\n")
        );
    }
}
