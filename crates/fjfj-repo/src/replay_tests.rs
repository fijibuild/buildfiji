//! Module extensions against Bazel 9.2.0 (buildfiji-mum.8.4): the table in
//! `replay_matrix` is what `bazel fetch` did with the same workspace, and the
//! tests here are what a table cannot say.

use crate::replay_matrix::EXT_ROWS;
use crate::{Options, Repos};
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

/// One workspace: its files, the repositories `bazel fetch` was asked for, and
/// what it did: the error it stopped with, what was printed, the root module's
/// repository mapping and each repository it made (files as `x ` if executable
/// and then their content, `<dir>`, `-> target`).
pub(crate) struct ExtRow {
    pub(crate) files: &'static [(&'static str, &'static str)],
    pub(crate) fetch: &'static [&'static str],
    pub(crate) error: Option<&'static str>,
    pub(crate) printed: &'static [&'static str],
    pub(crate) mapping: &'static [(&'static str, &'static str)],
    pub(crate) repos: &'static [(&'static str, &'static [(&'static str, &'static str)])],
}

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "unsupported binary operation",
    "unknown tag class",
    "has no field or method",
    "in call to hash()",
    "Facts",
    "syntax error at",
    "does not accept positional arguments",
    "missing 1 required positional argument",
];

struct Capture(RefCell<Vec<String>>);

impl starlark::PrintHandler for Capture {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0
            .borrow_mut()
            .push(text.trim_end_matches('\n').to_owned());
        Ok(())
    }
}

struct Outcome {
    error: Option<String>,
    printed: Vec<String>,
    mapping: BTreeMap<String, String>,
    repos: BTreeMap<String, BTreeMap<String, String>>,
}

fn walk(root: &Path, at: &Path, out: &mut BTreeMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap().display().to_string();
        let meta = std::fs::symlink_metadata(&path).unwrap();
        if meta.file_type().is_symlink() {
            let target = std::fs::read_link(&path).unwrap();
            out.insert(rel, format!("-> {}", target.display()));
        } else if meta.is_dir() {
            out.insert(rel.clone(), "<dir>".to_owned());
            walk(root, &path, out);
        } else {
            use std::os::unix::fs::PermissionsExt;
            let body = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            let exec = meta.permissions().mode() & 0o111 != 0;
            out.insert(rel, format!("{}{body}", if exec { "x " } else { "" }));
        }
    }
}

fn run(row: &ExtRow) -> Outcome {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    let base = dir.path().join("ob");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("BUILD.bazel"), "").unwrap();
    for (path, text) in row.files {
        let path = ws.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let capture = Capture(RefCell::new(Vec::new()));
    let clean = |s: &str| {
        s.replace(&base.join("modextwd").display().to_string(), "<work>")
            .replace(&base.join("external").display().to_string(), "<ext>")
            .replace(&ws.display().to_string(), "<ws>")
    };
    let mut error = None;
    let mut mapping = BTreeMap::new();
    let mut repos = BTreeMap::new();
    let module_text = row
        .files
        .iter()
        .find(|(p, _)| *p == "MODULE.bazel")
        .map(|(_, t)| *t)
        .unwrap();
    match eval_module_file("MODULE.bazel", module_text, &EvalOptions::root()) {
        Err(e) => error = Some(e.to_string()),
        Ok(file) => {
            let options = Options {
                workspace_root: ws.clone(),
                output_base: base.clone(),
                environ: BTreeMap::from([
                    ("HOME".to_owned(), "/home/probe".to_owned()),
                    (
                        "PATH".to_owned(),
                        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_owned()),
                    ),
                ]),
                downloader: None,
                repository_cache: None,
                registries: Vec::new(),
            };
            let mut made = Repos::new(options, file.module).unwrap();
            let result = made.run_extensions(Some(&capture)).and_then(|()| {
                for target in row.fetch {
                    let apparent = target.trim_start_matches('@');
                    let canonical = made
                        .imports()
                        .iter()
                        .find(|(local, _)| local == apparent)
                        .map(|(_, c)| c.clone())
                        .ok_or_else(|| crate::FetchError {
                            message: format!("no repository visible as '@{apparent}'"),
                        })?;
                    made.fetch(&canonical, Some(&capture))?;
                }
                Ok(())
            });
            if let Err(e) = result {
                error = Some(e.message);
            } else {
                for (local, canonical) in made.imports() {
                    mapping.insert(local.clone(), canonical.clone());
                }
            }
            let external = base.join("external");
            if let Ok(entries) = std::fs::read_dir(&external) {
                for entry in entries.flatten() {
                    // `@bazel_tools` is there, as in Bazel, and not something
                    // the workspace made.
                    if entry.file_name() == "bazel_tools" {
                        continue;
                    }
                    let mut tree = BTreeMap::new();
                    walk(&entry.path(), &entry.path(), &mut tree);
                    tree.remove("REPO.bazel");
                    repos.insert(entry.file_name().to_string_lossy().into_owned(), tree);
                }
            }
        }
    }
    Outcome {
        error: error.map(|e| clean(&e)),
        printed: capture.0.into_inner().iter().map(|p| clean(p)).collect(),
        mapping,
        repos,
    }
}

#[test]
fn module_extensions_replay_bazel() {
    let mut wrong = Vec::new();
    for row in EXT_ROWS {
        let got = run(row);
        let error_ok = match (&got.error, row.error) {
            (None, None) => true,
            (Some(got), Some(want)) => {
                got.contains(want) || GENERIC.iter().any(|g| want.contains(g))
            }
            _ => false,
        };
        let want_repos: BTreeMap<String, BTreeMap<String, String>> = row
            .repos
            .iter()
            .map(|(n, t)| {
                (
                    (*n).to_owned(),
                    t.iter()
                        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                        .collect(),
                )
            })
            .collect();
        // The root module's own names are in Bazel's mapping too; what is
        // compared is what `use_repo` added.
        let want_mapping: BTreeMap<String, String> = row
            .mapping
            .iter()
            .filter(|(k, v)| !k.is_empty() && *k != "bazel_tools" && v.contains('+'))
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        let got_mapping: BTreeMap<String, String> = got
            .mapping
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mapping_ok = row.error.is_some() || want_mapping == got_mapping;
        if !error_ok || got.printed != row.printed || got.repos != want_repos || !mapping_ok {
            wrong.push(format!(
                "{}\n  want: error {:?}, printed {:?}, mapping {:?}, repos {:?}\n  got:  error {:?}, printed {:?}, mapping {:?}, repos {:?}",
                row.files
                    .iter()
                    .map(|(p, t)| format!("--- {p}\n{t}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                row.error,
                row.printed,
                want_mapping,
                want_repos,
                got.error,
                got.printed,
                got_mapping,
                got.repos
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
