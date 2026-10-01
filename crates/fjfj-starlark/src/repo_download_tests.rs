//! `download()`, `download_and_extract()`, `extract()` and `patch()` against
//! Bazel 9.2.0 (buildfiji-mum.8.3): the table in `repo_download_matrix` is what
//! `bazel fetch` did for the same repository rule against a local HTTP server
//! (which this test stands in for), and the tests here are what a table cannot
//! say.

use crate::repo_download_matrix::DL_ROWS;
use crate::test_support::{Capture, module_in, probe_mappings};
use crate::{Downloader, HttpRequest, RepoEnv, repository_rule_defaults, run_repository_rule};
use fjfj_archive::testing::build_spec;
use regex::Regex;
use sha2::Digest as _;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// What the server serves under a name.
pub(crate) enum Serve {
    Text(&'static str),
    Hex(&'static str),
    Status(u16),
    /// An archive of a kind (`zip`, `tar.gz`, `deb:tar.xz`, ...), its members
    /// as (path, `f` file, `x` executable file, `d` directory, `l` symlink, `h`
    /// hard link, text or target).
    Archive(
        &'static str,
        &'static [(&'static str, &'static str, &'static str)],
    ),
}

/// One repository rule, what the server has, and what Bazel did with it: the
/// error it stopped with, what the rule printed, the repository it left, and
/// the paths the server was asked for (`--second--` marks a second fetch of
/// the same repository, from a clean output, when `twice`).
pub(crate) struct DlRow {
    pub(crate) bzl: &'static str,
    pub(crate) serve: &'static [(&'static str, Serve)],
    pub(crate) twice: bool,
    pub(crate) error: Option<&'static str>,
    pub(crate) printed: &'static [&'static str],
    pub(crate) tree: &'static [(&'static str, &'static str)],
    pub(crate) requests: &'static [&'static str],
}

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "unsupported comparison",
    "has no field or method",
    "in call to hash()",
];

fn bytes_of(serve: &Serve) -> (Vec<u8>, u16) {
    match serve {
        Serve::Text(text) => (text.as_bytes().to_vec(), 200),
        Serve::Hex(hex_text) => (hex::decode(hex_text).unwrap(), 200),
        Serve::Status(status) => (b"not here".to_vec(), *status),
        Serve::Archive(kind, files) => (build_spec(kind, files), 200),
    }
}

/// A server that is a table.
struct Served {
    files: HashMap<String, (Vec<u8>, u16)>,
    requests: Mutex<Vec<String>>,
}

impl Downloader for Served {
    fn get(&self, request: &HttpRequest) -> Result<Vec<u8>, String> {
        let path = request
            .url
            .strip_prefix("http://127.0.0.1:1")
            .unwrap_or(&request.url)
            .to_owned();
        self.requests.lock().unwrap().push(path.clone());
        let name = path.trim_start_matches('/').split('?').next().unwrap_or("");
        match self.files.get(name) {
            Some((bytes, 200)) => Ok(bytes.clone()),
            Some((_, status)) => Err(status_message(*status)),
            None => Err(status_message(404)),
        }
    }
}

fn status_message(status: u16) -> String {
    let reason = match status {
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Error",
    };
    format!("GET returned {status} {reason}")
}

/// A file's text, or a mark that it is not text (the bytes of an archive
/// differ from the ones Bazel was given, and only that they are there counts).
fn plain(value: &str) -> String {
    let (prefix, body) = match value.strip_prefix("x ") {
        Some(rest) => ("x ", rest),
        None => ("", value),
    };
    if body
        .chars()
        .all(|c| matches!(c, '\n' | '\t' | '\r') || (' '..='~').contains(&c))
    {
        value.to_owned()
    } else {
        format!("{prefix}<binary>")
    }
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

struct Outcome {
    error: Option<String>,
    printed: Vec<String>,
    tree: BTreeMap<String, String>,
    requests: Vec<String>,
}

fn run(row: &DlRow) -> Outcome {
    let dir = tempfile::tempdir().unwrap();
    let ext = dir.path().join("ext");
    let repo = ext.join("+r+NAME");
    let cache = dir.path().join("cache");
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ext).unwrap();
    std::fs::create_dir_all(&ws).unwrap();
    let files: HashMap<String, (Vec<u8>, u16)> = row
        .serve
        .iter()
        .map(|(name, serve)| ((*name).to_owned(), bytes_of(serve)))
        .collect();
    // The placeholders in the rule: the server's address, and what a served
    // file hashes to.
    let sha = Regex::new(r"@SHA:([\w./-]+)@").unwrap();
    let int = Regex::new(r"@INT:([\w./-]+)@").unwrap();
    let hash = |name: &str| sha2::Sha256::digest(&files[name].0);
    let bzl = row.bzl.replace("@URL@", "http://127.0.0.1:1");
    let bzl = sha.replace_all(&bzl, |c: &regex::Captures<'_>| hex::encode(hash(&c[1])));
    let bzl = int.replace_all(&bzl, |c: &regex::Captures<'_>| {
        use base64::Engine as _;
        format!(
            "sha256-{}",
            base64::engine::general_purpose::STANDARD.encode(hash(&c[1]))
        )
    });
    let module = module_in("", "", &bzl).unwrap();
    let served = Arc::new(Served {
        files,
        requests: Mutex::new(Vec::new()),
    });
    let capture = Capture(RefCell::new(Vec::new()));
    let mut error = None;
    let runs = if row.twice { 2 } else { 1 };
    let mut first_printed = Vec::new();
    for round in 0..runs {
        if round == 1 {
            served
                .requests
                .lock()
                .unwrap()
                .push("--second--".to_owned());
            let _ = std::fs::remove_dir_all(&repo);
        }
        let env = RepoEnv {
            name: "+r+NAME".to_owned(),
            original_name: "NAME".to_owned(),
            output: repo.clone(),
            workspace_root: ws.clone(),
            environ: BTreeMap::new(),
            labels: Box::new(|_| None),
            attrs: repository_rule_defaults(&module, "r"),
            downloader: Some(served.clone()),
            repository_cache: Some(cache.clone()),
            distdirs: Vec::new(),
            recorded: Default::default(),
        };
        let result = run_repository_rule(&module, "r", env, &probe_mappings(), Some(&capture));
        if let Err(e) = result {
            error = Some(e.message);
        }
        if round == 0 {
            first_printed = capture.0.borrow().clone();
        }
    }
    let mut tree = BTreeMap::new();
    walk(&repo, &repo, &mut tree);
    let temp = Regex::new(r"temp\d{6,}").unwrap();
    let clean = |s: &str| {
        let s = s
            .replace(&repo.display().to_string(), "<repo>")
            .replace(&ext.display().to_string(), "<ext>")
            .replace(&cache.display().to_string(), "<cache>")
            .replace("http://127.0.0.1:1", "@URL@");
        temp.replace_all(&s, "tempN").into_owned()
    };
    let requests = served.requests.lock().unwrap().clone();
    let mut deduped: Vec<String> = Vec::new();
    for request in requests {
        if deduped.last() != Some(&request) {
            deduped.push(request);
        }
    }
    Outcome {
        error: error.map(|e| clean(&e)),
        printed: first_printed
            .iter()
            // Bazel's log has the first line of a print on the line that says where.
            .map(|p| {
                clean(
                    p.trim_end_matches('\n')
                        .split('\n')
                        .next()
                        .unwrap_or_default(),
                )
            })
            .collect(),
        tree: tree
            .into_iter()
            .map(|(k, v)| (clean(&k), plain(&clean(&v))))
            .collect(),
        requests: deduped,
    }
}

#[test]
fn downloads_replay_bazel() {
    let mut wrong = Vec::new();
    for row in DL_ROWS {
        let got = run(row);
        let error_ok = match (&got.error, row.error) {
            (None, None) => true,
            (Some(got), Some(want)) => {
                got.contains(want) || GENERIC.iter().any(|g| want.contains(g))
            }
            _ => false,
        };
        let want_tree: BTreeMap<String, String> = row
            .tree
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        let want_requests: Vec<&str> = row.requests.to_vec();
        let got_requests: Vec<&str> = got.requests.iter().map(String::as_str).collect();
        if !error_ok
            || got.printed != row.printed
            || got.tree != want_tree
            || got_requests != want_requests
        {
            wrong.push(format!(
                "{}\n  want: error {:?}, printed {:?}, tree {:?}, requests {:?}\n  got:  error {:?}, printed {:?}, tree {:?}, requests {:?}",
                row.bzl, row.error, row.printed, want_tree, want_requests, got.error, got.printed, got.tree, got_requests
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

/// Run `bzl`'s rule `r` with `downloader` and the repository cache `cache`,
/// making the repository in `repo`.
fn run_with(
    bzl: &str,
    downloader: Arc<dyn Downloader>,
    cache: &Path,
    repo: &Path,
) -> Result<(), String> {
    let module = module_in("", "", bzl).unwrap();
    let env = RepoEnv {
        name: "+r+x".to_owned(),
        original_name: "x".to_owned(),
        output: repo.to_owned(),
        workspace_root: repo.to_owned(),
        environ: BTreeMap::new(),
        labels: Box::new(|_| None),
        attrs: repository_rule_defaults(&module, "r"),
        downloader: Some(downloader),
        repository_cache: Some(cache.to_owned()),
        distdirs: Vec::new(),
        recorded: Default::default(),
    };
    run_repository_rule(&module, "r", env, &probe_mappings(), None).map_err(|e| e.message)
}

struct Counting(Mutex<usize>, &'static [u8]);

impl Downloader for Counting {
    fn get(&self, _: &HttpRequest) -> Result<Vec<u8>, String> {
        *self.0.lock().unwrap() += 1;
        Ok(self.1.to_vec())
    }
}

struct Refusing;

impl Downloader for Refusing {
    fn get(&self, _: &HttpRequest) -> Result<Vec<u8>, String> {
        Err("the network is not there".to_owned())
    }
}

#[test]
fn download_and_extract_takes_the_deprecated_strip_prefix_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let archive: &'static [u8] =
        Box::leak(build_spec("tar.gz", &[("kotlinc/a.txt", "f", "hi")]).into_boxed_slice());
    let bzl = "def _impl(ctx):\n    ctx.download_and_extract('https://h/k.tar.gz', 'out', stripPrefix = 'kotlinc')\nr = repository_rule(_impl)\n";
    let counting = Arc::new(Counting(Mutex::new(0), archive));
    run_with(
        bzl,
        counting,
        &dir.path().join("cache"),
        &dir.path().join("one"),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("one/out/a.txt")).unwrap(),
        "hi"
    );
}

#[test]
fn a_download_that_names_its_sha256_is_kept_and_served_from_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache");
    let hex = hex::encode(sha2::Sha256::digest(b"hello"));
    let bzl = format!(
        "def _impl(ctx):\n    ctx.download('http://h/f', 'out', sha256 = '{hex}')\nr = repository_rule(_impl)\n"
    );
    let counting = Arc::new(Counting(Mutex::new(0), b"hello"));
    run_with(&bzl, counting.clone(), &cache, &dir.path().join("one")).unwrap();
    assert_eq!(*counting.0.lock().unwrap(), 1);
    // Kept by its hash, as Bazel's repository cache does.
    let kept = cache.join(format!("content_addressable/sha256/{hex}/file"));
    assert_eq!(std::fs::read(&kept).unwrap(), b"hello");
    // A second repository needs no network at all.
    run_with(&bzl, Arc::new(Refusing), &cache, &dir.path().join("two")).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("two/out")).unwrap(),
        "hello"
    );
    // Without the cache it would have failed.
    let error = run_with(
        &bzl,
        Arc::new(Refusing),
        &dir.path().join("empty"),
        &dir.path().join("three"),
    )
    .unwrap_err();
    assert!(error.contains("the network is not there"), "{error}");
}

#[test]
fn an_integrity_of_sha256_is_a_cache_key_too_and_a_file_url_is_cached() {
    use base64::Engine as _;
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache");
    let source = dir.path().join("source.txt");
    std::fs::write(&source, "from disk").unwrap();
    let sri = format!(
        "sha256-{}",
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(b"hello"))
    );
    let by_integrity = format!(
        "def _impl(ctx):\n    ctx.download('http://h/f', 'out', integrity = '{sri}')\nr = repository_rule(_impl)\n"
    );
    run_with(
        &by_integrity,
        Arc::new(Counting(Mutex::new(0), b"hello")),
        &cache,
        &dir.path().join("one"),
    )
    .unwrap();
    run_with(
        &by_integrity,
        Arc::new(Refusing),
        &cache,
        &dir.path().join("two"),
    )
    .unwrap();
    // A `file://` download needs no checksum and is cached by what it read.
    let by_file = format!(
        "def _impl(ctx):\n    ctx.download('file://{}', 'out')\nr = repository_rule(_impl)\n",
        source.display()
    );
    run_with(
        &by_file,
        Arc::new(Refusing),
        &cache,
        &dir.path().join("three"),
    )
    .unwrap();
    let hex = hex::encode(sha2::Sha256::digest(b"from disk"));
    assert!(
        cache
            .join(format!("content_addressable/sha256/{hex}/file"))
            .exists()
    );
}
