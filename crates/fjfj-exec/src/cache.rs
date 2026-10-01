//! The action cache (buildfiji-fyz, 23d.9): what is known of the last time
//! each action ran, so that an action whose command and inputs have not
//! changed, and whose outputs are still there, does not run again.
//!
//! An action's key is the SHA-256 of what it does (its kind, its command line
//! and environment) and of each input: the digest of its content. A file's
//! digest is remembered with its size and modification time, so a file that
//! has not changed is not read again.

use fjfj_graph::{Action, ActionKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    version: u32,
    actions: HashMap<String, Entry>,
    files: HashMap<String, FileDigest>,
}

const VERSION: u32 = 1;

pub struct ActionCache {
    path: PathBuf,
    state: Mutex<State>,
    /// Directories an action made, by the key it ran with: a directory
    /// stands for what made it.
    dirs: Mutex<HashMap<String, String>>,
}

impl ActionCache {
    /// The cache at `path`, empty if there is none or it is not one.
    pub fn load(path: impl Into<PathBuf>) -> ActionCache {
        let path = path.into();
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
            dirs: Mutex::new(HashMap::new()),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let state = self.state.lock().unwrap();
        let bytes = serde_json::to_vec(&*state).map_err(std::io::Error::other)?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(tmp, &self.path)
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

    /// The key of `action`: what it does and what its inputs hold. `None` if
    /// an input is not a file that can be read.
    pub fn key(&self, root: &Path, action: &Action) -> Option<String> {
        let mut hasher = Sha256::new();
        let mut field = |bytes: &[u8]| {
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        };
        field(action.mnemonic.as_bytes());
        match &action.kind {
            ActionKind::Spawn {
                argv,
                env,
                execution_requirements,
            } => {
                field(b"spawn");
                for arg in argv {
                    field(arg.as_bytes());
                }
                field(b"env");
                for (k, v) in env {
                    field(k.as_bytes());
                    field(v.as_bytes());
                }
                field(b"requirements");
                for (k, v) in execution_requirements {
                    field(k.as_bytes());
                    field(v.as_bytes());
                }
            }
            ActionKind::WriteFile {
                contents,
                executable,
            } => {
                field(b"write");
                field(contents);
                field(&[u8::from(*executable)]);
            }
            ActionKind::Symlink { target } => {
                field(b"symlink");
                field(target.as_bytes());
            }
            ActionKind::UnresolvedSymlink { target } => {
                field(b"unresolved-symlink");
                field(target.as_bytes());
            }
            ActionKind::RunfilesTree {
                dir,
                manifest,
                repo_mapping,
                repo_mapping_contents,
                entries,
            } => {
                field(b"runfiles");
                field(dir.as_bytes());
                field(manifest.as_bytes());
                field(repo_mapping.as_bytes());
                field(repo_mapping_contents.as_bytes());
                for (path, artifact) in entries {
                    field(path.as_bytes());
                    field(artifact.exec_path().as_bytes());
                }
            }
            ActionKind::Template {
                template,
                substitutions,
                executable,
            } => {
                field(b"template");
                field(template.as_bytes());
                for (k, v) in substitutions {
                    field(k.as_bytes());
                    field(v.as_bytes());
                }
                field(&[u8::from(*executable)]);
            }
        }
        field(b"inputs");
        let mut inputs: Vec<String> = action.inputs.iter().map(|a| a.exec_path()).collect();
        inputs.sort();
        inputs.dedup();
        let mut digests = Vec::with_capacity(inputs.len());
        for input in &inputs {
            digests.push(self.digest(root, input)?);
        }
        for (input, digest) in inputs.iter().zip(&digests) {
            field(input.as_bytes());
            field(digest.as_bytes());
        }
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
