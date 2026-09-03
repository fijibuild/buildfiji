//! Canonical REAPI encoding: the action key.
//!
//! Contract (beads buildfiji-c71.5, buildfiji-c71.9; Lean `Fjfj.ActionKey`):
//! for the same inputs, arguments, environment, outputs and platform, fjfj
//! must produce the same `Action` digest as Bazel 9.2.0. The rules that make
//! the encoding canonical:
//!
//! - `Directory.files`, `.directories`, `.symlinks` sorted by name, no
//!   duplicate names, no `.`/`..`/empty/slash-containing names.
//! - `Command.environment_variables` sorted by name; `output_paths` sorted;
//!   `Platform.properties` sorted by name. `output_files`/`output_directories`
//!   (deprecated) are left empty: Bazel 9 emits `output_paths` only.
//! - Digest = SHA-256 of the deterministic protobuf serialisation; proto3
//!   field order and default-value omission are what make prost and Java
//!   protobuf agree.
//!
//! Bazel-specific facts verified against Bazel 9.2.0 CAS blobs (testdata):
//! - Bazel sets `FileNode.is_executable = true` on **every** input file,
//!   independent of filesystem mode (so cache keys do not depend on umask).
//!   `input_file` does the same.
//! - `Action.salt` is a serialised `CacheSalt { may_be_executed_remotely }`
//!   message (Bazel's `remote_execution_log.proto`), not empty.
//! - `Command.platform` (deprecated in REAPI) is still populated, and the
//!   same `Platform` is repeated in `Action.platform`.

use std::collections::BTreeMap;

use prost::Message;
use sha2::Digest as _;

use crate::reapi::{
    Action, Command, Digest, Directory, DirectoryNode, FileNode, NodeProperties, Platform,
    SymlinkNode, command::EnvironmentVariable, platform::Property,
};

/// One input to an action, as a path relative to the execroot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub path: String,
    pub kind: InputKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    File { digest: Digest, executable: bool },
    Symlink { target: String },
}

impl Input {
    /// A regular file input, marked executable as Bazel does for all inputs.
    pub fn file(path: impl Into<String>, digest: Digest) -> Self {
        Input {
            path: path.into(),
            kind: InputKind::File {
                digest,
                executable: true,
            },
        }
    }
}

/// Bazel's `CacheSalt` (src/main/protobuf/remote_execution_log.proto),
/// serialised into `Action.salt`.
#[derive(Clone, PartialEq, Message)]
pub struct CacheSalt {
    #[prost(bool, tag = "1")]
    pub may_be_executed_remotely: bool,
    /// Set from `--experimental_remote_cache_key_workspace`-style options; empty by default.
    #[prost(string, tag = "2")]
    pub workspace: String,
}

pub fn digest_of(bytes: &[u8]) -> Digest {
    Digest {
        hash: hex::encode(sha2::Sha256::digest(bytes)),
        size_bytes: bytes.len() as i64,
    }
}

pub fn digest_of_message<M: Message>(m: &M) -> Digest {
    digest_of(&m.encode_to_vec())
}

#[derive(Default)]
struct Tree {
    files: BTreeMap<String, FileNode>,
    symlinks: BTreeMap<String, SymlinkNode>,
    dirs: BTreeMap<String, Tree>,
}

impl Tree {
    fn insert(&mut self, path: &str, kind: InputKind) -> anyhow::Result<()> {
        let (head, rest) = match path.split_once('/') {
            Some((h, r)) => (h, Some(r)),
            None => (path, None),
        };
        anyhow::ensure!(
            !head.is_empty() && head != "." && head != "..",
            "invalid path component in {path:?}"
        );
        match rest {
            Some(rest) => self
                .dirs
                .entry(head.to_string())
                .or_default()
                .insert(rest, kind),
            None => {
                anyhow::ensure!(
                    !self.dirs.contains_key(head),
                    "{path:?} is both a file and a directory"
                );
                match kind {
                    InputKind::File { digest, executable } => {
                        let prev = self.files.insert(
                            head.to_string(),
                            FileNode {
                                name: head.to_string(),
                                digest: Some(digest),
                                is_executable: executable,
                                node_properties: None::<NodeProperties>,
                            },
                        );
                        anyhow::ensure!(prev.is_none(), "duplicate input {path:?}");
                    }
                    InputKind::Symlink { target } => {
                        let prev = self.symlinks.insert(
                            head.to_string(),
                            SymlinkNode {
                                name: head.to_string(),
                                target,
                                node_properties: None,
                            },
                        );
                        anyhow::ensure!(prev.is_none(), "duplicate input {path:?}");
                    }
                }
                Ok(())
            }
        }
    }

    /// Serialise bottom-up, appending every `Directory` to `out`, returning this node's digest.
    fn finish(self, out: &mut Vec<Directory>) -> Digest {
        let directories = self
            .dirs
            .into_iter()
            .map(|(name, sub)| DirectoryNode {
                name,
                digest: Some(sub.finish(out)),
            })
            .collect();
        let dir = Directory {
            files: self.files.into_values().collect(),
            directories,
            symlinks: self.symlinks.into_values().collect(),
            node_properties: None,
        };
        let d = digest_of_message(&dir);
        out.push(dir);
        d
    }
}

/// Build the canonical input Merkle tree. Returns the root digest and every
/// `Directory` message (root last) for upload.
pub fn merkle_tree(
    inputs: impl IntoIterator<Item = Input>,
) -> anyhow::Result<(Digest, Vec<Directory>)> {
    let mut root = Tree::default();
    for i in inputs {
        root.insert(&i.path, i.kind)?;
    }
    let mut dirs = Vec::new();
    let d = root.finish(&mut dirs);
    Ok((d, dirs))
}

pub fn platform(properties: impl IntoIterator<Item = (String, String)>) -> Platform {
    let sorted: BTreeMap<String, String> = properties.into_iter().collect();
    Platform {
        properties: sorted
            .into_iter()
            .map(|(name, value)| Property { name, value })
            .collect(),
    }
}

/// Canonical `Command`. `platform` and the `output_files`/`output_directories`
/// fields are deprecated in REAPI but Bazel 9.2.0 still populates `platform`,
/// so we must too for byte parity.
#[allow(deprecated)]
pub fn command(
    arguments: Vec<String>,
    environment: impl IntoIterator<Item = (String, String)>,
    output_paths: impl IntoIterator<Item = String>,
    platform: Platform,
    working_directory: String,
) -> Command {
    let env: BTreeMap<String, String> = environment.into_iter().collect();
    let mut output_paths: Vec<String> = output_paths.into_iter().collect();
    output_paths.sort();
    output_paths.dedup();
    Command {
        arguments,
        environment_variables: env
            .into_iter()
            .map(|(name, value)| EnvironmentVariable { name, value })
            .collect(),
        output_files: vec![],
        output_directories: vec![],
        output_paths,
        platform: Some(platform),
        working_directory,
        output_node_properties: vec![],
        output_directory_format: 0,
    }
}

/// Canonical `Action`. `platform` is duplicated into the Action as Bazel does
/// (REAPI 2.2+ `Action.platform`).
#[allow(deprecated)]
pub fn action(
    command: &Command,
    input_root: Digest,
    timeout: Option<bazel_remote_apis::google::protobuf::Duration>,
    do_not_cache: bool,
    salt: &CacheSalt,
) -> Action {
    Action {
        command_digest: Some(digest_of_message(command)),
        input_root_digest: Some(input_root),
        timeout,
        do_not_cache,
        salt: salt.encode_to_vec(),
        platform: command.platform.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture recorded from `bazel build //pkg:hello --disk_cache=...
    /// --execution_log_json_file=...` with Bazel 9.2.0 on darwin_arm64; see
    /// testdata/genrule/BUILD.bazel.fixture. `action.bin` and `command.bin`
    /// are the CAS blobs Bazel wrote.
    const EXEC_LOG: &str = include_str!("../testdata/genrule/exec.json");
    const ACTION_BIN: &[u8] = include_bytes!("../testdata/genrule/action.bin");
    const COMMAND_BIN: &[u8] = include_bytes!("../testdata/genrule/command.bin");

    fn fixture() -> (Command, Action, Digest) {
        let log: serde_json::Value = serde_json::from_str(EXEC_LOG).unwrap();
        let strs = |v: &serde_json::Value| -> Vec<String> {
            v.as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect()
        };
        let pairs = |v: &serde_json::Value| -> Vec<(String, String)> {
            v.as_array()
                .map(|a| {
                    a.iter()
                        .map(|p| {
                            (
                                p["name"].as_str().unwrap().into(),
                                p["value"].as_str().unwrap().into(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let inputs = log["inputs"].as_array().unwrap().iter().map(|i| {
            Input::file(
                i["path"].as_str().unwrap(),
                Digest {
                    hash: i["digest"]["hash"].as_str().unwrap().to_string(),
                    size_bytes: i["digest"]["sizeBytes"].as_str().unwrap().parse().unwrap(),
                },
            )
        });
        let (root, _) = merkle_tree(inputs).unwrap();
        let cmd = command(
            strs(&log["commandArgs"]),
            pairs(&log["environmentVariables"]),
            strs(&log["listedOutputs"]),
            platform(pairs(&log["platform"]["properties"])),
            String::new(),
        );
        let salt = CacheSalt {
            may_be_executed_remotely: log["remotable"].as_bool().unwrap(),
            workspace: String::new(),
        };
        let act = action(&cmd, root, None, false, &salt);
        let expected = Digest {
            hash: log["digest"]["hash"].as_str().unwrap().to_string(),
            size_bytes: log["digest"]["sizeBytes"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        };
        (cmd, act, expected)
    }

    #[test]
    fn command_bytes_match_bazel() {
        let (cmd, _, _) = fixture();
        let bazel = Command::decode(COMMAND_BIN).unwrap();
        assert_eq!(cmd, bazel);
        assert_eq!(cmd.encode_to_vec(), COMMAND_BIN);
    }

    #[test]
    fn action_bytes_and_digest_match_bazel() {
        let (_, act, expected) = fixture();
        let bazel = Action::decode(ACTION_BIN).unwrap();
        assert_eq!(act, bazel);
        assert_eq!(act.encode_to_vec(), ACTION_BIN);
        assert_eq!(digest_of_message(&act), expected);
    }

    #[test]
    fn empty_tree_is_empty_blob_digest() {
        let (d, dirs) = merkle_tree([]).unwrap();
        assert_eq!(
            d.hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(d.size_bytes, 0);
        assert_eq!(dirs.len(), 1);
    }

    #[test]
    fn rejects_duplicates_and_file_dir_conflicts() {
        let f = |p: &str| Input {
            path: p.into(),
            kind: InputKind::File {
                digest: digest_of(b""),
                executable: false,
            },
        };
        assert!(merkle_tree([f("a"), f("a")]).is_err());
        assert!(merkle_tree([f("a/b"), f("a")]).is_err());
        assert!(merkle_tree([f("a/../b")]).is_err());
    }
}
