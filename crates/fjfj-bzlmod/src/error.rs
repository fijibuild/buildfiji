//! Errors from module resolution.
//!
//! The variants track Bazel's `ExternalDeps.Code` failure codes rather
//! than a shape of fjfj's own, because these are user-facing: a build that
//! fails resolution under Bazel should fail with a recognisable message
//! under fjfj.

use crate::module::ModuleKey;

pub type Result<T> = std::result::Result<T, BzlmodError>;

#[derive(Debug, thiserror::Error)]
pub enum BzlmodError {
    /// Bazel's `Code.BAD_MODULE`: the file exists but says something
    /// impossible.
    #[error("error in MODULE.bazel file for {key}: {message}")]
    BadModule { key: String, message: String },

    /// Bazel's `Code.MODULE_NOT_FOUND`.
    #[error("module {key} not found in registries:\n* {tried}")]
    ModuleNotFound { key: String, tried: String },

    /// A `bazel_dep` with no version on a module that has no non-registry
    /// override — the most common bzlmod mistake, and Bazel gives it its
    /// own message.
    #[error(
        "bad bazel_dep on module '{name}' with no version. Did you forget to specify a version, \
         or a non-registry override?"
    )]
    MissingVersion { name: String },

    /// Bazel's `Code.ERROR_ACCESSING_REGISTRY`.
    #[error("error accessing registry {registry}: {message}")]
    Registry { registry: String, message: String },

    /// Bazel's `Code.VERSION_RESOLUTION_ERROR`.
    #[error("{0}")]
    VersionResolution(String),

    /// A module needs contents fetched before its file can be read.
    #[error(
        "module {key} has a {kind} override, which requires fetching the module contents before \
         its MODULE.bazel can be read (buildfiji-mum.8)"
    )]
    FetchRequired { key: String, kind: &'static str },

    #[error("a MODULE.bazel directive was called outside module file evaluation")]
    NotAModuleFile,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl BzlmodError {
    pub(crate) fn bad_module(key: &ModuleKey, message: impl Into<String>) -> BzlmodError {
        BzlmodError::BadModule {
            key: key.to_string(),
            message: message.into(),
        }
    }

    pub(crate) fn resolution(message: impl Into<String>) -> BzlmodError {
        BzlmodError::VersionResolution(message.into())
    }
}
