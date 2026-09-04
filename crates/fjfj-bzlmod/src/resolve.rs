//! The top-level entry point: workspace `MODULE.bazel` in, resolved module
//! graph out.
//!
//! This is the sequence Bazel runs as `ModuleFileFunction` →
//! `Discovery` → `Selection` → `BazelDepGraphFunction`, minus the parts
//! that need repositories fetched (module extensions and the lockfile,
//! buildfiji-mum.8 and buildfiji-mum.7).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::rc::Rc;

use crate::discovery::{ModuleFileSource, discover};
use crate::error::{BzlmodError, Result};
use crate::eval::{
    EvalOptions, IncludeSource, ModuleFile, eval_module_file, validate_include_label,
};
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
    /// How to fetch the text an `include()` in the root module names.
    /// `None` makes an `include()` in the root file an error — same as
    /// having no include source at all (buildfiji-mum.22).
    pub include_source: Option<Rc<dyn IncludeSource>>,
}

/// Resolves `include()` labels against the workspace directory the root
/// `MODULE.bazel` lives in — the ordinary case, and the only one
/// buildfiji-mum.22 wires up; a non-registry override's own `include()`s
/// are still refused (buildfiji-mum.8 territory: that needs the override
/// fetched first).
#[derive(Debug)]
pub struct WorkspaceIncludeSource {
    workspace_root: PathBuf,
}

impl WorkspaceIncludeSource {
    pub fn new(workspace_root: impl Into<PathBuf>) -> WorkspaceIncludeSource {
        WorkspaceIncludeSource {
            workspace_root: workspace_root.into(),
        }
    }
}

impl IncludeSource for WorkspaceIncludeSource {
    fn read(&self, label: &str) -> Result<String> {
        let path = self.workspace_root.join(label_to_relative_path(label)?);
        std::fs::read_to_string(&path).map_err(|e| BzlmodError::BadModule {
            key: "<root>".to_owned(),
            message: format!(
                "include(\"{label}\") could not read {}: {e}",
                path.display()
            ),
        })
    }
}

/// `//dir1/dir2:name.MODULE.bazel` -> `dir1/dir2/name.MODULE.bazel`,
/// `//:name.MODULE.bazel` -> `name.MODULE.bazel`. Trusts the label has
/// already passed [`validate_include_label`] for everything except the
/// package/target split, which it re-derives the same way.
fn label_to_relative_path(label: &str) -> Result<PathBuf> {
    validate_include_label(label).map_err(|e| BzlmodError::BadModule {
        key: "<root>".to_owned(),
        message: e.into_anyhow().to_string(),
    })?;
    // Already validated to start with "//" and contain ':'.
    let rest = &label[2..];
    let (package, target) = rest.split_once(':').expect("validated above");
    let mut path = PathBuf::new();
    if !package.is_empty() {
        path.push(package);
    }
    path.push(target);
    Ok(path)
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
    if let Some(include_source) = &options.include_source {
        root_options = root_options.with_include_source(include_source.clone());
    }
    // `include()` executes inline during evaluation (eval.rs), landing in
    // `root.module`/`root.overrides` exactly as if the included text had
    // been pasted at the call site — nothing left to resolve here.
    // `root.includes` is only the audit trail of labels that were reached.
    let root = eval_module_file("MODULE.bazel", root_module_file, &root_options)?;

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
