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
    /// The command line of a spawn, `None` for the other kinds.
    pub fn argv(&self) -> Option<&[String]> {
        match &self.kind {
            ActionKind::Spawn { argv, .. } => Some(argv),
            _ => None,
        }
    }
}
