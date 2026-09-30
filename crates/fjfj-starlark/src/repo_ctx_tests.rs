//! `repository_ctx` against Bazel 9.2.0 (buildfiji-mum.8.2): the table in
//! `repo_ctx_matrix` is what `bazel fetch` did for the same repository rule,
//! and the tests here are what a table cannot say.

use crate::repo_ctx_matrix::REPO_CASES;
use crate::test_support::{Capture, module_in, probe_mappings};
use crate::{RepoEnv, repository_rule_defaults, run_repository_rule};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

/// One repository rule run: the `.bzl` that defines `r`, files in the main
/// repository, and what Bazel did: the error it stopped with, what the rule
/// printed, and the repository it left (files as `x ` if executable and then
/// their content, `<dir>`, `-> target`).
pub(crate) struct RepoRow {
    pub(crate) bzl: &'static str,
    pub(crate) files: &'static [(&'static str, &'static str)],
    pub(crate) error: Option<&'static str>,
    pub(crate) printed: &'static [&'static str],
    pub(crate) tree: &'static [(&'static str, &'static str)],
}

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "has no field or method",
    "unsupported binary operation",
    "unsupported comparison",
    "in call to hash()",
    "in call to len()",
    "cannot encode",
    "value does not support field assignment",
    "unknown attribute",
];

struct Outcome {
    error: Option<String>,
    printed: Vec<String>,
    tree: BTreeMap<String, String>,
}

fn names(text: &str, repo: &Path, ext: &Path, ws: &Path) -> String {
    text.replace(&repo.display().to_string(), "<repo>")
        .replace(&ext.display().to_string(), "<ext>")
        .replace(&ws.display().to_string(), "<ws>")
}

fn run_repo(bzl: &str, files: &[(&str, &str)]) -> Outcome {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    let ext = dir.path().join("ext");
    let repo = ext.join("+r+NAME");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::create_dir_all(&ext).unwrap();
    for (path, text) in files {
        let path = ws.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let module = module_in("", "", bzl).unwrap();
    let workspace = ws.clone();
    let env = RepoEnv {
        name: "+r+NAME".to_owned(),
        original_name: "NAME".to_owned(),
        output: repo.clone(),
        workspace_root: ws.clone(),
        environ: BTreeMap::from([
            ("HOME".to_owned(), "/home/probe".to_owned()),
            (
                "PATH".to_owned(),
                std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_owned()),
            ),
        ]),
        labels: Box::new(move |label| {
            label
                .repo
                .is_empty()
                .then(|| workspace.join(&label.package).join(&label.name))
        }),
        attrs: repository_rule_defaults(&module, "r"),
        downloader: None,
        repository_cache: None,
    };
    let capture = Capture(RefCell::new(Vec::new()));
    let result = run_repository_rule(&module, "r", env, &probe_mappings(), Some(&capture));
    let mut tree = BTreeMap::new();
    walk(&repo, &repo, &ws, &mut tree);
    let clean = |s: &str| names(s, &repo, &ext, &ws);
    Outcome {
        error: result.err().map(|e| clean(&e.message)),
        // Bazel's log is a line per print, so a print's own final newline is not in it.
        printed: capture
            .0
            .into_inner()
            .iter()
            .map(|p| clean(p.trim_end_matches('\n')))
            .collect(),
        tree: tree.into_iter().map(|(k, v)| (k, clean(&v))).collect(),
    }
}

fn walk(root: &Path, at: &Path, ws: &Path, out: &mut BTreeMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap().display().to_string();
        let meta = std::fs::symlink_metadata(&path).unwrap();
        if meta.file_type().is_symlink() {
            let target = std::fs::read_link(&path).unwrap();
            out.insert(
                rel,
                format!("-> {}", target.display())
                    .replace(&ws.display().to_string(), "<ws>")
                    .replace(&root.display().to_string(), "<repo>"),
            );
        } else if meta.is_dir() {
            out.insert(rel.clone(), "<dir>".to_owned());
            walk(root, &path, ws, out);
        } else {
            use std::os::unix::fs::PermissionsExt;
            let body = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            let exec = meta.permissions().mode() & 0o111 != 0;
            out.insert(rel, format!("{}{body}", if exec { "x " } else { "" }));
        }
    }
}

#[test]
fn repository_rules_replay_bazel() {
    let mut wrong = Vec::new();
    for row in REPO_CASES {
        let got = run_repo(row.bzl, row.files);
        let want_tree: BTreeMap<String, String> = row
            .tree
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        let error_ok = match (&got.error, row.error) {
            (None, None) => true,
            (Some(got), Some(want)) => {
                got.contains(want) || GENERIC.iter().any(|g| want.contains(g))
            }
            _ => false,
        };
        // A rule that failed leaves whatever it had made.
        if !error_ok || got.printed != row.printed || got.tree != want_tree {
            wrong.push(format!(
                "{}\n  want: error {:?}, printed {:?}, tree {:?}\n  got:  error {:?}, printed {:?}, tree {:?}",
                row.bzl, row.error, row.printed, want_tree, got.error, got.printed, got.tree
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

use crate::RepoAttr;
use fjfj_graph::Label;

/// Run the rule `r` of `bzl` with attributes the call gave, over the
/// defaults, and with no files in the main repository.
fn run_with(bzl: &str, given: Vec<(&str, RepoAttr)>) -> (Result<(), String>, Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let module = module_in("", "", bzl).unwrap();
    let mut attrs = repository_rule_defaults(&module, "r");
    for (name, value) in given {
        attrs.retain(|(n, _)| n != name);
        attrs.push((name.to_owned(), value));
    }
    let env = RepoEnv {
        name: "+r+x".to_owned(),
        original_name: "x".to_owned(),
        output: dir.path().join("repo"),
        workspace_root: dir.path().join("ws"),
        environ: BTreeMap::new(),
        labels: Box::new(|_| None),
        attrs,
        downloader: None,
        repository_cache: None,
    };
    let capture = Capture(RefCell::new(Vec::new()));
    let result = run_repository_rule(&module, "r", env, &probe_mappings(), Some(&capture))
        .map_err(|e| e.message);
    (result, capture.0.into_inner())
}

#[test]
fn attributes_reach_the_rule_as_values_of_their_types() {
    let (result, printed) = run_with(
        "def _impl(ctx):\n\
         \x20   print(ctx.attr.s, ctx.attr.i, ctx.attr.b, ctx.attr.sl, ctx.attr.il, ctx.attr.sd, ctx.attr.sld)\n\
         \x20   print(ctx.attr.l.package, ctx.attr.l.name, type(ctx.attr.l), ctx.attr.ll, ctx.attr.name)\n\
         \x20   ctx.file('done', '')\n\
         r = repository_rule(_impl, attrs = {\n\
         \x20   's': attr.string(), 'i': attr.int(), 'b': attr.bool(), 'sl': attr.string_list(),\n\
         \x20   'il': attr.int_list(), 'sd': attr.string_dict(), 'sld': attr.string_list_dict(),\n\
         \x20   'l': attr.label(), 'll': attr.label_list(), 'd': attr.string(default = 'dflt'),\n\
         })\n",
        vec![
            ("s", RepoAttr::String("text".into())),
            ("i", RepoAttr::Int(7)),
            ("b", RepoAttr::Bool(true)),
            ("sl", RepoAttr::StringList(vec!["a".into(), "b".into()])),
            ("il", RepoAttr::IntList(vec![1, 2])),
            ("sd", RepoAttr::StringDict(vec![("k".into(), "v".into())])),
            (
                "sld",
                RepoAttr::StringListDict(vec![("k".into(), vec!["v".into()])]),
            ),
            (
                "l",
                RepoAttr::Label(Label {
                    repo: String::new(),
                    package: "pkg".into(),
                    name: "t".into(),
                }),
            ),
            ("ll", RepoAttr::LabelList(vec![])),
        ],
    );
    assert_eq!(result, Ok(()));
    assert_eq!(
        printed,
        [
            "text 7 True [\"a\", \"b\"] [1, 2] {\"k\": \"v\"} {\"k\": [\"v\"]}",
            "pkg t Label [] +r+x",
        ]
    );
}

#[test]
fn a_default_fills_what_the_call_left_out() {
    let (result, printed) = run_with(
        "def _impl(ctx):\n    print(ctx.attr.d, ctx.attr.n)\n    ctx.file('done', '')\n\
         r = repository_rule(_impl, attrs = {'d': attr.string(default = 'dflt'), 'n': attr.int(default = 3)})\n",
        vec![],
    );
    assert_eq!(result, Ok(()));
    assert_eq!(printed, ["dflt 3"]);
}

#[test]
fn a_label_whose_file_cannot_be_found_is_an_error_not_a_panic() {
    let (result, _) = run_with(
        "def _impl(ctx):\n    ctx.read(Label('@other//:f'))\nr = repository_rule(_impl)\n",
        vec![],
    );
    let error = result.unwrap_err();
    assert!(error.contains("cannot find the file of label"), "{error}");
}

#[test]
fn what_the_rule_printed_before_it_failed_is_kept() {
    let (result, printed) = run_with(
        "def _impl(ctx):\n    print('before')\n    fail('boom')\nr = repository_rule(_impl)\n",
        vec![],
    );
    assert!(result.unwrap_err().contains("boom"));
    assert_eq!(printed, ["before"]);
}

#[test]
fn a_name_that_is_not_a_repository_rule_is_reported() {
    let module = module_in("", "", "x = 1\n").unwrap();
    let env = RepoEnv {
        name: "+r+x".to_owned(),
        original_name: "x".to_owned(),
        output: std::env::temp_dir().join("fjfj-never-made"),
        workspace_root: std::env::temp_dir(),
        environ: BTreeMap::new(),
        labels: Box::new(|_| None),
        attrs: vec![],
        downloader: None,
        repository_cache: None,
    };
    let error = run_repository_rule(&module, "x", env, &probe_mappings(), None).unwrap_err();
    assert!(error.message.contains("not a repository rule"), "{error}");
}

#[test]
fn a_process_that_is_killed_reports_the_signal_the_way_a_shell_does() {
    let (result, printed) = run_with(
        "def _impl(ctx):\n    print(ctx.execute(['sh', '-c', 'kill -9 $$']).return_code)\nr = repository_rule(_impl)\n",
        vec![],
    );
    assert_eq!(result, Ok(()));
    assert_eq!(printed, ["137"]);
}
