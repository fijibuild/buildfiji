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
            Some(frame) if !matches!(error.kind(), ErrorKind::Parser(_)) => {
                format!("Error in {}: {}", frame.name, plain(error))
            }
            _ => format!("Error: {}", plain(error)),
        },
    };
    text.push_str(&message);
    text
}

/// What the error says, with neither the location nor the call stack.
fn plain(error: &starlark::Error) -> String {
    let text = error.without_diagnostic().to_string();
    text.strip_prefix("fail: ").unwrap_or(&text).to_owned()
}

/// One `File "<name>", line L, column C, in <function>` and the line.
fn entry(text: &mut String, at: &FileSpan, function: &str) {
    let begin = at.resolve_span().begin;
    // Bazel locates a call at its opening parenthesis.
    let source = at.source_span();
    let call = source.find('(').filter(|&paren| {
        source[..paren]
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    });
    let column = begin.column + 1 + call.map_or(0, |paren| source[..paren].chars().count());
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

    #[test]
    fn an_operator_error_is_not_a_call() {
        let outcome = run_build("r = 1\n", "x = 1 + \"a\"\n");
        let text = outcome.fatal.expect("fails");
        assert!(text.contains("line 2, column"), "{text}");
        assert!(text.ends_with("\n\t\tx = 1 + \"a\"\nError: Operation `+` not supported for types `int` and `string`"), "{text}");
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
