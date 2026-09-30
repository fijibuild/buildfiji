//! Load visibility against Bazel 9.2.0 (buildfiji-ps4): what `visibility()`
//! accepts (the table `load_visibility_matrix` holds is what Bazel said for
//! each call), and who may load a file with a given declaration.

use crate::load_visibility_matrix::VISIBILITY_CALLS;
use crate::test_support::replay;
use crate::{
    BzlFile, RepoMappings, bzl_globals, check_load_visibility, evaluate_bzl, load_visibility,
};
use fjfj_graph::Label;
use starlark::environment::FrozenModule;
use starlark::eval::FileLoader;
use std::cell::RefCell;
use std::collections::HashMap;

/// Probes whose answer is the Starlark runtime's generic wording (buildfiji-v32).
const GENERIC: &[&str] = &["accepts no more than", "got unexpected keyword"];

#[test]
fn visibility_calls_replay_bazel() {
    let wrong = replay(VISIBILITY_CALLS, &[], GENERIC);
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// Loads files of one repo the way a loader with load visibility does: each
/// file is evaluated with a loader that knows which file is loading, and a
/// load the loaded file does not allow is an error.
struct Checked<'a> {
    files: &'a HashMap<String, String>,
    /// The file that loads.
    importer: Label,
    check: bool,
    cache: &'a RefCell<HashMap<String, FrozenModule>>,
}

/// The main repo names itself `@`.
fn main_mappings() -> RepoMappings {
    RepoMappings::from_repos([(String::new(), vec![(String::new(), String::new())])])
}

fn label_of(text: &str, from: &Label) -> Label {
    let (package, name) = match text.strip_prefix("//") {
        Some(rest) => {
            let (p, n) = rest.split_once(':').expect("a label with a colon");
            (p.to_owned(), n.to_owned())
        }
        None => (
            from.package.clone(),
            text.trim_start_matches(':').to_owned(),
        ),
    };
    Label {
        repo: from.repo.clone(),
        package,
        name,
    }
}

impl FileLoader for Checked<'_> {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        let file = label_of(path, &self.importer);
        let key = format!("{}:{}", file.package, file.name);
        let module = self.cache.borrow().get(&key).cloned();
        let module = match module {
            Some(m) => m,
            None => {
                let source = self
                    .files
                    .get(&key)
                    .unwrap_or_else(|| panic!("no file {key}"));
                let loader = Checked {
                    files: self.files,
                    importer: file.clone(),
                    check: self.check,
                    cache: self.cache,
                };
                let m = evaluate_bzl(&BzlFile {
                    file: &file,
                    source,
                    globals: &bzl_globals(),
                    mappings: &main_mappings(),
                    loader: &loader,
                    print: None,
                })?;
                self.cache.borrow_mut().insert(key, m.clone());
                m
            }
        };
        check_load_visibility(&self.importer, &file, &load_visibility(&module), self.check)
            .map_err(|message| starlark::Error::new_other(anyhow::anyhow!(message)))?;
        Ok(module)
    }
}

/// The error of loading `//a:lib.bzl`, which says `declaration`, from a file
/// in `importer_package`; `None` if it loads.
fn loading(declaration: &str, importer_package: &str, check: bool) -> Option<String> {
    let mut files = HashMap::new();
    files.insert("a:lib.bzl".to_owned(), format!("{declaration}\nX = 1\n"));
    let importer = format!("{importer_package}:m.bzl");
    files.insert(
        importer.clone(),
        "load('//a:lib.bzl', 'X')\nY = X\n".to_owned(),
    );
    let cache = RefCell::new(HashMap::new());
    let main = Label {
        repo: String::new(),
        package: importer_package.to_owned(),
        name: "m.bzl".to_owned(),
    };
    let loader = Checked {
        files: &files,
        importer: main.clone(),
        check,
        cache: &cache,
    };
    evaluate_bzl(&BzlFile {
        file: &main,
        source: &files[&importer],
        globals: &bzl_globals(),
        mappings: &main_mappings(),
        loader: &loader,
        print: None,
    })
    .err()
    .map(|e| e.kind().to_string())
}

fn refused(importer: &str) -> String {
    format!(
        "Starlark file //a:lib.bzl is not visible for loading from package {importer}. Check \
         the file's `visibility()` declaration."
    )
}

#[test]
fn who_may_load_a_file_is_what_its_visibility_says() {
    // Each row was probed: the declaration, the package that loads, and whether Bazel allowed it.
    let rows: &[(&str, &str, bool)] = &[
        ("", "b", true),
        ("visibility('public')", "b", true),
        ("visibility('private')", "b", false),
        ("visibility(['//a'])", "b", false),
        ("visibility(['//b'])", "b", true),
        ("visibility(['//b/...'])", "b", true),
        ("visibility(['//...'])", "b", true),
        ("visibility([])", "b", false),
        ("visibility(['public'])", "b", true),
        ("visibility(['private'])", "b", false),
        ("visibility(['//b', 'private'])", "b", true),
        ("visibility(['//a:__pkg__'])", "b", false),
        ("visibility(['//a:__subpackages__'])", "b", false),
        ("visibility(['@foo//x'])", "b", false),
        ("visibility(['@@//b'])", "b", true),
        ("visibility(['@//b'])", "b", true),
        // A package may load its own files, and a spec does not reach a subpackage.
        ("visibility('private')", "a", true),
        ("visibility(['//a'])", "a/sub", false),
        ("visibility(['//a/...'])", "a/sub", true),
        // The root package.
        ("visibility(['//'])", "", true),
        ("visibility(['//:__pkg__'])", "", true),
        ("visibility(['//b'])", "", false),
    ];
    for (declaration, importer, allowed) in rows {
        let got = loading(declaration, importer, true);
        let shown = if importer.is_empty() {
            "//".to_owned()
        } else {
            format!("//{importer}")
        };
        match (allowed, &got) {
            (true, None) => {}
            (false, Some(message)) => assert_eq!(message, &refused(&shown), "{declaration}"),
            _ => panic!("{declaration} from {importer:?}: got {got:?}, want allowed = {allowed}"),
        }
    }
}

#[test]
fn check_bzl_visibility_false_turns_the_check_off() {
    assert!(loading("visibility('private')", "b", false).is_none());
    assert!(loading("visibility('private')", "b", true).is_some());
}

#[test]
fn a_file_that_loads_a_private_one_is_the_importer_not_the_file_it_was_loaded_for() {
    // `//c:m.bzl` may load `//a:lib.bzl` (visible to `//c`); a BUILD file in
    // `//b` loads `//c:m.bzl`, which is public, and that is fine.
    let mut files = HashMap::new();
    files.insert(
        "a:lib.bzl".to_owned(),
        "visibility(['//c'])\nX = 1\n".to_owned(),
    );
    files.insert(
        "c:m.bzl".to_owned(),
        "load('//a:lib.bzl', 'X')\nY = X\n".to_owned(),
    );
    files.insert(
        "b:main.bzl".to_owned(),
        "load('//c:m.bzl', 'Y')\nZ = Y\n".to_owned(),
    );
    let cache = RefCell::new(HashMap::new());
    let main = Label {
        repo: String::new(),
        package: "b".to_owned(),
        name: "main.bzl".to_owned(),
    };
    let loader = Checked {
        files: &files,
        importer: main.clone(),
        check: true,
        cache: &cache,
    };
    let out = evaluate_bzl(&BzlFile {
        file: &main,
        source: &files["b:main.bzl"],
        globals: &bzl_globals(),
        mappings: &main_mappings(),
        loader: &loader,
        print: None,
    });
    assert!(out.is_ok(), "{:?}", out.err().map(|e| e.kind().to_string()));
}

#[test]
fn visibility_is_declared_once_and_at_the_top() {
    let run = |src: &str| crate::test_support::run(src).unwrap_err();
    assert!(
        run("visibility('public')\nvisibility('private')")
            .contains("may not be set more than once")
    );
    assert!(
        run("def f():\n    visibility('public')\nf()").contains("may only be set at the top level")
    );
    // Not called, a function that would is fine, and where the call is does not matter.
    assert!(
        crate::test_support::run("def f():\n    visibility('public')\nY = 1\nvisibility(['//b'])")
            .is_ok()
    );
    let err = crate::test_support::run_build("r = 1", "visibility('public')").fatal;
    assert!(err.is_some());
}
