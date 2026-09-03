//! The registry client: how fjfj asks a Bazel registry for a module file
//! and for the recipe to fetch that module's source.
//!
//! Ported from Bazel 9.2.0's `IndexRegistry.java`. A registry is a plain
//! index of files under a base URL — there is no API, so "the client" is
//! URL construction plus JSON parsing plus integrity checking:
//!
//! ```text
//! <registry>/bazel_registry.json                       # optional: mirrors, module_base_path
//! <registry>/modules/<name>/metadata.json              # versions, yanked versions
//! <registry>/modules/<name>/<version>/MODULE.bazel     # the module file
//! <registry>/modules/<name>/<version>/source.json      # how to fetch the source
//! <registry>/modules/<name>/<version>/patches/<name>   # registry-supplied patches
//! ```
//!
//! Transport is behind [`Fetcher`] rather than baked in, for two reasons:
//! a `file://` registry is a first-class case (it is how the BCR's own
//! tests run, and how fjfj's conformance fixtures work), and the HTTP
//! downloader is shared with repository rules, which are a separate bead.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use sha2::Digest as _;

use crate::error::{BzlmodError, Result};
use crate::module::ModuleKey;
use crate::overrides::{RepoRule, RepoSpec};
use crate::version::Version;

/// Fetches bytes for a URL. `Ok(None)` means "not there" — a 404 or a
/// missing file — which a registry treats as "ask the next registry",
/// not as a failure.
pub trait Fetcher: Send + Sync {
    fn fetch(&self, url: &str) -> Result<Option<Vec<u8>>>;
}

/// Reads `file://` URLs and bare paths. The registry used by fjfj's own
/// tests, and by anyone running a registry out of a directory.
#[derive(Debug, Default, Clone)]
pub struct FileFetcher;

impl Fetcher for FileFetcher {
    fn fetch(&self, url: &str) -> Result<Option<Vec<u8>>> {
        let path = file_url_to_path(url).ok_or_else(|| BzlmodError::Registry {
            registry: url.to_owned(),
            message: "not a file:// URL".to_owned(),
        })?;
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(BzlmodError::Registry {
                registry: url.to_owned(),
                message: e.to_string(),
            }),
        }
    }
}

/// `file:///a/b` and `/a/b` both denote the path `/a/b`; anything else is
/// not a local file.
fn file_url_to_path(url: &str) -> Option<PathBuf> {
    if let Some(rest) = url.strip_prefix("file://") {
        // `file:///a/b` has an empty authority; keep the leading slash.
        let path = rest.strip_prefix("localhost").unwrap_or(rest);
        return Some(PathBuf::from(path));
    }
    if url.starts_with('/') {
        return Some(PathBuf::from(url));
    }
    None
}

/// Fetches over HTTP(S), and over `file://` for URLs that name a local
/// path — a registry list can mix the two, and Bazel's `--registry` flag
/// accepts both.
///
/// `reqwest` with rustls, rather than a bespoke client, because
/// repository rules will need exactly the same downloader
/// (`repository_ctx.download`, `http_archive`) and one HTTP stack for the
/// whole tool beats two.
pub struct HttpFetcher {
    client: reqwest::blocking::Client,
    files: FileFetcher,
}

impl std::fmt::Debug for HttpFetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HttpFetcher")
    }
}

impl HttpFetcher {
    pub fn new() -> Result<HttpFetcher> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(concat!("fjfj/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| BzlmodError::Registry {
                registry: "<http client>".to_owned(),
                message: e.to_string(),
            })?;
        Ok(HttpFetcher {
            client,
            files: FileFetcher,
        })
    }
}

impl Fetcher for HttpFetcher {
    fn fetch(&self, url: &str) -> Result<Option<Vec<u8>>> {
        if file_url_to_path(url).is_some() {
            return self.files.fetch(url);
        }
        let registry_error = |message: String| BzlmodError::Registry {
            registry: url.to_owned(),
            message,
        };
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| registry_error(e.to_string()))?;
        // A missing file is how a registry says "I don't have this
        // module"; the caller moves on to the next registry.
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let response = response
            .error_for_status()
            .map_err(|e| registry_error(e.to_string()))?;
        let bytes = response
            .bytes()
            .map_err(|e| registry_error(e.to_string()))?;
        Ok(Some(bytes.to_vec()))
    }
}

/// A Bazel module registry addressed by a base URL.
pub struct Registry {
    url: String,
    fetcher: Box<dyn Fetcher>,
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry").field("url", &self.url).finish()
    }
}

/// The default registry, as in Bazel's `--registry` default.
pub const BAZEL_CENTRAL_REGISTRY: &str = "https://bcr.bazel.build";

impl Registry {
    pub fn new(url: impl Into<String>, fetcher: Box<dyn Fetcher>) -> Registry {
        Registry {
            url: url.into(),
            fetcher,
        }
    }

    /// A registry served out of a local directory.
    pub fn local(path: impl AsRef<Path>) -> Registry {
        Registry::new(
            format!("file://{}", path.as_ref().display()),
            Box::new(FileFetcher),
        )
    }

    /// A registry at a URL, over HTTP(S) or `file://`. This is what a
    /// `--registry` flag turns into.
    pub fn remote(url: impl Into<String>) -> Result<Registry> {
        Ok(Registry::new(url, Box::new(HttpFetcher::new()?)))
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// The module file for one version, or `None` if this registry does
    /// not carry it.
    pub fn module_file(&self, key: &ModuleKey) -> Result<Option<String>> {
        let url = self.module_file_url(key);
        let Some(bytes) = self.fetcher.fetch(&url)? else {
            return Ok(None);
        };
        let text = String::from_utf8(bytes).map_err(|_| BzlmodError::Registry {
            registry: self.url.clone(),
            message: format!("{url} is not valid UTF-8"),
        })?;
        Ok(Some(text))
    }

    pub fn module_file_url(&self, key: &ModuleKey) -> String {
        join_url(
            &self.url,
            &["modules", &key.name, key.version.as_str(), "MODULE.bazel"],
        )
    }

    /// Reads `metadata.json`, which lists a module's versions and which of
    /// them have been yanked.
    pub fn metadata(&self, module_name: &str) -> Result<Option<Metadata>> {
        let url = join_url(&self.url, &["modules", module_name, "metadata.json"]);
        let Some(bytes) = self.fetcher.fetch(&url)? else {
            return Ok(None);
        };
        let json: MetadataJson = self.parse_json(&url, &bytes)?;
        let mut yanked = BTreeMap::new();
        for (version, reason) in json.yanked_versions.unwrap_or_default() {
            let version = Version::parse(&version).map_err(|e| BzlmodError::Registry {
                registry: self.url.clone(),
                message: format!("bad yanked version in {url}: {e}"),
            })?;
            yanked.insert(version, reason);
        }
        Ok(Some(Metadata {
            versions: json.versions.unwrap_or_default(),
            yanked_versions: yanked,
        }))
    }

    /// Reads `source.json` and turns it into the repo rule call that would
    /// materialise the module. Running that call is the fetch phase's job.
    pub fn repo_spec(&self, key: &ModuleKey) -> Result<RepoSpec> {
        let url = join_url(
            &self.url,
            &["modules", &key.name, key.version.as_str(), "source.json"],
        );
        let bytes = self
            .fetcher
            .fetch(&url)?
            .ok_or_else(|| BzlmodError::Registry {
                registry: self.url.clone(),
                message: format!("module {key}'s source.json not found"),
            })?;
        let source: SourceJson = self.parse_json(&url, &bytes)?;
        match source.type_.as_deref().unwrap_or("archive") {
            "archive" => self.archive_repo_spec(key, &bytes, &url),
            "local_path" => self.local_path_repo_spec(key, &bytes, &url),
            "git_repository" => self.git_repo_spec(&bytes, &url),
            other => Err(BzlmodError::Registry {
                registry: self.url.clone(),
                message: format!("invalid source type \"{other}\" for module {key}"),
            }),
        }
    }

    fn archive_repo_spec(&self, key: &ModuleKey, bytes: &[u8], url: &str) -> Result<RepoSpec> {
        let json: ArchiveSourceJson = self.parse_json(url, bytes)?;
        let source_url = json.url.ok_or_else(|| BzlmodError::Registry {
            registry: self.url.clone(),
            message: format!("missing source URL for module {key}"),
        })?;
        let integrity = json.integrity.ok_or_else(|| BzlmodError::Registry {
            registry: self.url.clone(),
            message: format!("missing integrity for module {key}"),
        })?;

        // Mirrors are prefixes: the source URL's authority and path get
        // appended to each, and the original URL goes last as the
        // fallback.
        let mut urls: Vec<String> = Vec::new();
        for mirror in self.bazel_registry_json()?.mirrors.unwrap_or_default() {
            urls.push(mirror_url(&mirror, &source_url));
        }
        urls.push(source_url);
        urls.extend(json.mirror_urls.unwrap_or_default());

        let mut attrs = vec![
            (
                "urls".to_owned(),
                crate::attrs::AttrValue::List(
                    urls.into_iter()
                        .map(crate::attrs::AttrValue::String)
                        .collect(),
                ),
            ),
            (
                "integrity".to_owned(),
                crate::attrs::AttrValue::String(integrity),
            ),
        ];
        if let Some(strip_prefix) = json.strip_prefix {
            attrs.push((
                "strip_prefix".to_owned(),
                crate::attrs::AttrValue::String(strip_prefix),
            ));
        }
        // Registry-supplied patches are named relative to the module's
        // directory in the registry, so they become absolute URLs here.
        if let Some(patches) = json.patches
            && !patches.is_empty()
        {
            let remote_patches = patches
                .into_iter()
                .map(|(name, integrity)| {
                    (
                        crate::attrs::AttrValue::String(join_url(
                            &self.url,
                            &["modules", &key.name, key.version.as_str(), "patches", &name],
                        )),
                        crate::attrs::AttrValue::String(integrity),
                    )
                })
                .collect();
            attrs.push((
                "remote_patches".to_owned(),
                crate::attrs::AttrValue::Dict(remote_patches),
            ));
            attrs.push((
                "remote_patch_strip".to_owned(),
                crate::attrs::AttrValue::Int(json.patch_strip.unwrap_or(0).into()),
            ));
        }
        if let Some(archive_type) = json.archive_type.filter(|t| !t.is_empty()) {
            attrs.push((
                "type".to_owned(),
                crate::attrs::AttrValue::String(archive_type),
            ));
        }
        Ok(RepoSpec {
            rule: RepoRule::HttpArchive,
            attrs,
        })
    }

    fn local_path_repo_spec(&self, key: &ModuleKey, bytes: &[u8], url: &str) -> Result<RepoSpec> {
        let json: LocalPathSourceJson = self.parse_json(url, bytes)?;
        let path = json.path.ok_or_else(|| BzlmodError::Registry {
            registry: self.url.clone(),
            message: format!("missing path for module {key}"),
        })?;
        // A relative path is relative to `module_base_path`, and that in
        // turn may be relative to the registry — which only makes sense
        // for a local registry.
        let path = if Path::new(&path).is_absolute() {
            path
        } else {
            let base = self
                .bazel_registry_json()?
                .module_base_path
                .ok_or_else(|| BzlmodError::Registry {
                    registry: self.url.clone(),
                    message: format!(
                        "module {key} has a relative source path but the registry has no \
                         module_base_path"
                    ),
                })?;
            if Path::new(&base).is_absolute() {
                format!("{base}/{path}")
            } else {
                let registry_path =
                    file_url_to_path(&self.url).ok_or_else(|| BzlmodError::Registry {
                        registry: self.url.clone(),
                        message: format!("provided non-local registry for module {key}"),
                    })?;
                format!("{}/{base}/{path}", registry_path.display())
            }
        };
        Ok(RepoSpec {
            rule: RepoRule::LocalRepository,
            attrs: vec![("path".to_owned(), crate::attrs::AttrValue::String(path))],
        })
    }

    fn git_repo_spec(&self, bytes: &[u8], url: &str) -> Result<RepoSpec> {
        let json: GitSourceJson = self.parse_json(url, bytes)?;
        let mut attrs = Vec::new();
        let mut push_str = |name: &str, value: Option<String>| {
            if let Some(value) = value {
                attrs.push((name.to_owned(), crate::attrs::AttrValue::String(value)));
            }
        };
        push_str("remote", json.remote);
        push_str("commit", json.commit);
        push_str("shallow_since", json.shallow_since);
        push_str("tag", json.tag);
        push_str("strip_prefix", json.strip_prefix);
        if let Some(init_submodules) = json.init_submodules {
            // Bazel sets both attributes from the one JSON field.
            attrs.push((
                "init_submodules".to_owned(),
                crate::attrs::AttrValue::Bool(init_submodules),
            ));
            attrs.push((
                "recursive_init_submodules".to_owned(),
                crate::attrs::AttrValue::Bool(init_submodules),
            ));
        }
        if let Some(verbose) = json.verbose {
            attrs.push(("verbose".to_owned(), crate::attrs::AttrValue::Bool(verbose)));
        }
        Ok(RepoSpec {
            rule: RepoRule::GitRepository,
            attrs,
        })
    }

    fn bazel_registry_json(&self) -> Result<BazelRegistryJson> {
        let url = join_url(&self.url, &["bazel_registry.json"]);
        match self.fetcher.fetch(&url)? {
            Some(bytes) => self.parse_json(&url, &bytes),
            None => Ok(BazelRegistryJson::default()),
        }
    }

    fn parse_json<T: serde::de::DeserializeOwned>(&self, url: &str, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes).map_err(|e| BzlmodError::Registry {
            registry: self.url.clone(),
            message: format!("unable to parse json at url {url}: {e}"),
        })
    }
}

/// What `metadata.json` says about a module.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub versions: Vec<String>,
    /// Yanked version to the reason it was yanked. A yanked version is
    /// still served; it is selection that must refuse it unless the user
    /// allowed it.
    pub yanked_versions: BTreeMap<Version, String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct BazelRegistryJson {
    mirrors: Option<Vec<String>>,
    module_base_path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MetadataJson {
    versions: Option<Vec<String>>,
    yanked_versions: Option<BTreeMap<String, String>>,
}

#[derive(Debug, serde::Deserialize)]
struct SourceJson {
    #[serde(rename = "type")]
    type_: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ArchiveSourceJson {
    url: Option<String>,
    mirror_urls: Option<Vec<String>>,
    integrity: Option<String>,
    strip_prefix: Option<String>,
    patches: Option<BTreeMap<String, String>>,
    patch_strip: Option<i32>,
    archive_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct LocalPathSourceJson {
    path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct GitSourceJson {
    remote: Option<String>,
    commit: Option<String>,
    shallow_since: Option<String>,
    tag: Option<String>,
    init_submodules: Option<bool>,
    verbose: Option<bool>,
    strip_prefix: Option<String>,
}

/// Bazel's `constructUrl`: join with exactly one slash between segments.
fn join_url(base: &str, segments: &[&str]) -> String {
    let mut url = base.to_owned();
    for segment in segments {
        if !url.ends_with('/') && !segment.starts_with('/') {
            url.push('/');
        }
        url.push_str(segment);
    }
    url
}

/// A mirror URL is the mirror prefix with the source URL's authority and
/// path appended — `https://mirror/` + `example.com/a.tar.gz`.
fn mirror_url(mirror: &str, source_url: &str) -> String {
    let (authority, path, query) = split_url(source_url);
    let joined = join_url(mirror, &[authority, path]);
    match query {
        Some(query) => format!("{joined}?{query}"),
        None => joined,
    }
}

fn split_url(url: &str) -> (&str, &str, Option<&str>) {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let (before_query, query) = match after_scheme.split_once('?') {
        Some((before, query)) => (before, Some(query)),
        None => (after_scheme, None),
    };
    match before_query.split_once('/') {
        Some((authority, path)) => (authority, path, query),
        None => (before_query, "", query),
    }
}

/// Subresource Integrity as the registry writes it: `sha256-<base64>`.
///
/// This is the only thing standing between a registry index and arbitrary
/// code execution, so it is checked here rather than left to the fetcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Integrity {
    pub algorithm: String,
    pub digest: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IntegrityError {
    #[error("invalid integrity string '{0}': expected <algorithm>-<base64 digest>")]
    Malformed(String),
    #[error("unsupported integrity algorithm '{0}'")]
    UnsupportedAlgorithm(String),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },
}

impl Integrity {
    pub fn parse(value: &str) -> std::result::Result<Integrity, IntegrityError> {
        let (algorithm, encoded) = value
            .split_once('-')
            .ok_or_else(|| IntegrityError::Malformed(value.to_owned()))?;
        if !matches!(algorithm, "sha256" | "sha384" | "sha512") {
            return Err(IntegrityError::UnsupportedAlgorithm(algorithm.to_owned()));
        }
        let digest = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| IntegrityError::Malformed(value.to_owned()))?;
        Ok(Integrity {
            algorithm: algorithm.to_owned(),
            digest,
        })
    }

    pub fn verify(&self, content: &[u8]) -> std::result::Result<(), IntegrityError> {
        let actual = match self.algorithm.as_str() {
            "sha256" => sha2::Sha256::digest(content).to_vec(),
            "sha384" => sha2::Sha384::digest(content).to_vec(),
            "sha512" => sha2::Sha512::digest(content).to_vec(),
            other => return Err(IntegrityError::UnsupportedAlgorithm(other.to_owned())),
        };
        if actual == self.digest {
            Ok(())
        } else {
            Err(IntegrityError::Mismatch {
                expected: self.to_string(),
                actual: format!(
                    "{}-{}",
                    self.algorithm,
                    base64::engine::general_purpose::STANDARD.encode(&actual)
                ),
            })
        }
    }
}

impl std::fmt::Display for Integrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}",
            self.algorithm,
            base64::engine::general_purpose::STANDARD.encode(&self.digest)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_urls_with_one_slash() {
        assert_eq!(
            join_url("https://bcr.bazel.build", &["modules", "foo"]),
            "https://bcr.bazel.build/modules/foo"
        );
        assert_eq!(
            join_url("https://bcr.bazel.build/", &["modules", "foo"]),
            "https://bcr.bazel.build/modules/foo"
        );
    }

    #[test]
    fn mirrors_prefix_the_source_url() {
        assert_eq!(
            mirror_url(
                "https://mirror.example",
                "https://github.com/a/b/archive/v1.tar.gz"
            ),
            "https://mirror.example/github.com/a/b/archive/v1.tar.gz"
        );
        assert_eq!(
            mirror_url("https://mirror.example/", "https://host/p?q=1"),
            "https://mirror.example/host/p?q=1"
        );
    }

    #[test]
    fn parses_and_verifies_sri_integrity() {
        let content = b"hello";
        let digest = sha2::Sha256::digest(content);
        let value = format!(
            "sha256-{}",
            base64::engine::general_purpose::STANDARD.encode(digest)
        );
        let integrity = Integrity::parse(&value).unwrap();
        integrity.verify(content).unwrap();
        assert!(matches!(
            integrity.verify(b"goodbye"),
            Err(IntegrityError::Mismatch { .. })
        ));
        assert_eq!(integrity.to_string(), value);
    }

    #[test]
    fn rejects_bad_integrity_strings() {
        assert!(matches!(
            Integrity::parse("sha256"),
            Err(IntegrityError::Malformed(_))
        ));
        assert!(matches!(
            Integrity::parse("md5-YWJj"),
            Err(IntegrityError::UnsupportedAlgorithm(_))
        ));
        assert!(matches!(
            Integrity::parse("sha256-not base64!"),
            Err(IntegrityError::Malformed(_))
        ));
    }

    #[test]
    fn file_urls_map_to_paths() {
        assert_eq!(
            file_url_to_path("file:///tmp/registry"),
            Some(PathBuf::from("/tmp/registry"))
        );
        assert_eq!(
            file_url_to_path("/tmp/registry"),
            Some(PathBuf::from("/tmp/registry"))
        );
        assert_eq!(file_url_to_path("https://bcr.bazel.build"), None);
    }
}
