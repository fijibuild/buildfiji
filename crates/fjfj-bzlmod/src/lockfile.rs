//! `MODULE.bazel.lock`: what Bazel 9.2.0 writes next to `MODULE.bazel`, and
//! what it does with it on the next run.
//!
//! Read off probes against Bazel 9.2.0 (lockfile version 28), not its
//! documentation:
//!
//! - The file is JSON, two-space indented, with a trailing newline and the
//!   keys `lockFileVersion`, `registryFileHashes`, `selectedYankedVersions`,
//!   `moduleExtensions`, `facts` and `factsVersions`, in that order. An
//!   empty object or array is written inline (`{}`, `[]`). There is no
//!   module graph in it: version 28 re-resolves every time.
//! - `registryFileHashes` maps the URL of every registry file read over the
//!   network to the SHA-256 of its content, or to `"not found"` for a file
//!   the registry did not have. Keys are sorted. Only `bazel_registry.json`,
//!   `MODULE.bazel` and (for selected modules) `source.json` are recorded;
//!   `metadata.json` is not, and `file://` registries are not recorded at
//!   all. A file whose content no longer matches its recorded hash is an
//!   error (`Checksum was X but wanted Y`); a file recorded as not found is
//!   not asked for again, and the error for the module says so.
//! - Only what a run read is written back: entries no longer used are
//!   dropped.
//! - `selectedYankedVersions` holds `name@version` and the reason for each
//!   selected version the registry has yanked (whether or not it was
//!   allowed). A run that finds an entry for a module there does not fetch
//!   that module's `metadata.json`.
//! - `--lockfile_mode` is `off` (neither read nor written), `update`,
//!   `refresh` (like `update`, and additionally asks again for files
//!   recorded as not found and for `metadata.json`), or `error` (never
//!   written; a registry file with no recorded hash is an error).
//! - A lockfile that is not version 28, or is not JSON, is not used:
//!   `update` and `refresh` replace it, `error` refuses it.
//!
//! What is here and what is not: `factsVersions` is carried through untouched
//! and `facts` is what extensions kept ([`LockSession::set_facts`]), merged as
//! `moduleExtensions` is. `moduleExtensions` is what the
//! extensions a run evaluated give ([`LockSession::set_module_extensions`],
//! from `fjfj-repo`'s `LockedExtension`s): the file's entries for extensions the
//! graph still uses stay, the others go, and the ids and factors are sorted. One known difference:
//! Bazel writes `selectedYankedVersions` in a Java `HashMap` iteration
//! order, fjfj in selection order, so the two agree only when at most one
//! version is yanked (buildfiji-avh).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest as _;

use crate::error::{BzlmodError, Result};
use crate::module::ModuleKey;
use crate::registry::Fetcher;

/// The lockfile version Bazel 9.2.0 reads and writes.
pub const LOCK_FILE_VERSION: u64 = 28;

/// What `--lockfile_mode` said.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockfileMode {
    Off,
    #[default]
    Update,
    Refresh,
    Error,
}

impl LockfileMode {
    /// `--lockfile_mode`'s value; the error is Bazel's own wording.
    pub fn parse(value: &str) -> std::result::Result<LockfileMode, String> {
        match value {
            "off" => Ok(LockfileMode::Off),
            "update" => Ok(LockfileMode::Update),
            "refresh" => Ok(LockfileMode::Refresh),
            "error" => Ok(LockfileMode::Error),
            other => Err(format!(
                "Invalid value '{other}' for option 'lockfile_mode': valid values are off, \
                 update, refresh, error"
            )),
        }
    }

    /// Whether a changed lockfile is written back.
    pub fn writes(self) -> bool {
        matches!(self, LockfileMode::Update | LockfileMode::Refresh)
    }
}

/// What the lockfile knows about one registry file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileHash {
    /// Lower-case hex SHA-256 of the content.
    Sha256(String),
    /// The registry had no such file.
    NotFound,
}

const NOT_FOUND: &str = "not found";

/// JSON with its object keys in the order they were read, which is the
/// order Bazel wrote them in (neither sorted nor a map's own order).
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn empty_object() -> Json {
        Json::Object(Vec::new())
    }
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Json, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Json;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_unit<E>(self) -> std::result::Result<Json, E> {
                Ok(Json::Null)
            }
            fn visit_bool<E>(self, v: bool) -> std::result::Result<Json, E> {
                Ok(Json::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> std::result::Result<Json, E> {
                Ok(Json::Number(v.into()))
            }
            fn visit_u64<E>(self, v: u64) -> std::result::Result<Json, E> {
                Ok(Json::Number(v.into()))
            }
            fn visit_f64<E>(self, v: f64) -> std::result::Result<Json, E> {
                Ok(serde_json::Number::from_f64(v).map_or(Json::Null, Json::Number))
            }
            fn visit_str<E>(self, v: &str) -> std::result::Result<Json, E> {
                Ok(Json::String(v.to_owned()))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<Json, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = a.next_element()? {
                    items.push(item);
                }
                Ok(Json::Array(items))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> std::result::Result<Json, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = a.next_entry()? {
                    entries.push(entry);
                }
                Ok(Json::Object(entries))
            }
        }
        d.deserialize_any(V)
    }
}

impl Serialize for Json {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Json::Null => s.serialize_unit(),
            Json::Bool(b) => s.serialize_bool(*b),
            Json::Number(n) => n.serialize(s),
            Json::String(v) => s.serialize_str(v),
            Json::Array(items) => {
                let mut seq = s.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Json::Object(entries) => {
                let mut map = s.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

/// Why a `MODULE.bazel.lock` could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unusable {
    /// Not JSON, or not in the shape of any version Bazel wrote.
    Malformed,
    /// A version this Bazel does not read (0 when the field is absent).
    Version(u64),
}

/// A parsed `MODULE.bazel.lock`.
#[derive(Debug, Clone, PartialEq)]
pub struct Lockfile {
    pub registry_file_hashes: BTreeMap<String, FileHash>,
    /// `name@version` and the reason, in file order.
    pub selected_yanked_versions: Vec<(String, String)>,
    /// Carried through: module extension results (buildfiji-mum.8).
    pub module_extensions: Json,
    pub facts: Json,
    pub facts_versions: Json,
}

impl Default for Lockfile {
    fn default() -> Lockfile {
        Lockfile {
            registry_file_hashes: BTreeMap::new(),
            selected_yanked_versions: Vec::new(),
            module_extensions: Json::empty_object(),
            facts: Json::empty_object(),
            facts_versions: Json::empty_object(),
        }
    }
}

/// Reads a list of string pairs from a JSON object, keeping file order.
mod pairs {
    use super::*;
    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<Vec<(String, String)>, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<(String, String)>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an object of strings")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut out = Vec::new();
                while let Some(entry) = a.next_entry()? {
                    out.push(entry);
                }
                Ok(out)
            }
        }
        d.deserialize_map(V)
    }
}

impl Lockfile {
    /// Reads lockfile text.
    pub fn parse(text: &str) -> std::result::Result<Lockfile, Unusable> {
        // A second, tolerant pass to learn the version first: a file of
        // another version may have a shape this one cannot read.
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|_| Unusable::Malformed)?;
        let version = value
            .get("lockFileVersion")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                if value.is_object() {
                    Unusable::Version(0)
                } else {
                    Unusable::Malformed
                }
            })?;
        if version != LOCK_FILE_VERSION {
            return Err(Unusable::Version(version));
        }
        let wire: Wire = serde_json::from_str(text).map_err(|_| Unusable::Malformed)?;
        let mut hashes = BTreeMap::new();
        for (url, hash) in wire.registry_file_hashes {
            hashes.insert(
                url,
                if hash == NOT_FOUND {
                    FileHash::NotFound
                } else {
                    FileHash::Sha256(hash)
                },
            );
        }
        Ok(Lockfile {
            registry_file_hashes: hashes,
            selected_yanked_versions: wire.selected_yanked_versions,
            module_extensions: wire.module_extensions.unwrap_or_else(Json::empty_object),
            facts: wire.facts.unwrap_or_else(Json::empty_object),
            facts_versions: wire.facts_versions.unwrap_or_else(Json::empty_object),
        })
    }

    /// The file's text, as Bazel writes it.
    pub fn to_text(&self) -> String {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Out<'a> {
            lock_file_version: u64,
            registry_file_hashes: Entries<'a>,
            selected_yanked_versions: Entries<'a>,
            module_extensions: &'a Json,
            facts: &'a Json,
            facts_versions: &'a Json,
        }
        struct Entries<'a>(Vec<(&'a str, &'a str)>);
        impl Serialize for Entries<'_> {
            fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                let mut map = s.serialize_map(Some(self.0.len()))?;
                for (k, v) in &self.0 {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
        let out = Out {
            lock_file_version: LOCK_FILE_VERSION,
            registry_file_hashes: Entries(
                self.registry_file_hashes
                    .iter()
                    .map(|(url, hash)| {
                        (
                            url.as_str(),
                            match hash {
                                FileHash::Sha256(h) => h.as_str(),
                                FileHash::NotFound => NOT_FOUND,
                            },
                        )
                    })
                    .collect(),
            ),
            selected_yanked_versions: Entries(
                self.selected_yanked_versions
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect(),
            ),
            module_extensions: &self.module_extensions,
            facts: &self.facts,
            facts_versions: &self.facts_versions,
        };
        let mut text = serde_json::to_string_pretty(&out).expect("a lockfile serialises");
        // Gson escapes these two line separators even with HTML escaping
        // off; serde_json does not.
        text = text
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029");
        text.push('\n');
        text
    }
}

/// The file as read, with the hash and yanked maps kept in file order.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Wire {
    #[serde(default, deserialize_with = "pairs::deserialize")]
    registry_file_hashes: Vec<(String, String)>,
    #[serde(default, deserialize_with = "pairs::deserialize")]
    selected_yanked_versions: Vec<(String, String)>,
    module_extensions: Option<Json>,
    facts: Option<Json>,
    facts_versions: Option<Json>,
}

/// One run's dealings with the lockfile: what it said before, and what the
/// run read, which is what gets written.
pub struct LockSession {
    mode: LockfileMode,
    previous: Option<Lockfile>,
    /// What the previous lockfile was like, for the `error` mode refusal.
    unusable: Option<Unusable>,
    touched: Mutex<BTreeMap<String, FileHash>>,
    /// The module extensions this run evaluated, and those the module graph
    /// uses (what stays of the previous lockfile's).
    extensions: Mutex<Option<(Json, Vec<String>)>>,
    /// The facts extensions kept this run.
    facts: Mutex<Vec<(String, Json)>>,
}

impl LockSession {
    /// Starts a run from the text of the existing `MODULE.bazel.lock`, if
    /// there was one. `Err` is the refusal `error` mode gives a lockfile
    /// it cannot use; the other modes start over from nothing.
    pub fn new(mode: LockfileMode, existing: Option<&str>) -> Result<Arc<LockSession>> {
        let (previous, unusable) = match existing.map(Lockfile::parse) {
            Some(Ok(lockfile)) => (Some(lockfile), None),
            Some(Err(why)) => (None, Some(why)),
            None => (None, None),
        };
        if mode == LockfileMode::Error && unusable.is_some() {
            return Err(BzlmodError::Lockfile(
                "The version of MODULE.bazel.lock is not supported by this version of Bazel. \
                 Please run `bazel mod deps --lockfile_mode=update` to update your lockfile."
                    .to_owned(),
            ));
        }
        Ok(Arc::new(LockSession {
            mode,
            previous,
            unusable,
            touched: Mutex::new(BTreeMap::new()),
            extensions: Mutex::new(None),
            facts: Mutex::new(Vec::new()),
        }))
    }

    /// The module extensions this run evaluated, as `moduleExtensions` holds
    /// them (an object of extension ids, each an object of the factors it
    /// ran under), and the ids of all the extensions the module graph uses: an
    /// extension the graph no longer uses leaves the file, one it still uses
    /// but this run did not evaluate keeps what the file had.
    /// The `facts` the extensions kept this run, each under its extension's id:
    /// they replace what the file had for those extensions (the file's for the
    /// extensions the graph no longer uses go, as `moduleExtensions`' do).
    pub fn set_facts(&self, kept: Vec<(String, Json)>) {
        *self.facts.lock().expect("lock") = kept;
    }

    pub fn set_module_extensions(&self, evaluated: Json, used: Vec<String>) {
        *self.extensions.lock().expect("lock") = Some((evaluated, used));
    }

    pub fn mode(&self) -> LockfileMode {
        self.mode
    }

    /// Whether the previous lockfile was discarded as unusable.
    pub fn discarded_previous(&self) -> Option<&Unusable> {
        self.unusable.as_ref()
    }

    fn previous_hash(&self, url: &str) -> Option<&FileHash> {
        self.previous.as_ref()?.registry_file_hashes.get(url)
    }

    /// A fetcher for `registry`'s files that checks them against, and
    /// records them in, the lockfile. `file://` URLs and `metadata.json`
    /// pass through untouched.
    pub fn fetcher(self: &Arc<Self>, registry: &str, inner: Box<dyn Fetcher>) -> Box<dyn Fetcher> {
        Box::new(LockedFetcher {
            session: Arc::clone(self),
            registry: registry.to_owned(),
            inner,
        })
    }

    /// Whether the lockfile recorded `url` as not found, which is worth
    /// saying when a module turns out to be missing.
    pub fn recorded_missing(&self, url: &str) -> bool {
        self.mode != LockfileMode::Refresh && self.previous_hash(url) == Some(&FileHash::NotFound)
    }

    /// The yanked versions the lockfile knows of for `module`, if it knows
    /// of any (and this is not a `refresh`): those stand in for reading
    /// `metadata.json`.
    pub fn locked_yanked(&self, module: &str) -> Option<Vec<(String, String)>> {
        if self.mode == LockfileMode::Refresh {
            return None;
        }
        let prefix = format!("{module}@");
        let found: Vec<(String, String)> = self
            .previous
            .as_ref()?
            .selected_yanked_versions
            .iter()
            .filter_map(|(key, reason)| {
                key.strip_prefix(&prefix)
                    .map(|version| (version.to_owned(), reason.clone()))
            })
            .collect();
        (!found.is_empty()).then_some(found)
    }

    /// The lockfile this run ends up with: what it read, the yanked
    /// versions it selected, and everything else as it was.
    pub fn finish(&self, selected_yanked: &[(ModuleKey, String)]) -> Lockfile {
        let previous = self.previous.clone().unwrap_or_default();
        let registry_file_hashes = self.touched.lock().expect("lock").clone();
        Lockfile {
            registry_file_hashes,
            selected_yanked_versions: selected_yanked
                .iter()
                .map(|(key, reason)| (key.to_string(), reason.clone()))
                .collect(),
            module_extensions: self.merged_module_extensions(&previous.module_extensions),
            facts: self.merged_facts(&previous.facts),
            facts_versions: previous.facts_versions,
        }
    }

    /// `previous` with what this run kept put in, sorted by id.
    fn merged_facts(&self, previous: &Json) -> Json {
        let kept = self.facts.lock().expect("lock").clone();
        let used = self
            .extensions
            .lock()
            .expect("lock")
            .as_ref()
            .map(|(_, used)| used.clone());
        if kept.is_empty() && used.is_none() {
            return previous.clone();
        }
        let mut merged: BTreeMap<String, Json> = BTreeMap::new();
        if let Json::Object(items) = previous {
            for (id, facts) in items {
                if used.as_ref().is_none_or(|used| used.contains(id)) {
                    merged.insert(id.clone(), facts.clone());
                }
            }
        }
        merged.extend(kept);
        Json::Object(merged.into_iter().collect())
    }

    /// `previous` with what this run evaluated put in, sorted by id and factors.
    fn merged_module_extensions(&self, previous: &Json) -> Json {
        let Some((evaluated, used)) = self.extensions.lock().expect("lock").clone() else {
            return previous.clone();
        };
        let entries = |json: &Json| -> Vec<(String, Json)> {
            match json {
                Json::Object(items) => items.clone(),
                _ => Vec::new(),
            }
        };
        let mut merged: BTreeMap<String, BTreeMap<String, Json>> = BTreeMap::new();
        for (id, factors) in entries(previous) {
            if used.contains(&id) {
                merged.entry(id).or_default().extend(entries(&factors));
            }
        }
        for (id, factors) in entries(&evaluated) {
            merged.entry(id).or_default().extend(entries(&factors));
        }
        Json::Object(
            merged
                .into_iter()
                .map(|(id, factors)| (id, Json::Object(factors.into_iter().collect())))
                .collect(),
        )
    }

    /// The text to write to `MODULE.bazel.lock`, or `None` if the mode does
    /// not write or the file would not change.
    pub fn to_write(
        &self,
        selected_yanked: &[(ModuleKey, String)],
        existing: Option<&str>,
    ) -> Option<String> {
        if !self.mode.writes() {
            return None;
        }
        let text = self.finish(selected_yanked).to_text();
        (existing != Some(text.as_str())).then_some(text)
    }
}

struct LockedFetcher {
    session: Arc<LockSession>,
    registry: String,
    inner: Box<dyn Fetcher>,
}

impl Fetcher for LockedFetcher {
    fn fetch(&self, url: &str) -> Result<Option<Vec<u8>>> {
        let session = &self.session;
        if url.starts_with("file://") || url.starts_with('/') || url.ends_with("/metadata.json") {
            return self.inner.fetch(url);
        }
        let record = |hash: FileHash| {
            session
                .touched
                .lock()
                .expect("lock")
                .insert(url.to_owned(), hash);
        };
        match session.previous_hash(url) {
            Some(FileHash::NotFound) if session.mode != LockfileMode::Refresh => {
                record(FileHash::NotFound);
                return Ok(None);
            }
            Some(FileHash::Sha256(wanted)) => {
                let fetched = self.inner.fetch(url)?;
                let actual = fetched.as_deref().map(sha256_hex);
                if actual.as_deref() != Some(wanted.as_str()) {
                    return Err(BzlmodError::Registry {
                        registry: self.registry.clone(),
                        message: format!(
                            "Failed to fetch registry file {url}: Checksum was {} but wanted \
                             {wanted}",
                            actual.as_deref().unwrap_or(NOT_FOUND)
                        ),
                    });
                }
                record(FileHash::Sha256(wanted.clone()));
                return Ok(fetched);
            }
            _ => {}
        }
        if session.mode == LockfileMode::Error {
            return Err(BzlmodError::Lockfile(format!(
                "Missing checksum for registry file {url} not permitted with \
                 --lockfile_mode=error. Please run `bazel mod deps --lockfile_mode=update` to \
                 update your lockfile."
            )));
        }
        let fetched = self.inner.fetch(url)?;
        record(match &fetched {
            Some(bytes) => FileHash::Sha256(sha256_hex(bytes)),
            None => FileHash::NotFound,
        });
        Ok(fetched)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "lockfile_tests.rs"]
mod tests;
