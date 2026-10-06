//! What Bazel 9.2.0 says of a package that does not load (buildfiji-bj1): the
//! events it reports, then the error of the package, which `bazel build
//! //<package>:all` shows as `ERROR: Skipping '<pattern>': <error>`. Each row
//! is a workspace of one BUILD file (and `.bzl` files) with what Bazel
//! printed; `{ws}` is the workspace's directory.

use crate::{Options, Repos};
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_loading::resolve;
use std::collections::BTreeMap;

struct Row {
    files: &'static [(&'static str, &'static str)],
    /// The events, each shown after `ERROR: `.
    events: &'static [&'static str],
    error: &'static str,
}

const ROWS: &[Row] = &[
    // A load of a file that is not there: no event, the reason is the error.
    Row {
        files: &[("a/BUILD", "load('//a:missing.bzl', 'x')\n")],
        events: &[],
        error: "while parsing '//a:all': error loading package 'a': cannot load '//a:missing.bzl': no such file",
    },
    // An error in the BUILD file: the traceback, then the package says so.
    Row {
        files: &[("b/BUILD", "x = [1][3]\n")],
        events: &[
            "Traceback (most recent call last):\n\tFile \"{ws}/b/BUILD\", line 1, column 8, in <toplevel>\n\t\tx = [1][3]\nError: index out of range (index is 3, but sequence has 1 elements)",
            "package contains errors: b: Traceback (most recent call last):\n\tFile \"{ws}/b/BUILD\", line 1, column 8, in <toplevel>\n\t\tx = [1][3]\nError: index out of range (index is 3, but sequence has 1 elements)",
        ],
        error: "Error evaluating '//b:all': error loading package 'b': Package 'b' contains errors",
    },
    // A call that fails in a macro: every call is in the traceback.
    Row {
        files: &[
            ("c/BUILD", "load('//c:l.bzl', 'f')\nf()\n"),
            ("c/l.bzl", "def f():\n    fail('boom')\n"),
        ],
        events: &[
            "Traceback (most recent call last):\n\tFile \"{ws}/c/BUILD\", line 2, column 2, in <toplevel>\n\t\tf()\n\tFile \"{ws}/c/l.bzl\", line 2, column 9, in f\n\t\tfail('boom')\nError in fail: boom",
            "package contains errors: c: Traceback (most recent call last):\n\tFile \"{ws}/c/BUILD\", line 2, column 2, in <toplevel>\n\t\tf()\n\tFile \"{ws}/c/l.bzl\", line 2, column 9, in f\n\t\tfail('boom')\nError in fail: boom",
        ],
        error: "Error evaluating '//c:all': error loading package 'c': Package 'c' contains errors",
    },
    // A .bzl that fails when it is loaded: its traceback, and the load fails.
    Row {
        files: &[
            ("e/BUILD", "load('//e:l.bzl', 'x')\n"),
            ("e/l.bzl", "fail('top')\n"),
        ],
        events: &[
            "Traceback (most recent call last):\n\tFile \"{ws}/e/l.bzl\", line 1, column 5, in <toplevel>\n\t\tfail('top')\nError in fail: top",
        ],
        error: "while parsing '//e:all': error loading package 'e': initialization of module 'e/l.bzl' failed",
    },
    // A rule the BUILD file calls wrongly: an event at the call.
    Row {
        files: &[(
            "h/BUILD",
            "genrule(name = 't', outs = ['o'], cmd = 'x', bogus = 1)\n",
        )],
        events: &[
            "{ws}/h/BUILD:1:8: //h:t: no such attribute 'bogus' in 'genrule' rule",
            "package contains errors: h: //h:t: no such attribute 'bogus' in 'genrule' rule",
        ],
        error: "Error evaluating '//h:all': error loading package 'h': Package 'h' contains errors",
    },
];

#[test]
fn a_package_that_does_not_load_says_what_bazel_says() {
    for row in ROWS {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        for (file, text) in row.files {
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
        let package = row.files[0].0.split('/').next().unwrap();
        let text = format!("//{package}:all");
        let ctx = PatternContext {
            repo: "",
            offset: "",
        };
        let pattern = TargetPattern::parse(&text, ctx, &mut |r| r.to_owned()).unwrap();
        let out = resolve(&[pattern], &repos);
        let shown = |text: String| text.replace(&ws.display().to_string(), "{ws}");
        assert_eq!(
            out.failures
                .into_iter()
                .map(|f| shown(f.message))
                .collect::<Vec<_>>(),
            [row.error],
            "{text}"
        );
        let events: Vec<String> = repos.take_events().into_iter().map(shown).collect();
        assert_eq!(events, row.events, "{text}");
    }
}
