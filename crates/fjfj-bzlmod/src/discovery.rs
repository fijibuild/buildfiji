//! Module discovery: reading module files outward from the root until the
//! whole dependency graph is known.
//!
//! Ported from Bazel 9.2.0's `Discovery.java`. It is a breadth-first
//! search with one wrinkle — *nodep* edges. A `bazel_dep(..., repo_name =
//! None)` says "if this module is in the graph anyway, I depend on it";
//! whether it is in the graph can change as more of the graph is
//! discovered, so discovery runs in rounds and repeats while a previously
//! unfulfilled nodep edge has become fulfillable.
//!
//! Overrides are applied here, before selection ever sees the graph: an
//! override changes which version of a module gets *fetched*, so it has to
//! act on the edge, not on the result.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{BzlmodError, Result};
use crate::eval::{EvalOptions, ModuleFile, eval_module_file};
use crate::module::{DepSpec, Module, ModuleKey};
use crate::overrides::ModuleOverride;
use crate::registry::Registry;
use crate::selection::{DepGraph, Overrides};
use crate::version::Version;

/// Where a module file comes from, given the module's key and the override
/// (if any) that applies to it.
pub trait ModuleFileSource {
    /// Returns the file's text and the registry URL it came from, or
    /// `None` for a module that did not come from a registry.
    fn module_file(
        &self,
        key: &ModuleKey,
        module_override: Option<&ModuleOverride>,
    ) -> Result<(String, Option<String>)>;

    /// The versions of a module the registry has yanked, and why.
    /// Consulted after selection, since only a selected version matters.
    fn yanked_versions(&self, _module_name: &str) -> Result<BTreeMap<Version, String>> {
        Ok(BTreeMap::new())
    }
}

/// The ordinary source: a list of registries tried in order, with a
/// module's own override able to replace that list.
pub struct RegistrySource {
    registries: Vec<Registry>,
    /// Registries named by an override, keyed by URL so that two modules
    /// pointing at the same one share it.
    overridden: BTreeMap<String, Registry>,
    /// Module files supplied by fjfj rather than by a registry — today
    /// just `bazel_tools`, which Bazel ships inside the binary.
    builtin: BTreeMap<String, String>,
}

/// The `bazel_tools` module file fjfj supplies until it ships an embedded
/// tools repository (buildfiji-mum.23).
///
/// Bazel's real one carries `bazel_dep`s of its own (`rules_cc`,
/// `rules_java`, `rules_license`, ...), so a graph resolved against this
/// placeholder is missing them. It is invisible to `mod graph`, which
/// hides the `bazel_tools` subtree, but it is not invisible to a build.
pub const PLACEHOLDER_BAZEL_TOOLS_MODULE: &str = "module(name = \"bazel_tools\")\n";

impl RegistrySource {
    pub fn new(registries: Vec<Registry>) -> RegistrySource {
        RegistrySource {
            registries,
            overridden: BTreeMap::new(),
            builtin: BTreeMap::from([(
                "bazel_tools".to_owned(),
                PLACEHOLDER_BAZEL_TOOLS_MODULE.to_owned(),
            )]),
        }
    }

    /// Registers a module file that fjfj supplies itself. `bazel_tools` is
    /// the only one Bazel has, and it needs one because every module
    /// implicitly depends on it.
    pub fn with_builtin_module(
        mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        self.builtin.insert(name.into(), source.into());
        self
    }

    /// Adds registries named by `single_version_override` /
    /// `multiple_version_override` so they can be reached later.
    pub fn with_override_registries(mut self, registries: Vec<Registry>) -> Self {
        for registry in registries {
            self.overridden.insert(registry.url().to_owned(), registry);
        }
        self
    }
}

impl ModuleFileSource for RegistrySource {
    fn module_file(
        &self,
        key: &ModuleKey,
        module_override: Option<&ModuleOverride>,
    ) -> Result<(String, Option<String>)> {
        if let Some(source) = self.builtin.get(&key.name) {
            return Ok((source.clone(), None));
        }
        match module_override {
            Some(ModuleOverride::NonRegistry(non_registry)) => {
                // A local path is readable now; an archive or a git
                // checkout is not, since the module file only exists once
                // the repo has been fetched.
                if let Some(path) = local_repository_path(non_registry) {
                    let file = std::path::Path::new(&path).join("MODULE.bazel");
                    let source = std::fs::read_to_string(&file).map_err(|e| {
                        BzlmodError::bad_module(key, format!("{}: {e}", file.display()))
                    })?;
                    return Ok((source, None));
                }
                Err(BzlmodError::FetchRequired {
                    key: key.to_string(),
                    kind: match non_registry.repo_spec.rule {
                        crate::overrides::RepoRule::HttpArchive => "archive",
                        crate::overrides::RepoRule::GitRepository => "git",
                        crate::overrides::RepoRule::LocalRepository => "local path",
                    },
                })
            }
            _ => {
                if key.version.is_empty() {
                    return Err(BzlmodError::MissingVersion {
                        name: key.name.clone(),
                    });
                }
                // An override may replace the whole registry list.
                let registries: Vec<&Registry> = match module_override.and_then(|o| o.registry()) {
                    Some(url) => self.overridden.get(url).into_iter().collect(),
                    None => self.registries.iter().collect(),
                };
                let mut tried = Vec::new();
                for registry in registries {
                    match registry.module_file(key)? {
                        Some(source) => return Ok((source, Some(registry.url().to_owned()))),
                        None => tried.push(format!("{}: not found", registry.module_file_url(key))),
                    }
                }
                Err(BzlmodError::ModuleNotFound {
                    key: key.to_string(),
                    tried: tried.join("\n* "),
                })
            }
        }
    }

    fn yanked_versions(&self, module_name: &str) -> Result<BTreeMap<Version, String>> {
        self.metadata_yanked(module_name)
    }
}

impl RegistrySource {
    /// Reads `metadata.json` from the first registry that has one for the
    /// module.
    fn metadata_yanked(&self, module_name: &str) -> Result<BTreeMap<Version, String>> {
        for registry in self.registries.iter().chain(self.overridden.values()) {
            if let Some(metadata) = registry.metadata(module_name)? {
                return Ok(metadata.yanked_versions);
            }
        }
        Ok(BTreeMap::new())
    }
}

fn local_repository_path(non_registry: &crate::overrides::NonRegistryOverride) -> Option<String> {
    if non_registry.repo_spec.rule != crate::overrides::RepoRule::LocalRepository {
        return None;
    }
    non_registry
        .repo_spec
        .attrs
        .iter()
        .find(|(name, _)| name == "path")
        .and_then(|(_, value)| value.as_str())
        .map(str::to_owned)
}

/// Discovers the whole dependency graph starting from an evaluated root
/// module file.
pub fn discover(
    root: &ModuleFile,
    overrides: &Overrides,
    source: &dyn ModuleFileSource,
) -> Result<DepGraph> {
    // Nodep edges are fulfilled against the module *names* that existed at
    // the end of the previous round, so the first round only knows the
    // root's own name.
    let mut previous_round_names: BTreeSet<String> = BTreeSet::from([root.module.name.clone()]);
    loop {
        let round = discover_round(root, overrides, source, &previous_round_names)?;
        let names: BTreeSet<String> = round
            .graph
            .values()
            .map(|module| module.name.clone())
            .collect();
        // Another round is only worth running if a nodep edge that went
        // unfulfilled now names a module that has since appeared.
        let progressed = round
            .unfulfilled_nodep_names
            .iter()
            .any(|name| names.contains(name));
        previous_round_names = names;
        if !progressed {
            return Ok(round.graph);
        }
    }
}

struct Round {
    graph: DepGraph,
    unfulfilled_nodep_names: BTreeSet<String>,
}

fn discover_round(
    root: &ModuleFile,
    overrides: &Overrides,
    source: &dyn ModuleFileSource,
    previous_round_names: &BTreeSet<String>,
) -> Result<Round> {
    let mut graph = DepGraph::new();
    let mut unfulfilled_nodep_names = BTreeSet::new();

    let apply = |dep: &DepSpec| apply_overrides(dep, &root.module.name, overrides);
    graph.insert(ModuleKey::root(), root.module.with_deps_transformed(apply));

    let mut horizon = vec![ModuleKey::root()];
    while !horizon.is_empty() {
        let mut next_keys: Vec<ModuleKey> = Vec::new();
        for key in &horizon {
            let module = &graph[key];
            for dep in &module.deps {
                let dep_key = dep.spec.to_module_key();
                if !graph.contains_key(&dep_key) && !next_keys.contains(&dep_key) {
                    next_keys.push(dep_key);
                }
            }
            for dep in &module.nodep_deps {
                let dep_key = dep.to_module_key();
                if graph.contains_key(&dep_key) || next_keys.contains(&dep_key) {
                    continue;
                }
                if !previous_round_names.contains(&dep.name) {
                    unfulfilled_nodep_names.insert(dep.name.clone());
                    continue;
                }
                next_keys.push(dep_key);
            }
        }

        let mut next_horizon = Vec::new();
        for key in next_keys {
            let module = read_module(&key, overrides, source, apply)?;
            graph.insert(key.clone(), module);
            next_horizon.push(key);
        }
        horizon = next_horizon;
    }

    // A nodep edge whose module never showed up is dropped entirely, as if
    // it had never been written.
    let present: BTreeSet<ModuleKey> = graph.keys().cloned().collect();
    for module in graph.values_mut() {
        module
            .nodep_deps
            .retain(|dep| present.contains(&dep.to_module_key()));
    }

    Ok(Round {
        graph,
        unfulfilled_nodep_names,
    })
}

fn read_module(
    key: &ModuleKey,
    overrides: &Overrides,
    source: &dyn ModuleFileSource,
    apply: impl Fn(&DepSpec) -> DepSpec,
) -> Result<Module> {
    let module_override = overrides.get(&key.name);
    let (text, registry) = source.module_file(key, module_override)?;
    let file = eval_module_file("MODULE.bazel", &text, &EvalOptions::dependency(key.clone()))?;
    let mut module = file.module.with_deps_transformed(apply);
    module.registry = registry;
    Ok(module)
}

/// Rewrites a dependency edge for the overrides in force.
///
/// Only the *root* module's overrides reach this function — a dependency's
/// overrides were already dropped during evaluation — which is what makes
/// an override a property of the workspace rather than of whoever you
/// happen to depend on.
fn apply_overrides(dep: &DepSpec, root_module_name: &str, overrides: &Overrides) -> DepSpec {
    // A dep on the root module is the root module. This is how a module
    // can be developed against itself: `local_path_override` on your own
    // name is not needed, the edge just folds back to the root.
    if !root_module_name.is_empty() && dep.name == root_module_name {
        return DepSpec::new(String::new(), Version::EMPTY);
    }
    match overrides.get(&dep.name) {
        // A non-registry module has no version at all; it is whatever the
        // override points at.
        Some(o) if o.is_non_registry() => dep.with_version(Version::EMPTY),
        Some(ModuleOverride::SingleVersion(svo)) if !svo.version.is_empty() => {
            dep.with_version(svo.version.clone())
        }
        _ => dep.clone(),
    }
}
