//! Which targets a pattern selects, against Bazel 9.2.0 (buildfiji-gwl.3):
//! each row is what `bazel build --nobuild` expanded the patterns to, or
//! what it said when it skipped one, in a workspace of local packages.

use crate::{Options, Repos};
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_loading::resolve;
use std::collections::BTreeMap;

const FILES: &[(&str, &str)] = &[
    (
        "BUILD.bazel",
        "filegroup(name = 'm', srcs = ['f.txt'])\nfilegroup(name = 'top', srcs = [])\n",
    ),
    ("f.txt", ""),
    (
        "a/BUILD.bazel",
        "filegroup(name = 'a', srcs = ['x.txt'])\nfilegroup(name = 'other', srcs = [])\n",
    ),
    ("a/x.txt", ""),
    (
        "a/b/BUILD.bazel",
        "filegroup(name = 'b', srcs = ['y.txt'])\n",
    ),
    ("a/b/y.txt", ""),
    (
        "c/BUILD.bazel",
        "filegroup(name = 'c', srcs = [])\n\
         filegroup(name = 'hidden', srcs = ['h.txt'], tags = ['manual'])\n\
         exports_files(['e.txt'])\n\
         package_group(name = 'pg', packages = ['//...'])\n",
    ),
    ("c/h.txt", ""),
    ("c/e.txt", ""),
    ("c/unref.txt", ""),
    ("nopkg/deep/f.txt", ""),
];

fn resolved(offset: &str, patterns: &[&str]) -> (Vec<String>, Vec<(String, String)>, String) {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    for (file, text) in FILES {
        let at = ws.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
        .unwrap()
        .module;
    let repos = Repos::new(
        Options {
            workspace_root: ws.clone(),
            output_base: dir.path().join("ob"),
            environ: BTreeMap::new(),
            downloader: None,
            repository_cache: None,
            distdirs: Vec::new(),
            registries: Vec::new(),
            facts: Vec::new(),
            repo_overrides: Vec::new(),
        },
        module,
    )
    .unwrap();
    let ctx = PatternContext { repo: "", offset };
    let parsed: Vec<TargetPattern> = patterns
        .iter()
        .map(|p| TargetPattern::parse(p, ctx, &mut |r| r.to_owned()).unwrap())
        .collect();
    let out = resolve(&parsed, &repos);
    let mut targets: Vec<String> = out.targets.iter().map(ToString::to_string).collect();
    targets.sort();
    let failures = out
        .failures
        .into_iter()
        .map(|f| (f.pattern, f.message))
        .collect();
    (targets, failures, ws.display().to_string())
}

#[test]
fn patterns_select_what_bazel_selects() {
    for (offset, patterns, want) in [
        ("", &["//a"][..], &["//a:a"][..]),
        ("", &["//a:a"], &["//a:a"]),
        ("", &["//a:all"], &["//a:a", "//a:other"]),
        (
            "",
            &["//a:*"],
            &["//a:BUILD.bazel", "//a:a", "//a:other", "//a:x.txt"],
        ),
        ("", &["//a:x.txt"], &["//a:x.txt"]),
        ("", &["//:f.txt"], &["//:f.txt"]),
        ("", &["//c:all"], &["//c:c"]),
        (
            "",
            &["//c:*"],
            &[
                "//c:BUILD.bazel",
                "//c:c",
                "//c:e.txt",
                "//c:h.txt",
                "//c:pg",
            ],
        ),
        // Named, a `manual` rule is selected.
        ("", &["//c:hidden"], &["//c:hidden"]),
        ("", &["//c/..."], &["//c:c"]),
        (
            "",
            &["//..."],
            &["//:m", "//:top", "//a/b:b", "//a:a", "//a:other", "//c:c"],
        ),
        ("", &["a/..."], &["//a/b:b", "//a:a", "//a:other"]),
        ("", &["a"], &["//a:a"]),
        ("", &["a/b"], &["//a/b:b"]),
        ("", &["a/b/y.txt"], &["//a/b:y.txt"]),
        ("", &["c/h.txt"], &["//c:h.txt"]),
        ("", &["c/e.txt"], &["//c:e.txt"]),
        // A negative pattern removes, wherever it is written.
        ("", &["//...", "-//a/..."], &["//:m", "//:top", "//c:c"]),
        ("", &["-//a:a", "//a:all"], &["//a:other"]),
        ("a", &["b"], &["//a/b:b"]),
        ("a", &[":a"], &["//a:a"]),
        ("a", &["x.txt"], &["//a:x.txt"]),
        ("a", &["..."], &["//a/b:b", "//a:a", "//a:other"]),
        ("a", &["a"], &["//a:a"]),
    ] {
        let (targets, failures, _) = resolved(offset, patterns);
        assert_eq!(failures, [], "in {offset:?}: {patterns:?}");
        assert_eq!(targets, want, "in {offset:?}: {patterns:?}");
    }
}

#[test]
fn a_pattern_that_selects_nothing_is_skipped_with_bazels_reason() {
    for (pattern, message) in [
        (
            "//a:nope",
            "no such target '//a:nope': target 'nope' not declared in package 'a' defined by {ws}/a/BUILD.bazel",
        ),
        (
            "//a:A",
            "no such target '//a:A': target 'A' not declared in package 'a' defined by {ws}/a/BUILD.bazel (did you mean a?)",
        ),
        (
            "a/nothere",
            "no such target '//a:nothere': target 'nothere' not declared in package 'a' defined by {ws}/a/BUILD.bazel (did you mean other?)",
        ),
        (
            "//a:b",
            "no such target '//a:b': target 'b' not declared in package 'a' defined by {ws}/a/BUILD.bazel; however, a source directory of this name exists.  (Perhaps add 'exports_files([\"b\"])' to a/BUILD.bazel, or define a filegroup?)",
        ),
        (
            "c/unref.txt",
            "no such target '//c:unref.txt': target 'unref.txt' not declared in package 'c' defined by {ws}/c/BUILD.bazel; however, a source file of this name exists.  (Perhaps add 'exports_files([\"unref.txt\"])' to c/BUILD.bazel?)",
        ),
        (
            "nopkg",
            "no such target '//:nopkg': target 'nopkg' not declared in package '' defined by {ws}/BUILD.bazel; however, a source directory of this name exists.  (Perhaps add 'exports_files([\"nopkg\"])' to BUILD.bazel, or define a filegroup?)",
        ),
        (
            "nopkg/deep/f.txt",
            "no such target '//:nopkg/deep/f.txt': target 'nopkg/deep/f.txt' not declared in package '' defined by {ws}/BUILD.bazel; however, a source file of this name exists.  (Perhaps add 'exports_files([\"nopkg/deep/f.txt\"])' to BUILD.bazel?)",
        ),
        (
            "//nonexistent",
            "no such package 'nonexistent': BUILD file not found in any of the following directories. Add a BUILD file to a directory to mark it as a package.\n - nonexistent",
        ),
        (
            "//nonexistent/...",
            "no targets found beneath 'nonexistent'",
        ),
        ("nopkg/...", "no targets found beneath 'nopkg'"),
    ] {
        let (targets, failures, ws) = resolved("", &["//a:a", pattern]);
        assert_eq!(targets, ["//a:a"], "{pattern}");
        assert_eq!(
            failures,
            [(pattern.to_owned(), message.replace("{ws}", &ws))],
            "{pattern}"
        );
    }
}
