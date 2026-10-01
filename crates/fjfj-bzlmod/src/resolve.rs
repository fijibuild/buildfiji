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
use crate::extension_repos::ExtensionInstance;
use crate::module::{Module, ModuleKey};
use crate::overrides::{ModuleOverride, NonRegistryOverride, RepoRule, RepoSpec};
use crate::registry::Registry;
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

impl YankedPolicy {
    /// Parses `--allow_yanked_versions`' value (also
    /// `BZLMOD_ALLOW_YANKED_VERSIONS`'s): the literal `all`, or a
    /// comma-separated `name@version` list. Bazel calls this exact grammar
    /// out in the flag's own help text.
    pub fn parse(value: &str) -> std::result::Result<YankedPolicy, String> {
        if value == "all" {
            return Ok(YankedPolicy::AllowAll);
        }
        let mut keys = BTreeSet::new();
        for entry in value.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let (name, version) = entry.split_once('@').ok_or_else(|| {
                format!("invalid --allow_yanked_versions entry '{entry}': expected name@version")
            })?;
            let version = Version::parse(version)
                .map_err(|e| format!("invalid --allow_yanked_versions entry '{entry}': {e}"))?;
            keys.insert(ModuleKey::new(name, version));
        }
        Ok(YankedPolicy::Allow(keys))
    }
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
    /// The run writes a lockfile (`--lockfile_mode` other than `off`): look
    /// up every selected version's yanked status even when
    /// `--allow_yanked_versions=all` makes the answer moot, and read each
    /// selected module's `source.json`, as Bazel does, so both are recorded.
    pub for_lockfile: bool,
    /// `--experimental_isolated_extension_usages`.
    pub experimental_isolated_extension_usages: bool,
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
    /// Each selected version the registry has yanked, and why, in selection
    /// order. Only filled when [`ResolveOptions::for_lockfile`] is set or
    /// yanked versions are not all allowed.
    pub selected_yanked: Vec<(ModuleKey, String)>,
}

impl Resolution {
    /// The resolution of a workspace whose root module depends on nothing
    /// that needs fetching: just the root, for tests and tools that run
    /// extensions and repository rules of the main repository.
    pub fn root_only(root: Module) -> Resolution {
        Resolution {
            selection: Selection {
                resolved: vec![(ModuleKey::root(), root.clone())],
                unpruned: Vec::new(),
            },
            root,
            overrides: Overrides::new(),
            warnings: Vec::new(),
            selected_yanked: Vec::new(),
        }
    }

    /// How to make the repository of a selected module: its non-registry
    /// override's rule, or what its registry's `source.json` says.
    pub fn module_repo_spec(&self, key: &ModuleKey, registries: &[Registry]) -> Result<RepoSpec> {
        if let Some(ModuleOverride::NonRegistry(o)) = self.overrides.get(&key.name) {
            return Ok(o.repo_spec.clone());
        }
        let module = self
            .selection
            .resolved
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, m)| m)
            .ok_or_else(|| BzlmodError::bad_module(key, "not a selected module"))?;
        let url = module.registry.as_deref().ok_or_else(|| {
            BzlmodError::bad_module(key, "the module did not come from a registry")
        })?;
        registries
            .iter()
            .find(|r| r.url() == url)
            .ok_or_else(|| BzlmodError::bad_module(key, format!("no registry {url}")))?
            .repo_spec(key)
    }

    /// The mapping from canonical repo name to the module that backs it —
    /// the half of repo mapping that module resolution owns (the apparent
    /// side is buildfiji-mum.15).
    pub fn canonical_repo_names(&self) -> BTreeMap<String, ModuleKey> {
        self.selection
            .keys()
            .map(|key| (key.canonical_repo_name(), key.clone()))
            .collect()
    }

    /// The canonical repo name of a selected module: with its version when
    /// more than one version of the module was selected (a
    /// `multiple_version_override`), which is when the name alone would not
    /// tell them apart.
    pub fn canonical_name_of(&self, key: &ModuleKey) -> String {
        let versions = self.selection.keys().filter(|k| k.name == key.name).count();
        if versions > 1
            && let Some(with_version) = key.canonical_repo_name_with_version()
        {
            return with_version;
        }
        key.canonical_repo_name()
    }

    /// The toolchain patterns modules register, in the order Bazel considers
    /// them (the root module's first, then the others'), each with the
    /// canonical repo of the module that wrote it and the pattern as written.
    pub fn registered_toolchains(&self) -> Vec<(String, String)> {
        self.registered(|m| &m.toolchains_to_register)
    }

    /// The same for `register_execution_platforms`.
    pub fn registered_execution_platforms(&self) -> Vec<(String, String)> {
        self.registered(|m| &m.execution_platforms_to_register)
    }

    fn registered(&self, which: impl Fn(&Module) -> &Vec<String>) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (key, module) in self.selection.resolved.iter() {
            let repo = self.canonical_name_of(key);
            for pattern in which(module) {
                out.push((repo.clone(), pattern.clone()));
            }
        }
        out
    }

    /// What each apparent repo name means in each repo of the module graph
    /// (Bazel's repo mapping, what `bazel mod dump_repo_mapping` prints): for
    /// every selected module, in breadth-first order, its canonical repo
    /// name and `(apparent name, canonical name)` rows. The repos its
    /// `use_repo`s import come first, in the order they are written; then a
    /// module sees itself under its `repo_name`, then each `bazel_dep`
    /// under the name the dependency was given, built-in `bazel_tools` last;
    /// the root module sees the main repo as `""` too. The root module's
    /// `override_repo` calls change what an import means for every module.
    pub fn repo_mappings(&self) -> Vec<(String, Vec<(String, String)>)> {
        let extensions = self.extensions();
        let overrides = self.override_targets(&extensions);
        self.selection
            .resolved
            .iter()
            .map(|(key, module)| {
                let own = self.canonical_name_of(key);
                (
                    own,
                    self.module_mapping(key, module, &extensions, Some(&overrides)),
                )
            })
            .collect()
    }

    /// The rows of one module's mapping (see [`Resolution::repo_mappings`]),
    /// with the root module's overrides applied when `overrides` is given.
    pub(crate) fn module_mapping(
        &self,
        key: &ModuleKey,
        module: &Module,
        extensions: &[ExtensionInstance],
        overrides: Option<&BTreeMap<(usize, String), String>>,
    ) -> Vec<(String, String)> {
        let mut rows: Vec<(String, String)> = Vec::new();
        for (usage_index, usage) in module.extension_usages.iter().enumerate() {
            let Some((at, extension)) = extensions
                .iter()
                .enumerate()
                .find(|(_, e)| e.usages.iter().any(|(k, i)| k == key && *i == usage_index))
            else {
                continue;
            };
            for (local, exported) in &usage.imports {
                let canonical = overrides
                    .and_then(|o| o.get(&(at, exported.clone())))
                    .cloned()
                    .unwrap_or_else(|| extension.repo_name(exported));
                rows.push((local.clone(), canonical));
            }
        }
        rows.extend(self.base_rows(key, module));
        rows
    }

    /// A module's own name and its `bazel_dep`s, what needs no extension.
    pub(crate) fn base_rows(&self, key: &ModuleKey, module: &Module) -> Vec<(String, String)> {
        let own = self.canonical_name_of(key);
        let mut rows: Vec<(String, String)> = Vec::new();
        if key.is_root() {
            rows.push((String::new(), own.clone()));
        }
        rows.push((module.repo_name.clone(), own));
        for dep in &module.deps {
            let target = self.canonical_name_of(&dep.spec.to_module_key());
            rows.push((dep.repo_name.clone(), target));
        }
        rows
    }
}

/// Resolves the module graph for a workspace.
pub fn resolve(
    root_module_file: &str,
    source: &dyn ModuleFileSource,
    options: &ResolveOptions,
) -> Result<Resolution> {
    let mut root_options = EvalOptions::root();
    root_options.isolated_extension_usages = options.experimental_isolated_extension_usages;
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
    let selected_yanked = check_yanked(&selection, source, options)?;
    if options.for_lockfile {
        for (key, module) in &selection.resolved {
            if let Some(registry) = &module.registry {
                source.read_source(key, registry)?;
            }
        }
    }

    Ok(Resolution {
        root: root.module,
        overrides,
        selection,
        warnings: root.warnings,
        selected_yanked,
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

/// Rejects selected versions the registry has yanked, and returns the ones
/// it has that were allowed anyway.
fn check_yanked(
    selection: &Selection,
    source: &dyn ModuleFileSource,
    options: &ResolveOptions,
) -> Result<Vec<(ModuleKey, String)>> {
    let mut yanked = Vec::new();
    if options.yanked == YankedPolicy::AllowAll && !options.for_lockfile {
        return Ok(yanked);
    }
    // The registry is asked about every selected module; ask for them together.
    // What fails here fails again, in order, below.
    let names: BTreeSet<&str> = selection
        .keys()
        .filter(|k| !k.version.is_empty())
        .map(|k| k.name.as_str())
        .collect();
    let names: Vec<&str> = names.into_iter().collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..names.len().min(16) {
            scope.spawn(|| {
                loop {
                    let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(name) = names.get(at) else { break };
                    let _ = source.yanked_versions(name);
                }
            });
        }
    });
    // The order Bazel's map of every selected module iterates in: which one it
    // complains about first, and the order they are written in.
    for key in crate::java_map::hash_map_order(selection.keys(), selection.resolved.len()) {
        if key.version.is_empty() {
            continue;
        }
        let Some(reason) = source
            .yanked_versions(&key.name)?
            .get(&key.version)
            .cloned()
        else {
            continue;
        };
        let allowed = match &options.yanked {
            YankedPolicy::AllowAll => true,
            YankedPolicy::Allow(allowed) => allowed.contains(key),
            YankedPolicy::Deny => false,
        };
        if !allowed {
            return Err(BzlmodError::resolution(format!(
                "Yanked version detected in your resolved dependency graph: {key}, for the \
                 reason: {reason}.\nYanked versions may contain serious vulnerabilities and \
                 should not be used. To fix this, use a bazel_dep on a newer version of this \
                 module. To continue using this version, allow it using the \
                 --allow_yanked_versions flag or the BZLMOD_ALLOW_YANKED_VERSIONS env variable."
            )));
        }
        yanked.push((key.clone(), reason));
    }
    Ok(yanked)
}

/// A yanked version, and why.
pub type YankedVersions = BTreeMap<Version, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yanked_policy_parses_all() {
        assert_eq!(YankedPolicy::parse("all").unwrap(), YankedPolicy::AllowAll);
    }

    #[test]
    fn yanked_policy_parses_name_at_version_list() {
        let policy = YankedPolicy::parse("a@1.2.3,b@2.0").unwrap();
        assert_eq!(
            policy,
            YankedPolicy::Allow(BTreeSet::from([
                ModuleKey::new("a", Version::parse("1.2.3").unwrap()),
                ModuleKey::new("b", Version::parse("2.0").unwrap()),
            ]))
        );
    }

    #[test]
    fn yanked_policy_rejects_missing_at_version() {
        assert!(YankedPolicy::parse("a").is_err());
    }

    #[test]
    fn include_label_to_path_splits_package_and_target() {
        assert_eq!(
            label_to_relative_path("//dir1/dir2:name.MODULE.bazel").unwrap(),
            PathBuf::from("dir1/dir2/name.MODULE.bazel")
        );
        assert_eq!(
            label_to_relative_path("//:name.MODULE.bazel").unwrap(),
            PathBuf::from("name.MODULE.bazel")
        );
        assert!(label_to_relative_path("not/a/label").is_err());
    }
}
