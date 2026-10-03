//! Actions: what the execution phase runs (buildfiji-136.3, 136.13).
//!
//! An action reads artifacts and creates artifacts. Its three kinds are what
//! a rule can ask for without a program: write a file, link to one, and run a
//! command.

use crate::Label;
use crate::artifact::Artifact;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ActionKind {
    /// Run a command in the execroot.
    Spawn {
        argv: Vec<String>,
        /// The environment the command sees, and nothing else.
        env: BTreeMap<String, String>,
        /// `execution_requirements` (`local`, `no-sandbox`, ...).
        execution_requirements: BTreeMap<String, String>,
    },
    /// Create a file with these contents.
    WriteFile { contents: Vec<u8>, executable: bool },
    /// Make the output a symlink to `target`, an exec path.
    Symlink { target: String },
    /// Make the output a symlink whose text is `target`, as written (a path
    /// that need not exist, relative to the link).
    UnresolvedSymlink { target: String },
    /// Make the runfiles tree of an executable: a directory of symlinks at
    /// `dir`, a manifest and a repository mapping beside it.
    RunfilesTree {
        /// `bazel-out/k8-fastbuild/bin/pkg/bin.runfiles`.
        dir: String,
        /// `bin.runfiles_manifest`.
        manifest: String,
        /// `bin.repo_mapping`, and what it holds.
        repo_mapping: String,
        repo_mapping_contents: String,
        /// Each link as a path under `dir` and the file it leads to.
        entries: Vec<(String, Artifact)>,
        /// Paths under `dir` of empty regular files.
        empty_files: Vec<String>,
    },
    /// Write the file `template`, with each key replaced by its value.
    Template {
        template: String,
        substitutions: Vec<(String, String)>,
        executable: bool,
    },
}

/// One action of a configured target.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Action {
    /// The target that registered it.
    pub owner: Label,
    /// The rule class of the owner (`genrule`), for messages.
    pub owner_kind: String,
    /// Where the owner is declared, `BUILD.bazel:3:8` from its package's
    /// repository, for messages.
    pub location: String,
    /// The configuration of the owner, `k8-fastbuild`.
    pub configuration: String,
    /// `Genrule`, `CppCompile`, `FileWrite`: what is counted by kind.
    pub mnemonic: String,
    /// What the console says while it runs; `None` is Bazel's default
    /// for the mnemonic.
    pub progress_message: Option<String>,
    pub kind: ActionKind,
    pub inputs: Vec<Artifact>,
    pub outputs: Vec<Artifact>,
}

impl Action {
    /// Feed `field` what the action does: its mnemonic and what its kind
    /// carries, each as one field. What its inputs and outputs are is left to
    /// the caller.
    pub fn definition(&self, field: &mut dyn FnMut(&[u8])) {
        field(self.mnemonic.as_bytes());
        match &self.kind {
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
                empty_files,
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
                field(b"empty");
                for path in empty_files {
                    field(path.as_bytes());
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
    }

    /// A digest of the action as defined, without what its inputs hold, which
    /// `aquery` shows as the `ActionKey`.
    pub fn key(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut field = |bytes: &[u8]| {
            hasher.update((bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        };
        self.definition(&mut field);
        field(b"inputs");
        let mut inputs: Vec<String> = self.inputs.iter().map(|a| a.exec_path()).collect();
        inputs.sort();
        inputs.dedup();
        for input in &inputs {
            field(input.as_bytes());
        }
        field(b"outputs");
        for out in &self.outputs {
            field(out.exec_path().as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    /// The command line of a spawn, `None` for the other kinds.
    pub fn argv(&self) -> Option<&[String]> {
        match &self.kind {
            ActionKind::Spawn { argv, .. } => Some(argv),
            _ => None,
        }
    }
}
