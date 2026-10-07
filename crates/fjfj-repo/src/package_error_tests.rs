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
    // The resolver runs on what the parser recovered of a .bzl: its names
    // are reported after the syntax errors, in a def after the bad one.
    Row {
        files: &[
            ("w/BUILD", "load('//w:l.bzl', 'x')\n"),
            (
                "w/l.bzl",
                "def a():\n  return 2 ** 3\ndef b():\n  return chr\n",
            ),
        ],
        events: &[
            "{ws}/w/l.bzl:2:12: syntax error at '**': expected newline",
            "{ws}/w/l.bzl:4:10: name 'chr' is not defined",
        ],
        error: "while parsing '//w:all': error loading package 'w': compilation of module 'w/l.bzl' failed",
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

/// The events of `text` against a package whose BUILD file fails, after the
/// traceback (buildfiji-wtzd).
fn events_of(text: &str) -> Vec<String> {
    events_in(text, "x = [1][3]\nfilegroup(name='g')\n", None)
}

/// The events of `text` against a workspace of one BUILD file and perhaps a
/// REPO.bazel.
fn events_in(text: &str, build: &str, repo_file: Option<&str>) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("BUILD"), build).unwrap();
    if let Some(repo_file) = repo_file {
        std::fs::write(ws.join("REPO.bazel"), repo_file).unwrap();
    }
    let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
        .unwrap()
        .module;
    let repos = Repos::new(
        Options {
            workspace_root: ws,
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
    let ctx = PatternContext {
        repo: "",
        offset: "",
    };
    let pattern = TargetPattern::parse(text, ctx, &mut |r| r.to_owned()).unwrap();
    resolve(&[pattern], &repos);
    repos
        .take_events()
        .into_iter()
        .map(|e| e.lines().next().unwrap().to_owned())
        .collect()
}

#[test]
fn what_a_repo_file_prints_is_an_event_of_every_command_that_reads_it() {
    let events = events_in("//:g", "filegroup(name='g')\n", Some("print('hi')\n"));
    assert_eq!(events.len(), 1);
    assert!(events[0].starts_with("DEBUG: ") && events[0].ends_with("REPO.bazel:1:6: hi"));
}

#[test]
fn one_target_of_a_package_with_errors_gets_no_package_contains_errors_event_and_a_tree_gets_an_empty_one_first()
 {
    assert_eq!(events_of("//:g").len(), 1);
    let all = events_of("//:all");
    assert_eq!(all.len(), 2);
    assert!(all[1].starts_with("package contains errors: : "));
    let tree = events_of("//...");
    assert_eq!(tree.len(), 3);
    assert_eq!(tree[1], "package contains errors: ");
    assert!(tree[2].starts_with("package contains errors: : "));
}

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

/// Probed on Bazel 9.2.0: a package whose `load` cannot be satisfied is named
/// by the pattern that asked for it only as a wildcard of the package does;
/// a label says nothing of it, and `...` says it was under a directory.
#[test]
fn the_pattern_that_found_a_package_that_does_not_load_is_named_only_as_bazel_names_it() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(ws.join("a")).unwrap();
    std::fs::write(ws.join("a/BUILD"), "load('//a:missing.bzl', 'x')\n").unwrap();
    let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
        .unwrap()
        .module;
    let repos = Repos::new(
        Options {
            workspace_root: ws,
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
    let reason = "error loading package 'a': cannot load '//a:missing.bzl': no such file";
    for (text, want) in [
        ("//a:t", reason.to_owned()),
        ("//a", reason.to_owned()),
        ("//a:t+", reason.to_owned()),
        ("//a:all", format!("while parsing '//a:all': {reason}")),
        ("//a:*", format!("while parsing '//a:*': {reason}")),
        (
            "//a/...",
            format!("error loading package under directory 'a': {reason}"),
        ),
    ] {
        let ctx = PatternContext {
            repo: "",
            offset: "",
        };
        let pattern = TargetPattern::parse(text, ctx, &mut |r| r.to_owned()).unwrap();
        let out = resolve(&[pattern], &repos);
        let messages: Vec<String> = out.failures.into_iter().map(|f| f.message).collect();
        assert_eq!(messages, [want], "{text}");
    }
}

/// Probed on Bazel 9.2.0 (buildfiji-mum.29): a BUILD file whose rules reported
/// errors still defines them, so a target named in it, or all of it, is a
/// target that fails to analyse; the other packages under a `...` are
/// selected whatever happens in the one with errors, a syntax error included.
#[test]
fn a_package_with_rule_errors_keeps_its_targets_and_the_others_under_a_tree_are_selected() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    for (file, text) in [
        (
            "pkg/BUILD",
            "genrule(name='ok', outs=['o'], cmd='true')\ngenrule(name='bad', cmd='true')\n",
        ),
        (
            "good/BUILD",
            "genrule(name='g', outs=['g.txt'], cmd='true')\n",
        ),
        ("syn/BUILD", "x = = 1\n"),
    ] {
        let at = ws.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
        .unwrap()
        .module;
    let repos = Repos::new(
        Options {
            workspace_root: ws,
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
    let ctx = PatternContext {
        repo: "",
        offset: "",
    };
    let label = |package: &str, name: &str| fjfj_graph::Label {
        repo: String::new(),
        package: package.to_owned(),
        name: name.to_owned(),
    };
    let run = |text: &str| {
        let pattern = TargetPattern::parse(text, ctx, &mut |r| r.to_owned()).unwrap();
        resolve(&[pattern], &repos)
    };

    let one = run("//pkg:ok");
    assert!(one.targets.is_empty());
    assert_eq!(one.in_error, [label("pkg", "ok")]);
    let failure = &one.failures[0];
    assert_eq!(
        failure.message,
        "Error evaluating '//pkg:ok': error loading package 'pkg': Package 'pkg' contains errors"
    );
    assert!(failure.defined && !failure.tree);

    // A name the package did not get to define is not one.
    let missing = run("//pkg:nonesuch");
    assert!(missing.in_error.is_empty());
    assert!(missing.failures[0].message.starts_with("no such target"));
    assert!(!missing.failures[0].defined);

    let all = run("//pkg:all");
    assert_eq!(all.in_error, [label("pkg", "ok"), label("pkg", "bad")]);

    let tree = run("//...");
    assert_eq!(tree.targets, [label("good", "g")]);
    assert_eq!(tree.in_error, [label("pkg", "ok"), label("pkg", "bad")]);
    assert!(tree.failures.iter().all(|f| f.tree));
    assert!(tree.failures[0].defined);
    // The syntax error defines nothing.
    let syntax = run("//syn/...");
    assert!(syntax.targets.is_empty() && syntax.in_error.is_empty());
    assert!(!syntax.failures[0].defined);
}
