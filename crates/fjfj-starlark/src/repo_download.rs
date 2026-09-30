//! `repository_ctx.download()`, `download_and_extract()`, `extract()` and
//! `patch()` (buildfiji-mum.8.3), and the repository cache they share.
//!
//! What Bazel 9.2.0 does, read off probes against a local HTTP server:
//!
//! - **What is asked for.** `url` is a string or a sequence of them, tried in
//!   order; a URL must have a scheme (`Bad URL: x`) that is `http`, `https` or
//!   `file` (`Unsupported protocol: ftp`). A plain `http` URL with no `sha256`
//!   or `integrity` is refused (`No URLs left after removing plain http URLs
//!   due to missing checksum. Please provide either a checksum or an https
//!   download location.`). `sha256` and `integrity` (an SRI string) may not both
//!   be given. The output is written only inside the repository
//!   (`Cannot write outside...`, before anything is asked for); the file is
//!   executable only if `executable = True`; there is no directory magic
//!   (`output = "dir/"` is a file called `dir`).
//! - **The checksum** is verified after the download: `Error downloading
//!   [urls] to <path>: Checksum was X but wanted Y` (both in the form it was
//!   given, hex or SRI), and no other URL is tried. A checksum that is not one
//!   (`Invalid SHA-256 checksum 'abc'`, `Unsupported checksum algorithm: 'x'
//!   (expected SHA-1, SHA-256, SHA-384, or SHA-512)`) is only found out after.
//! - **The repository cache** keeps each download by the SHA-256 of what it
//!   got, `content_addressable/sha256/<hex>/file`; a download that names its
//!   SHA-256 (as `sha256` or as a `sha256-` SRI) is served from it without a
//!   request, whatever `canonical_id` says, and anything downloaded is put
//!   there (a `file://` URL too).
//! - **Failure.** Every failure of the download (a 404, a refused URL, a bad
//!   checksum, no URLs left) is `java.io.IOException: Error downloading [urls]
//!   to <path>: <why>`; with `allow_fail = True` it is `struct(success =
//!   False)` instead. Success is `struct(integrity = "sha256-...", sha256 =
//!   "...", success = True)`. With `block = False` the same happens at the call
//!   and `wait()` on the result gives it (or the error).
//! - **download_and_extract** downloads into a `temp<digits>` directory of the
//!   output, names the file for the URL's last segment plus `.type` if given,
//!   extracts it, and deletes the directory (it stays if the download failed
//!   with `allow_fail`). A name with no known suffix: `Expected a file with a
//!   .zip, .jar, ... suffix (got <path>)`; a failed extraction: `Error
//!   extracting <archive> to <dir>: <why>`.
//! - **extract** of a missing archive is `Archive path '<p>' does not exist.`;
//!   **patch** reports `Error applying patch <p>: <why>`.

use crate::args::fatal;
use crate::decl::{P, bind_checked, is_bool, is_dict, p};
use crate::repo_ctx::{
    RepoEnv, Resolved, check_watch, env_of, flag, io_error, is_path_like, path_arg, read_text,
    resolve, writable,
};
use crate::structs::new_struct;
use allocative::Allocative;
use base64::Engine as _;
use sha2::Digest as _;
use starlark::eval::{Arguments, Evaluator};
use starlark::values::dict::DictRef;
use starlark::values::list::ListRef;
use starlark::values::none::NoneType;
use starlark::values::tuple::TupleRef;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;
use std::path::{Path, PathBuf};

// ---- the network, which the caller provides -------------------------------------------

/// A request for a URL over HTTP or HTTPS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    /// Header names with each of their values.
    pub headers: Vec<(String, Vec<String>)>,
    /// What `auth` said for this URL: a `type` and its `login` and `password`
    /// (or `pattern`), as given.
    pub auth: Vec<(String, String)>,
}

/// Fetches URLs. An error is the words Bazel gives for it after `Error
/// downloading [...] to <path>: `, such as `GET returned 404 Not Found`.
pub trait Downloader: Send + Sync {
    fn get(&self, request: &HttpRequest) -> Result<Vec<u8>, String>;
}

// ---- the repository cache -------------------------------------------------------------

fn cache_file(root: &Path, hex: &str) -> PathBuf {
    root.join("content_addressable")
        .join("sha256")
        .join(hex)
        .join("file")
}

fn cache_get(env: &RepoEnv, hex: &str) -> Option<Vec<u8>> {
    std::fs::read(cache_file(env.repository_cache.as_deref()?, hex)).ok()
}

fn cache_put(env: &RepoEnv, hex: &str, bytes: &[u8]) {
    let Some(root) = env.repository_cache.as_deref() else {
        return;
    };
    let file = cache_file(root, hex);
    if file.exists() {
        return;
    }
    if let Some(dir) = file.parent()
        && std::fs::create_dir_all(dir).is_ok()
    {
        let _ = std::fs::write(&file, bytes);
    }
}

// ---- checksums ------------------------------------------------------------------------

/// What a download is expected to hash to.
enum Expected {
    /// `sha256 = "..."`: hex.
    Hex(String),
    /// `integrity = "algo-base64"`.
    Sri { algo: String, value: String },
}

fn base64_of(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn digest(algo: &str, bytes: &[u8]) -> Option<Vec<u8>> {
    match algo {
        "sha256" => Some(sha2::Sha256::digest(bytes).to_vec()),
        "sha384" => Some(sha2::Sha384::digest(bytes).to_vec()),
        "sha512" => Some(sha2::Sha512::digest(bytes).to_vec()),
        _ => None,
    }
}

impl Expected {
    /// The expected checksum, or Bazel's complaint about it, which it makes
    /// only once it has the file.
    fn check(&self, repo: &str, bytes: &[u8]) -> Result<Option<String>, String> {
        let bad = |why: String| format!("Checksum error in repository @@{repo}: {why}");
        match self {
            Expected::Hex(hex) => {
                if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(bad(format!("Invalid SHA-256 checksum '{hex}'")));
                }
                let actual = hex::encode(sha2::Sha256::digest(bytes));
                Ok((!actual.eq_ignore_ascii_case(hex))
                    .then(|| format!("Checksum was {actual} but wanted {hex}")))
            }
            Expected::Sri { algo, value } => {
                let Some(actual) = digest(algo, bytes) else {
                    return Err(bad(format!(
                        "Unsupported checksum algorithm: '{algo}' (expected SHA-1, SHA-256, \
                         SHA-384, or SHA-512)"
                    )));
                };
                let actual = format!("{algo}-{}", base64_of(&actual));
                let wanted = format!("{algo}-{value}");
                Ok(
                    (actual != wanted)
                        .then(|| format!("Checksum was {actual} but wanted {wanted}")),
                )
            }
        }
    }

    /// The SHA-256 the cache is keyed by, if the expectation is one.
    fn cache_key(&self) -> Option<String> {
        match self {
            Expected::Hex(hex) if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) => {
                Some(hex.to_ascii_lowercase())
            }
            Expected::Sri { algo, value } if algo == "sha256" => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(value)
                    .ok()?;
                (bytes.len() == 32).then(|| hex::encode(bytes))
            }
            _ => None,
        }
    }
}

fn expected_of(sha256: &str, integrity: &str) -> Result<Option<Expected>, String> {
    match (sha256.is_empty(), integrity.is_empty()) {
        (false, false) => Err("Expected either 'sha256' or 'integrity', but not both".to_owned()),
        (false, true) => Ok(Some(Expected::Hex(sha256.to_owned()))),
        (true, false) => Ok(Some(match integrity.split_once('-') {
            Some((algo, value)) => Expected::Sri {
                algo: algo.to_owned(),
                value: value.to_owned(),
            },
            None => Expected::Sri {
                algo: integrity.to_owned(),
                value: String::new(),
            },
        })),
        (true, true) => Ok(None),
    }
}

// ---- fetching -------------------------------------------------------------------------

/// What a finished download says.
#[derive(Debug, Clone)]
struct Info {
    sha256: String,
    integrity: String,
}

/// Why a download did not happen.
enum Failure {
    /// Bazel's `java.io.IOException`: `allow_fail` makes it a result.
    Io(String),
    /// Anything else.
    Fatal(String),
}

struct Request<'a> {
    urls: Vec<String>,
    headers: Vec<(String, Vec<String>)>,
    auth: Vec<(String, Vec<(String, String)>)>,
    sha256: &'a str,
    integrity: &'a str,
}

fn listed(urls: &[String]) -> String {
    format!("[{}]", urls.join(", "))
}

/// The bytes of the URLs, the first that gives them, checked.
fn fetch(env: &RepoEnv, request: &Request<'_>, to: &str) -> Result<(Vec<u8>, Info), Failure> {
    let expected = expected_of(request.sha256, request.integrity).map_err(Failure::Fatal)?;
    if request.urls.is_empty() {
        return Err(Failure::Io("java.io.IOException: urls not set".to_owned()));
    }
    let mut usable: Vec<&String> = Vec::new();
    for url in &request.urls {
        let Some((scheme, _)) = url.split_once("://") else {
            return Err(Failure::Io(format!("java.io.IOException: Bad URL: {url}")));
        };
        if !matches!(scheme, "http" | "https" | "file") {
            return Err(Failure::Io(format!(
                "java.io.IOException: Unsupported protocol: {scheme}"
            )));
        }
        if scheme != "http" || expected.is_some() {
            usable.push(url);
        }
    }
    if usable.is_empty() {
        return Err(Failure::Io(
            "java.io.IOException: No URLs left after removing plain http URLs due to missing \
             checksum. Please provide either a checksum or an https download location."
                .to_owned(),
        ));
    }
    let info_of = |bytes: &[u8]| Info {
        sha256: hex::encode(sha2::Sha256::digest(bytes)),
        integrity: format!("sha256-{}", base64_of(&sha2::Sha256::digest(bytes))),
    };
    // A download that names its SHA-256 is one the cache may already have.
    if let Some(key) = expected.as_ref().and_then(Expected::cache_key)
        && let Some(bytes) = cache_get(env, &key)
    {
        return Ok((bytes.clone(), info_of(&bytes)));
    }
    let mut last = String::new();
    for url in usable {
        let fetched = match url.strip_prefix("file://") {
            Some(path) => std::fs::read(path).map_err(|e| {
                let why = if e.kind() == std::io::ErrorKind::NotFound {
                    "No such file or directory".to_owned()
                } else {
                    e.to_string()
                };
                format!("{path} ({why})")
            }),
            None => match &env.downloader {
                Some(downloader) => downloader.get(&HttpRequest {
                    url: url.clone(),
                    headers: request.headers.clone(),
                    auth: request
                        .auth
                        .iter()
                        .find(|(pattern, _)| pattern == url)
                        .map(|(_, fields)| fields.clone())
                        .unwrap_or_default(),
                }),
                None => Err("no downloader is available".to_owned()),
            },
        };
        let bytes = match fetched {
            Ok(bytes) => bytes,
            Err(why) => {
                tracing::warn!(%url, %why, "download failed");
                last = why;
                continue;
            }
        };
        if let Some(expected) = &expected
            && let Some(mismatch) = expected.check(&env.name, &bytes).map_err(Failure::Fatal)?
        {
            return Err(Failure::Io(format!(
                "java.io.IOException: Error downloading {} to {to}: {mismatch}",
                listed(&request.urls)
            )));
        }
        let info = info_of(&bytes);
        cache_put(env, &info.sha256, &bytes);
        return Ok((bytes, info));
    }
    Err(Failure::Io(format!(
        "java.io.IOException: Error downloading {} to {to}: {last}",
        listed(&request.urls)
    )))
}

// ---- the values -----------------------------------------------------------------------

/// What a download hands back: the success struct.
fn success<'v>(heap: Heap<'v>, info: &Info) -> Value<'v> {
    new_struct(
        heap,
        vec![
            (
                "integrity".to_owned(),
                heap.alloc_str(&info.integrity).to_value(),
            ),
            ("sha256".to_owned(), heap.alloc_str(&info.sha256).to_value()),
            ("success".to_owned(), Value::new_bool(true)),
        ],
    )
}

fn failed<'v>(heap: Heap<'v>) -> Value<'v> {
    new_struct(heap, vec![("success".to_owned(), Value::new_bool(false))])
}

/// A download that was asked not to block: already done, and waiting says how
/// it went.
#[derive(ProvidesStaticType, NoSerialize, Allocative)]
struct PendingDownload {
    #[allocative(skip)]
    outcome: Result<Option<Info>, String>,
}

impl fmt::Debug for PendingDownload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingDownload")
    }
}

impl fmt::Display for PendingDownload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.repository.starlark.\
             StarlarkBaseExternalContext$PendingDownload>"
        )
    }
}

starlark::starlark_simple_value!(PendingDownload);

#[starlark_value(type = "PendingDownload")]
impl<'v> StarlarkValue<'v> for PendingDownload {
    fn get_methods() -> Option<&'static starlark::environment::Methods> {
        static RES: starlark::environment::MethodsStatic =
            starlark::environment::MethodsStatic::new("PendingDownload", pending_members);
        Some(RES.methods())
    }
}

#[starlark::starlark_module]
fn pending_members(builder: &mut starlark::environment::MethodsBuilder) {
    /// Wait for the download and give what it came to.
    fn wait<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let pending = this
            .downcast_ref::<PendingDownload>()
            .expect("wait is called on a PendingDownload");
        match &pending.outcome {
            Ok(Some(info)) => Ok(success(heap, info)),
            Ok(None) => Ok(failed(heap)),
            Err(message) => Err(fatal(message.clone())),
        }
    }
}

// ---- reading the arguments ------------------------------------------------------------

fn is_urls(v: Value<'_>) -> bool {
    v.unpack_str().is_some() || matches!(v.get_type(), "list" | "tuple" | "range" | "set")
}

fn is_str(v: Value<'_>) -> bool {
    v.unpack_str().is_some()
}

fn urls_of<'v>(value: Value<'v>, heap: Heap<'v>) -> starlark::Result<Vec<String>> {
    if let Some(one) = value.unpack_str() {
        return Ok(vec![one.to_owned()]);
    }
    let mut urls = Vec::new();
    for item in crate::attr::sequence(value, heap).unwrap_or_default() {
        match item.unpack_str() {
            Some(s) => urls.push(s.to_owned()),
            None => {
                return Err(fatal(format!(
                    "at index {}, got element of type {}, want string",
                    urls.len(),
                    item.get_type()
                )));
            }
        }
    }
    Ok(urls)
}

/// `headers = {name: value or [values]}`.
fn headers_of(value: Option<Value<'_>>) -> starlark::Result<Vec<(String, Vec<String>)>> {
    let Some(dict) = value.and_then(DictRef::from_value) else {
        return Ok(Vec::new());
    };
    let wrong = || {
        fatal(
            "headers argument must be a dict whose keys are string and whose values are either \
             string or sequence of string",
        )
    };
    let strings = |value: Value<'_>| -> Option<Vec<String>> {
        if let Some(one) = value.unpack_str() {
            return Some(vec![one.to_owned()]);
        }
        let items: Vec<Value<'_>> = match ListRef::from_value(value) {
            Some(list) => list.iter().collect(),
            None => TupleRef::from_value(value)?.iter().collect(),
        };
        items
            .into_iter()
            .map(|i| i.unpack_str().map(str::to_owned))
            .collect()
    };
    let mut out = Vec::new();
    for (key, value) in dict.iter() {
        let key = key.unpack_str().ok_or_else(wrong)?.to_owned();
        out.push((key, strings(value).ok_or_else(wrong)?));
    }
    Ok(out)
}

fn auth_of(value: Option<Value<'_>>) -> Vec<(String, Vec<(String, String)>)> {
    let Some(dict) = value.and_then(DictRef::from_value) else {
        return Vec::new();
    };
    dict.iter()
        .filter_map(|(url, fields)| {
            let fields = DictRef::from_value(fields)?;
            Some((
                url.unpack_str()?.to_owned(),
                fields
                    .iter()
                    .filter_map(|(k, v)| {
                        Some((k.unpack_str()?.to_owned(), v.unpack_str()?.to_owned()))
                    })
                    .collect(),
            ))
        })
        .collect()
}

fn string_arg<'v>(params: &[P], bound: &[Option<Value<'v>>], name: &str) -> &'v str {
    let at = params.iter().position(|p| p.name == name).expect("known");
    bound[at].and_then(|v| v.unpack_str()).unwrap_or_default()
}

fn arg<'v>(params: &[P], bound: &[Option<Value<'v>>], name: &str) -> Option<Value<'v>> {
    let at = params.iter().position(|p| p.name == name).expect("known");
    bound[at]
}

const DOWNLOAD_PARAMS: &[P] = &[
    p("url", true, true, "string or Iterable", is_urls),
    p(
        "output",
        true,
        false,
        "string, Label, or path",
        is_path_like,
    ),
    p("sha256", true, false, "string", is_str),
    p("executable", true, false, "bool", is_bool),
    p("allow_fail", true, false, "bool", is_bool),
    p("canonical_id", true, false, "string", is_str),
    p("auth", true, false, "dict", is_dict),
    p("headers", true, false, "dict", is_dict),
    p("integrity", false, false, "string", is_str),
    p("block", false, false, "bool", is_bool),
];

const EXTRACT_AND_DOWNLOAD_PARAMS: &[P] = &[
    p("url", true, true, "string or Iterable", is_urls),
    p(
        "output",
        true,
        false,
        "string, Label, or path",
        is_path_like,
    ),
    p("sha256", true, false, "string", is_str),
    p("type", true, false, "string", is_str),
    p("strip_prefix", true, false, "string", is_str),
    p("allow_fail", true, false, "bool", is_bool),
    p("canonical_id", true, false, "string", is_str),
    p("auth", true, false, "dict", is_dict),
    p("headers", true, false, "dict", is_dict),
    p("integrity", false, false, "string", is_str),
    p("rename_files", false, false, "dict", is_dict),
    p("strip_components", false, false, "int", is_int),
];

const EXTRACT_PARAMS: &[P] = &[
    p(
        "archive",
        true,
        true,
        "string, Label, or path",
        is_path_like,
    ),
    p(
        "output",
        true,
        false,
        "string, Label, or path",
        is_path_like,
    ),
    p("strip_prefix", true, false, "string", is_str),
    p("rename_files", false, false, "dict", is_dict),
    p("watch_archive", false, false, "string", is_str),
    p("strip_components", false, false, "int", is_int),
];

const PATCH_PARAMS: &[P] = &[
    path_arg("patch_file"),
    p("strip", true, false, "int", is_int),
    p("watch_patch", false, false, "string", is_str),
];

fn is_int(v: Value<'_>) -> bool {
    v.unpack_i32().is_some()
}

fn renames_of(value: Option<Value<'_>>) -> starlark::Result<Vec<(String, String)>> {
    let Some(dict) = value.and_then(DictRef::from_value) else {
        return Ok(Vec::new());
    };
    if let Some((k, v)) = dict
        .iter()
        .find(|(k, v)| k.unpack_str().is_none() || v.unpack_str().is_none())
    {
        return Err(fatal(format!(
            "got dict<{}, {}> for 'rename_files', want dict<string, string>",
            k.get_type(),
            v.get_type()
        )));
    }
    Ok(dict
        .iter()
        .map(|(k, v)| {
            (
                k.unpack_str().expect("checked").to_owned(),
                v.unpack_str().expect("checked").to_owned(),
            )
        })
        .collect())
}

/// Write the downloaded bytes where `output` says.
fn write_output(
    resolved: &Resolved,
    urls: &[String],
    bytes: &[u8],
    executable: bool,
) -> Result<(), Failure> {
    let path = &resolved.path;
    let problem = |why: String| {
        Failure::Io(format!(
            "java.io.IOException: Error downloading {} to {}: {why}",
            listed(urls),
            path.display()
        ))
    };
    if path.is_dir() {
        return Err(problem(format!("{} (Is a directory)", path.display())));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| problem(e.to_string()))?;
    }
    let _ = std::fs::remove_file(path);
    std::fs::write(path, bytes).map_err(|e| problem(e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|e| problem(e.to_string()))?;
    }
    #[cfg(not(unix))]
    let _ = executable;
    Ok(())
}

// ---- the operations -------------------------------------------------------------------

pub(crate) fn op_download<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let env = env_of(this);
    let bound = bind_checked("download", DOWNLOAD_PARAMS, args, eval)?;
    let heap = eval.heap();
    let urls = urls_of(arg(DOWNLOAD_PARAMS, &bound, "url").expect("required"), heap)?;
    let output = match arg(DOWNLOAD_PARAMS, &bound, "output") {
        Some(o) => resolve(env, o)?,
        None => resolve(env, heap.alloc_str("").to_value())?,
    };
    writable(env, &output)?;
    let executable = flag(arg(DOWNLOAD_PARAMS, &bound, "executable"), false);
    let allow_fail = flag(arg(DOWNLOAD_PARAMS, &bound, "allow_fail"), false);
    let block = flag(arg(DOWNLOAD_PARAMS, &bound, "block"), true);
    let request = Request {
        urls: urls.clone(),
        headers: headers_of(arg(DOWNLOAD_PARAMS, &bound, "headers"))?,
        auth: auth_of(arg(DOWNLOAD_PARAMS, &bound, "auth")),
        sha256: string_arg(DOWNLOAD_PARAMS, &bound, "sha256"),
        integrity: string_arg(DOWNLOAD_PARAMS, &bound, "integrity"),
    };
    let outcome =
        fetch(env, &request, &output.path.display().to_string()).and_then(|(bytes, info)| {
            write_output(&output, &urls, &bytes, executable)?;
            Ok(info)
        });
    let outcome: Result<Option<Info>, String> = match outcome {
        Ok(info) => Ok(Some(info)),
        Err(Failure::Io(_)) if allow_fail => Ok(None),
        Err(Failure::Io(message) | Failure::Fatal(message)) => Err(message),
    };
    if !block {
        return Ok(heap.alloc(PendingDownload { outcome }));
    }
    match outcome {
        Ok(Some(info)) => Ok(success(heap, &info)),
        Ok(None) => Ok(failed(heap)),
        Err(message) => Err(fatal(message)),
    }
}

/// The last path segment of a URL, without a query: what a downloaded archive
/// is called.
fn file_name_of(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let path = path.split_once("://").map_or(path, |(_, rest)| rest);
    path.rsplit('/').next().unwrap_or(path).to_owned()
}

fn extraction_message(archive: &Path, to: &Path, why: &str) -> String {
    format!(
        "java.io.IOException: Error extracting {} to {}: {why}",
        archive.display(),
        to.display()
    )
}

fn unknown_suffix(path: &Path) -> starlark::Error {
    fatal(format!(
        "Expected a file with a {} suffix (got {})",
        fjfj_archive::SUFFIXES,
        path.display()
    ))
}

/// `strip_components`, checked against `strip_prefix` the way Bazel does.
fn strip_components_of(
    function: &str,
    strip_prefix: &str,
    value: Option<Value<'_>>,
) -> starlark::Result<usize> {
    let components = value.and_then(|v| v.unpack_i32()).unwrap_or(0);
    if components < 0 {
        return Err(fatal(format!(
            "{function}() has an invalid argument for 'strip_components': {components}. Must be \
             non-negative."
        )));
    }
    if components > 0 && !strip_prefix.is_empty() {
        return Err(fatal(format!(
            "{function}() got multiple strip values. Only one of 'strip_prefix' or \
             'strip_components' can be set"
        )));
    }
    Ok(components as usize)
}

fn unpack(
    archive: &Path,
    output: &Path,
    shown_to: &Path,
    strip_prefix: &str,
    strip_components: usize,
    rename: &[(String, String)],
) -> starlark::Result<()> {
    let name = archive
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Some(format) = fjfj_archive::format_for(&name) else {
        return Err(unknown_suffix(archive));
    };
    fjfj_archive::extract(&fjfj_archive::ExtractRequest {
        archive,
        format,
        output,
        strip_prefix,
        strip_components,
        rename,
    })
    .map_err(|why| fatal(extraction_message(archive, shown_to, &why)))
}

pub(crate) fn op_download_and_extract<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let env = env_of(this);
    let params = EXTRACT_AND_DOWNLOAD_PARAMS;
    let bound = bind_checked("download_and_extract", params, args, eval)?;
    let heap = eval.heap();
    let urls = urls_of(arg(params, &bound, "url").expect("required"), heap)?;
    let output = match arg(params, &bound, "output") {
        Some(o) => resolve(env, o)?,
        None => resolve(env, heap.alloc_str("").to_value())?,
    };
    writable(env, &output)?;
    let rename = renames_of(arg(params, &bound, "rename_files"))?;
    let components = strip_components_of(
        "download_and_extract",
        string_arg(params, &bound, "strip_prefix"),
        arg(params, &bound, "strip_components"),
    )?;
    let allow_fail = flag(arg(params, &bound, "allow_fail"), false);
    let kind = string_arg(params, &bound, "type");
    let temp = output.path.join(format!("temp{}", temp_number()));
    let mut name = file_name_of(urls.first().map_or("", String::as_str));
    if !kind.is_empty() {
        name = format!("{name}.{}", kind.trim_start_matches('.'));
    }
    let archive = Resolved {
        path: temp.join(&name),
        label_path: None,
    };
    let request = Request {
        urls: urls.clone(),
        headers: headers_of(arg(params, &bound, "headers"))?,
        auth: auth_of(arg(params, &bound, "auth")),
        sha256: string_arg(params, &bound, "sha256"),
        integrity: string_arg(params, &bound, "integrity"),
    };
    std::fs::create_dir_all(&temp).map_err(|e| io_error(&e))?;
    let outcome =
        fetch(env, &request, &archive.path.display().to_string()).and_then(|(bytes, info)| {
            write_output(&archive, &urls, &bytes, false)?;
            Ok(info)
        });
    let info = match outcome {
        Ok(info) => info,
        // The directory stays behind when the download is let to fail.
        Err(Failure::Io(_)) if allow_fail => return Ok(failed(heap)),
        Err(Failure::Io(message) | Failure::Fatal(message)) => return Err(fatal(message)),
    };
    unpack(
        &archive.path,
        &output.path,
        &temp,
        string_arg(params, &bound, "strip_prefix"),
        components,
        &rename,
    )?;
    let _ = std::fs::remove_dir_all(&temp);
    Ok(success(heap, &info))
}

/// A number for the name of a temporary directory, which differs each time.
fn temp_number() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    nanos
        .wrapping_mul(1_000_003)
        .wrapping_add(NEXT.fetch_add(1, Ordering::Relaxed))
}

pub(crate) fn op_extract<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let env = env_of(this);
    let params = EXTRACT_PARAMS;
    let bound = bind_checked("extract", params, args, eval)?;
    let heap = eval.heap();
    let archive = resolve(env, arg(params, &bound, "archive").expect("required"))?;
    let output = match arg(params, &bound, "output") {
        Some(o) => resolve(env, o)?,
        None => resolve(env, heap.alloc_str("").to_value())?,
    };
    writable(env, &output)?;
    check_watch(env, &archive, arg(params, &bound, "watch_archive"))?;
    let rename = renames_of(arg(params, &bound, "rename_files"))?;
    let components = strip_components_of(
        "extract",
        string_arg(params, &bound, "strip_prefix"),
        arg(params, &bound, "strip_components"),
    )?;
    if !archive.path.exists() {
        return Err(fatal(format!(
            "Archive path '{}' does not exist.",
            archive.path.display()
        )));
    }
    unpack(
        &archive.path,
        &output.path,
        &output.path,
        string_arg(params, &bound, "strip_prefix"),
        components,
        &rename,
    )?;
    Ok(NoneType)
}

pub(crate) fn op_patch<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let env = env_of(this);
    let bound = bind_checked("patch", PATCH_PARAMS, args, eval)?;
    let file = resolve(
        env,
        arg(PATCH_PARAMS, &bound, "patch_file").expect("required"),
    )?;
    check_watch(env, &file, arg(PATCH_PARAMS, &bound, "watch_patch"))?;
    let strip = arg(PATCH_PARAMS, &bound, "strip")
        .and_then(|s| s.unpack_i32())
        .unwrap_or(0)
        .max(0) as usize;
    let failed_with = |why: &str| {
        fatal(format!(
            "Error applying patch {}: {why}",
            file.path.display()
        ))
    };
    if !file.path.is_file() {
        return Err(failed_with(&format!(
            "Cannot find patch file: {}",
            file.path.display()
        )));
    }
    let text = read_text(&file)?;
    fjfj_archive::apply_patch(&text, strip, &env.output).map_err(|why| failed_with(&why))?;
    Ok(NoneType)
}
