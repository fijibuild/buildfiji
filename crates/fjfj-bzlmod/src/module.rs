//! The module graph's data model: keys, dependency specs and the module
//! record produced by evaluating one `MODULE.bazel` file.
//!
//! Ported from Bazel 9.2.0's `ModuleKey.java` and `InterimModule.java`.
//! Two things Bazel carries are deliberately absent, because Bazel 9.2.0
//! no longer has them either:
//!
//! - `compatibility_level` on `module()` is a **no-op**: Bazel parses the
//!   argument, warns when the root module sets it, then hard-codes the
//!   module's level to 0 (`ModuleFileGlobals.module`, which calls
//!   `setCompatibilityLevel(0)` unconditionally).
//! - `max_compatibility_level` on `bazel_dep()` is a no-op for the same
//!   reason, so a `DepSpec` resolves to exactly one version.
//!
//! Modelling either would be modelling a Bazel that no longer exists, and
//! it would put a field in the lockfile and in `mod` output that Bazel
//! does not. See `docs/design/starlark-and-loading.md` for what changes if
//! compatibility levels ever come back.

use std::fmt;

use crate::attrs::Attrs;
use crate::version::Version;

/// A `(name, version)` pair: the identity of a node in the module graph.
///
/// The root module's key has both parts empty; a module under a
/// non-registry override has an empty version, since such a module can
/// only appear once in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub struct ModuleKey {
    pub name: String,
    pub version: Version,
}

/// Bazel hard-codes the repo name of these modules instead of deriving it,
/// for backwards compatibility: `@bazel_tools` and `@platforms` are
/// referred to by those exact names from rules that predate bzlmod.
/// Kept in sync with Bazel's `ModuleKey.WELL_KNOWN_MODULES`.
const WELL_KNOWN_MODULES: &[&str] = &["bazel_tools", "platforms"];

impl ModuleKey {
    /// The root module — the workspace fjfj was invoked in.
    pub fn root() -> ModuleKey {
        ModuleKey {
            name: String::new(),
            version: Version::EMPTY,
        }
    }

    pub fn new(name: impl Into<String>, version: Version) -> ModuleKey {
        ModuleKey {
            name: name.into(),
            version,
        }
    }

    pub fn is_root(&self) -> bool {
        self.name.is_empty() && self.version.is_empty()
    }

    /// The canonical repo name including the version, e.g. `rules_foo+1.2.3`.
    /// Always unique. Only meaningful for a module that came from a
    /// registry — a module with a non-registry override has no version, so
    /// [`Self::canonical_repo_name`] is the one to use there.
    pub fn canonical_repo_name_with_version(&self) -> Option<String> {
        if let Some(well_known) = self.well_known_repo_name() {
            return Some(well_known.to_owned());
        }
        if self.version.is_empty() {
            return None;
        }
        Some(format!("{}+{}", self.name, self.version))
    }

    /// The canonical repo name without the version, e.g. `rules_foo+`.
    /// Unique only when a single version of the module survives selection,
    /// which is the normal case (a `multiple_version_override` is the
    /// exception).
    ///
    /// The trailing `+` is not decoration. Bazel appends it even when the
    /// version is dropped so that a canonical name is never equal to the
    /// apparent name it maps from: code that forgets to apply a repo
    /// mapping then breaks immediately instead of working by accident
    /// until someone adds a `multiple_version_override`.
    pub fn canonical_repo_name(&self) -> String {
        if let Some(well_known) = self.well_known_repo_name() {
            return well_known.to_owned();
        }
        if self.is_root() {
            // The main repository has the empty canonical name.
            return String::new();
        }
        format!("{}+", self.name)
    }

    fn well_known_repo_name(&self) -> Option<&str> {
        WELL_KNOWN_MODULES.iter().copied().find(|&m| m == self.name)
    }
}

impl fmt::Display for ModuleKey {
    /// Bazel's `ModuleKey.toString`: `<root>`, `name@version`, or
    /// `name@_` for a module with a non-registry override.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_root() {
            return f.write_str("<root>");
        }
        if self.version.is_empty() {
            write!(f, "{}@_", self.name)
        } else {
            write!(f, "{}@{}", self.name, self.version)
        }
    }
}

/// A requested dependency: the module name and the version asked for,
/// before selection rewrites it to the version that won.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DepSpec {
    pub name: String,
    pub version: Version,
}

impl DepSpec {
    pub fn new(name: impl Into<String>, version: Version) -> DepSpec {
        DepSpec {
            name: name.into(),
            version,
        }
    }

    pub fn to_module_key(&self) -> ModuleKey {
        ModuleKey::new(self.name.clone(), self.version.clone())
    }

    /// This dep with its version replaced — the operation selection
    /// applies to every edge in the graph.
    pub fn with_version(&self, version: Version) -> DepSpec {
        DepSpec {
            name: self.name.clone(),
            version,
        }
    }
}

/// One `bazel_dep` edge: the apparent repo name the depending module sees,
/// and what it points at.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Dep {
    /// The name this dependency is visible under inside the depending
    /// module — `bazel_dep(name = "x", repo_name = "y")` makes it `y`.
    pub repo_name: String,
    pub spec: DepSpec,
}

/// A use of a module extension, recorded but not executed.
///
/// Running the extension is the fetch phase's job (buildfiji-mum.8); what
/// module resolution needs from it is only that the syntax is accepted and
/// the imports are known, so `mod` and repo mapping can see them.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ExtensionUsage {
    /// The `.bzl` file the extension is defined in, as written.
    pub bzl_file: String,
    pub extension_name: String,
    pub isolate: bool,
    pub dev_dependency: bool,
    /// `use_repo` imports, as `(local name, name exported by the
    /// extension)`. They are equal unless `use_repo(ext, local = "远")`
    /// renamed one.
    pub imports: Vec<(String, String)>,
    /// Tag calls such as `rust.toolchain(edition = "2024")`, in source
    /// order — the order tag classes are evaluated in.
    pub tags: Vec<Tag>,
    /// `override_repo` and `inject_repo` calls on this extension.
    pub repo_overrides: Vec<RepoOverride>,
}

/// A repo an extension generates, redirected to one the module already
/// has. `override_repo` requires the extension to generate that repo;
/// `inject_repo` adds it if the extension does not.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RepoOverride {
    /// The name inside the extension being replaced or added.
    pub overridden_repo_name: String,
    /// The module's own repo that takes its place.
    pub overriding_repo_name: String,
    /// True for `override_repo`, false for `inject_repo`.
    pub must_exist: bool,
}

/// A single tag call on an extension proxy.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Tag {
    pub tag_class: String,
    pub attrs: Attrs,
    pub dev_dependency: bool,
}

/// One evaluated `MODULE.bazel` file.
///
/// "Interim" in Bazel's sense: the deps still name the versions that were
/// *requested*, so this is the input to selection, not its output.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Module {
    pub key: ModuleKey,
    /// The name from `module()`, empty if the file has no `module()` call
    /// (legal only for the root module).
    pub name: String,
    pub version: Version,
    /// The repo name the module refers to itself by; defaults to `name`.
    pub repo_name: String,
    /// `bazel_dep`s in declaration order. A list rather than a map because
    /// the order decides the BFS order of the resolved graph, and so the
    /// order of `fjfj mod graph` output.
    pub deps: Vec<Dep>,
    /// `bazel_dep(..., repo_name = None)` edges: honoured only if the
    /// target module is already in the graph for some other reason.
    pub nodep_deps: Vec<DepSpec>,
    /// The registry this module's file was fetched from; `None` for the
    /// root module and for non-registry overrides.
    pub registry: Option<String>,
    pub bazel_compatibility: Vec<String>,
    pub toolchains_to_register: Vec<String>,
    pub execution_platforms_to_register: Vec<String>,
    pub extension_usages: Vec<ExtensionUsage>,
    /// `flag_alias(name = ..., starlark_flag = ...)`: a short name for a
    /// Starlark build setting, usable on the command line. Root module
    /// only in practice, since only the root's command line is parsed.
    pub flag_aliases: Vec<(String, String)>,
}

impl Module {
    /// Looks up a dep by the apparent repo name it is visible under.
    pub fn dep(&self, repo_name: &str) -> Option<&DepSpec> {
        self.deps
            .iter()
            .find(|d| d.repo_name == repo_name)
            .map(|d| &d.spec)
    }

    /// This module with every dep and nodep edge rewritten — how selection
    /// applies a resolution to a module without mutating the original.
    pub fn with_deps_transformed(&self, f: impl Fn(&DepSpec) -> DepSpec) -> Module {
        Module {
            deps: self
                .deps
                .iter()
                .map(|d| Dep {
                    repo_name: d.repo_name.clone(),
                    spec: f(&d.spec),
                })
                .collect(),
            nodep_deps: self.nodep_deps.iter().map(&f).collect(),
            ..self.clone()
        }
    }
}

/// Bazel's `RepositoryName.VALID_MODULE_NAME`: lowercase letters, digits,
/// `.`, `-` and `_`, beginning with a letter and ending with a letter or
/// digit.
///
/// Narrower than a repo name on purpose — a module name becomes part of
/// registry URLs and of canonical repo names, so it has to survive
/// case-insensitive filesystems and URL paths.
pub fn validate_module_name(name: &str) -> Result<(), InvalidModuleName> {
    let mut chars = name.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    let last_ok = name
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    let body_ok = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'));
    if first_ok && last_ok && body_ok {
        Ok(())
    } else {
        Err(InvalidModuleName(name.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid module name '{0}': valid names must 1) only contain lowercase letters (a-z), digits \
     (0-9), dots (.), hyphens (-), and underscores (_); 2) begin with a lowercase letter; 3) end \
     with a lowercase letter or digit."
)]
pub struct InvalidModuleName(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn canonical_repo_names_follow_bazels_plus_format() {
        let key = ModuleKey::new("rules_foo", v("1.2.3"));
        assert_eq!(
            key.canonical_repo_name_with_version().unwrap(),
            "rules_foo+1.2.3"
        );
        assert_eq!(key.canonical_repo_name(), "rules_foo+");
        // A canonical name is never equal to the apparent name, so code
        // that skips repo mapping fails loudly.
        assert_ne!(key.canonical_repo_name(), key.name);
    }

    #[test]
    fn well_known_modules_keep_their_bare_names() {
        for name in ["bazel_tools", "platforms"] {
            let key = ModuleKey::new(name, v("1.0"));
            assert_eq!(key.canonical_repo_name(), name);
            assert_eq!(key.canonical_repo_name_with_version().unwrap(), name);
        }
    }

    #[test]
    fn root_module_is_the_main_repository() {
        let root = ModuleKey::root();
        assert!(root.is_root());
        assert_eq!(root.canonical_repo_name(), "");
        assert_eq!(root.to_string(), "<root>");
    }

    #[test]
    fn non_registry_override_has_no_versioned_repo_name() {
        let key = ModuleKey::new("rules_foo", Version::EMPTY);
        assert_eq!(key.canonical_repo_name_with_version(), None);
        assert_eq!(key.canonical_repo_name(), "rules_foo+");
        assert_eq!(key.to_string(), "rules_foo@_");
    }

    #[test]
    fn module_names_follow_bazels_grammar() {
        for good in ["foo", "rules_foo", "a", "a.b-c_d0", "abseil-cpp"] {
            validate_module_name(good).unwrap();
        }
        for bad in [
            "", "Foo", "0foo", "_foo", ".foo", "foo_", "foo-", "foo.", "foo bar", "föö",
        ] {
            assert!(
                validate_module_name(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }
}
