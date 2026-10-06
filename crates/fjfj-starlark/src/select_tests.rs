//! `select()` against Bazel 9.2.0 (buildfiji-mum.3.6): the tables in
//! `select_matrix` are what Bazel printed and reported for the same code, and
//! the tests here are what a table cannot say.

use crate::WithoutSites;
use crate::select_matrix::{SELECT_BUILD_CASES, SELECT_CASES};
use crate::test_support::{replay, replay_build};

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[];

/// Probes the crate cannot answer the way Bazel does.
const SKIPPED: &[&str] = &[
    // A plain dict on the left of `|` has no hook for the right operand to
    // take over (buildfiji-v32).
    "} | select(",
    // The crate prints `range(2)` where Bazel prints `range(0, 2)`.
    "range(",
    // The crate calls a builtin a `function`, prints it and a function by
    // another name (buildfiji-v32), and lacks the natives Bazel lists.
    "print(type(select))",
    "print(select)",
    "select({\"a\":len})",
    "def f()",
    "print(dir(native))",
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
fn select_replays_bazel() {
    assert_replays(replay(SELECT_CASES, SKIPPED, GENERIC));
}

#[test]
fn select_in_attributes_replays_bazel() {
    assert_replays(replay_build(SELECT_BUILD_CASES, SKIPPED, GENERIC));
}

use crate::test_support::{run_build, run_files};
use fjfj_graph::Label;
use fjfj_graph::package::TargetKind;
use fjfj_graph::rule::{AttrValue, SelectorList, default_condition};

const STRINGS: &str =
    r#"r = rule(implementation=lambda ctx: [], attrs={"sl": attr.string_list()})"#;

/// What the target `t` of `build` holds for `attr`.
fn held(bzl: &str, build: &str, attr: &str) -> Option<AttrValue> {
    let out = run_build(bzl, build);
    assert!(out.fatal.is_none() && out.events.is_empty(), "{out:?}");
    let package = out.package.expect("the package");
    let TargetKind::Rule { attrs, .. } = &package.target("t").expect("t").kind else {
        panic!("t is not a rule")
    };
    attrs
        .iter()
        .find(|(k, _)| k == attr)
        .map(|(_, v)| v.clone())
}

fn condition(name: &str) -> Label {
    Label {
        repo: String::new(),
        package: String::new(),
        name: name.to_owned(),
    }
}

fn selectors(value: Option<AttrValue>) -> SelectorList {
    match value {
        Some(AttrValue::Select(list)) => list,
        other => panic!("not a select: {other:?}"),
    }
}

#[test]
fn a_select_keeps_its_branches_in_order_with_labels_for_keys() {
    let list = selectors(held(
        STRINGS,
        r#"r(name="t", sl=select({"b": ["y"], ":a": ["x"], "//conditions:default": []}, no_match_error="no"))"#,
        "sl",
    ));
    assert!(!list.pipe && list.elements.len() == 1);
    let one = &list.elements[0];
    assert_eq!(one.no_match_error, "no");
    assert!(!one.unconditional);
    let keys: Vec<_> = one.branches.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(keys, [condition("b"), condition("a"), default_condition()]);
    assert_eq!(
        one.branches[0].1,
        Some(AttrValue::StringList(vec!["y".to_owned()]))
    );
}

#[test]
fn plain_values_between_selects_are_unconditional_selectors() {
    let list = selectors(held(
        STRINGS,
        r#"r(name="t", sl=["p"] + select({"a": ["x"]}) + ["q"])"#,
        "sl",
    ));
    let unconditional: Vec<bool> = list.elements.iter().map(|e| e.unconditional).collect();
    assert_eq!(unconditional, [true, false, true]);
    assert_eq!(
        list.elements[0].branches,
        [(
            default_condition(),
            Some(AttrValue::StringList(vec!["p".to_owned()]))
        )]
    );
}

#[test]
fn what_no_configuration_can_change_is_the_value_itself() {
    let strings = |items: &[&str]| {
        Some(AttrValue::StringList(
            items.iter().map(|s| s.to_string()).collect(),
        ))
    };
    for (written, want) in [
        (
            r#"select({"//conditions:default": ["x"]})"#,
            strings(&["x"]),
        ),
        (
            r#"["p"] + select({"//conditions:default": ["x"]}) + ["q"]"#,
            strings(&["p", "x", "q"]),
        ),
        // Two spellings of one condition are one branch, but the select was
        // not written with only the default.
    ] {
        assert_eq!(
            held(STRINGS, &format!(r#"r(name="t", sl={written})"#), "sl"),
            want,
            "{written}"
        );
    }
    let two_spellings = held(
        "def m():\n  native.filegroup(name=\"t\", srcs=select({Label(\"//conditions:default\"): [\"f1.txt\"], \"//conditions:default\": []}))\nr = m",
        "r()",
        "srcs",
    );
    assert!(
        matches!(two_spellings, Some(AttrValue::Select(_))),
        "{two_spellings:?}"
    );
}

#[test]
fn ints_add_and_strings_join_when_unconditional() {
    let bzl =
        r#"r = rule(implementation=lambda ctx: [], attrs={"i": attr.int(), "s": attr.string()})"#;
    assert_eq!(
        held(
            bzl,
            r#"r(name="t", i=1 + select({"//conditions:default": 2}) + 3)"#,
            "i"
        ),
        Some(AttrValue::Int(6))
    );
    assert_eq!(
        held(
            bzl,
            r#"r(name="t", s="a" + select({"//conditions:default": "b"}))"#,
            "s"
        ),
        Some(AttrValue::String("ab".to_owned()))
    );
}

#[test]
fn a_select_is_itself_once_frozen_and_loaded() {
    let printed = run_files(
        &[(
            "a.bzl",
            "S = select({'a': ['x']})\nT = S + ['y']\nU = select({'a': {'k': 1}}) | select({'b': {'j': 2}})",
        )],
        "load(':a.bzl', 'S', 'T', 'U')\nprint(S)\nprint(T)\nprint(U)\nprint(S == select({'a': ['x']}))\nprint(T + ['z'])\nprint(type(T))",
    )
    .unwrap();
    assert_eq!(
        printed,
        [
            r#"select({"a": ["x"]})"#,
            r#"select({"a": ["x"]}) + ["y"]"#,
            r#"select({"a": {"k": 1}}) | select({"b": {"j": 2}})"#,
            "True",
            r#"select({"a": ["x"]}) + ["y"] + ["z"]"#,
            "select",
        ]
    );
}

#[test]
fn a_frozen_select_is_an_attribute_value() {
    let bzl = format!("{STRINGS}\nS = select({{'a': ['x']}})");
    let out = run_build(&bzl, "load(':u.bzl', 'S')\nr(name='t', sl=S + ['y'])");
    assert!(out.fatal.is_none() && out.events.is_empty(), "{out:?}");
}

#[test]
fn a_select_is_never_hashable_iterable_or_empty() {
    let cases = [
        (r#"print({select({"a": 1}): 1})"#, "hashable"),
        (r#"print(len(select({"a": 1})))"#, "select"),
        (r#"print(select({}))"#, "an empty dictionary"),
        (r#"print(select({1: 2}))"#, "got int for dict key"),
    ];
    for (src, want) in cases {
        let err = crate::test_support::run(src).unwrap_err();
        assert!(err.contains(want), "{src}: {err}");
    }
    assert_eq!(
        crate::test_support::run(r#"print(bool(select({"a": 1})))"#).unwrap(),
        ["True"]
    );
}

#[test]
fn a_select_is_a_global_of_a_build_file_and_a_bzl_and_not_a_native() {
    assert_eq!(
        run_build("r = 1", r#"print(select({"a": 1}))"#)
            .printed
            .without_sites(),
        [r#"select({"a": 1})"#]
    );
    let native = crate::test_support::run("print(native.select)").unwrap_err();
    assert!(native.contains("select"), "{native}");
}
