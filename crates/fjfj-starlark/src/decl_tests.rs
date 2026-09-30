//! The declaration builtins against Bazel 9.2.0 (buildfiji-mum.3.7): the
//! tables in `decl_matrix` are what Bazel printed and reported for the same
//! code, and the tests here are what a table cannot say.

use crate::decl_matrix::{DECL_BUILD_CASES, DECL_CASES};
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
    "cannot set .",
];

/// Probes the crate cannot answer the way Bazel does.
const SKIPPED: &[&str] = &[
    // The crate calls a builtin a `function`, and accepts one where Bazel
    // wants a Starlark function (buildfiji-v32).
    "=len",
    "=config.bool)",
    // A builtin is a `function` to the crate, prints by its name, and a bound
    // method by that name too (buildfiji-v32).
    "print(config.exec)",
    "print(config.target)",
    "type(config.target)",
    ".and_then)",
    "type(config.exec().and_then)",
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
fn declarations_replay_bazel() {
    assert_replays(replay(DECL_CASES, SKIPPED, GENERIC));
}

#[test]
fn declarations_in_rules_replay_bazel() {
    assert_replays(replay_build(DECL_BUILD_CASES, SKIPPED, GENERIC));
}

use crate::decl::{aspect_arg, subrule_arg};
use crate::test_support::{module_in, run_files};

const DECLARATIONS: &str = r#"
def _impl(target, ctx): return []
def _set(settings, attr): return {}
P = provider()
A = aspect(_impl, attr_aspects = ["deps"], provides = [P], fragments = ["cpp"], doc = "d")
B = aspect(_impl, requires = [A], attrs = {"mode": attr.string(values = ["a", "b"])})
T = transition(implementation = _set, inputs = ["//command_line_option:cpu"], outputs = ["//command_line_option:cpu"])
U = T.and_then(T)
E = exec_group(exec_compatible_with = ["//os:linux"], toolchains = ["//tc:t"])
S = subrule(implementation = _impl, attrs = {"_t": attr.label(default = "//x:y")})
F = configuration_field("cpp", "zipper")
K = config.bool(flag = True)
"#;

#[test]
fn declarations_are_themselves_once_frozen_and_loaded() {
    let printed = run_files(
        &[("a.bzl", DECLARATIONS)],
        "load(':a.bzl', 'A', 'B', 'T', 'U', 'E', 'S', 'F', 'K')\n\
         print(A, A == A, A == B, S, T == T, U == U, U == T.and_then(T), E == E, F == F)\n\
         print(type(A), type(T), type(E), type(S), type(F), type(K))\n\
         print(rule(implementation = lambda ctx: [], cfg = T, subrules = [S], exec_groups = {'g': E}))",
    )
    .unwrap();
    assert_eq!(
        printed[0],
        "<aspect> True False <subrule S> True True False True True"
    );
    assert_eq!(
        printed[1],
        "Aspect transition exec_group Subrule LateBoundDefault BuildSetting"
    );
}

#[test]
fn what_an_aspect_was_made_with_is_readable_from_rust() {
    let module = module_in("", "", DECLARATIONS).unwrap();
    let a_owned = module.get("A").unwrap();
    let a = a_owned.value();
    assert_eq!(
        aspect_arg(a, "attr_aspects").unwrap().to_repr(),
        r#"["deps"]"#
    );
    assert_eq!(aspect_arg(a, "fragments").unwrap().to_repr(), r#"["cpp"]"#);
    assert_eq!(aspect_arg(a, "doc").unwrap().to_repr(), r#""d""#);
    assert!(aspect_arg(a, "requires").is_none());
    let b_owned = module.get("B").unwrap();
    let b = b_owned.value();
    assert!(aspect_arg(b, "requires").is_some() && aspect_arg(b, "attrs").is_some());
    let s_owned = module.get("S").unwrap();
    let s = s_owned.value();
    assert!(subrule_arg(s, "attrs").is_some() && subrule_arg(s, "toolchains").is_none());
}

#[test]
fn an_aspect_needs_a_name_to_be_used_and_only_while_a_bzl_runs() {
    use crate::test_support::run;
    let err = run("load(':a.bzl', 'x')").unwrap_err();
    assert!(
        err.contains("no loader") || err.contains("cannot load"),
        "{err}"
    );
    let err = run("def f(t, c): return []\nx = attr.label(aspects = [aspect(f)])").unwrap_err();
    assert!(err.contains("Aspects should be top-level values"), "{err}");
    assert_eq!(
        run("def f(t, c): return []\nA = aspect(f)\nx = attr.label(aspects = [A])\nprint(1)")
            .unwrap(),
        ["1"]
    );
    let out = crate::test_support::run_build("def f(t, c): return []\ndef r(): aspect(f)", "r()");
    // `r` is not loaded by the BUILD prefix as a rule, so this is the call of a
    // macro that makes an aspect.
    assert!(
        out.fatal
            .as_deref()
            .is_some_and(|f| f.contains("aspect() can only be used during .bzl initialization")),
        "{out:?}"
    );
}

#[test]
fn a_build_setting_adds_its_default_and_help() {
    let out = crate::test_support::run_build(
        "r = rule(implementation = lambda ctx: [], build_setting = config.string(flag = True), attrs = {'z': attr.int()})",
        "r(name = 'a', build_setting_default = 'x', z = 1)\nprint(existing_rule('a')['build_setting_default'], existing_rule('a')['help'])",
    );
    assert!(out.fatal.is_none() && out.events.is_empty(), "{out:?}");
    assert_eq!(out.printed, [r#"x "#]);
}
