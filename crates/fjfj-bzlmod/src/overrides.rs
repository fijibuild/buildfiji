//! Module overrides and the repo specs they produce.
//!
//! Ported from Bazel 9.2.0's `ModuleOverride` hierarchy. The rule that
//! shapes everything here: **only the root module's overrides are
//! honoured**. A module used as a dependency has its own overrides
//! discarded, which is what stops a transitive dep from redirecting the
//! build (Bazel enforces this in `ModuleThreadContext.addOverride`, which
//! returns early whenever dev dependencies are being ignored — that flag
//! is set for every non-root module).

use crate::attrs::Attrs;
use crate::version::Version;

/// An override on one module, as declared in the root `MODULE.bazel`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ModuleOverride {
    /// `single_version_override`: still comes from a registry, but pinned,
    /// and/or from a different registry, and/or patched.
    SingleVersion(SingleVersionOverride),
    /// `multiple_version_override`: still from a registry, but several
    /// versions are allowed to coexist in the final graph.
    MultipleVersion(MultipleVersionOverride),
    /// `archive_override`, `git_override`, `local_path_override`: the
    /// module leaves the registry world entirely and stops participating
    /// in version selection.
    NonRegistry(NonRegistryOverride),
}

impl ModuleOverride {
    /// The registry to use for this module, if the override names one.
    /// `None` means "use the configured registry list".
    pub fn registry(&self) -> Option<&str> {
        let registry = match self {
            ModuleOverride::SingleVersion(o) => o.registry.as_deref(),
            ModuleOverride::MultipleVersion(o) => o.registry.as_deref(),
            ModuleOverride::NonRegistry(_) => None,
        };
        registry.filter(|r| !r.is_empty())
    }

    /// Whether this override takes the module out of version selection.
    pub fn is_non_registry(&self) -> bool {
        matches!(self, ModuleOverride::NonRegistry(_))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SingleVersionOverride {
    /// The version to pin to. Empty means "don't pin" — the override is
    /// then only about the registry or the patches, and the module still
    /// takes part in selection.
    pub version: Version,
    pub registry: Option<String>,
    /// Labels of patch files in the root module's source tree.
    pub patches: Vec<String>,
    pub patch_cmds: Vec<String>,
    pub patch_strip: i32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct MultipleVersionOverride {
    /// The versions allowed to coexist. Bazel requires at least two —
    /// one version is what `single_version_override` is for.
    pub versions: Vec<Version>,
    pub registry: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct NonRegistryOverride {
    pub repo_spec: RepoSpec,
}

/// How to materialise a repo: which repo rule to call and with what
/// attributes. Produced both by non-registry overrides and by a
/// registry's `source.json`; consumed by the fetch phase
/// (buildfiji-mum.8), which is the only thing that knows how to run a
/// repository rule.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RepoSpec {
    pub rule: RepoRule,
    pub attrs: Attrs,
}

/// The repo rules bzlmod can name. All three live in `@bazel_tools` and
/// are bootstrapped without a repo mapping, since they are what defines
/// the modules a mapping would be built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum RepoRule {
    /// `@@bazel_tools//tools/build_defs/repo:http.bzl%http_archive`
    HttpArchive,
    /// `@@bazel_tools//tools/build_defs/repo:git.bzl%git_repository`
    GitRepository,
    /// `@@bazel_tools//tools/build_defs/repo:local.bzl%local_repository`
    LocalRepository,
}

impl RepoRule {
    /// The `label%name` form Bazel writes into the lockfile.
    pub fn id(self) -> &'static str {
        match self {
            RepoRule::HttpArchive => "@@bazel_tools//tools/build_defs/repo:http.bzl%http_archive",
            RepoRule::GitRepository => {
                "@@bazel_tools//tools/build_defs/repo:git.bzl%git_repository"
            }
            RepoRule::LocalRepository => {
                "@@bazel_tools//tools/build_defs/repo:local.bzl%local_repository"
            }
        }
    }
}
