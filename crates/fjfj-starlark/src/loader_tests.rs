//! The loader (buildfiji-mum.19): what it says of a load, as Bazel 9.2.0 does,
//! and that it evaluates each file once from any number of threads.

use crate::{BuildFileError, BzlLoader, RepoMappings};
use fjfj_loading::PackageLookup;
use std::collections::HashMap;

/// A workspace of the given files, which has the main repo `""` and a mapping for it.
fn workspace(files: &[(&str, &str)], check_visibility: bool) -> (tempfile::TempDir, BzlLoader) {
    let dir = tempfile::tempdir().unwrap();
    for (name, body) in files {
        let path = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
    let repos = HashMap::from([(String::new(), PackageLookup::new(dir.path()).unwrap())]);
    let mappings = RepoMappings::from_repos([(
        String::new(),
        vec![(String::new(), String::new()), ("me".into(), String::new())],
    )]);
    (dir, BzlLoader::new(repos, mappings, check_visibility))
}

/// What loading `package` does: what it printed, or the error.
fn load(loader: &BzlLoader, package: &str) -> Result<Vec<String>, String> {
    loader
        .load_package("", package)
        .map(|out| out.printed)
        .map_err(|e| match e {
            BuildFileError::Eval(e) => format!("{e:#}"),
            BuildFileError::Package { events, .. } => events.join("\n"),
        })
}

fn fails(files: &[(&str, &str)], package: &str, want: &str) {
    let (_dir, loader) = workspace(files, true);
    let err = load(&loader, package).expect_err("should fail");
    assert!(err.contains(want), "want {want:?} in {err}");
}

#[test]
fn a_load_reads_the_file_and_gives_its_symbols() {
    let (_dir, loader) = workspace(
        &[
            (
                "b/BUILD.bazel",
                "load('//a:x.bzl', 'X', Z = 'Y')\nload(':l.bzl', 'L')\nload('l2.bzl', 'L2')\nload('@me//a:x.bzl', X2 = 'X')\nprint(X, Z, L, L2, X2)\n",
            ),
            ("a/BUILD.bazel", ""),
            ("a/x.bzl", "X = 1\nY = 2\n"),
            ("b/l.bzl", "L = 3\n"),
            ("b/l2.bzl", "L2 = 4\n"),
        ],
        true,
    );
    assert_eq!(load(&loader, "b").unwrap(), ["1 2 3 4 1"]);
    // Three `.bzl` files (x.bzl is loaded twice), each evaluated once.
    assert_eq!(loader.evaluations(), 3);
}

#[test]
fn what_bazel_says_of_a_load_that_cannot_happen() {
    let main = "b/BUILD.bazel";
    fails(
        &[(main, "load('//a:nope.bzl', 'X')\n"), ("a/BUILD.bazel", "")],
        "b",
        "cannot load '//a:nope.bzl': no such file",
    );
    fails(
        &[
            (main, "load('//a:x.bzl', 'X')\n"),
            ("a/BUILD.bazel", ""),
            ("a/x.bzl/y", ""),
        ],
        "b",
        "cannot load '//a:x.bzl': is a directory",
    );
    fails(
        &[(main, "load('//nopkg:x.bzl', 'X')\n")],
        "b",
        "Every .bzl file must have a corresponding package, but '//nopkg:x.bzl' does not have one.",
    );
    fails(
        &[(main, "load('//a:x.bzl', 'X')\n"), ("a/x.bzl", "X = 1\n")],
        "b",
        "Every .bzl file must have a corresponding package, but '//a:x.bzl' does not have one.",
    );
    fails(
        &[(main, "load('//a:x.txt', 'X')\n"), ("a/BUILD.bazel", "")],
        "b",
        "in load statement: The label must reference a file with extension \".bzl\" or \".scl\"",
    );
    fails(
        &[(main, "load('//a:x:y.bzl', 'X')\n")],
        "b",
        "in load statement: invalid target name 'x:y.bzl': target names may not contain ':'",
    );
    fails(
        &[(main, "load('@nope//a:x.bzl', 'X')\n")],
        "b",
        "Unable to find package for @@[unknown repo 'nope' requested from @@]//a:x.bzl: The repository '@@[unknown repo 'nope' requested from @@]' could not be resolved: No repository visible as '@nope' from main repository.",
    );
    fails(
        &[
            (main, "load('//a:sub/x.bzl', 'X')\n"),
            ("a/BUILD.bazel", ""),
            ("a/sub/BUILD.bazel", ""),
            ("a/sub/x.bzl", "X = 1\n"),
        ],
        "b",
        "Label '//a:sub/x.bzl' is invalid because 'a/sub' is a subpackage; perhaps you meant to put the colon here: '//a/sub:x.bzl'?",
    );
    fails(
        &[
            (main, "load('//a:x.bzl', 'X')\n"),
            ("a/BUILD.bazel", ""),
            ("a/x.bzl", "fail('boom')\nX = 1\n"),
        ],
        "b",
        "boom",
    );
    fails(
        &[
            (main, "load('//a:x.bzl', 'X')\n"),
            ("a/BUILD.bazel", ""),
            ("a/x.bzl", "load('//a:nope.bzl', 'Y')\nX = 1\n"),
        ],
        "b",
        "cannot load '//a:nope.bzl': no such file",
    );
}

#[test]
fn a_cycle_is_an_error_naming_its_files() {
    fails(
        &[
            ("b/BUILD.bazel", "load('//a:x.bzl', 'X')\n"),
            ("a/BUILD.bazel", ""),
            ("a/x.bzl", "load('//a:y.bzl', 'Y')\nX = 1\n"),
            ("a/y.bzl", "load('//a:x.bzl', 'X')\nY = 1\n"),
        ],
        "b",
        "cycle detected in extension files: \n.-> //a:x.bzl\n|   //a:y.bzl\n`-- //a:x.bzl",
    );
    fails(
        &[
            ("b/BUILD.bazel", "load('//a:x.bzl', 'X')\n"),
            ("a/BUILD.bazel", ""),
            ("a/x.bzl", "load('//a:x.bzl', 'X')\nZ = 1\n"),
        ],
        "b",
        "cycle detected in extension files",
    );
}

#[test]
fn load_visibility_is_checked_against_the_file_that_loads() {
    let files = [
        ("b/BUILD.bazel", "load('//a:lib.bzl', 'X')\n"),
        ("a/BUILD.bazel", ""),
        ("a/lib.bzl", "visibility(['//c'])\nX = 1\n"),
        ("c/BUILD.bazel", "load('//c:m.bzl', 'Y')\n"),
        ("c/m.bzl", "load('//a:lib.bzl', 'X')\nY = X\n"),
    ];
    let refused = "Starlark file //a:lib.bzl is not visible for loading from package //b. Check the file's `visibility()` declaration.";
    let (_dir, loader) = workspace(&files, true);
    assert!(load(&loader, "b").unwrap_err().contains(refused));
    // `//c` may, from its BUILD file and through its own `.bzl`.
    assert!(load(&loader, "c").is_ok());
    // And `--check_bzl_visibility=false` lets `//b` too.
    let (_dir, loader) = workspace(&files, false);
    assert!(load(&loader, "b").is_ok());
}

#[test]
fn each_file_is_evaluated_once_by_many_threads() {
    const PACKAGES: usize = 120;
    let mut files: Vec<(String, String)> = Vec::new();
    // A chain of ten `.bzl` files every package loads the top of.
    for i in 0..10 {
        let body = if i == 9 {
            "V9 = 0\n".to_owned()
        } else {
            format!(
                "load(':l{}.bzl', 'V{}')\nV{i} = V{} + 1\n",
                i + 1,
                i + 1,
                i + 1
            )
        };
        files.push((format!("lib/l{i}.bzl"), body));
    }
    files.push(("lib/BUILD.bazel".to_owned(), String::new()));
    for p in 0..PACKAGES {
        files.push((
            format!("p{p}/BUILD.bazel"),
            "load('//lib:l0.bzl', 'V0')\nload(':own.bzl', 'O')\nprint(V0, O)\n".to_owned(),
        ));
        files.push((format!("p{p}/own.bzl"), format!("O = {p}\n")));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_dir, loader) = workspace(&refs, true);
    std::thread::scope(|scope| {
        for thread in 0..8 {
            let loader = &loader;
            scope.spawn(move || {
                for p in (thread..PACKAGES).step_by(8) {
                    let out = load(loader, &format!("p{p}")).unwrap();
                    assert_eq!(out, [format!("9 {p}")]);
                }
            });
        }
    });
    // The ten of the chain and one own file a package, each once.
    assert_eq!(loader.evaluations(), 10 + PACKAGES);
}

#[test]
fn two_threads_loading_the_two_ends_of_a_cycle_do_not_wait_forever() {
    let files = [
        ("a/BUILD.bazel", "load('//lib:x.bzl', 'X')\n"),
        ("b/BUILD.bazel", "load('//lib:y.bzl', 'Y')\n"),
        ("lib/BUILD.bazel", ""),
        ("lib/x.bzl", "load(':y.bzl', 'Y')\nX = 1\n"),
        ("lib/y.bzl", "load(':x.bzl', 'X')\nY = 1\n"),
    ];
    for _ in 0..30 {
        let (_dir, loader) = workspace(&files, true);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            for package in ["a", "b"] {
                let (loader, tx) = (&loader, tx.clone());
                scope.spawn(move || {
                    tx.send(load(loader, package)).unwrap();
                });
            }
            drop(tx);
            for _ in 0..2 {
                let result = rx
                    .recv_timeout(std::time::Duration::from_secs(20))
                    .expect("a load hung");
                assert!(result.is_err());
            }
        });
    }
}

/// Loading a few thousand small packages through one loader, by thread count
/// (`cargo test -p fjfj-starlark --release loader_scale -- --ignored --nocapture`).
/// Not run by the gate: it measures, it does not assert.
#[test]
#[ignore]
fn loader_scale() {
    const PACKAGES: usize = 4000;
    let mut files: Vec<(String, String)> = Vec::new();
    files.push(("lib/BUILD.bazel".to_owned(), String::new()));
    files.push((
        "lib/defs.bzl".to_owned(),
        "def lib(name, **kw):\n    native.filegroup(name = name, srcs = kw.get('srcs', []))\n"
            .to_owned(),
    ));
    for p in 0..PACKAGES {
        let mut build = "load('//lib:defs.bzl', 'lib')\n".to_owned();
        for t in 0..8 {
            build.push_str(&format!("lib(name = 't{t}', srcs = ['f{t}.txt'])\n"));
        }
        files.push((format!("p{p}/BUILD.bazel"), build));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    for threads in [1usize, 2, 4, 8] {
        let (_dir, loader) = workspace(&refs, true);
        let started = std::time::Instant::now();
        std::thread::scope(|scope| {
            for thread in 0..threads {
                let loader = &loader;
                scope.spawn(move || {
                    for p in (thread..PACKAGES).step_by(threads) {
                        load(loader, &format!("p{p}")).unwrap();
                    }
                });
            }
        });
        let peak = std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmHWM"))
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        eprintln!(
            "{PACKAGES} packages, {threads} threads: {:?} ({} .bzl evaluated; process {peak})",
            started.elapsed(),
            loader.evaluations()
        );
    }
}
