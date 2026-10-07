//! Build graph: labels, targets, configured targets, aspects, actions.
//!
//! This crate is the analysis-phase data model. It is intentionally free of
//! I/O so it can be unit-tested and so that an incremental engine (a
//! Skyframe/DICE-style memoising key/value graph) can be layered on top.

use std::fmt;

pub mod action;
pub mod artifact;
mod build_options;
pub mod config;
pub mod conflicts;
pub mod expand;
pub mod label;
pub mod native_rules;
pub mod package;
pub mod parse;
pub mod pattern;
pub mod rule;
pub mod schema;
pub mod test_info;
pub mod visibility;

pub use action::{Action, ActionKind};
pub use artifact::{Artifact, NestedSet, Root, Runfiles};
pub use config::{CompilationMode, Configuration, SettingValue};
pub use label::LabelError;
pub use parse::{LabelContext, LabelParseError};
pub use test_info::TestInfo;

/// A Bazel label, e.g. `@repo//pkg/sub:name`.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Label {
    pub repo: String,
    pub package: String,
    pub name: String,
}

impl Label {
    /// Build a `Label`, validating `package` and `name` against Bazel's
    /// own character rules (see [`label`]'s module doc for why they
    /// differ). Does not parse a `@repo//pkg:name` string — that's
    /// [`pattern::TargetPattern`]'s job, which also handles the
    /// wildcard/negation syntax a concrete `Label` doesn't have.
    pub fn new(
        repo: impl Into<String>,
        package: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Label, LabelError> {
        let package = package.into();
        let name = name.into();
        label::validate_package_name(&package)?;
        label::validate_target_name(&name)?;
        Ok(Label {
            repo: repo.into(),
            package,
            name,
        })
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.repo.is_empty() {
            write!(f, "//{}:{}", self.package, self.name)
        } else {
            write!(f, "@{}//{}:{}", self.repo, self.package, self.name)
        }
    }
}

/// A content digest as used by REAPI: SHA-256 hex + size in bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Digest {
    pub hash: String,
    pub size_bytes: u64,
}

impl Digest {
    pub fn of_bytes(b: &[u8]) -> Self {
        use sha2::Digest as _;
        Digest {
            hash: hex::encode(sha2::Sha256::digest(b)),
            size_bytes: b.len() as u64,
        }
    }
}
