//! Symbolic macros against Bazel 9.2.0 (buildfiji-mum.3.8): the tables in
//! `macros_matrix` are what Bazel printed and reported for the same code,
//! and the tests here are what a table cannot say.

use crate::WithoutSites;
use crate::macros_matrix::{MACRO_BUILD_CASES, MACRO_CASES};
use crate::test_support::{replay, replay_build};

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "has no field or method",
    "has no operator",
    "unsupported binary operation",
    "unsupported comparison",
    "unhashable type",
    "in call to len()",
    "in call to list()",
    "in call to hash()",
    "duplicate keyword argument",
    "no native function or rule",
    "not found in view",
    "dictionary expression has duplicate key",
    "got unexpected keyword argument",
    "got unexpected keyword arguments",
    "missing 1 required positional argument",
    "**kwargs arguments",
    "does not contain symbol",
];

/// Probes the crate cannot answer the way Bazel does.
const SKIPPED: &[&str] = &[
    // `print(label)` writes `//a:b` in Bazel and `str(label)` `@@//a:b`; the
    // crate's `print` writes the latter (buildfiji-v32).
    "print(native.package_relative_label(':x'))",
    "attrs={'x':attr.output()}",
    // The probe workspace had no `other`, and the test one has.
    "@other//foo",
    // `**kwargs` of a call in a BUILD file is an event in Bazel (buildfiji-v32).
    "**{",
    "(**args)",
    // The crate calls a builtin a `function` (buildfiji-v32).
    "implementation=len",
    "native.cc_library",
    // Natives are not rule values here (buildfiji-136.10).
    "inherit_attrs=native.",
    // A value made in a function and then bound is named when the module is
    // done, not at once (buildfiji-10f).
    "m=mk()",
    // A `.bzl`'s own `print` is not a BUILD file's.
    "print(r)\nm=r",
];

fn assert_replays(wrong: Vec<String>) {
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn macro_declarations_replay_bazel() {
    assert_replays(replay(MACRO_CASES, SKIPPED, GENERIC));
}

#[test]
fn macro_instantiations_replay_bazel() {
    assert_replays(replay_build(MACRO_BUILD_CASES, SKIPPED, GENERIC));
}

use crate::test_support::{run, run_build};

#[test]
fn a_value_is_named_where_it_is_bound() {
    let printed = run(
        "P = provider()\n_R = rule(implementation = lambda c: [])\nM = macro(implementation = lambda name: None)\n\
         S = subrule(implementation = lambda: None)\nprint(_R, M, S)\n\
         def f():\n    return macro(implementation = lambda name: None)\nN = f()\nprint(N)",
    )
    .unwrap();
    // One a function made is named when its module is done.
    assert_eq!(printed, ["<rule _R> <macro M> <subrule S>", "<macro>"]);
}

#[test]
fn a_macro_runs_once_and_returns_none() {
    let out = run_build(
        "def _i(name, **kw):\n    native.filegroup(name = name + '_g')\nr = macro(implementation = _i)",
        "print(r(name = 'a'))\nprint(sorted(existing_rules().keys()))",
    );
    assert!(out.fatal.is_none() && out.events.is_empty(), "{out:?}");
    assert_eq!(out.printed.without_sites(), ["None", r#"["a_g"]"#]);
}

#[test]
fn a_finalizer_runs_after_the_file_and_sees_the_rules_before_it() {
    let out = run_build(
        "def _f(name, **kw):\n    print('fin', name, sorted(native.existing_rules().keys()))\n    native.filegroup(name = name + '_x')\n\
         def _i(name, **kw):\n    native.filegroup(name = name + '_g')\n\
         fin = macro(implementation = _f, finalizer = True)\nr = macro(implementation = _i)",
        "load(':u.bzl', 'fin')\nfin(name = 'f1')\nr(name = 'a')\nfin(name = 'f2')\nprint('file done')",
    );
    assert!(out.fatal.is_none() && out.events.is_empty(), "{out:?}");
    assert_eq!(
        out.printed.without_sites(),
        ["file done", r#"fin f1 ["a_g"]"#, r#"fin f2 ["a_g"]"#]
    );
    let package = out.package.expect("the package");
    for name in ["a_g", "f1_x", "f2_x"] {
        assert!(package.target(name).is_some(), "{name}");
    }
}
