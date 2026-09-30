//! `repository_rule()`, `module_extension()` and `tag_class()` against Bazel
//! 9.2.0 (buildfiji-mum.8.1): the tables in `ext_matrix` are what Bazel
//! printed and reported for the same code, and the tests here are what a
//! table cannot say.

use crate::ext_matrix::{EXT_BUILD_CASES, EXT_CASES};
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
    "dictionary expression has duplicate key",
    "is not iterable",
    "cannot encode",
    "not callable",
    "no native function or rule",
    "not found in view",
    "cannot set .",
];

/// Probes the crate cannot answer the way Bazel does.
const SKIPPED: &[&str] = &[
    // Bazel refuses `*args` and `**kwargs` at any call in a BUILD file, as
    // an event; the crate does not (buildfiji-v32).
    "r(*[1])", "r(**{",
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
fn extension_declarations_replay_bazel() {
    assert_replays(replay(EXT_CASES, SKIPPED, GENERIC));
}

#[test]
fn extension_declarations_in_a_build_replay_bazel() {
    assert_replays(replay_build(EXT_BUILD_CASES, SKIPPED, GENERIC));
}

use crate::ext::{module_extension_arg, repository_rule_arg, tag_class_arg};
use crate::test_support::{module_in, run_files};

const DECLARATIONS: &str = r#"
def _impl(ctx): pass
x = repository_rule(_impl, attrs = {"url": attr.string()}, local = True, environ = ["HOME"], doc = "d")
_y = repository_rule(_impl)
z = module_extension(_impl, tag_classes = {"t": tag_class(attrs = {"a": attr.string()}, doc = "t")}, environ = ["E"], os_dependent = True)
t = tag_class(attrs = {"a": attr.string()}, doc = "t")
u = tag_class(attrs = {"a": attr.string()}, doc = "t")
s = struct(r = repository_rule(_impl))
"#;

#[test]
fn declarations_keep_their_file_and_name_once_frozen_and_loaded() {
    let printed = run_files(
        &[("a.bzl", DECLARATIONS)],
        "load(':a.bzl', 'x', 'z', 't', 'u', 's')\n\
         print(x)\nprint(z)\nprint(t)\nprint(s.r)\n\
         print(x == x, t == u, z == z, x == s.r)\n\
         print(type(x), type(z), type(t))\n",
    )
    .unwrap();
    assert_eq!(
        printed,
        [
            "<starlark repository rule @@//:a.bzl%x>",
            "<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>",
            "<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>",
            "<anonymous starlark repository rule>",
            "True True True False",
            "repository_rule ModuleExtension tag_class",
        ]
    );
}

#[test]
fn a_repository_rule_is_named_in_its_own_package() {
    let printed = crate::test_support::run_in(
        "",
        "pkg/sub",
        "def _f(ctx): pass\nx = repository_rule(_f)\nprint(x)\n",
    )
    .unwrap();
    assert_eq!(printed, ["<starlark repository rule @@//pkg/sub:t.bzl%x>"]);
}

#[test]
fn what_a_declaration_was_made_with_can_be_read_back() {
    let module = module_in("", "", DECLARATIONS).unwrap();
    let (x, z, t) = (
        module.get("x").unwrap(),
        module.get("z").unwrap(),
        module.get("t").unwrap(),
    );
    let (x, z, t) = (x.value(), z.value(), t.value());
    assert_eq!(
        repository_rule_arg(x, "environ").unwrap().to_string(),
        "[\"HOME\"]"
    );
    assert_eq!(repository_rule_arg(x, "local").unwrap().to_string(), "True");
    assert!(repository_rule_arg(x, "configure").is_none());
    assert_eq!(repository_rule_arg(x, "doc").unwrap().to_string(), "\"d\"");
    assert_eq!(
        repository_rule_arg(x, "implementation").unwrap().get_type(),
        "function"
    );
    assert_eq!(
        module_extension_arg(z, "os_dependent").unwrap().to_string(),
        "True"
    );
    assert!(module_extension_arg(z, "arch_dependent").is_none());
    assert_eq!(
        module_extension_arg(z, "tag_classes").unwrap().get_type(),
        "dict"
    );
    assert_eq!(tag_class_arg(t, "doc").unwrap().to_string(), "\"t\"");
    assert!(repository_rule_arg(t, "doc").is_none());
}

#[test]
fn tag_classes_equal_by_what_they_declare_not_by_identity() {
    let printed = crate::test_support::run(
        "a = tag_class(attrs = {'x': attr.string()})\n\
         print(a == tag_class(attrs = {'x': attr.string()}))\n\
         print(a == tag_class(attrs = {'x': attr.string(), 'y': attr.int()}))\n\
         print(tag_class() == tag_class(attrs = {}))\n\
         print(tag_class() == tag_class(doc = None))\n\
         print(tag_class(doc = 'd') == tag_class(doc = 'e'))\n\
         print(a == 1)\n",
    )
    .unwrap();
    assert_eq!(printed, ["True", "False", "True", "True", "False", "False"]);
}

#[test]
fn only_a_repository_rule_hashes() {
    let run = |src: &str| crate::test_support::run(src);
    assert_eq!(
        run("def f(ctx): pass\nprint({repository_rule(f): 1}.values())").unwrap(),
        ["[1]"]
    );
    for ctor in ["module_extension(f)", "tag_class()"] {
        let err = run(&format!("def f(ctx): pass\nprint({{{ctor}: 1}})")).unwrap_err();
        assert!(err.contains("unhashable type"), "{err}");
    }
}
