//! Shared by the tests that replay Bazel 9.2.0 probes.

use crate::WithoutSites;
use crate::{BuildFile, BuildFileError, BzlFile, RepoMappings, bzl_globals, evaluate_bzl};
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

/// What a BUILD file did, for [`replay_build`]: what it printed, the events
/// it reported, and the fatal error that stopped it.
#[derive(Debug, Default)]
pub(crate) struct BuildOutcome {
    pub(crate) printed: Vec<String>,
    pub(crate) events: Vec<String>,
    pub(crate) fatal: Option<String>,
    pub(crate) package: Option<fjfj_graph::package::Package>,
}

/// Load `build`, after a line that loads `r` from `:u.bzl`, as the BUILD file
/// of the main repo of the probe workspace
/// (`f1.txt` and `sub/x.txt` are files in it, and `sub` is a package), with
/// `bzl` as the `.bzl` file `:u.bzl` it may load.
pub(crate) fn run_build(bzl: &str, build: &str) -> BuildOutcome {
    use std::fs;
    let dir = tempfile::tempdir().unwrap();
    for file in ["f1.txt", "sub/x.txt", "sub/BUILD.bazel", "BUILD.bazel"] {
        let path = dir.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }
    let lookup = fjfj_loading::PackageLookup::new(dir.path()).unwrap();
    let mappings = probe_mappings();
    let loader = Files {
        sources: [("u.bzl", bzl)].into_iter().collect(),
        loaded: RefCell::new(HashMap::new()),
    };
    let out = crate::evaluate_build_file(&BuildFile {
        module: None,
        repo: "",
        package: "",
        lookup: &lookup,
        mappings: &mappings,
        path: "BUILD.bazel",
        source: &format!("load(':u.bzl', 'r')\n{build}"),
        loader: &loader,
    });
    match out {
        Ok(out) => BuildOutcome {
            printed: out.printed.without_sites(),
            package: Some(out.package),
            ..Default::default()
        },
        Err(BuildFileError::Eval(e, _)) => BuildOutcome {
            fatal: Some(format!("{e:#}")),
            ..Default::default()
        },
        Err(BuildFileError::Load(reason) | BuildFileError::Absent(reason)) => BuildOutcome {
            fatal: Some(reason),
            ..Default::default()
        },
        Err(BuildFileError::Package {
            events, printed, ..
        }) => BuildOutcome {
            printed: printed.without_sites(),
            events,
            ..Default::default()
        },
    }
}

/// A row of a BUILD-file probe table: the `.bzl` and the BUILD file, and
/// what Bazel 9.2.0 printed, reported and stopped with.
pub(crate) struct BuildRow {
    pub(crate) bzl: &'static str,
    pub(crate) build: &'static str,
    pub(crate) printed: &'static [&'static str],
    pub(crate) events: &'static [&'static str],
    pub(crate) fatal: Option<&'static str>,
}

/// Replay [`BuildRow`]s. A row that stopped with a fatal error (or whose
/// `.bzl` failed to load, which Bazel reports as an event at the `.bzl`)
/// must stop with it; a row with events must report exactly those, and
/// print what it did; any other prints what Bazel did. Rows containing one
/// of `skip` are known differences, and a fatal message containing one of
/// `relaxed` only has to be some fatal error.
pub(crate) fn replay_build(rows: &[BuildRow], skip: &[&str], relaxed: &[&str]) -> Vec<String> {
    let mut wrong = Vec::new();
    for row in rows {
        if skip
            .iter()
            .any(|marker| row.bzl.contains(marker) || row.build.contains(marker))
        {
            continue;
        }
        // `print("")` is a line Bazel's own log drops the end of.
        let mut got = run_build(row.bzl, row.build);
        got.printed.retain(|p| !p.is_empty());
        let want_printed: Vec<&str> = row
            .printed
            .iter()
            .copied()
            .filter(|p| !p.is_empty())
            .collect();
        // A `.bzl` that does not load is an event at `u.bzl:L:C:`.
        let load_failure = row
            .events
            .first()
            .and_then(|e| e.strip_prefix("u.bzl:"))
            .and_then(|e| e.splitn(3, ':').nth(2))
            .map(|m| m.trim_start());
        let want_fatal = row.fatal.or(load_failure);
        let ok = match want_fatal {
            Some(message) => got.fatal.as_deref().is_some_and(|f| {
                f.contains(message) || relaxed.iter().any(|p| message.contains(p))
            }),
            None if !row.events.is_empty() => {
                got.fatal.is_none() && got.events == row.events && got.printed == want_printed
            }
            None => got.fatal.is_none() && got.events.is_empty() && got.printed == want_printed,
        };
        if !ok {
            wrong.push(format!(
                "{}\n---\n{}\n  want: printed {:?} events {:?} fatal {:?}\n  got:  printed {:?} events {:?} fatal {:?}",
                row.bzl, row.build, row.printed, row.events, row.fatal, got.printed, got.events, got.fatal
            ));
        }
    }
    wrong
}
