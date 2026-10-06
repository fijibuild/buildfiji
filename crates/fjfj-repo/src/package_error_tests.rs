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
    // A BUILD file that does not parse: one event, at the token.
    Row {
        files: &[("s/BUILD", "x = 1 +\n")],
        events: &[
            "{ws}/s/BUILD:1:8: syntax error at 'newline': expected expression",
            "package contains errors: s: syntax error at 'newline': expected expression",
        ],
        error: "Error evaluating '//s:all': error loading package 's': Package 's' contains errors",
    },
    // A name that is not defined is an event at the name, not a traceback.
    Row {
        files: &[("t/BUILD", "x = 1\ny = z\n")],
        events: &[
            "{ws}/t/BUILD:2:5: name 'z' is not defined",
            "package contains errors: t: name 'z' is not defined",
        ],
        error: "Error evaluating '//t:all': error loading package 't': Package 't' contains errors",
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
    // A .bzl that does not parse: its syntax error, the recovery's event at
    // the start of the bracket that failed, and a compile failure.
    Row {
        files: &[
            ("p/BUILD", "load('//p:l.bzl', 'x')\n"),
            ("p/l.bzl", "y = 1\nx = (\n"),
        ],
        events: &[
            "{ws}/p/l.bzl:3:1: syntax error at 'newline': expected expression",
            "{ws}/p/l.bzl:2:5: contains syntax errors",
        ],
        error: "while parsing '//p:all': error loading package 'p': compilation of module 'p/l.bzl' failed",
    },
    // A .bzl with a name that is not defined is not run either.
    Row {
        files: &[
            ("n/BUILD", "load('//n:l.bzl', 'x')\n"),
            ("n/l.bzl", "def f():\n  return z\nx = 1\n"),
        ],
        events: &["{ws}/n/l.bzl:2:10: name 'z' is not defined"],
        error: "while parsing '//n:all': error loading package 'n': compilation of module 'n/l.bzl' failed",
    },
    // What a BUILD file printed before it failed is shown first, as a DEBUG
    // line, and what a .bzl it loads printed before that.
    Row {
        files: &[
            (
                "d/BUILD",
                "load('//d:l.bzl', 'x')\nprint('a')\ny = [1][3]\nprint('b')\n",
            ),
            ("d/l.bzl", "print('in bzl')\nx = 1\n"),
        ],
        events: &[
            "DEBUG: {ws}/d/l.bzl:1:6: in bzl",
            "DEBUG: {ws}/d/BUILD:2:6: a",
            "Traceback (most recent call last):\n\tFile \"{ws}/d/BUILD\", line 3, column 8, in <toplevel>\n\t\ty = [1][3]\nError: index out of range (index is 3, but sequence has 1 elements)",
            "package contains errors: d: Traceback (most recent call last):\n\tFile \"{ws}/d/BUILD\", line 3, column 8, in <toplevel>\n\t\ty = [1][3]\nError: index out of range (index is 3, but sequence has 1 elements)",
        ],
        error: "Error evaluating '//d:all': error loading package 'd': Package 'd' contains errors",
    },
    // A .bzl that prints and then fails.
    Row {
        files: &[
            ("f/BUILD", "load('//f:l.bzl', 'x')\n"),
            ("f/l.bzl", "print('before')\nfail('after')\n"),
        ],
        events: &[
            "DEBUG: {ws}/f/l.bzl:1:6: before",
            "Traceback (most recent call last):\n\tFile \"{ws}/f/l.bzl\", line 2, column 5, in <toplevel>\n\t\tfail('after')\nError in fail: after",
        ],
        error: "while parsing '//f:all': error loading package 'f': initialization of module 'f/l.bzl' failed",
    },
    // Every syntax error of a BUILD file is an event, and names are not
    // looked at once there is one.
    Row {
        files: &[("m/BUILD", "x = \"\\q\"\ny = \"\\z\"\nz = w\n")],
        events: &[
            "{ws}/m/BUILD:1:7: invalid escape sequence: \\q. Use '\\\\' to insert '\\'.",
            "{ws}/m/BUILD:2:7: invalid escape sequence: \\z. Use '\\\\' to insert '\\'.",
            "package contains errors: m: invalid escape sequence: \\q. Use '\\\\' to insert '\\'.",
        ],
        error: "Error evaluating '//m:all': error loading package 'm': Package 'm' contains errors",
    },
    Row {
        files: &[("o/BUILD", "x = [1 2]\ny = (3 4)\n")],
        events: &[
            "{ws}/o/BUILD:1:8: syntax error at '2': expected ',', 'for' or ']'",
            "{ws}/o/BUILD:2:8: syntax error at '4': expected )",
            "package contains errors: o: syntax error at '2': expected ',', 'for' or ']'",
        ],
        error: "Error evaluating '//o:all': error loading package 'o': Package 'o' contains errors",
    },
    // With none, every name that is not defined is, and a duplicate keyword
    // argument comes before the name of the function called.
    Row {
        files: &[("r/BUILD", "x = a\ny = b\nf(k=1, k=2)\n")],
        events: &[
            "{ws}/r/BUILD:1:5: name 'a' is not defined",
            "{ws}/r/BUILD:2:5: name 'b' is not defined",
            "{ws}/r/BUILD:3:8: duplicate keyword argument: k",
            "{ws}/r/BUILD:3:1: name 'f' is not defined",
            "package contains errors: r: name 'a' is not defined",
        ],
        error: "Error evaluating '//r:all': error loading package 'r': Package 'r' contains errors",
    },
    // A .bzl says the same, and 'contains syntax errors' after.
    Row {
        files: &[
            ("u/BUILD", "load('//u:l.bzl', 'x')\n"),
            ("u/l.bzl", "x = [1 2]\ny = 1 2\n"),
        ],
        events: &[
            "{ws}/u/l.bzl:1:8: syntax error at '2': expected ',', 'for' or ']'",
            "{ws}/u/l.bzl:2:7: syntax error at '2': expected newline",
            "{ws}/u/l.bzl:1:5: contains syntax errors",
        ],
        error: "while parsing '//u:all': error loading package 'u': compilation of module 'u/l.bzl' failed",
    },
    Row {
        files: &[
            ("v/BUILD", "load('//v:l.bzl', 'x')\n"),
            ("v/l.bzl", "x = a\ny = b\n"),
        ],
        events: &[
            "{ws}/v/l.bzl:1:5: name 'a' is not defined",
            "{ws}/v/l.bzl:2:5: name 'b' is not defined",
        ],
        error: "while parsing '//v:all': error loading package 'v': compilation of module 'v/l.bzl' failed",
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
