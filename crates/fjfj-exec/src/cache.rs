//! The action cache (buildfiji-fyz, 23d.9): what is known of the last time
//! each action ran, so that an action whose command and inputs have not
//! changed, and whose outputs are still there, does not run again.
//!
//! An action's key is the SHA-256 of what it does (its kind, its command line
//! and environment) and of each input: the digest of its content. A file's
//! digest is remembered with its size and modification time, so a file that
//! has not changed is not read again.

use fjfj_graph::Action;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileDigest {
    mtime_ns: u128,
    size: u64,
    sha256: String,
}

/// What an action did, by its first output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Entry {
    key: String,
    /// How long the action took, for the console to say again when it does not run.
    #[serde(default)]
    duration_ms: u64,
    /// Each output's exec path and digest after the action ran.
    outputs: Vec<(String, String)>,
}

/// What actions of one mnemonic have taken, as a mean over the last runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Mean {
    runs: u64,
    mean_ms: u64,
}

/// Runs older than this many fade from a mnemonic's mean.
const MEAN_WINDOW: u64 = 32;

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    version: u32,
    actions: HashMap<String, Entry>,
    files: HashMap<String, FileDigest>,
}

const VERSION: u32 = 1;

/// An input's fragment of a key, once it is known.
type Fragment = Arc<Mutex<Option<[u8; 32]>>>;

pub struct ActionCache {
    path: PathBuf,
    state: Mutex<State>,
    /// How long actions of each mnemonic take, for ranking one never timed.
    /// Kept in a file of its own, outside the output base, so a build in a
    /// fresh output base still knows what a compile costs.
    mnemonics: Mutex<HashMap<String, Mean>>,
    history: Option<PathBuf>,
    /// Directories an action made, by the key it ran with: a directory
    /// stands for what made it.
    dirs: Mutex<HashMap<String, String>>,
    /// The fragments [`ActionCache::input_fragment`] has found this run: an input
    /// is read after what makes it has finished, so it does not change again.
    /// Each input has a slot of its own, held while it is found, so actions
    /// that need the same input at once wait for one reading of it.
    inputs: RwLock<HashMap<String, Fragment>>,
}

impl ActionCache {
    /// The cache at `path`, empty if there is none or it is not one.
    pub fn load(path: impl Into<PathBuf>) -> ActionCache {
        Self::load_with_history(path, None)
    }

    /// The cache at `path`, and the per-mnemonic durations at `history`.
    pub fn load_with_history(path: impl Into<PathBuf>, history: Option<PathBuf>) -> ActionCache {
        let path = path.into();
        let mnemonics = history
            .as_ref()
            .and_then(|h| std::fs::read(h).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let state = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<State>(&bytes).ok())
            .filter(|s| s.version == VERSION)
            .unwrap_or_else(|| State {
                version: VERSION,
                ..State::default()
            });
        ActionCache {
            path,
            state: Mutex::new(state),
            mnemonics: Mutex::new(mnemonics),
            history,
            dirs: Mutex::new(HashMap::new()),
            inputs: RwLock::new(HashMap::new()),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let state = self.state.lock().unwrap();
        let bytes = serde_json::to_vec(&*state).map_err(std::io::Error::other)?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(tmp, &self.path)?;
        if let Some(history) = &self.history {
            let bytes = serde_json::to_vec(&*self.mnemonics.lock().unwrap())
                .map_err(std::io::Error::other)?;
            if let Some(dir) = history.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let tmp = history.with_extension("tmp");
            std::fs::write(&tmp, bytes)?;
            std::fs::rename(tmp, history)?;
        }
        Ok(())
    }

    /// The digest of the file at `root/exec_path`, `None` if it is not a file
    /// (a missing file, a directory).
    pub fn digest(&self, root: &Path, exec_path: &str) -> Option<String> {
        let path = root.join(exec_path);
        let meta = std::fs::metadata(&path).ok()?;
        if meta.is_dir() {
            return self.dirs.lock().unwrap().get(exec_path).cloned();
        }
        if !meta.is_file() {
            return None;
        }
        let mtime_ns = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_nanos());
        if let Some(known) = self.state.lock().unwrap().files.get(exec_path)
            && known.mtime_ns == mtime_ns
            && known.size == meta.len()
        {
            return Some(known.sha256.clone());
        }
        let mut file = std::fs::File::open(&path).ok()?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buffer).ok()?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let sha256 = hex::encode(hasher.finalize());
        self.state.lock().unwrap().files.insert(
            exec_path.to_owned(),
            FileDigest {
                mtime_ns,
                size: meta.len(),
                sha256: sha256.clone(),
            },
        );
        Some(sha256)
    }

    /// What an input of an action about to be keyed adds to its key: the
    /// SHA-256 of the input's path and digest, found once however many
    /// actions read it.
    fn input_fragment(&self, root: &Path, exec_path: &str) -> Option<[u8; 32]> {
        let known = self.inputs.read().unwrap().get(exec_path).cloned();
        let slot = match known {
            Some(slot) => slot,
            None => self
                .inputs
                .write()
                .unwrap()
                .entry(exec_path.to_owned())
                .or_default()
                .clone(),
        };
        let mut found = slot.lock().unwrap();
        if let Some(fragment) = *found {
            return Some(fragment);
        }
        let digest = self.digest(root, exec_path)?;
        let mut hasher = Sha256::new();
        for bytes in [exec_path.as_bytes(), digest.as_bytes()] {
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        }
        let fragment: [u8; 32] = hasher.finalize().into();
        *found = Some(fragment);
        Some(fragment)
    }

    /// The key of `action`: what it does and what its inputs hold. `None` if
    /// an input is not a file that can be read.
    pub fn key(&self, root: &Path, action: &Action) -> Option<String> {
        let mut hasher = Sha256::new();
        let mut field = |bytes: &[u8]| {
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        };
        action.definition(&mut field);
        field(b"inputs");
        let mut inputs: Vec<String> = action.inputs.iter().map(|a| a.exec_path()).collect();
        inputs.sort();
        inputs.dedup();
        let mut fragments = Vec::with_capacity(inputs.len() * 32);
        for input in &inputs {
            fragments.extend_from_slice(&self.input_fragment(root, input)?);
        }
        field(&fragments);
        field(b"outputs");
        for out in &action.outputs {
            field(out.exec_path().as_bytes());
        }
        Some(hex::encode(hasher.finalize()))
    }

    /// Whether `action` ran with `key` and its outputs are as it left them.
    pub fn is_current(&self, root: &Path, action: &Action, key: &str) -> bool {
        let Some(first) = action.outputs.first() else {
            return false;
        };
        let entry = self
            .state
            .lock()
            .unwrap()
            .actions
            .get(&first.exec_path())
            .cloned();
        let Some(entry) = entry else { return false };
        let current = entry.key == key
            && entry.outputs.len() == action.outputs.len()
            && entry.outputs.iter().all(|(path, digest)| {
                // A symlink output is current while it is a link.
                match std::fs::symlink_metadata(root.join(path)) {
                    Ok(meta) if meta.file_type().is_symlink() => digest == "symlink",
                    Ok(meta) if meta.is_dir() => digest == &format!("directory:{key}"),
                    Ok(_) => self.digest(root, path).as_deref() == Some(digest.as_str()),
                    Err(_) => false,
                }
            });
        if current {
            self.note_dirs(root, action, key);
        }
        current
    }

    /// Remember what made the directories among `action`'s outputs.
    fn note_dirs(&self, root: &Path, action: &Action, key: &str) {
        for out in &action.outputs {
            let path = out.exec_path();
            if std::fs::metadata(root.join(&path)).is_ok_and(|m| m.is_dir()) {
                self.dirs
                    .lock()
                    .unwrap()
                    .insert(path, format!("directory:{key}"));
            }
        }
    }

    /// How long `action` took the last time it ran.
    pub fn duration(&self, action: &Action) -> Option<std::time::Duration> {
        let first = action.outputs.first()?;
        self.state
            .lock()
            .unwrap()
            .actions
            .get(&first.exec_path())
            .map(|e| std::time::Duration::from_millis(e.duration_ms))
    }

    /// How long an action of `mnemonic` took, on average, over earlier runs.
    pub fn mnemonic_mean(&self, mnemonic: &str) -> Option<std::time::Duration> {
        self.mnemonics
            .lock()
            .unwrap()
            .get(mnemonic)
            .filter(|m| m.runs > 0)
            .map(|m| std::time::Duration::from_millis(m.mean_ms))
    }

    /// Remember that `action` ran with `key`, took `took`, and left its outputs.
    pub fn record(&self, root: &Path, action: &Action, key: String, took: std::time::Duration) {
        let Some(first) = action.outputs.first() else {
            return;
        };
        self.note_dirs(root, action, &key);
        let outputs = action
            .outputs
            .iter()
            .map(|out| {
                let path = out.exec_path();
                let digest = match std::fs::symlink_metadata(root.join(&path)) {
                    Ok(meta) if meta.file_type().is_symlink() => "symlink".to_owned(),
                    Ok(meta) if meta.is_dir() => format!("directory:{key}"),
                    _ => self.digest(root, &path).unwrap_or_default(),
                };
                (path, digest)
            })
            .collect();
        let mut mnemonics = self.mnemonics.lock().unwrap();
        let mean = mnemonics.entry(action.mnemonic.clone()).or_default();
        mean.runs = (mean.runs + 1).min(MEAN_WINDOW);
        let took_ms = took.as_millis() as u64;
        mean.mean_ms = if mean.runs == 1 {
            took_ms
        } else {
            (mean.mean_ms * (mean.runs - 1) + took_ms) / mean.runs
        };
        drop(mnemonics);
        self.state.lock().unwrap().actions.insert(
            first.exec_path(),
            Entry {
                key,
                duration_ms: took.as_millis() as u64,
                outputs,
            },
        );
    }
}
