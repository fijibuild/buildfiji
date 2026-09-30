//! Shared by the tests that replay Bazel 9.2.0 probes.

use crate::{BzlFile, RepoMappings, bzl_globals, evaluate_bzl};
use fjfj_graph::Label;
use starlark::environment::FrozenModule;
use starlark::eval::FileLoader;
use std::cell::RefCell;
use std::collections::HashMap;

pub(crate) struct Capture(pub(crate) RefCell<Vec<String>>);

impl starlark::PrintHandler for Capture {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0.borrow_mut().push(text.to_owned());
        Ok(())
    }
}

/// The repos of the probe workspace the Label tables were taken in: the
/// main repo is `probe` and depends on `dep` (as `mydep`) and `other`, and
/// `dep+` depends on `other` (as `oth`).
pub(crate) fn probe_mappings() -> RepoMappings {
    let rows = |rows: &[(&str, &str)]| {
        rows.iter()
            .map(|(a, c)| (a.to_string(), c.to_string()))
            .collect::<Vec<_>>()
    };
    let mut mappings = RepoMappings::new();
    mappings.insert(
        "",
        rows(&[
            ("", ""),
            ("probe", ""),
            ("mydep", "dep+"),
            ("other", "other+"),
        ]),
    );
    mappings.insert("dep+", rows(&[("dep", "dep+"), ("oth", "other+")]));
    mappings
}

/// Run `src` as the body of `//:t.bzl` in the main repo of the probe
/// workspace, returning what it printed or its error.
pub(crate) fn run(src: &str) -> Result<Vec<String>, String> {
    run_in("", "", src)
}

/// [`run`] for the file `t.bzl` in `package` of `repo`.
pub(crate) fn run_in(repo: &str, package: &str, src: &str) -> Result<Vec<String>, String> {
    let capture = Capture(RefCell::new(Vec::new()));
    module_with(repo, package, src, Some(&capture))?;
    Ok(capture.0.into_inner())
}

/// The frozen module of the file `t.bzl` in `package` of `repo`, for a test
/// that looks at what a global holds.
pub(crate) fn module_in(repo: &str, package: &str, src: &str) -> Result<FrozenModule, String> {
    module_with(repo, package, src, None)
}

fn module_with(
    repo: &str,
    package: &str,
    src: &str,
    print: Option<&dyn starlark::PrintHandler>,
) -> Result<FrozenModule, String> {
    let file = Label {
        repo: repo.to_owned(),
        package: package.to_owned(),
        name: "t.bzl".to_owned(),
    };
    evaluate_bzl(&BzlFile {
        file: &file,
        source: src,
        globals: &bzl_globals(),
        mappings: &probe_mappings(),
        loader: &NoLoads,
        print,
    })
    .map_err(|e| format!("{:#}", e.into_anyhow()))
}

/// Run `main` as `//:t.bzl` in a workspace of the other `files` (named
/// `a.bzl`, loaded as `:a.bzl`), returning what it printed or its error. A
/// file is evaluated once, however many others load it.
pub(crate) fn run_files(files: &[(&str, &str)], main: &str) -> Result<Vec<String>, String> {
    let loader = Files {
        sources: files.iter().copied().collect(),
        loaded: RefCell::new(HashMap::new()),
    };
    let capture = Capture(RefCell::new(Vec::new()));
    let file = Label {
        repo: String::new(),
        package: String::new(),
        name: "t.bzl".to_owned(),
    };
    evaluate_bzl(&BzlFile {
        file: &file,
        source: main,
        globals: &bzl_globals(),
        mappings: &probe_mappings(),
        loader: &loader,
        print: Some(&capture),
    })
    .map_err(|e| format!("{:#}", e.into_anyhow()))?;
    Ok(capture.0.into_inner())
}

/// The loader of [`run_files`].
struct Files<'a> {
    sources: HashMap<&'a str, &'a str>,
    loaded: RefCell<HashMap<String, FrozenModule>>,
}

impl FileLoader for Files<'_> {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        let name = path.trim_start_matches(':');
        if let Some(done) = self.loaded.borrow().get(name) {
            return Ok(done.clone());
        }
        let source = self.sources.get(name).ok_or_else(|| {
            starlark::Error::new_other(anyhow::anyhow!("cannot load '{path}': no such file"))
        })?;
        let file = Label {
            repo: String::new(),
            package: String::new(),
            name: name.to_owned(),
        };
        let module = evaluate_bzl(&BzlFile {
            file: &file,
            source,
            globals: &bzl_globals(),
            mappings: &probe_mappings(),
            loader: self,
            print: None,
        })?;
        self.loaded
            .borrow_mut()
            .insert(name.to_owned(), module.clone());
        Ok(module)
    }
}

/// A loader for a file that loads nothing.
struct NoLoads;

impl FileLoader for NoLoads {
    fn load(&self, path: &str) -> starlark::Result<starlark::environment::FrozenModule> {
        Err(starlark::Error::new_other(anyhow::anyhow!(
            "cannot load '{path}': no loader"
        )))
    }
}

/// Replay `(source, outcome)` rows: `Ok` is every line `print` gave, `Err`
/// is a substring of the error. Rows whose source contains one of `skip`
/// are known differences owned by another bead, and an `Err` row whose
/// message contains one of `relaxed` (the runtime's generic wording, which
/// is buildfiji-v32's) only has to fail. Returns what differs.
pub(crate) fn replay(
    rows: &[(&str, Result<&str, &str>)],
    skip: &[&str],
    relaxed: &[&str],
) -> Vec<String> {
    replay_in("", "", rows, skip, relaxed)
}

/// [`replay`] for a file in `package` of `repo`.
pub(crate) fn replay_in(
    repo: &str,
    package: &str,
    rows: &[(&str, Result<&str, &str>)],
    skip: &[&str],
    relaxed: &[&str],
) -> Vec<String> {
    let run = |src: &str| run_in(repo, package, src);
    let mut wrong = Vec::new();
    for (src, want) in rows {
        if skip.iter().any(|marker| src.contains(marker)) {
            continue;
        }
        let ok = match (run(src), want) {
            (Ok(got), Ok(want)) => got.join("\n") == *want,
            (Err(got), Err(want)) => {
                got.contains(want) || relaxed.iter().any(|pattern| want.contains(pattern))
            }
            _ => false,
        };
        if !ok {
            wrong.push(format!("{src}\n  want: {want:?}\n  got:  {:?}", run(src)));
        }
    }
    wrong
}
