//! A Starlark error as Bazel 9.2.0 prints it:
//!
//! ```text
//! Traceback (most recent call last):
//!     File "/ws/pkg/BUILD", line 2, column 2, in <toplevel>
//!         f()
//!     File "/ws/pkg/l.bzl", line 2, column 9, in f
//!         fail("boom")
//! Error in fail: boom
//! ```
//!
//! (each `File` line is indented by a tab, and the source line by two).
//!
//! A column is where the operation is: the opening parenthesis of a call. The
//! files are named as the parser was told, and made into paths by whoever
//! knows where they are (see [`absolute_files`]).

use starlark::ErrorKind;
use starlark::codemap::FileSpan;
use starlark::values::ValueError;

/// A `load()` that could not be served, for the file that asked: Bazel says
/// why in the error of the package, and names no place in the file.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct LoadFailed(pub String);

impl LoadFailed {
    /// The reason, if `error` is a failed `load()`.
    pub fn of(error: &starlark::Error) -> Option<&LoadFailed> {
        match error.kind() {
            ErrorKind::Other(inner) => inner.downcast_ref::<LoadFailed>(),
            _ => None,
        }
    }
}

/// `error` as a traceback, ending in `Error in <function>: <message>` when a
/// function the file called failed and `Error: <message>` otherwise.
pub fn traceback(error: &starlark::Error) -> String {
    let frames = &error.call_stack().frames;
    let mut text = String::from("Traceback (most recent call last):\n");
    let mut function = "<toplevel>";
    for frame in frames {
        if let Some(at) = &frame.location {
            entry(&mut text, at, function);
        }
        function = if frame.name == "<module>" {
            "<toplevel>"
        } else {
            &frame.name
        };
    }
    // An error inside a function is at a place of its own; one a native
    // function gave is at the call, which the last frame has.
    let native = frames.last().filter(|last| {
        last.location.as_ref().map(FileSpan::resolve_span)
            == error.span().map(FileSpan::resolve_span)
    });
    if native.is_none()
        && let Some(at) = error.span()
    {
        entry(&mut text, at, function);
    }
    let message = match error.kind() {
        ErrorKind::Fail(e) => format!("Error in fail: {}", e.to_string().trim_start()),
        _ => match native {
            Some(frame)
                if !matches!(error.kind(), ErrorKind::Parser(_)) && !not_callable(error) =>
            {
                format!("Error in {}: {}", frame.name, plain(error))
            }
            _ => format!("Error: {}", plain(error)),
        },
    };
    let (message, text) = match missing_symbol(error) {
        Some((label, symbol, column)) => (
            format!("Error: file '{label}' does not contain symbol '{symbol}'"),
            with_last_column(text, column),
        ),
        None => (message, text),
    };
    let mut text = text;
    text.push_str(&message);
    text
}

/// A `load()` of a symbol the file does not have: the label as written, the
/// symbol and the column Bazel puts it at, which is inside the quotes of the
/// symbol's string.
fn missing_symbol(error: &starlark::Error) -> Option<(String, String, usize)> {
    let message = error.without_diagnostic().to_string();
    let symbol = message
        .strip_prefix("Module has no symbol `")?
        .split('`')
        .next()?
        .to_owned();
    let at = error.span()?;
    let source = at.file.source();
    let begin = at.span.begin().get() as usize;
    let statement = &source[source[..begin].rfind("load(")?..];
    let quoted = |from: &str| -> Option<(usize, String)> {
        let open = from.find(['"', '\''])?;
        let quote = from[open..].chars().next()?;
        let close = from[open + 1..].find(quote)?;
        Some((open, from[open + 1..open + 1 + close].to_owned()))
    };
    let (_, label) = quoted(statement)?;
    let (open, _) = quoted(&source[begin..])?;
    let line = at
        .file
        .find_line(starlark::codemap::Pos::new((begin + open) as u32));
    let line_start = at.file.line_span(line).begin().get() as usize;
    let column = source[line_start..begin + open].chars().count() + 2;
    Some((label, symbol, column))
}

/// `text` with the column of its last entry replaced.
fn with_last_column(text: String, column: usize) -> String {
    const KEY: &str = ", column ";
    let Some(at) = text.rfind(KEY) else {
        return text;
    };
    let from = at + KEY.len();
    let digits = text[from..]
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len() - from);
    format!("{}{column}{}", &text[..from], &text[from + digits..])
}

/// Whether the error is a call of a value that is not a function, which Bazel
/// does not place in any function.
fn not_callable(error: &starlark::Error) -> bool {
    match error.kind() {
        ErrorKind::Value(inner) => matches!(
            inner.downcast_ref::<ValueError>(),
            Some(ValueError::OperationNotSupported { op, .. }) if op == "call()"
        ),
        _ => false,
    }
}

/// What the error says, with neither the location nor the call stack.
fn plain(error: &starlark::Error) -> String {
    let text = error.without_diagnostic().to_string();
    text.strip_prefix("fail: ").unwrap_or(&text).to_owned()
}

/// One `File "<name>", line L, column C, in <function>` and the line.
fn entry(text: &mut String, at: &FileSpan, function: &str) {
    let begin = at.resolve_span().begin;
    // The compiler begins the span of a call at its parenthesis, as Bazel does.
    let column = begin.column + 1;
    text.push_str(&format!(
        "\tFile \"{}\", line {}, column {column}, in {function}\n\t\t{}\n",
        at.filename(),
        begin.line + 1,
        at.file.source_line(begin.line).trim_start(),
    ));
}

/// `text` with the name in each `File "<name>"` replaced by `path_of(name)`.
pub fn absolute_files(text: &str, path_of: &dyn Fn(&str) -> String) -> String {
    const OPEN: &str = "File \"";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(OPEN) {
        let (before, after) = rest.split_at(at + OPEN.len());
        out.push_str(before);
        match after.split_once('"') {
            Some((name, tail)) => {
                out.push_str(&path_of(name));
                out.push('"');
                rest = tail;
            }
            None => {
                rest = after;
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::run_build;

    #[test]
    fn a_traceback_lists_calls_from_the_file_to_the_failure() {
        let outcome = run_build("def r():\n    fail(\"boom\")\n", "r()\n");
        let text = outcome.fatal.expect("fails");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "Traceback (most recent call last):");
        assert_eq!(
            lines[1],
            "\tFile \"BUILD.bazel\", line 2, column 2, in <toplevel>"
        );
        assert_eq!(lines[2], "\t\tr()");
        assert!(
            lines[3].ends_with("u.bzl\", line 2, column 9, in r"),
            "{text}"
        );
        assert_eq!(lines[4], "\t\tfail(\"boom\")");
        assert_eq!(lines[5], "Error in fail: boom");
    }

    /// What Bazel 9.2.0 reports for a BUILD file of one statement, as probed:
    /// the column where the failing operation is, and the words.
    const PROBED: &[(&str, &str, &str)] = &[
        (
            "r = 1\n",
            "x = 1 + \"a\"",
            "column 7, in <toplevel>\n\t\tx = 1 + \"a\"\nError: unsupported binary operation: int + string",
        ),
        (
            "r = 1\n",
            "x = 1 in 2",
            "column 7, in <toplevel>\n\t\tx = 1 in 2\nError: unsupported binary operation: int in int",
        ),
        (
            "r = 1\n",
            "x = [] - 1",
            "column 8, in <toplevel>\n\t\tx = [] - 1\nError: unsupported binary operation: list - int",
        ),
        (
            "r = 1\n",
            "x = 1 < \"a\"",
            "column 7, in <toplevel>\n\t\tx = 1 < \"a\"\nError: unsupported comparison: int <=> string",
        ),
        (
            "r = 1\n",
            "x = -\"a\"",
            "column 5, in <toplevel>\n\t\tx = -\"a\"\nError: unsupported unary operation: -string",
        ),
        (
            "r = 1\n",
            "x = [1][3]",
            "column 8, in <toplevel>\n\t\tx = [1][3]\nError: index out of range (index is 3, but sequence has 1 elements)",
        ),
        (
            "r = 1\n",
            "x = \"abc\"[5]",
            "column 10, in <toplevel>\n\t\tx = \"abc\"[5]\nError: index out of range (index is 5, but sequence has 3 elements)",
        ),
        (
            "r = 1\n",
            "x = depset([])[0]",
            "column 15, in <toplevel>\n\t\tx = depset([])[0]\nError: type 'depset' has no operator [](int)",
        ),
        (
            "r = 1\n",
            "x = {\"a\": 1}[\"b\"]",
            "column 13, in <toplevel>\n\t\tx = {\"a\": 1}[\"b\"]\nError: key \"b\" not found in dictionary",
        ),
        (
            "r = 1\n",
            "x = \"a\".nope",
            "column 8, in <toplevel>\n\t\tx = \"a\".nope\nError: 'string' value has no field or method 'nope'",
        ),
        (
            "r = 1\n",
            "x = [].foo",
            "column 7, in <toplevel>\n\t\tx = [].foo\nError: 'list' value has no field or method 'foo'",
        ),
        (
            "r = 1\n",
            "x = depset([]).foo",
            "column 15, in <toplevel>\n\t\tx = depset([]).foo\nError: 'depset' value has no field or method 'foo'",
        ),
        (
            "r = 1\n",
            "x = None.x",
            "column 9, in <toplevel>\n\t\tx = None.x\nError: 'NoneType' value has no field or method 'x'",
        ),
        (
            "r = 1\n",
            "x = [x for x in 1]",
            "column 17, in <toplevel>\n\t\tx = [x for x in 1]\nError: type 'int' is not iterable",
        ),
        (
            "r = 1\n",
            "x = \"a\" % (1, 2)",
            "column 9, in <toplevel>\n\t\tx = \"a\" % (1, 2)\nError: not all arguments converted during string formatting",
        ),
        (
            "r = 1\n",
            "x = 1 // 0",
            "column 7, in <toplevel>\n\t\tx = 1 // 0\nError: integer division by zero",
        ),
        (
            "r = 1\n",
            "x = 1 % 0",
            "column 7, in <toplevel>\n\t\tx = 1 % 0\nError: integer modulo by zero",
        ),
        (
            "r = 1\n",
            "x = int(\"z\")",
            "column 8, in <toplevel>\n\t\tx = int(\"z\")\nError in int: invalid base-10 literal: \"z\"",
        ),
        (
            "r = 1\n",
            "x = int(\"z\", 16)",
            "column 8, in <toplevel>\n\t\tx = int(\"z\", 16)\nError in int: invalid base-16 literal: \"z\"",
        ),
        (
            "r = 1\n",
            "x = {}.get()",
            "column 11, in <toplevel>\n\t\tx = {}.get()\nError in get: get() missing 1 required positional argument: key",
        ),
        (
            "r = 1\n",
            "x = [].append(1, 2)",
            "column 14, in <toplevel>\n\t\tx = [].append(1, 2)\nError in append: append() accepts no more than 1 positional argument but got 2",
        ),
        (
            "r = 1\n",
            "x = str(1, 2)",
            "column 8, in <toplevel>\n\t\tx = str(1, 2)\nError in str: str() accepts no more than 1 positional argument but got 2",
        ),
        (
            "r = 1\n",
            "x = \"abc\".split(1)",
            "column 16, in <toplevel>\n\t\tx = \"abc\".split(1)\nError in split: in call to split(), parameter 'sep' got value of type 'int', want 'string'",
        ),
        (
            "r = 1\n",
            "load(':u.bzl', 'nope')",
            "column 17, in <toplevel>\n\t\tload(':u.bzl', 'nope')\nError: file ':u.bzl' does not contain symbol 'nope'",
        ),
        (
            "r = 1\n",
            "load(':u.bzl', Y = 'nope')",
            "column 21, in <toplevel>\n\t\tload(':u.bzl', Y = 'nope')\nError: file ':u.bzl' does not contain symbol 'nope'",
        ),
        (
            "r = 1\n",
            "x = \"%\" % ()",
            "column 9, in <toplevel>\n\t\tx = \"%\" % ()\nError: incomplete format pattern ends with %: \"%\"",
        ),
        (
            "r = 1\n",
            "x = \"%z\" % 1",
            "column 10, in <toplevel>\n\t\tx = \"%z\" % 1\nError: unsupported format character \"z\" at index 1 in \"%z\"",
        ),
        (
            "r = 1\n",
            "x = \"%s %s\" % (1,)",
            "column 13, in <toplevel>\n\t\tx = \"%s %s\" % (1,)\nError: not enough arguments for format pattern \"%s %s\": (1,)",
        ),
        (
            "r = 1\n",
            "x = \"%c\" % 65",
            "column 10, in <toplevel>\n\t\tx = \"%c\" % 65\nError: unsupported format character \"c\" at index 1 in \"%c\"",
        ),
        (
            "r = 1\n",
            "x = \"abc%\" % 1",
            "column 12, in <toplevel>\n\t\tx = \"abc%\" % 1\nError: incomplete format pattern ends with %: \"abc%\"",
        ),
        (
            "r = 1\n",
            "x = max([])",
            "column 8, in <toplevel>\n\t\tx = max([])\nError in max: expected at least one item",
        ),
    ];

    /// A call that does not fit a `def` is reported at the `def`, in the callee.
    const PROBED_CALLS: &[(&str, &str)] = &[
        ("r(1)", "does not accept positional arguments, but got 1"),
        ("r(a=1)", "got unexpected keyword argument: a"),
        ("r(1, 2)", "does not accept positional arguments, but got 2"),
    ];

    /// `text` with the path of `u.bzl` as the bare name.
    fn bare(text: &str) -> String {
        text.lines()
            .map(|line| match (line.find("File \""), line.find("u.bzl\"")) {
                (Some(from), Some(to)) => format!("{}File \"{}", &line[..from], &line[to..]),
                _ => line.to_owned(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_runtime_error_is_worded_and_placed_as_bazel_does() {
        let mut wrong = Vec::new();
        for (bzl, build, want) in PROBED {
            let text = run_build(bzl, build).fatal.expect("fails");
            // The load line before it makes the statement's line 2.
            let at = format!("line 2, {want}");
            if !text.contains(&at) {
                wrong.push(format!("{build}\n  want: {at}\n  got:  {text}"));
            }
        }
        assert!(
            wrong.is_empty(),
            "{} differ:\n{}",
            wrong.len(),
            wrong.join("\n")
        );
    }

    #[test]
    fn a_call_that_does_not_fit_is_placed_at_the_def() {
        for (build, words) in PROBED_CALLS {
            let text = bare(&run_build("def r(): pass\n", build).fatal.expect("fails"));
            let want = format!(
                "\tFile \"u.bzl\", line 1, column 5, in r\n\t\tdef r(): pass\nError: r() {words}"
            );
            assert!(text.ends_with(&want), "{build}: {text}");
        }
        let text = bare(&run_build("def r(a): pass\n", "r()").fatal.expect("fails"));
        assert!(
            text.ends_with("Error: r() missing 1 required positional argument: a"),
            "{text}"
        );
    }

    #[test]
    fn files_are_named_by_the_caller() {
        let text = "\tFile \"a/BUILD\", line 1, column 2, in <toplevel>\n\t\tx\n";
        assert_eq!(
            absolute_files(text, &|name| format!("/root/{name}")),
            "\tFile \"/root/a/BUILD\", line 1, column 2, in <toplevel>\n\t\tx\n"
        );
    }
}
