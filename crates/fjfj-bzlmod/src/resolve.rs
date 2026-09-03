//! The top-level entry point: workspace `MODULE.bazel` in, resolved module
//! graph out.
//!
//! This is the sequence Bazel runs as `ModuleFileFunction` →
//! `Discovery` → `Selection` → `BazelDepGraphFunction`, minus the parts
//! that need repositories fetched (module extensions and the lockfile,
//! buildfiji-mum.8 and buildfiji-mum.7).

use std::collections::{BTreeMap, BTreeSet};

use crate::discovery::{ModuleFileSource, discover};
use crate::error::{BzlmodError, Result};
use crate::eval::{EvalOptions, ModuleFile, eval_module_file};
use crate::module::{Module, ModuleKey};
use crate::overrides::{ModuleOverride, NonRegistryOverride, RepoRule, RepoSpec};
use crate::selection::{self, Overrides, Selection};
use crate::version::Version;

/// Whether a yanked module version may be used.
///
/// A yanked version is still served by the registry — pulling it would
/// break every build that already selected it — so refusing to *use* one
/// is the resolver's job, and the user can override that per version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum YankedPolicy {
    /// `--allow_yanked_versions` unset: a selected yanked version is an
    /// error.
    #[default]
    Deny,
    /// `--allow_yanked_versions=all`.
    AllowAll,
    /// `--allow_yanked_versions=foo@1.2.3,...`.
    Allow(BTreeSet<ModuleKey>),
}

/// How to resolve.
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub yanked: YankedPolicy,
    /// `--ignore_dev_dependency`: treat the root module's own dev deps and
    /// dev-only extension usages as if it were a dependency of something
    /// else.
    pub ignore_dev_dependency: bool,
    /// Overrides supplied on the command line, which win over the ones in
    /// the file.
    pub command_overrides: Vec<(String, ModuleOverride)>,
}

/// A resolved module graph.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolution {
    pub root: Module,
    pub overrides: Overrides,
    pub selection: Selection,
    pub warnings: Vec<String>,
}

impl Resolution {
    /// The mapping from canonical repo name to the module that backs it —
    /// the half of repo mapping that module resolution owns (the apparent
    /// side is buildfiji-mum.15).
    pub fn canonical_repo_names(&self) -> BTreeMap<String, ModuleKey> {
        self.selection
            .keys()
            .map(|key| (key.canonical_repo_name(), key.clone()))
            .collect()
    }
}

/// Resolves the module graph for a workspace.
pub fn resolve(
    root_module_file: &str,
    source: &dyn ModuleFileSource,
    options: &ResolveOptions,
) -> Result<Resolution> {
    let mut root_options = EvalOptions::root();
    root_options.ignore_dev_deps = options.ignore_dev_dependency;
    let root = eval_module_file("MODULE.bazel", root_module_file, &root_options)?;

    // `include()` pulls in more directives from another file, so ignoring
    // it would silently resolve a different graph than the one the user
    // wrote. Refuse instead.
    if !root.includes.is_empty() {
        return Err(BzlmodError::BadModule {
            key: "<root>".to_owned(),
            message: format!(
                "include() is not implemented yet (buildfiji-mum.22); found {}",
                root.includes.join(", ")
            ),
        });
    }

    let overrides = build_overrides(&root, options)?;
    let dep_graph = discover(&root, &overrides, source)?;
    let selection = selection::run(&dep_graph, &overrides)?;
    check_yanked(&selection, source, options)?;

    Ok(Resolution {
        root: root.module,
        overrides,
        selection,
        warnings: root.warnings,
    })
}

/// Collects the overrides in force: the root file's, then the command
/// line's (which win), then the implicit ones for built-in modules.
fn build_overrides(root: &ModuleFile, options: &ResolveOptions) -> Result<Overrides> {
    let mut overrides: Overrides = BTreeMap::new();
    for (name, module_override) in root
        .overrides
        .iter()
        .chain(options.command_overrides.iter())
    {
        overrides.insert(name.clone(), module_override.clone());
    }

    // Pinning a dep *below* what the root itself asks for cannot be what
    // the author meant, and MVS would silently ignore it, so Bazel makes
    // it an error rather than a surprise.
    for (name, module_override) in &overrides {
        let ModuleOverride::SingleVersion(svo) = module_override else {
            continue;
        };
        let Some(dep) = root.module.deps.iter().find(|d| d.spec.name == *name) else {
            continue;
        };
        if !dep.spec.version.is_empty() && svo.version < dep.spec.version {
            return Err(BzlmodError::BadModule {
                key: "<root>".to_owned(),
                message: format!(
                    "module '{name}' is overridden to use version '{}', which is lower than the \
                     version '{}' requested by the root module",
                    svo.version, dep.spec.version
                ),
            });
        }
    }

    if let Some(module_override) = overrides.get(&root.module.name)
        && !root.module.name.is_empty()
    {
        return Err(BzlmodError::BadModule {
            key: "<root>".to_owned(),
            message: format!("invalid override for the root module found: {module_override:?}"),
        });
    }

    // `bazel_tools` never comes from a registry.
    overrides
        .entry("bazel_tools".to_owned())
        .or_insert_with(|| {
            ModuleOverride::NonRegistry(NonRegistryOverride {
                repo_spec: RepoSpec {
                    rule: RepoRule::LocalRepository,
                    attrs: Vec::new(),
                },
            })
        });
    Ok(overrides)
}

/// Rejects selected versions the registry has yanked.
fn check_yanked(
    selection: &Selection,
    source: &dyn ModuleFileSource,
    options: &ResolveOptions,
) -> Result<()> {
    if options.yanked == YankedPolicy::AllowAll {
        return Ok(());
    }
    for key in selection.keys() {
        if key.version.is_empty() {
            continue;
        }
        if let YankedPolicy::Allow(allowed) = &options.yanked
            && allowed.contains(key)
        {
            continue;
        }
        if let Some(reason) = source.yanked_versions(&key.name)?.get(&key.version) {
            return Err(BzlmodError::resolution(format!(
                "Yanked version detected in your resolved dependency graph: {key}, for the \
                 reason: {reason}.\nYanked versions may contain serious vulnerabilities and \
                 should not be used. To fix this, use a bazel_dep on a newer version of this \
                 module. To continue using this version, allow it using the \
                 --allow_yanked_versions flag or the BZLMOD_ALLOW_YANKED_VERSIONS env variable."
            )));
        }
    }
    Ok(())
}

/// A yanked version, and why.
pub type YankedVersions = BTreeMap<Version, String>;
