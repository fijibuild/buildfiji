//! What the replay table cannot say about [`Repos`].

use crate::{Options, Repos};
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use std::collections::BTreeMap;

const EXT: &str = r#"
def _repo(ctx):
    ctx.file("BUILD.bazel", "")
    ctx.file("name.txt", ctx.attr.name + "/" + ctx.original_name)
repo = repository_rule(_repo)
def _impl(mctx):
    repo(name = "one")
    repo(name = "two")
ext = module_extension(_impl)
other = module_extension(_impl)
"#;

fn repos(module: &str) -> (tempfile::TempDir, Repos) {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("BUILD.bazel"), "").unwrap();
    std::fs::write(ws.join("ext.bzl"), EXT).unwrap();
    let file = eval_module_file("MODULE.bazel", module, &EvalOptions::root()).unwrap();
    let repos = Repos::new(
        Options {
            workspace_root: ws,
            output_base: dir.path().join("ob"),
            environ: BTreeMap::new(),
        },
        file.module,
    )
    .unwrap();
    (dir, repos)
}

#[test]
fn an_extension_makes_every_repository_its_implementation_calls_and_imports_name_some() {
    let (_dir, mut repos) = repos(
        "module(name = 'm')\next = use_extension('//:ext.bzl', 'ext')\nuse_repo(ext, 'one')\n",
    );
    repos.run_extensions(None).unwrap();
    // `two` was generated too, though nothing imports it.
    assert_eq!(
        repos.generated().collect::<Vec<_>>(),
        ["+ext+one", "+ext+two"]
    );
    assert_eq!(repos.imports(), [("one".to_owned(), "+ext+one".to_owned())]);
    assert_eq!(repos.extension_of("+ext+two"), Some("@@//:ext.bzl%ext"));
}

#[test]
fn a_repository_is_made_where_the_output_base_says_and_knows_its_names() {
    let (dir, mut repos) = repos(
        "module(name = 'm')\next = use_extension('//:ext.bzl', 'ext')\nuse_repo(ext, mine = 'two')\n",
    );
    repos.run_extensions(None).unwrap();
    let made = repos.fetch("+ext+two", None).unwrap();
    assert_eq!(made, dir.path().join("ob/external/+ext+two"));
    assert_eq!(
        std::fs::read_to_string(made.join("name.txt")).unwrap(),
        "+ext+two/two"
    );
    assert!(made.join("REPO.bazel").exists());
    assert_eq!(
        repos.imports(),
        [("mine".to_owned(), "+ext+two".to_owned())]
    );
}

#[test]
fn two_extensions_of_one_file_are_two_extensions() {
    let (_dir, mut repos) = repos(
        "module(name = 'm')\na = use_extension('//:ext.bzl', 'ext')\nuse_repo(a, 'one')\n\
         b = use_extension('//:ext.bzl', 'other')\nuse_repo(b, other_one = 'one')\n",
    );
    repos.run_extensions(None).unwrap();
    assert_eq!(
        repos.imports(),
        [
            ("one".to_owned(), "+ext+one".to_owned()),
            ("other_one".to_owned(), "+other+one".to_owned())
        ]
    );
}

#[test]
fn a_name_that_is_not_a_generated_repository_cannot_be_fetched() {
    let (_dir, mut repos) = repos("module(name = 'm')\n");
    repos.run_extensions(None).unwrap();
    let error = repos.fetch("+ext+nothing", None).unwrap_err();
    assert_eq!(error.message, "no repository named '@@+ext+nothing'");
}
