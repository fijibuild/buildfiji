//! Bazel's own `http_archive` and `http_file` (the Starlark of `@bazel_tools`)
//! against Bazel 9.2.0 (buildfiji-mum.12): the table in `http_archive_matrix` is
//! what `bazel fetch` made of the same `MODULE.bazel` against a local HTTP server
//! (which this test stands in for).

use crate::http_archive_matrix::HA_ROWS;
use crate::{CredentialHelper, CredentialHelpers, Options, Repos};
use fjfj_archive::testing::build_spec;
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use fjfj_starlark::{Downloader, HttpRequest};
use regex::Regex;
use sha2::Digest as _;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What the server serves under a name.
#[allow(dead_code)]
pub(crate) enum Serve {
    Text(&'static str),
    Status(u16),
    Archive(
        &'static str,
        &'static [(&'static str, &'static str, &'static str)],
    ),
}

/// A `MODULE.bazel`, what is beside it, what the server has, the repositories
/// `bazel fetch` was asked for, and what it did: the error, what was printed,
/// the repositories it made, and the paths the server was asked for.
pub(crate) struct HaRow {
    pub(crate) module: &'static str,
    /// `--credential_helper` flags, the only ones the rows use.
    pub(crate) flags: &'static [&'static str],
    /// What the helper scripts of the row logged: a line per call.
    pub(crate) helper_log: &'static [&'static str],
    /// Git repositories to make: a name, the files of each commit, and the
    /// commits (by index) that are tagged.
    pub(crate) git: &'static [Git],
    pub(crate) files: &'static [(&'static str, &'static str)],
    pub(crate) serve: &'static [(&'static str, Serve)],
    pub(crate) fetch: &'static [&'static str],
    pub(crate) error: Option<&'static str>,
    pub(crate) printed: &'static [&'static str],
    pub(crate) tree: &'static [(&'static str, &'static str)],
    pub(crate) requests: &'static [&'static str],
}

pub(crate) type Git = (
    &'static str,
    &'static [&'static [(&'static str, &'static str)]],
    &'static [(usize, &'static str)],
);

/// Make the git repository `name` in `dir`: what `git init` and one commit per
/// set of files give, with a fixed author and date so that it is the same
/// repository each time. Returns its commits.
fn make_git(dir: &Path, spec: &Git) -> Vec<String> {
    let (name, commits, tags) = spec;
    let root = dir.join(format!("git_{name}"));
    std::fs::create_dir_all(&root).unwrap();
    let git = |args: &[&str]| -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .env("GIT_AUTHOR_NAME", "a")
            .env("GIT_AUTHOR_EMAIL", "a@x")
            .env("GIT_COMMITTER_NAME", "a")
            .env("GIT_COMMITTER_EMAIL", "a@x")
            .env("GIT_AUTHOR_DATE", "2023-01-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2023-01-01T00:00:00Z")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };
    git(&["init", "-q", "-b", "main"]);
    let mut shas = Vec::new();
    for (i, files) in commits.iter().enumerate() {
        for (path, text) in *files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", &format!("c{i}")]);
        shas.push(git(&["rev-parse", "HEAD"]));
        for (at, tag) in *tags {
            if *at == i {
                git(&["tag", tag]);
            }
        }
    }
    shas
}

struct Served {
    files: HashMap<String, (Vec<u8>, u16)>,
    requests: Mutex<Vec<String>>,
    helpers: Option<CredentialHelpers>,
}

impl Downloader for Served {
    fn get(&self, request: &HttpRequest) -> Result<Vec<u8>, String> {
        let path = request
            .url
            .strip_prefix("http://127.0.0.1:1")
            .unwrap_or(&request.url)
            .to_owned();
        // What `HttpDownloader` sends: a helper's `Authorization` wins.
        let helped = self
            .helpers
            .as_ref()
            .and_then(|h| h.headers_for(&request.url).ok().flatten())
            .and_then(|headers| {
                headers
                    .into_iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
                    .and_then(|(_, values)| values.into_iter().next())
            });
        let shown = match helped.or_else(|| crate::http::authorization(&request.auth)) {
            Some(value) => format!("{path} auth={value}"),
            None => path.clone(),
        };
        self.requests.lock().unwrap().push(shown);
        match self.files.get(path.trim_start_matches('/')) {
            Some((bytes, 200)) => Ok(bytes.clone()),
            Some((_, status)) => Err(format!("GET returned {status} Error")),
            None => Err("GET returned 404 Not Found".to_owned()),
        }
    }
}

struct Capture(std::cell::RefCell<Vec<String>>);

impl starlark::PrintHandler for Capture {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0.borrow_mut().push(text.to_owned());
        Ok(())
    }
}

/// A file's text, or a mark that it is not text.
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

fn walk(root: &Path, at: &Path, prefix: &str, out: &mut BTreeMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = format!("{prefix}/{}", path.strip_prefix(root).unwrap().display());
        let meta = std::fs::symlink_metadata(&path).unwrap();
        if meta.file_type().is_symlink() {
            let target = std::fs::read_link(&path).unwrap();
            out.insert(rel, format!("-> {}", target.display()));
        } else if meta.is_dir() {
            out.insert(rel.clone(), "<dir>".to_owned());
            walk(root, &path, prefix, out);
        } else {
            use std::os::unix::fs::PermissionsExt;
            let body = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            let exec = meta.permissions().mode() & 0o111 != 0;
            out.insert(
                rel,
                plain(&format!("{}{body}", if exec { "x " } else { "" })),
            );
        }
    }
}

struct Outcome {
    error: Option<String>,
    printed: Vec<String>,
    tree: BTreeMap<String, String>,
    requests: Vec<String>,
    helper_log: Vec<String>,
}

fn run(row: &HaRow) -> Outcome {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    let base = dir.path().join("ob");
    std::fs::create_dir_all(&ws).unwrap();
    std::fs::write(ws.join("BUILD.bazel"), "").unwrap();
    let files: HashMap<String, (Vec<u8>, u16)> = row
        .serve
        .iter()
        .map(|(name, serve)| {
            let value = match serve {
                Serve::Text(t) => (t.as_bytes().to_vec(), 200),
                Serve::Status(s) => (b"not here".to_vec(), *s),
                Serve::Archive(kind, members) => (build_spec(kind, members), 200),
            };
            ((*name).to_owned(), value)
        })
        .collect();
    let sha = Regex::new(r"@SHA:([\w./-]+)@").unwrap();
    let int = Regex::new(r"@INT:([\w./-]+)@").unwrap();
    let hash = |name: &str| sha2::Sha256::digest(&files[name].0);
    let gits: HashMap<&str, Vec<String>> = row
        .git
        .iter()
        .map(|spec| (spec.0, make_git(dir.path(), spec)))
        .collect();
    let git_path = Regex::new(r"@GIT:(\w+)@").unwrap();
    let git_commit = Regex::new(r"@COMMIT(\d*):(\w+)@").unwrap();
    let substitute = |text: &str| {
        let text = text
            .replace("@URL@", "http://127.0.0.1:1")
            .replace("@WS@", &ws.display().to_string());
        let text = git_path.replace_all(&text, |c: &regex::Captures<'_>| {
            dir.path()
                .join(format!("git_{}", &c[1]))
                .display()
                .to_string()
        });
        let text = git_commit.replace_all(&text, |c: &regex::Captures<'_>| {
            let all = &gits[&c[2]];
            let at: usize = c[1].parse().unwrap_or(all.len());
            all[at - 1].clone()
        });
        let text = sha.replace_all(&text, |c: &regex::Captures<'_>| hex::encode(hash(&c[1])));
        int.replace_all(&text, |c: &regex::Captures<'_>| {
            use base64::Engine as _;
            format!(
                "sha256-{}",
                base64::engine::general_purpose::STANDARD.encode(hash(&c[1]))
            )
        })
        .into_owned()
    };
    for (path, text) in row.files {
        let path = ws.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, substitute(text)).unwrap();
        if path.extension().is_some_and(|e| e == "sh") {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    // The flags, as Bazel parses them and then starts the helpers.
    let mut error = None;
    let mut parsed = Vec::new();
    for flag in row.flags {
        let value = substitute(flag.strip_prefix("--credential_helper=").unwrap());
        match CredentialHelper::parse(&value) {
            Ok(helper) => parsed.push(helper),
            Err(e) => {
                error = Some(format!(
                    "While parsing option --credential_helper={value}: {e}"
                ));
                break;
            }
        }
    }
    let helpers = if error.is_some() || parsed.is_empty() {
        None
    } else {
        let path_env = std::env::var("PATH").unwrap_or_default();
        match CredentialHelpers::new(&parsed, ws.clone(), &path_env, Duration::from_secs(10)) {
            Ok(h) => Some(h),
            Err(e) => {
                error = Some(e);
                None
            }
        }
    };
    let served = Arc::new(Served {
        files: files.clone(),
        requests: Mutex::new(Vec::new()),
        helpers,
    });
    let capture = Capture(std::cell::RefCell::new(Vec::new()));
    let module = match eval_module_file(
        "MODULE.bazel",
        &substitute(row.module),
        &EvalOptions::root(),
    ) {
        Ok(file) => Some(file.module),
        Err(e) => {
            error = Some(e.to_string());
            None
        }
    };
    if let Some(module) = module.filter(|_| error.is_none()) {
        let mut repos = Repos::new(
            Options {
                workspace_root: ws.clone(),
                output_base: base.clone(),
                environ: BTreeMap::from([
                    ("HOME".to_owned(), "/home/probe".to_owned()),
                    (
                        "PATH".to_owned(),
                        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_owned()),
                    ),
                ]),
                downloader: Some(served.clone()),
                repository_cache: Some(dir.path().join("cache")),
                registries: Vec::new(),
                facts: Vec::new(),
                repo_overrides: Vec::new(),
            },
            module,
        )
        .unwrap();
        let result = repos.run_extensions(Some(&capture)).and_then(|()| {
            for target in row.fetch {
                let apparent = target.trim_start_matches('@');
                let canonical = repos
                    .imports()
                    .iter()
                    .find(|(local, _)| local == apparent)
                    .map(|(_, c)| c.clone())
                    .ok_or_else(|| crate::FetchError {
                        message: format!("no repository visible as '@{apparent}'"),
                    })?;
                repos.fetch(&canonical, Some(&capture))?;
            }
            Ok(())
        });
        if let Err(e) = result {
            error = Some(e.message);
        }
    }
    let mut tree = BTreeMap::new();
    if error.is_none() {
        let external = base.join("external");
        if let Ok(entries) = std::fs::read_dir(&external) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with('+') {
                    walk(&entry.path(), &entry.path(), &name, &mut tree);
                }
            }
        }
        tree.retain(|k, _| !k.ends_with("REPO.bazel"));
    }
    let temp = Regex::new(r"temp\d{6,}").unwrap();
    let clean = |s: &str| {
        let s = Regex::new(r"[0-9a-f]{40}")
            .unwrap()
            .replace_all(s, "<commit>")
            .into_owned();
        let s = s
            .replace(&base.join("external").display().to_string(), "<ext>")
            .replace(&ws.display().to_string(), "<ws>")
            .replace("http://127.0.0.1:1", "@URL@");
        temp.replace_all(&s, "tempN").into_owned()
    };
    let mut requests: Vec<String> = Vec::new();
    for request in served.requests.lock().unwrap().iter() {
        if requests.last() != Some(request) {
            requests.push(request.clone());
        }
    }
    let helper_log = std::fs::read_to_string(ws.join("helper.log"))
        .unwrap_or_default()
        .lines()
        .map(&clean)
        .collect();
    Outcome {
        helper_log,
        error: error.map(|e| clean(&e)),
        printed: capture
            .0
            .into_inner()
            .iter()
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
            .map(|(k, v)| (clean(&k), clean(&v)))
            .collect(),
        requests,
    }
}

#[test]
fn bazels_http_archive_and_http_file_replay_bazel() {
    let mut wrong = Vec::new();
    for row in HA_ROWS {
        let got = run(row);
        let error_ok = match (&got.error, row.error) {
            (None, None) => true,
            // buildfiji-b9c: `"%s" % path` quotes the path, as repr does.
            (Some(got), Some(want)) => {
                got.contains(want) || got.replace("\"\"", "\"").contains(want)
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
            || got.helper_log != row.helper_log
        {
            wrong.push(format!(
                "{}\n  want: error {:?}, printed {:?}, tree {:?}, requests {:?}\n  got:  error {:?}, printed {:?}, tree {:?}, requests {:?}",
                row.module, row.error, row.printed, want_tree, want_requests, got.error, got.printed, got.tree, got_requests
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
