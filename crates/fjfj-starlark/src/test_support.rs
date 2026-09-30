//! Shared by the tests that replay Bazel 9.2.0 probes.

use crate::{FileKind, bzl_globals, parse};
use starlark::environment::Module;
use starlark::eval::Evaluator;
use std::cell::RefCell;

pub(crate) struct Capture(pub(crate) RefCell<Vec<String>>);

impl starlark::PrintHandler for Capture {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0.borrow_mut().push(text.to_owned());
        Ok(())
    }
}

/// Run `src` as a `.bzl` body, returning what it printed or its error.
pub(crate) fn run(src: &str) -> Result<Vec<String>, String> {
    let ast = parse("t.bzl", src, FileKind::Bzl).map_err(|e| format!("{e:#}"))?;
    let capture = Capture(RefCell::new(Vec::new()));
    let result = Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.set_print_handler(&capture);
        eval.eval_module(ast, &bzl_globals())
            .map(|_| ())
            .map_err(|e| format!("{:#}", e.into_anyhow()))
    });
    result.map(|()| capture.0.into_inner())
}

/// Replay `(source, outcome)` rows: `Ok` is every line `print` gave, `Err`
/// is a substring of the error. Rows whose source contains one of `skip`
/// are known differences owned by another bead, and an `Err` row whose
/// message contains one of `relaxed` (the runtime's generic wording, which
/// is buildfiji-v32's) only has to fail. Returns what differs.
pub(crate) fn replay(
    rows: &[(&str, Result<&str, &str>)],
    skip: &[&str],
    relaxed: &[&str],
) -> Vec<String> {
    let mut wrong = Vec::new();
    for (src, want) in rows {
        if skip.iter().any(|marker| src.contains(marker)) {
            continue;
        }
        let ok = match (run(src), want) {
            (Ok(got), Ok(want)) => got.join("\n") == *want,
            (Err(got), Err(want)) => {
                got.contains(want) || relaxed.iter().any(|pattern| want.contains(pattern))
            }
            _ => false,
        };
        if !ok {
            wrong.push(format!("{src}\n  want: {want:?}\n  got:  {:?}", run(src)));
        }
    }
    wrong
}
