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

/// The second event Bazel prints for a `.bzl` that does not parse,
/// `<file>:<line>:<column>: contains syntax errors`, if it prints one.
///
/// Bazel's parser recovers from a bad expression by putting an error
/// expression in its place, and the resolver reports that. The error
/// expression starts at the failing token, unless the token is inside a list,
/// a dict or a parenthesis that is not a call or an index: those fail whole,
/// and the event is at the start of the outermost one.
pub fn contains_event(file: &str, source: &str, error: &starlark::Error) -> Option<String> {
    if matches!(error.kind(), ErrorKind::Scope(_)) {
        return None;
    }
    let message = message(error);
    let wanted = message.split_once(": expected ")?.1;
    let end_of_file = error.span().is_none_or(|at| {
        at.span.begin().get() == 0 && at.span.end().get() == 0 && !source.is_empty()
    });
    let offset = match error.span() {
        Some(at) if !end_of_file => at.span.begin().get() as usize,
        _ => source.len(),
    };
    let chain = literal_chain(source, offset);
    let start = match wanted {
        "expression" => chain.map(|c| c.start).or(Some(offset)),
        "]" | "}" | ":" | "',', 'for' or ']'" | "']', 'for' or 'if'" | "'}', 'for' or 'if'" => {
            chain.map(|c| c.start)
        }
        ")" => chain.filter(|c| c.top_is_plain_paren).map(|c| c.start),
        _ => None,
    }?;
    let (line, column) = line_column(source, start);
    Some(format!("{file}:{line}:{column}: contains syntax errors"))
}

/// The line and column (from 1) of the byte `offset` of `source`.
fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset.min(source.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, column)
}

/// The brackets open at an error that fail whole.
struct Chain {
    /// Where the outermost starts.
    start: usize,
    /// Whether the innermost is a parenthesis with no comma in it, which is
    /// not a tuple.
    top_is_plain_paren: bool,
}

/// Where the outermost of the brackets that are open at `upto`, and are not
/// a call or an index, and have no call or index inside them, starts.
fn literal_chain(source: &str, upto: usize) -> Option<Chain> {
    // (is a literal, where it starts, has a comma directly inside)
    let mut open: Vec<(bool, usize, bool, u8)> = Vec::new();
    let bytes = source.as_bytes();
    let mut value_before = false;
    let mut i = 0;
    while i < bytes.len() && i < upto {
        let c = bytes[i];
        match c {
            b'#' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'"' | b'\'' => {
                let triple = bytes[i..].starts_with(&[c, c, c]);
                i += if triple { 3 } else { 1 };
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                    } else if triple && bytes[i..].starts_with(&[c, c, c]) {
                        i += 3;
                        break;
                    } else if !triple && (bytes[i] == c || bytes[i] == b'\n') {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                value_before = true;
                continue;
            }
            b'(' | b'[' | b'{' => {
                open.push((c == b'{' || !value_before, i, false, c));
                value_before = false;
            }
            b')' | b']' | b'}' => {
                open.pop();
                value_before = true;
            }
            c if c.is_ascii_alphanumeric() || c == b'_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = &source[start..i];
                value_before = !matches!(
                    word,
                    "in" | "not"
                        | "and"
                        | "or"
                        | "if"
                        | "else"
                        | "for"
                        | "return"
                        | "lambda"
                        | "load"
                );
                continue;
            }
            b',' => {
                if let Some(top) = open.last_mut() {
                    top.2 = true;
                }
                value_before = false;
            }
            c if c.is_ascii_whitespace() => {}
            _ => value_before = false,
        }
        i += 1;
    }
    let run = open
        .iter()
        .rev()
        .take_while(|(literal, ..)| *literal)
        .last()?;
    let top = open.last()?;
    Some(Chain {
        start: run.1,
        top_is_plain_paren: top.3 == b'(' && !top.2,
    })
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
    if text.contains("`if` cannot be used outside `def`") {
        return "`if` statements are not allowed in BUILD files. You may move conditional logic to a function definition (in a .bzl file), or use an `if` expression for simple cases.".to_owned();
    }
    if text.contains("`return` cannot be used outside of a `def`") {
        return "return statements must be inside a function".to_owned();
    }
    if text.contains("Python-style generator expressions") {
        return "syntax error at 'for': Starlark does not support Python-style generator expressions".to_owned();
    }
    if let Some(c) = between(text, "Parse error: invalid input `", "`") {
        return format!("invalid character: '{c}'");
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
        // A string is named by its literal, which already has its quotes.
        other if other.starts_with("string literal ") => {
            other["string literal ".len()..].to_owned()
        }
        other => quoted(other).unwrap_or(other).to_owned(),
    }
}

/// How Bazel writes what the parser wanted.
fn wanted_token(wanted: &str) -> String {
    match wanted {
        "new line" => "newline".to_owned(),
        "expression" => "expression".to_owned(),
        // What can follow a comprehension: the list is Bazel's, quotes and all.
        "']', 'for' or 'if'" | "'}', 'for' or 'if'" | "',', 'for' or ']'" => wanted.to_owned(),
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
            "if x: pass\n",
            "BUILD:1:1: `if` statements are not allowed in BUILD files. You may move conditional logic to a function definition (in a .bzl file), or use an `if` expression for simple cases.",
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
        (
            "x = [1 2]\n",
            "BUILD:1:8: syntax error at '2': expected ',', 'for' or ']'",
        ),
        (
            "x = a.\n",
            "BUILD:1:7: syntax error at 'newline': expected identifier after dot",
        ),
        ("f(1 2)\n", "BUILD:1:5: syntax error at '2': expected ,"),
        ("f(a=1 2)\n", "BUILD:1:7: syntax error at '2': expected ,"),
        (
            "x = \"a\" \"b\"\n",
            "BUILD:1:9: Implicit string concatenation is forbidden, use the + operator",
        ),
        ("x = $\n", "BUILD:1:5: invalid character: '$'"),
        ("f(a=1, a=2)\n", "BUILD:1:8: duplicate keyword argument: a"),
        (
            "print(a=1, 2)\n",
            "BUILD:1:12: positional argument may not follow keyword argument",
        ),
    ];

    /// A string after a string is the concatenation event, and then the
    /// syntax error with the token as written (a single-quoted string is
    /// named with double quotes).
    #[test]
    fn a_string_after_a_string_is_two_events() {
        let parsed = crate::dialect::parse_all("BUILD", "x = \"a\" 'b'\n", FileKind::Build);
        let events: Vec<String> = parsed
            .syntax
            .iter()
            .map(|e| syntax_event("BUILD", e))
            .collect();
        assert_eq!(
            events,
            [
                "BUILD:1:9: Implicit string concatenation is forbidden, use the + operator",
                "BUILD:1:9: syntax error at '\"b\"': expected newline",
            ]
        );
    }

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
