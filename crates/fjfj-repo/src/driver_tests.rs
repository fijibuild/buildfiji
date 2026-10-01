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
            downloader: None,
            repository_cache: None,
            distdirs: Vec::new(),
            registries: Vec::new(),
            facts: Vec::new(),
            repo_overrides: Vec::new(),
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
    assert_eq!(
        repos.extension_of("+ext+two").as_deref(),
        Some("@@//:ext.bzl%ext")
    );
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
fn a_repository_fetched_again_starts_from_nothing() {
    let (dir, mut repos) = repos(
        "module(name = 'm')\next = use_extension('//:ext.bzl', 'ext')\nuse_repo(ext, 'one')\n",
    );
    repos.run_extensions(None).unwrap();
    // What an earlier process left in the output base.
    let left = dir.path().join("ob/external/+ext+one");
    std::fs::create_dir_all(&left).unwrap();
    std::fs::write(left.join("stale.txt"), "left over").unwrap();
    // An archive's read-only directory does not stop it.
    std::fs::create_dir_all(left.join("ro")).unwrap();
    std::fs::write(left.join("ro/f"), "x").unwrap();
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(left.join("ro"), std::fs::Permissions::from_mode(0o555)).unwrap();
    }
    let made = repos.fetch("+ext+one", None).unwrap();
    assert!(!made.join("stale.txt").exists());
    assert!(!made.join("ro").exists());
    assert!(made.join("name.txt").exists());
}

#[test]
fn a_path_formats_as_its_text_with_str_and_percent_s_and_format() {
    let (dir, mut repos) = repos(
        "module(name = 'm')\next = use_extension('//:ext.bzl', 'ext')\nuse_repo(ext, 'one')\n",
    );
    std::fs::write(
        dir.path().join("ws/ext.bzl"),
        r#"
def _repo(ctx):
    ctx.file("args", "x")
    p = ctx.path("args")
    ctx.file("out.txt", "|".join([str(p), "{}".format(p), "%s" % p, "{!s}".format(p), repr(p)]))
repo = repository_rule(_repo)
def _impl(mctx):
    repo(name = "one")
ext = module_extension(_impl)
"#,
    )
    .unwrap();
    repos.run_extensions(None).unwrap();
    let made = repos.fetch("+ext+one", None).unwrap();
    let out = std::fs::read_to_string(made.join("out.txt")).unwrap();
    let at = made.join("args").display().to_string();
    assert_eq!(out, [at.as_str(); 4].join("|") + "|\"" + &at + "\"");
}

#[test]
fn a_name_that_is_not_a_generated_repository_cannot_be_fetched() {
    let (_dir, mut repos) = repos("module(name = 'm')\n");
    repos.run_extensions(None).unwrap();
    let error = repos.fetch("+ext+nothing", None).unwrap_err();
    assert_eq!(error.message, "no repository named '@@+ext+nothing'");
}

mod http_archive {
    use super::*;
    use fjfj_archive::testing::{Member, build};
    use fjfj_starlark::{Downloader, HttpRequest};
    use sha2::Digest as _;
    use std::sync::{Arc, Mutex};

    /// A server with one file.
    struct One(String, Vec<u8>, Mutex<Vec<String>>);

    impl Downloader for One {
        fn get(&self, request: &HttpRequest) -> Result<Vec<u8>, String> {
            self.2.lock().unwrap().push(request.url.clone());
            if request.url == self.0 {
                Ok(self.1.clone())
            } else {
                Err("GET returned 404 Not Found".to_owned())
            }
        }
    }

    pub(super) fn repos_serving(
        module: &str,
        url: &str,
        bytes: Vec<u8>,
    ) -> (tempfile::TempDir, Repos) {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("BUILD.bazel"), "").unwrap();
        let file = eval_module_file("MODULE.bazel", module, &EvalOptions::root()).unwrap();
        let repos = Repos::new(
            Options {
                workspace_root: ws,
                output_base: dir.path().join("ob"),
                environ: BTreeMap::new(),
                downloader: Some(Arc::new(One(url.to_owned(), bytes, Mutex::new(Vec::new())))),
                repository_cache: Some(dir.path().join("cache")),
                distdirs: Vec::new(),
                registries: Vec::new(),
                facts: Vec::new(),
                repo_overrides: Vec::new(),
            },
            file.module,
        )
        .unwrap();
        (dir, repos)
    }

    #[test]
    fn bazels_own_http_archive_fetches_unpacks_and_writes_a_build_file() {
        let zip = build(
            "zip",
            &[
                ("top/a.txt", Member::file("A")),
                ("top/sub/b.txt", Member::file("B")),
            ],
        );
        let sha = hex::encode(sha2::Sha256::digest(&zip));
        let module = format!(
            "module(name = 'm')\n\
             http_archive = use_repo_rule('@bazel_tools//tools/build_defs/repo:http.bzl', 'http_archive')\n\
             http_archive(name = 'dep', urls = ['http://h/dep.zip'], sha256 = '{sha}', \
             strip_prefix = 'top', build_file_content = 'filegroup(name = \"all\")')\n"
        );
        let (_dir, mut repos) = repos_serving(&module, "http://h/dep.zip", zip);
        repos.run_extensions(None).unwrap();
        assert_eq!(
            repos.imports(),
            [("dep".to_owned(), "+http_archive+dep".to_owned())]
        );
        let made = repos.fetch("+http_archive+dep", None).unwrap();
        let read = |p: &str| std::fs::read_to_string(made.join(p)).unwrap();
        assert_eq!(read("a.txt"), "A");
        assert_eq!(read("sub/b.txt"), "B");
        assert!(read("BUILD.bazel").contains("filegroup(name = \"all\")"));
    }
}

#[test]
fn a_local_repository_with_no_boundary_file_is_refused_as_bazel_does() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(ws.join("sub")).unwrap();
    std::fs::write(ws.join("BUILD.bazel"), "").unwrap();
    std::fs::write(ws.join("sub/BUILD.bazel"), "").unwrap();
    let module = "module(name = 'm')\n\
        local_repository = use_repo_rule('@bazel_tools//tools/build_defs/repo:local.bzl', \
        'local_repository')\n\
        local_repository(name = 'x', path = 'sub')\n";
    let file = eval_module_file("MODULE.bazel", module, &EvalOptions::root()).unwrap();
    let base = dir.path().join("ob");
    let repos = Repos::new(
        Options {
            workspace_root: ws.clone(),
            output_base: base.clone(),
            environ: BTreeMap::new(),
            downloader: None,
            repository_cache: None,
            distdirs: Vec::new(),
            registries: Vec::new(),
            facts: Vec::new(),
            repo_overrides: Vec::new(),
        },
        file.module,
    )
    .unwrap();
    let err = repos.fetch("+local_repository+x", None).unwrap_err();
    assert_eq!(
        err.message,
        format!(
            "No MODULE.bazel, REPO.bazel, or WORKSPACE file found in {}/external/+local_repository+x",
            base.display()
        )
    );
    // With a MODULE.bazel in it the directory is what the repository is.
    std::fs::write(ws.join("sub/MODULE.bazel"), "module(name = 'x')\n").unwrap();
    std::fs::remove_dir_all(base.join("external/+local_repository+x")).ok();
    std::fs::remove_file(base.join("external/+local_repository+x")).ok();
    let dir = repos.fetch("+local_repository+x", None).unwrap();
    assert!(dir.join("BUILD.bazel").is_file());
}

#[test]
fn an_extension_reads_the_facts_a_previous_run_kept() {
    use fjfj_bzlmod::lockfile::Json;
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("BUILD.bazel"), "").unwrap();
    std::fs::write(
        ws.join("ext.bzl"),
        r#"
def _r(rctx):
    rctx.file("BUILD.bazel", "")

r = repository_rule(implementation = _r)

def _impl(mctx):
    f = mctx.facts
    print("type=%s get=%s dflt=%s index=%s missing=%s in=%s out=%s" % (
        type(f), f.get("a"), f.get("zz", "d"), f["b"], f["nope"], "a" in f, "zz" in f))
    print("nested=%s" % f["c"]["d"])
    r(name = "x")
    return mctx.extension_metadata(facts = {"k": [1, 2.5, None], "n": None})

ext = module_extension(_impl)
"#,
    )
    .unwrap();
    let module =
        "module(name = 'm')\next = use_extension('//:ext.bzl', 'ext')\nuse_repo(ext, 'x')\n";
    let file = eval_module_file("MODULE.bazel", module, &EvalOptions::root()).unwrap();
    let facts = Json::Object(vec![
        ("a".to_owned(), Json::Number(1.into())),
        (
            "b".to_owned(),
            Json::Array(vec![Json::String("x".to_owned())]),
        ),
        (
            "c".to_owned(),
            Json::Object(vec![("d".to_owned(), Json::Bool(true))]),
        ),
    ]);
    let mut repos = Repos::new(
        Options {
            workspace_root: ws,
            output_base: dir.path().join("ob"),
            environ: BTreeMap::new(),
            downloader: None,
            repository_cache: None,
            distdirs: Vec::new(),
            registries: Vec::new(),
            facts: vec![("//:ext.bzl%ext".to_owned(), facts)],
            repo_overrides: Vec::new(),
        },
        file.module,
    )
    .unwrap();
    struct Capture(std::cell::RefCell<Vec<String>>);
    impl starlark::PrintHandler for Capture {
        fn println(&self, text: &str) -> starlark::Result<()> {
            self.0.borrow_mut().push(text.to_owned());
            Ok(())
        }
    }
    let capture = Capture(std::cell::RefCell::new(Vec::new()));
    repos.run_extensions(Some(&capture)).unwrap();
    assert_eq!(
        *capture.0.borrow(),
        [
            "type=Facts get=1 dflt=d index=[\"x\"] missing=None in=True out=False",
            "nested=True"
        ]
    );
    // What it keeps is what the lockfile will have: sorted keys, None left out.
    let kept = repos.locked_facts();
    assert_eq!(kept.len(), 1);
    assert_eq!(
        serde_json::to_string(&kept[0].1).unwrap(),
        r#"{"k":[1,2.5,null]}"#
    );
}
