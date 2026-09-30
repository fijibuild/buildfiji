//! `rule()` against Bazel 9.2.0 (buildfiji-mum.3.5): the tables at the bottom
//! are what Bazel printed and reported for the same `.bzl` and BUILD files,
//! and the tests here are what a table cannot say.
//!
//! `RULE_CASES` are `.bzl` bodies (what `rule()` accepts and refuses), and
//! `BUILD_CASES` are a `.bzl` and a BUILD file, with what the BUILD file
//! printed, the events it reported and the fatal error that stopped it.

use crate::test_support::{BuildRow, replay, replay_build, run_build, run_files};
use fjfj_graph::package::TargetKind;
use fjfj_graph::rule::AttrValue;

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "has no field or method",
    "has no operator",
    "unsupported binary operation",
    "unsupported comparison",
    "unhashable type",
    "cannot set .",
    "is not callable",
    "in call to len()",
    "in call to list()",
    "in call to hash()",
    "duplicate keyword argument",
    "positional argument may not follow keyword argument",
    "print() got unexpected keyword argument",
    "**kwargs arguments are not allowed",
    "*args arguments are not allowed",
    "lambda() ",
];

/// Probes that need something no bead has built yet, or differ in the
/// crate.
const SKIPPED: &[&str] = &[
    "DefaultInfo",
    "aspect(",
    "transition(",
    "config.",
    "exec_group(",
    "subrule(",
    "macro(",
    // `print(R)` right after `R = rule(...)` says `<rule R>` in Bazel, which
    // names a rule at the assignment (buildfiji-10f).
    "print(R)\n",
    // The crate calls a builtin a `function` and accepts it where Bazel
    // does not (buildfiji-v32).
    "implementation=print",
    "implementation=len",
    // The probe workspace had no `mydep`, and the test one has.
    "@mydep",
    // The crate refuses a `**` that repeats a keyword before the call, and
    // a repeated keyword in a BUILD file is an event in Bazel and fatal here.
    "**{",
    "name=\"a\", name=\"b\"",
    // `1 << 31` wraps in the crate (buildfiji-sbj).
    "1 << 31",
    // Bazel crashes with both `applicable_licenses` and `package_metadata`.
    "applicable_licenses=1",
    // The crate prints `range(2)` where Bazel prints `range(0, 2)`.
    "range(",
    // A `return` outside a function is a parse error, worded differently.
    "return 5",
    // `visibility()` and `configuration_field()` are other beads'
    // (buildfiji-mum.3.7, buildfiji-mum.3.8).
    "configuration_field(",
    "visibility(",
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
fn rule_takes_what_bazel_takes() {
    assert_replays(replay(RULE_CASES, SKIPPED, GENERIC));
}

#[test]
fn a_rule_is_called_as_bazel_calls_it() {
    assert_replays(replay_build(BUILD_CASES, SKIPPED, GENERIC));
}

/// The BUILD file of `build`, which loads `r` from `bzl`.
fn built(bzl: &str, build: &str) -> fjfj_graph::package::Package {
    let out = run_build(bzl, build);
    assert!(out.fatal.is_none() && out.events.is_empty(), "{out:?}");
    out.package.expect("the package")
}

#[test]
fn a_call_is_a_target_with_its_class_and_what_it_set() {
    let p = built(
        r#"r = rule(implementation = lambda ctx: None, attrs = {"s": attr.string(default = "d"), "n": attr.int()})"#,
        "load(':u.bzl', 'r')\nr(name = 'a', n = 3, tags = ['t'], visibility = ['//visibility:public'])\nr(name = 'b', s = None)\n",
    );
    let a = p.target("a").unwrap();
    let TargetKind::Rule { rule_class, attrs } = &a.kind else {
        panic!("{a:?}")
    };
    assert_eq!(rule_class, "r");
    // Only what the call set, in the order it wrote it, and no default.
    let names: Vec<&str> = attrs.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(names, ["n", "tags", "visibility"]);
    assert_eq!(attrs[0].1, AttrValue::Int(3));
    assert_eq!(attrs[1].1, AttrValue::StringList(vec!["t".to_owned()]));
    let TargetKind::Rule { attrs, .. } = &p.target("b").unwrap().kind else {
        panic!()
    };
    assert!(attrs.is_empty());
}

#[test]
fn a_rule_makes_the_files_it_declares() {
    let p = built(
        r#"r = rule(implementation = lambda ctx: None, outputs = {"o": "%{name}.txt", "p": "%{name}_p"}, attrs = {"x": attr.output()})"#,
        "load(':u.bzl', 'r')\nr(name = 'a', x = 'out.bin')\n",
    );
    for file in ["a.txt", "a_p", "out.bin"] {
        let target = p.target(file).unwrap_or_else(|| panic!("no {file}"));
        assert_eq!(
            target.kind,
            TargetKind::GeneratedFile {
                rule: "a".to_owned()
            }
        );
    }
    assert!(matches!(
        p.target("a").unwrap().kind,
        TargetKind::Rule { .. }
    ));
}

#[test]
fn a_rule_is_itself_once_frozen_and_loaded() {
    let a = "r = rule(implementation = lambda ctx: None)\nd = {r: 'r'}\n";
    let main = "load(':a.bzl', 'r', 'd')\nprint(r == r, d[r], type(r), r)\ns = rule(implementation = lambda ctx: None)\nprint(r == s)\n";
    let out = run_files(&[("a.bzl", a)], main).unwrap();
    assert_eq!(out, ["True r rule <rule r>", "False"]);
}

const RULE_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"R = rule(implementation=lambda ctx: None)
print(R)
print(type(R))
print(repr(R))
print(str(R))
print(dir(R))
print(R == R)
print(bool(R))
def X(): pass
X()"#,
        Ok(r#"<rule R>
rule
<rule R>
<rule R>
[]
True
True"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule())
def X(): pass
X()"#,
        Err(r#"rule() missing 1 required positional argument: implementation"#),
    ),
    (
        r#"print(rule(lambda ctx: None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(lambda ctx: None, {}))
def X(): pass
X()"#,
        Err(r#"rule() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(rule(1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'NoneType', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'NoneType', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda: None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda a, b: None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda *a: None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx, x=1: None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=print))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'builtin_function_or_method', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=len))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'builtin_function_or_method', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'implementation' got value of type 'Provider', want 'function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, foo=1))
def X(): pass
X()"#,
        Err(r#"rule() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, implemntation=1))
def X(): pass
X()"#,
        Err(
            r#"rule() got unexpected keyword argument 'implemntation' (did you mean 'implementation'?)"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attr={}))
def X(): pass
X()"#,
        Err(r#"rule() got unexpected keyword argument 'attr' (did you mean 'attrs'?)"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=None))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'NoneType', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=[]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": 1}))
def X(): pass
X()"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={1: attr.string()}))
def X(): pass
X()"#,
        Err(r#"got dict<int, Attribute> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.string()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x y": attr.string()}))
def X(): pass
X()"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"": attr.string()}))
def X(): pass
X()"#,
        Err(r#"attribute name `` is not a valid identifier."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"1x": attr.string()}))
def X(): pass
X()"#,
        Err(r#"attribute name `1x` is not a valid identifier."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"name": attr.string()}))
def X(): pass
X()"#,
        Err(r#"attribute `name`: built-in attributes cannot be overridden."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"visibility": attr.string()}))
def X(): pass
X()"#,
        Err(r#"attribute `visibility`: built-in attributes cannot be overridden."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"tags": attr.string_list()}))
def X(): pass
X()"#,
        Err(r#"attribute `tags`: built-in attributes cannot be overridden."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"testonly": attr.bool()}))
def X(): pass
X()"#,
        Err(r#"attribute `testonly`: built-in attributes cannot be overridden."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"deprecation": attr.string()}))
def X(): pass
X()"#,
        Err(r#"attribute `deprecation`: built-in attributes cannot be overridden."#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.string()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.string(default="d")}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.label()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.label(default="//a:b")}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.label_list()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.int()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.bool()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.string_list()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"_x": attr.output()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.label(default="//a:b", mandatory=True)}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.string(mandatory=True, default="a")}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.output(), "y": attr.output_list()}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a"], default="b")}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a"], default="")}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a"])}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.int(values=[1], default=2)}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=[1])}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"x": attr.int(values=["a"])}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=dict([("a", attr.string()), ("a", attr.int())])))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=None))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'NoneType', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test="s"))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=""))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=[]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=["a"]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=[1]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test={}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=("a",)))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=lambda: 1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'function', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=depset(["a"])))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'depset', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=struct()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'struct', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=attr.string()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'Attribute', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=provider()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'Provider', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, test=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'test' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=True))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'bool', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=False))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'bool', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs="s"))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'string', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=""))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'string', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=["a"]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=[1]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"got dict<string, string> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=("a",)))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'tuple', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=lambda: 1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'function', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=depset(["a"])))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'depset', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=struct()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'struct', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=attr.string()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'Attribute', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=provider()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'Provider', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, attrs=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'bool', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'bool', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'int', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'int', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'int', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'string', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'string', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'list', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'list', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'list', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs={}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs={"a":"b"}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'tuple', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=lambda: 1))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'depset', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'struct', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'Attribute', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'Provider', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, outputs=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'outputs' got value of type 'list', want 'dict, NoneType, or function'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'executable' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable="s"))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=""))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=[]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=["a"]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=[1]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable={}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=("a",)))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'executable' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=depset(["a"])))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'depset', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=struct()))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'struct', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'executable' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'executable' got value of type 'Provider', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, executable=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'struct', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'Provider', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, output_to_genfiles=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'output_to_genfiles' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'fragments' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'fragments' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'fragments' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=[]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=["a"]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=[1]))
def X(): pass
X()"#,
        Err(r#"at index 0 of fragments, got element of type int, want string"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=("a",)))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'fragments' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, fragments=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"at index 0 of fragments, got element of type Attribute, want string"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=[]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=["a"]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=[1]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=("a",)))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'host_fragments' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, host_fragments=[attr.string()]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'struct', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'Provider', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, _skylark_testable=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter '_skylark_testable' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=[]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=["a"]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=[1]))
def X(): pass
X()"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=("a",)))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'toolchains' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, toolchains=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Descriptor"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc="s"))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=""))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'struct', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'Provider', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, doc=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=True))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'bool', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=False))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'bool', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=[]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=["a"]))
def X(): pass
X()"#,
        Err(r#"at index 0 of provides, got element of type string, want Provider"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=[1]))
def X(): pass
X()"#,
        Err(r#"at index 0 of provides, got element of type int, want Provider"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides={}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'provides' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=("a",)))
def X(): pass
X()"#,
        Err(r#"at index 0 of provides, got element of type string, want Provider"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'provides' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, provides=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"at index 0 of provides, got element of type Attribute, want Provider"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'struct', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'Provider', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, dependency_resolution_rule=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'dependency_resolution_rule' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=[]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=["a"]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=[1]))
def X(): pass
X()"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=("a",)))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_compatible_with' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_compatible_with=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type Attribute, want string"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'analysis_test' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'analysis_test' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'analysis_test' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'struct', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'Provider', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, analysis_test=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'analysis_test' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'bool', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'bool', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'int', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'int', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'int', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'string', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'string', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'list', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'list', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'list', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'dict', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'dict', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'tuple', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'function', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'depset', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'struct', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'Attribute', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'Provider', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, build_setting=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'build_setting' got value of type 'list', want 'BuildSetting or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=True))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=False))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=1))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=-1))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=0))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg="s"))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=""))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=[]))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=["a"]))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=[1]))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg={}))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=("a",)))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=struct()))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=provider()))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, cfg=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=True))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'bool', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=False))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'bool', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'int', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'int', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'int', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'string', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'string', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'list', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'list', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'list', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups={}))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"got dict<string, string> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'tuple', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'function', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'depset', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'struct', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'Attribute', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'Provider', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, exec_groups=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'exec_groups' got value of type 'list', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, initializer=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, initializer=lambda: 1))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=True))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was bool"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=False))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was bool"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=1))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was int"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=-1))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was int"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=0))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was int"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent="s"))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was string"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=""))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was string"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=[]))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was list"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=["a"]))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was list"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=[1]))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was list"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent={}))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was dict"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was dict"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=("a",)))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was tuple"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=lambda: 1))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was function"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=depset(["a"])))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was depset"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=struct()))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was struct"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=attr.string()))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was Attribute"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=provider()))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was Provider"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, parent=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"Parent needs to be a Starlark rule, was list"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=None))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=True))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=False))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'int', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=-1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'int', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=0))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'int', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable="s"))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=""))
def X(): pass
X()"#,
        Err(r#"Unable to parse label '': invalid target name '': empty target name"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'list', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=["a"]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'list', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=[1]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'list', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable={}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'dict', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable={"a":"b"}))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'dict', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=("a",)))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'tuple', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'function', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'depset', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'struct', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'Attribute', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'Provider', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, extendable=[attr.string()]))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'extendable' got value of type 'list', want 'bool, Label, string, or NoneType'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=None))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=True))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'bool', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=False))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'bool', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=-1))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=0))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules="s"))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=""))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=[]))
def X(): pass
X()"#,
        Ok(r#"<rule>"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=["a"]))
def X(): pass
X()"#,
        Err(r#"at index 0 of subrules, got element of type string, want Subrule"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=[1]))
def X(): pass
X()"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules={}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules={"a":"b"}))
def X(): pass
X()"#,
        Err(r#"in call to rule(), parameter 'subrules' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=("a",)))
def X(): pass
X()"#,
        Err(r#"at index 0 of subrules, got element of type string, want Subrule"#),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=lambda: 1))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=depset(["a"])))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=struct()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=attr.string()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=provider()))
def X(): pass
X()"#,
        Err(
            r#"in call to rule(), parameter 'subrules' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"print(rule(implementation=lambda ctx: None, subrules=[attr.string()]))
def X(): pass
X()"#,
        Err(r#"at index 0 of subrules, got element of type Attribute, want Subrule"#),
    ),
];

const BUILD_CASES: &[BuildRow] = &[
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True)"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "args": (), "output_licenses": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, test=True)"#,
        build: r#"r(name="a_test")
print(dict(existing_rule("a_test")))"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:9: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, test=True)"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:9: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, test=True, executable=True)"#,
        build: r#"r(name="a_test")
print(dict(existing_rule("a_test")))"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:9: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, test=False, executable=True)"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "args": (), "output_licenses": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, test=True, executable=False)"#,
        build: r#"r(name="a_test")
print(dict(existing_rule("a_test")))"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:9: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"s": attr.string(), "i": attr.int(), "b": attr.bool(), "sl": attr.string_list(), "il": attr.int_list(), "l": attr.label(), "ll": attr.label_list(), "sd": attr.string_dict(), "sld": attr.string_list_dict(), "lksd": attr.label_keyed_string_dict(), "skld": attr.string_keyed_label_dict(), "lld": attr.label_list_dict(), "o": attr.output(), "ol": attr.output_list()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "s": "", "i": 0, "b": False, "sl": (), "il": (), "ll": (), "sd": {}, "sld": {}, "lksd": {}, "skld": {}, "lld": {}, "ol": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"s": attr.string(default="d"), "i": attr.int(default=3), "b": attr.bool(default=True), "sl": attr.string_list(default=["x"]), "il": attr.int_list(default=[1]), "l": attr.label(default="//a:b"), "ll": attr.label_list(default=["//a:b", ":c"]), "sd": attr.string_dict(default={"k":"v"}), "sld": attr.string_list_dict(default={"k":["v"]}), "lksd": attr.label_keyed_string_dict(default={"//a:b":"v"}), "skld": attr.string_keyed_label_dict(default={"k":"//a:b"}), "lld": attr.label_list_dict(default={"k":["//a:b"]})})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "s": "d", "i": 3, "b": True, "sl": ("x",), "il": (1,), "l": "//a:b", "ll": ("//a:b", ":c"), "sd": {"k": "v"}, "sld": {"k": ("v",)}, "lksd": {"//a:b": "v"}, "skld": {"k": "//a:b"}, "lld": {"k": ("//a:b",)}}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"_p": attr.label(default="//a:b"), "_q": attr.string(default="x"), "_n": attr.label()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"z": attr.string(), "a": attr.string(), "m": attr.string()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "z", "a", "m"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"s": attr.string(default="d")})"#,
        build: r#"r(name="a", s="e")
print(dict(existing_rule("a")).get("s"))"#,
        printed: &[r#"e"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"s": attr.string(default="d")})"#,
        build: r#"r(name="a")
print(existing_rule("a")["kind"])
print(existing_rule("a")["name"])"#,
        printed: &[r#"r"#, r#"a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r_test", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "testonly": True, "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "size": "medium", "flaky": False, "shard_count": -1, "local": False, "args": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, executable=True)
r = r_test"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r_test", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "testonly": True, "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "size": "medium", "flaky": False, "shard_count": -1, "local": False, "args": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, executable=False)
r = r_test"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r_test", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "testonly": True, "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "size": "medium", "flaky": False, "shard_count": -1, "local": False, "args": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, executable=True)
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:14: Invalid rule class name 'r_test', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None)
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:14: Invalid rule class name 'r_test', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=False)
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"u.bzl:1:14: Invalid rule class name 'r_test', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, test=False)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, attrs={"args": attr.string()})
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `args`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, attrs={"size": attr.string()})
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `size`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, attrs={"flaky": attr.bool()})
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `flaky`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True, attrs={"args": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `args`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True, attrs={"output_licenses": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `output_licenses`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"args": attr.string()})"#,
        build: r#"r(name="a", args="x")
print(dict(existing_rule("a")).get("args"))"#,
        printed: &[r#"x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"expect_failure": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `expect_failure`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"toolchains": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `toolchains`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"exec_properties": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `exec_properties`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"generator_name": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `generator_name`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"features": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `features`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"compatible_with": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `compatible_with`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"restricted_to": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `restricted_to`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"aspect_hints": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `aspect_hints`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"transitive_configs": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `transitive_configs`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"exec_compatible_with": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute `exec_compatible_with`: built-in attributes cannot be overridden."#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"target_compatible_with": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute `target_compatible_with`: built-in attributes cannot be overridden."#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"package_metadata": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `package_metadata`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"applicable_licenses": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"distribs": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"licenses": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"exec_group_compatible_with": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute `exec_group_compatible_with`: built-in attributes cannot be overridden."#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"generator_function": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `generator_function`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"generator_location": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `generator_location`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"kind": attr.string()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).get("kind"))"#,
        printed: &[r#"r"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"name": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `name`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"data": attr.label_list()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).get("data"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"deprecation": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `deprecation`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"testonly": attr.bool()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `testonly`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {"a": "b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {"a": ["b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {"//a:b": "c"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {"a": "//a:b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {"a": ["//a:b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {"a": [1]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got {1: "a"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"1"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"2"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"-1"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {"a": "b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {"a": ["b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {"//a:b": "c"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {"a": "//a:b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {"a": ["//a:b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {"a": [1]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got {1: "a"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"s"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//a:b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":c"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//bad//x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": "b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": ["b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"//a:b": "c"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": "//a:b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": ["//a:b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": [1]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {1: "a"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(1,)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(1,)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(1, 2)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for element 0 of attribute 'x' of 'r', but got "a" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(int)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a", "a")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b", "//a:b")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":s"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '' in attribute 'x' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//a:b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":c"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '//bad//x' in attribute 'x' of 'r': invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": "b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": ["b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"//a:b": "c"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": "//a:b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": ["//a:b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": [1]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {1: "a"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a", ":a")"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:a' is duplicated in the 'x' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b", "//a:b")"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//a:b' is duplicated in the 'x' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": "b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"//a:b": "c"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": "//a:b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for dict value element, but got "b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("b",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for dict value element, but got "c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for dict value element, but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("//a:b",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(string))' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{":a": "b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"//a:b": "c"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{":a": "//a:b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(label, string)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ":b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"//a:b": ":c"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": "//a:b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, label)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for dict value element, but got "b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": (":b",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for dict value element, but got "c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for dict value element, but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("//a:b",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":s"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '' in attribute 'x' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[r#"BUILD.bazel:2:2: //:a: label '//a:b' is not in the current package"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":c"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '//bad//x' in attribute 'x' of 'r': invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [1] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got [True] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["a", "a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ["//a:b", "//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": "b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": ["b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"//a:b": "c"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": "//a:b"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": ["//a:b"]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {"a": [1]} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got {1: "a"} (dict)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got (1, 2) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=None)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=True)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=0)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=2)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=-1)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got -1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x="s")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got "s" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x="")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got "" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x="//a:b")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got "//a:b" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=":c")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got ":c" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x="//bad//x")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got "//bad//x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=[])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=["a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=["//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[r#"BUILD.bazel:2:2: //:a: label '//a:b' is not in the current package"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=[1])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=[True])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=["a","a"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"rule 'a' has more than one generated file named 'a'"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=["//a:b","//a:b"])
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[r#"BUILD.bazel:2:2: //:a: label '//a:b' is not in the current package"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={"a":"b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={"a":["b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={"//a:b":"c"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[r#"BUILD.bazel:2:2: //:a: label '//a:b' is not in the current package"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={"a":"//a:b"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={"a":["//a:b"]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={"a":[1]})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x={1:"a"})
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=(1,2))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=("a",))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=depset(["a"]))
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})"#,
        build: r#"r(name="a", x=1.5)
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(mandatory=True)})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'x' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(mandatory=True)})"#,
        build: r#"r(name="a", x=None)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'x' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(mandatory=True)})"#,
        build: r#"r(name="a", x="")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label(mandatory=True)})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'x' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label(mandatory=True), "y": attr.string(mandatory=True)})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'x' in 'r' rule"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'y' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list(allow_empty=False)})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list(allow_empty=False)})"#,
        build: r#"r(name="a", x=[])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list(allow_empty=False)})"#,
        build: r#"r(name="a", x=["a"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list(allow_empty=False)})"#,
        build: r#"r(name="a", x=[])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list(allow_empty=False)})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict(allow_empty=False)})"#,
        build: r#"r(name="a", x={})"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list(allow_empty=False)})"#,
        build: r#"r(name="a", x=[])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a", "b"])})"#,
        build: r#"r(name="a", x="c")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid value in 'x' attribute: has to be one of 'a' or 'b' instead of 'c'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a", "b"])})"#,
        build: r#"r(name="a", x="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a", "b"])})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a", "b"], default="a")})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string(values=["a", "b"], default="q")})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int(values=[1, 2])})"#,
        build: r#"r(name="a", x=3)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid value in 'x' attribute: has to be one of '1' or '2' instead of '3'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int(values=[1, 2])})"#,
        build: r#"r(name="a", x=2)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int(values=[1, 2])})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", y=1)"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'y' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"xyz": attr.string_list()})"#,
        build: r#"r(name="a", xzy=1)"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'xzy' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"xyz": attr.string_list()})"#,
        build: r#"r(name="a", xzy=1, qqq=2)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'xzy' in 'r' rule"#,
            r#"BUILD.bazel:2:2: //:a: no such attribute 'qqq' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"_x": attr.string(default="d")})"#,
        build: r#"r(name="a", _x="e")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute '_x' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"_x": attr.label(default="//a:b")})"#,
        build: r#"r(name="a", _x="//c:d")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute '_x' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"r rule has no 'name' attribute"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"r 'name' attribute must be a string"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name=None)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"r 'name' attribute must be a string"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Unexpected positional arguments"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r("a", name="b")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Unexpected positional arguments"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", name="b")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:13: duplicate keyword argument: name"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="../x")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"illegal rule name: ../x: invalid target name '../x': target names may not contain up-level references '..'"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="sub/y")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"r rule 'a' conflicts with existing r rule, defined at BUILD.bazel:2:2"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"filegroup(name="a")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"r rule 'a' conflicts with existing filegroup rule, defined at BUILD.bazel:2:10"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a")
filegroup(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a' conflicts with existing r rule, defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(r(name="a"))"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", visibility=["//visibility:public"], tags=["x"], testonly=True, deprecation="d", features=["f"], compatible_with=[], restricted_to=[], exec_properties={"k":"v"}, exec_compatible_with=[], target_compatible_with=[], toolchains=[], aspect_hints=[], transitive_configs=["t"], expect_failure="e", generator_name="g", generator_function="gf", generator_location="gl", exec_group_compatible_with={})
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "e", "visibility": ("//visibility:public",), "transitive_configs": (":t",), "deprecation": "d", "tags": ("x",), "generator_name": "g", "generator_function": "gf", "generator_location": "gl", "testonly": True, "features": ("f",), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {"k": "v"}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", package_metadata=[":a"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", applicable_licenses=[":a"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", licenses=["notice"])"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'licenses' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", distribs=["client"])"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'distribs' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", size="small")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'size' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", args=["x"])"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'args' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True)"#,
        build: r#"r(name="a", args=["x"], output_licenses=["notice"])
print(dict(existing_rule("a"))["args"], dict(existing_rule("a"))["output_licenses"])"#,
        printed: &[r#"("x",) ("notice",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="small", flaky=True, shard_count=3, local=True, args=["x"], timeout="short", env={"A": "b"}, env_inherit=["X"])
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r_test", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "testonly": True, "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "size": "small", "timeout": "short", "flaky": True, "shard_count": 3, "local": True, "args": ("x",)}"#,
        ],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'env' in 'r_test' rule"#,
            r#"BUILD.bazel:2:2: //:a: no such attribute 'env_inherit' in 'r_test' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="huge")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule 'a', size 'huge' is not a valid size."#,
            r#"BUILD.bazel:2:2: In rule 'a', timeout 'illegal' is not a valid timeout."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", timeout="x")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: In rule 'a', timeout 'x' is not a valid timeout."#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", shard_count=100)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", shard_count=-2)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", flaky="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'flaky' of 'r_test', but got "x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", env=1)"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'env' in 'r_test' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", testonly=False)
print(dict(existing_rule("a")).get("testonly"))"#,
        printed: &[r#"False"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, attrs={"timeout": attr.string(), "env": attr.string(), "env_inherit": attr.string()})
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute `timeout`: built-in attributes cannot be overridden."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", package_metadata=[":a"], aspect_hints=[":h"])
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "package_metadata", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True, attrs={"z": attr.string(), "a": attr.string()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "args", "output_licenses", "z", "a"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True, attrs={"z": attr.string(), "a": attr.string()})
r = r_test"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "testonly", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "size", "flaky", "shard_count", "local", "args", "z", "a"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt", "p": "%{name}_p"})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt"})"#,
        build: r#"r(name="a")
r(name="b")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "out.txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "out.txt"})"#,
        build: r#"r(name="a")
r(name="b")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"generated file 'out.txt' in rule 'b' conflicts with existing generated file from rule 'a', defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt"})"#,
        build: r#"r(name="a")
filegroup(name="a.txt")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a.txt' conflicts with existing generated file from rule 'a', defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt"})"#,
        build: r#"filegroup(name="a.txt")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"generated file 'a.txt' in rule 'a' conflicts with existing filegroup rule, defined at BUILD.bazel:2:10"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{nme}.txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: For attribute 'o' in outputs: Invalid placeholder(s) in template"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}/x"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt", "p": "%{name}.txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"rule 'a' has more than one generated file named 'a.txt'"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": ""})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: illegal output file name '' in rule //:a due to: invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "//x:y"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: illegal output file name '//x:y' in rule //:a due to: invalid target name '//x:y': target names may not start with '/'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": 1})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"got dict<string, int> for 'implicit outputs of the rule class', want dict<string, string>"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={1: "a"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"got dict<int, string> for 'implicit outputs of the rule class', want dict<string, string>"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="q")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x="q")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.label(allow_single_file=True)})"#,
        build: r#"r(name="a", x="q.c")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.string_list()})"#,
        build: r#"r(name="a", x=["q"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs=lambda name: {"o": name + ".txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs=lambda name, x: {"o": name + x}, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="q")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs=lambda: {"o": "a"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs=lambda name: 1)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: got int for 'implicit outputs function return value', want dict"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs=lambda name: {"o": 1})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: got dict<string, int> for 'implicit outputs function return value', want dict<string, string>"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a", o="out.txt")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a", o="out.txt")
r(name="b", o="out.txt")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"generated file 'out.txt' in rule 'b' conflicts with existing generated file from rule 'a', defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a", o="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a", o="sub/x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output_list()})"#,
        build: r#"r(name="a", o=["x", "y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output_list()})"#,
        build: r#"r(name="a", o=["x", "x"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"rule 'a' has more than one generated file named 'x'"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a", o="f1.txt")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"o": attr.output()})"#,
        build: r#"r(name="a", o=":z")
print(dict(existing_rule("a"))["o"])"#,
        printed: &[r#":z"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})
def r(name): _r(name=name, x=Label("//a:b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got Label("//a:b") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})
def r(name): _r(name=name, x=range(2))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got range(0, 2) (range)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})
def r(name): _r(name=name, x=set(["a"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got set(["a"]) (set)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})
def r(name): _r(name=name, x=1.5)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got 1.5 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})
def r(name): _r(name=name, x=[])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#""#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})
def r(name): _r(name=name, x=1<<20)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"1048576"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})
def r(name): _r(name=name, x=-(1<<30))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"-1073741824"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})
def r(name): _r(name=name, x=2147483647)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"2147483647"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})
def r(name): _r(name=name, x=True)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of '_r', but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})
def r(name): _r(name=name, x=1.0)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of '_r', but got 1.0 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})
def r(name): _r(name=name, x=Label("//a:b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"0"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'x' of '_r', but got Label("//a:b") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=Label("//a:b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//a:b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=Label("@dep//a:b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"@@[unknown repo 'dep' requested from @@]//a:b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=Label("@@other+//a:b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"@@other+//a:b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=["//a:b"])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=select({"//conditions:default": "//a:b"}))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//a:b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=depset(["a"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got depset(["a"]) (depset)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=struct(a=1))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got struct(a = 1) (struct)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="@mydep//x:y")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"@@[unknown repo 'mydep' requested from @@]//x:y"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="@nope//x:y")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"@@[unknown repo 'nope' requested from @@]//x:y"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="@@other+//x:y")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"@@other+//x:y"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="//a")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//a:a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="a:b")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label 'a:b' in attribute 'x' of '_r': invalid label 'a:b': absolute label must begin with '@' or '//'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="a/b")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":a/b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="//a:b/c")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//a:b/c"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="//:x")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="@//:x")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="//sub:x")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"//sub:x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x="sub:x")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label 'sub:x' in attribute 'x' of '_r': invalid label 'sub:x': absolute label must begin with '@' or '//'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})
def r(name): _r(name=name, x=":sub/x")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":sub/x"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=[Label("//a:b")])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=[Label("//a:b"), "//a:b"])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b", "//a:b")"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//a:b' is duplicated in the 'x' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=["//a:b", "@mydep//x:y"])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b", "@@[unknown repo 'mydep' requested from @@]//x:y")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=range(2))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=set(["a"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x="abc")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'x' of '_r', but got "abc" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=[None])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=[[]])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=[["a"]])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got ["a"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=("a", "b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a", ":b")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=["a", None])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 1 of attribute 'x' of '_r', but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x={"a": 1, "b": 2})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a", ":b")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=depset(["a", "b"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a", ":b")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=select({"//conditions:default": ["//a:b"]}))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=["//a:b"] + select({"//conditions:default": ["//c:d"]}))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("//a:b", "//c:d")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})
def r(name): _r(name=name, x=[":a", "a"])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":a", ":a")"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:a' is duplicated in the 'x' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=range(2))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=set(["a"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x="abc")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'x' of '_r', but got "abc" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=[None])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=[[]])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got [] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=(("a",),))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got ("a",) (tuple)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=[Label("//a:b")])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'x' of '_r', but got Label("//a:b") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=depset(["a", "b"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a", "b")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list()})
def r(name): _r(name=name, x=depset(["a"], order="topological"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"("a",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=range(3))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(0, 1, 2)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=set([1]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(1,)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=(1, 2))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(1, 2)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=depset([1, 2]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(1, 2)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=[1 << 31])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: for element 0 of attribute 'x' of '_r', got 2147483648, want value in signed 32-bit range"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=[2147483647])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(2147483647,)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})
def r(name): _r(name=name, x=[-2147483648])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(-2147483648,)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x={"a": Label("//a:b")})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got Label("//a:b") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x={Label("//a:b"): "a"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got Label("//a:b") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x={"a": None})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x={None: "a"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x=struct())"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of '_r', but got struct() (struct)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x=[("a", "b")])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'x' of '_r', but got [("a", "b")] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_dict()})
def r(name): _r(name=name, x={"b": "1", "a": "2"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"b": "1", "a": "2"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})
def r(name): _r(name=name, x={Label("//a:b"): "a"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"//a:b": "a"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})
def r(name): _r(name=name, x={"//a:b": "x", Label("//a:b"): "y"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: duplicate labels in attribute 'x' of '_r': //a:b (as ["//a:b", Label("//a:b")])"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})
def r(name): _r(name=name, x={"a": "x", ":a": "y"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: duplicate labels in attribute 'x' of '_r': //:a (as ["a", ":a"])"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})
def r(name): _r(name=name, x={"//a:b": Label("//c:d")})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got Label("//c:d") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})
def r(name): _r(name=name, x={"b": "1", "a": "2"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{":b": "1", ":a": "2"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})
def r(name): _r(name=name, x={"a": Label("//a:b")})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": "//a:b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})
def r(name): _r(name=name, x={"a": "//a:b", "b": "//a:b"})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": "//a:b", "b": "//a:b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})
def r(name): _r(name=name, x={"a": None})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})
def r(name): _r(name=name, x={"a": ["//a:b"]})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got ["//a:b"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})
def r(name): _r(name=name, x={"a": [Label("//a:b")]})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("//a:b",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})
def r(name): _r(name=name, x={"a": ["//a:b", "//a:b"]})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("//a:b", "//a:b")}"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//a:b' is duplicated in the 'x' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})
def r(name): _r(name=name, x={"a": depset(["//a:b"])})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("//a:b",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})
def r(name): _r(name=name, x={"a": ("x",)})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": (":x",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})
def r(name): _r(name=name, x={"a": None})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for dict value element, but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})
def r(name): _r(name=name, x={"a": depset(["x"])})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("x",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})
def r(name): _r(name=name, x={"a": ("x",)})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ("x",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})
def r(name): _r(name=name, x={"a": None})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for dict value element, but got None (NoneType)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})
def r(name): _r(name=name, x={"a": []})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"a": ()}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_list_dict()})
def r(name): _r(name=name, x={"b": ["1"], "a": ["2"]})"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"{"b": ("1",), "a": ("2",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})
def r(name): _r(name=name, x=Label("//a:b"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of '_r', but got Label("//a:b") (Label)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})
def r(name): _r(name=name, x=True)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})
def r(name): _r(name=name, x=1.0)"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"False"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'x' of '_r', but got 1.0 (float)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.bool()})
def r(name): _r(name=name, x=select({"//conditions:default": True}))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x=Label("//:z"))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":z"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x=["z"])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'x' of '_r', but got ["z"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x="z")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":z"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x="a")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x="//:z")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":z"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x="@//:z")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#":z"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output()})
def r(name): _r(name=name, x="@mydep//x:y")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"<unset>"#],
        events: &[r#"BUILD.bazel:2:2: //:a: label '@mydep//x:y' is not in the current package"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})
def r(name): _r(name=name, x=[Label("//:z")])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":z",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})
def r(name): _r(name=name, x="z")"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(output)' for attribute 'x' of '_r', but got "z" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})
def r(name): _r(name=name, x=depset(["z"]))"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":z",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_r = rule(implementation=lambda ctx: None, attrs={"x": attr.output_list()})
def r(name): _r(name=name, x=["z", "a"])"#,
        build: r#"r("a")
print(dict(existing_rule("a")).get("x", "<unset>"))"#,
        printed: &[r#"(":z", ":a")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(): return rule(implementation=lambda ctx: None)
r = f()"#,
        build: r#"r(name="a")
print(existing_rule("a")["kind"])"#,
        printed: &[r#"r"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(): return rule(implementation=lambda ctx: None)
_x = f()
def g(name): _x(name=name)
def r(name): g(name)"#,
        build: r#"r("a")
print(existing_rule("a")["kind"])"#,
        printed: &[r#"_x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"_q = rule(implementation=lambda ctx: None)
def r(name): _q(name=name)"#,
        build: r#"r("a")
print(existing_rule("a")["kind"])"#,
        printed: &[r#"_q"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"q = rule(implementation=lambda ctx: None)
r = q"#,
        build: r#"r(name="a")
print(existing_rule("a")["kind"])"#,
        printed: &[r#"q"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"d = {"r": rule(implementation=lambda ctx: None)}
r = d["r"]"#,
        build: r#"r(name="a")
print(existing_rule("a")["kind"])"#,
        printed: &[r#"r"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"d = {"r": rule(implementation=lambda ctx: None)}
def r(name): d["r"](name=name)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Invalid rule class hasn't been exported by a bzl file"#),
    },
    BuildRow {
        bzl: r#"l = [rule(implementation=lambda ctx: None)]
def r(name): l[0](name=name)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Invalid rule class hasn't been exported by a bzl file"#),
    },
    BuildRow {
        bzl: r#"def r(name): rule(implementation=lambda ctx: None)(name=name)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"rule() can only be used during .bzl initialization (top-level evaluation)"#),
    },
    BuildRow {
        bzl: r#"s = struct(r=rule(implementation=lambda ctx: None))
def r(name): s.r(name=name)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Invalid rule class hasn't been exported by a bzl file"#),
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: None)
R(name="x")
r = R"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"a rule can only be instantiated while evaluating a BUILD file or a legacy or symbolic macro"#,
        ),
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: None)
def r(name): R(name=name)"#,
        build: r#"r("a")
r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"R rule 'a' conflicts with existing R rule, defined at BUILD.bazel:2:2"#),
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: None)
def r(name): R(name=name)
return 5"#,
        build: r#"print(r("a"))"#,
        printed: &[],
        events: &[r#"u.bzl:3:1: return statements must be inside a function"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(r)
print(type(r))
print(repr(r))
print(str(r))"#,
        printed: &[r#"<rule r>"#, r#"rule"#, r#"<rule r>"#, r#"<rule r>"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(dir(r))"#,
        printed: &[r#"[]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(r == r)"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(r.kind)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"'rule' value has no field or method 'kind'"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(r.__name__)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"'rule' value has no field or method '__name__'"#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print({r: 1})"#,
        printed: &[r#"{<rule r>: 1}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"print(bool(r))"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(**{"name": "a"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:3: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(*[], name="a")
r(name="b", **{})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:3: *args arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
            r#"BUILD.bazel:3:13: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", **{"tags": ["x"]})
print(dict(existing_rule("a"))["tags"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:13: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", **{"tags": ["x"], "tags2": 1})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:13: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", tags=["x"], **{"tags": ["y"]})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:25: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="huge", timeout="long")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: In rule 'a', size 'huge' is not a valid size."#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="small", timeout="illegal")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: In rule 'a', timeout 'illegal' is not a valid timeout."#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule 'a', size '' is not a valid size."#,
            r#"BUILD.bazel:2:2: In rule 'a', timeout 'illegal' is not a valid timeout."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size=None)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'size' of 'r_test', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="Small")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule 'a', size 'Small' is not a valid size."#,
            r#"BUILD.bazel:2:2: In rule 'a', timeout 'illegal' is not a valid timeout."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", size="enormous")
print(dict(existing_rule("a"))["size"])"#,
        printed: &[r#"enormous"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", timeout="eternal")
print(dict(existing_rule("a"))["timeout"])"#,
        printed: &[r#"eternal"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", timeout=None)
print(dict(existing_rule("a")).get("timeout"))"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", args="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'args' of 'r_test', but got "x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", shard_count="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'shard_count' of 'r_test', but got "x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", local=2)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'local' of 'r_test', but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a", testonly=False, tags=["t"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True)"#,
        build: r#"r(name="a", args=[1])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for element 0 of attribute 'args' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True)"#,
        build: r#"r(name="a", output_licenses="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'output_licenses' of 'r', but got "x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"deprecation2": attr.string()})"#,
        build: r#"r(name="a", deprecation=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'deprecation' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", testonly="x", tags="x", features=1, compatible_with=1, restricted_to="x", visibility=1, exec_properties=1, exec_compatible_with=1, target_compatible_with=1, toolchains=1, aspect_hints=1, transitive_configs=1, expect_failure=1, generator_name=1, exec_group_compatible_with=1, package_metadata=1, applicable_licenses=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'testonly' of 'r', but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'tags' of 'r', but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'features' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'compatible_with' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'restricted_to' of 'r', but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'visibility' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, string)' for attribute 'exec_properties' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'exec_compatible_with' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'target_compatible_with' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'toolchains' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'aspect_hints' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'transitive_configs' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'expect_failure' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'generator_name' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'dict(string, list(label))' for attribute 'exec_group_compatible_with' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for attribute 'package_metadata' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_properties={"a": 1})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict value element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_properties={1: "a"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for dict key element, but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_group_compatible_with={"a": ["//x:y"]})
print(dict(existing_rule("a"))["exec_group_compatible_with"])"#,
        printed: &[r#"{"a": ("//x:y",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_group_compatible_with={"a": "x"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(label)' for dict value element, but got "x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", toolchains=["//x:y"])
print(dict(existing_rule("a"))["toolchains"])"#,
        printed: &[r#"("//x:y",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", transitive_configs=["//x:y", "z"])
print(dict(existing_rule("a"))["transitive_configs"])"#,
        printed: &[r#"(":z", "//x:y")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", aspect_hints=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//x:y' is duplicated in the 'aspect_hints' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", toolchains=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//x:y' is duplicated in the 'toolchains' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_compatible_with=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//x:y' is duplicated in the 'exec_compatible_with' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", package_metadata=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//x:y' is duplicated in the 'package_metadata' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", restricted_to=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//x:y' is duplicated in the 'restricted_to' attribute of rule 'a'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", visibility=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", transitive_configs=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs={"x":1})
print(dict(existing_rule("a"))["srcs"])"#,
        printed: &[r#"(":x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=depset(["x", "y"]))
print(dict(existing_rule("a"))["srcs"])"#,
        printed: &[r#"(":x", ":y")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=set(["x"]))
print(dict(existing_rule("a"))["srcs"])"#,
        printed: &[r#"(":x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=range(2))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:10: //:a: expected value of type 'string' for element 0 of attribute 'srcs' of 'filegroup', but got 0 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", tags=set(["x"]))
print(dict(existing_rule("a"))["tags"])"#,
        printed: &[r#"("x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=[":sub/x"])
print(dict(existing_rule("a"))["srcs"])"#,
        printed: &[r#"(":sub/x",)"#],
        events: &[
            r#"BUILD.bazel:2:10: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=["//:sub/x"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:10: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", data=["sub/x"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:10: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", transitive_configs=["t"])
print(dict(existing_rule("a"))["transitive_configs"])"#,
        printed: &[r#"(":t",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", transitive_configs=["//x:y", "//x:y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"alias(name="a", actual=":sub/x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:6: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", visibility=[":sub/x"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=[":sub/x", ":sub/x"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:10: Label '//:sub/x' is duplicated in the 'srcs' attribute of rule 'a'"#,
            r#"BUILD.bazel:2:10: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = 1"#,
        build: r#"filegroup(name="a", srcs=["sub:x"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:10: //:a: invalid label 'sub:x' in element 0 of attribute 'srcs' of 'filegroup': invalid label 'sub:x': absolute label must begin with '@' or '//'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt", "p": "%{name}_p"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x="//p:q.c")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=["q"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=3)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: For attribute 'x' in outputs: Attributes of type int cannot be used in an outputs substitution template"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.bool()})"#,
        build: r#"r(name="a", x=True)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: For attribute 'x' in outputs: Attributes of type boolean cannot be used in an outputs substitution template"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{x}.txt"}, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x="//p:q")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: illegal output file name '//p:q.txt' in rule //:a due to: invalid target name '//p:q.txt': target names may not start with '/'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.%{x}"}, attrs={"x": attr.string(default="d")})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{visibility}"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: For attribute 'visibility' in outputs: Attributes of type list(label) cannot be used in an outputs substitution template"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{tags}.txt"})"#,
        build: r#"r(name="a", tags=["z"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "x%"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{}.txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: In rule //:a: For attribute 'o' in outputs: Invalid placeholder(s) in template"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "a b.txt"})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt"})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, outputs={"o": "%{name}.txt"}, executable=True)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, executable=True)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r_test = rule(implementation=lambda ctx: None, test=True)
r = r_test"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"srcss": attr.label_list()})"#,
        build: r#"r(name="a", srcs=[])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'srcs' in 'r' rule (did you mean 'srcss'?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"deps": attr.label_list()})"#,
        build: r#"r(name="a", dep=[])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'dep' in 'r' rule (did you mean 'deps'?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"deps": attr.label_list(), "dep": attr.label_list()})"#,
        build: r#"r(name="a", depz=[])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'depz' in 'r' rule (did you mean 'deps'?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", tag=[])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'tag' in 'r' rule (did you mean 'tags'?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", visibilty=[])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'visibilty' in 'r' rule (did you mean 'visibility'?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", nme="b")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: no such attribute 'nme' in 'r' rule (did you mean 'name'?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list()})"#,
        build: r#"r(name="a", x=[":sub/y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label()})"#,
        build: r#"r(name="a", x=":sub/y")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={":sub/y": "v"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"v": ":sub/y"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"v": [":sub/y"]})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string()})"#,
        build: r#"r(name="a", x=":sub/y")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", transitive_configs=[":sub/y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_compatible_with=[":sub/y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", toolchains=[":sub/y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", compatible_with=[":sub/y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", visibility=[":sub/y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", aspect_hints=[":sub/y"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", exec_group_compatible_with={"g": [":sub/y"]})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/y' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:y'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"//bad//x": "v"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '//bad//x' in dict key element: invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.string_keyed_label_dict()})"#,
        build: r#"r(name="a", x={"v": "//bad//x"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '//bad//x' in dict value element: invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"v": ["//bad//x"]})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '//bad//x' in element 0 of dict value element: invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list_dict()})"#,
        build: r#"r(name="a", x={"v": ["//ok:x", "//bad//x"]})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '//bad//x' in element 1 of dict value element: invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"": "v"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: invalid label '' in dict key element: invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=2147483648)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: for attribute 'x' of 'r', got 2147483648, want value in signed 32-bit range"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=-2147483649)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: for attribute 'x' of 'r', got -2147483649, want value in signed 32-bit range"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=-2147483648)
print(dict(existing_rule("a"))["x"])"#,
        printed: &[r#"-2147483648"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int_list()})"#,
        build: r#"r(name="a", x=[0, 2147483648])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: for element 1 of attribute 'x' of 'r', got 2147483648, want value in signed 32-bit range"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.int()})"#,
        build: r#"r(name="a", x=99999999999999999999)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: for attribute 'x' of 'r', got 99999999999999999999, want value in signed 32-bit range"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", applicable_licenses=[":q"])
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "package_metadata", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", package_metadata=[":q"])
print(dict(existing_rule("a")).get("package_metadata"))"#,
        printed: &[r#"(":q",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", package_metadata=None)
print(dict(existing_rule("a")).get("package_metadata"))"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", deprecation=None, testonly=None, tags=None, visibility=None)
print(dict(existing_rule("a")).get("deprecation", "-"), dict(existing_rule("a")).get("testonly", "-"))"#,
        printed: &[r#"- -"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None)"#,
        build: r#"r(name="a", visibility=None)
print(dict(existing_rule("a"))["visibility"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, provides=[provider()])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Providers should be top-level values in extension files that define them."#),
    },
    BuildRow {
        bzl: r#"P = provider()
r = rule(implementation=lambda ctx: None, provides=[P])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, provides=[DefaultInfo])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, attrs={"x": attr.label_list(providers=[provider()])})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Providers should be top-level values in extension files that define them."#),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, toolchains=["//x:y", "bad label"])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, toolchains=["//x:y", Label("//a:b")])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, toolchains=["//bad//x"])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Unable to parse toolchain_type label '//bad//x': invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, exec_compatible_with=["//bad//x"])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Unable to parse label '//bad//x' in attribute 'exec_compatible_with': invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant ":x"?)"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, toolchains=[""])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Unable to parse toolchain_type label '': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, fragments=["cpp", "cpp"])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r = rule(implementation=lambda ctx: None, doc="d", fragments=["cpp"], host_fragments=["cpp"], provides=[], exec_compatible_with=[], toolchains=[], analysis_test=False, dependency_resolution_rule=False, _skylark_testable=True, output_to_genfiles=True, subrules=[])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): provider()"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): attr.string()"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): attr.label_list()"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = Label("//a:b")"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = depset([])"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = struct()"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = rule(implementation=lambda c: None)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"rule() can only be used during .bzl initialization (top-level evaluation)"#),
    },
    BuildRow {
        bzl: r#"def r(name): x = aspect(implementation=lambda t, c: None)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"aspect() can only be used during .bzl initialization (top-level evaluation)"#,
        ),
    },
    BuildRow {
        bzl: r#"def r(name): x = select({"//conditions:default": 1})"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = transition(implementation=lambda s, a: {}, inputs=[], outputs=[])"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = configuration_field("cpp", "x")"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"configuration_field() can only be used during .bzl initialization (top-level evaluation)"#,
        ),
    },
    BuildRow {
        bzl: r#"def r(name): x = exec_group()"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = subrule(implementation=lambda: None)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = macro(implementation=lambda name: None)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"macro() can only be used during .bzl initialization (top-level evaluation)"#,
        ),
    },
    BuildRow {
        bzl: r#"def r(name): x = visibility(["//visibility:public"])"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"visibility() can only be used during .bzl initialization (top-level evaluation)"#,
        ),
    },
    BuildRow {
        bzl: r#"def r(name): x = analysis_test_transition(settings={})"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r(name): x = config.string()"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
];
