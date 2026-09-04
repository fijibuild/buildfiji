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
///
/// `Sync`: one horizon's worth of module files are fetched concurrently
/// (buildfiji-mum.24), sharing one `&dyn ModuleFileSource` across threads.
/// `RegistrySource`'s own `Fetcher` bound is already `Send + Sync`, so
/// nothing had to change for it to satisfy this.
pub trait ModuleFileSource: Sync {
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

        // One horizon's module files are independent fetches — fetching
        // them one at a time paid a full round trip's latency per module,
        // which against a real registry (not this crate's in-memory or
        // local-file test doubles) dominates resolution time. `apply` only
        // borrows `root.module.name` and `overrides`, so it's `Copy` and
        // each thread gets its own.
        let modules: Vec<Result<(ModuleKey, Module)>> = std::thread::scope(|scope| {
            next_keys
                .iter()
                .map(|key| {
                    let key = key.clone();
                    scope.spawn(move || {
                        let module = read_module(&key, overrides, source, apply)?;
                        Ok((key, module))
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().expect("module-fetch thread panicked"))
                .collect()
        });

        let mut next_horizon = Vec::with_capacity(modules.len());
        for result in modules {
            let (key, module) = result?;
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
    let options = dependency_eval_options(key, module_override);
    let file = eval_module_file("MODULE.bazel", &text, &options)?;
    let mut module = file.module.with_deps_transformed(apply);
    module.registry = registry;
    Ok(module)
}

/// [`EvalOptions`] for a dependency module: `include()` is allowed exactly
/// when Bazel allows it there — a non-registry override (root modules go
/// through [`EvalOptions::root`] instead, never this function).
///
/// A `local_path_override`'s contents sit on disk already, so its
/// `include()`s resolve against that path right away. An `archive_override`
/// or `git_override`'s contents don't exist as filesystem content until
/// fetched (buildfiji-mum.8), so those get `allow_include: true` with no
/// source: an `include()` call there still validates, but resolving one
/// fails with "not configured" until fetching lands.
fn dependency_eval_options(
    key: &ModuleKey,
    module_override: Option<&ModuleOverride>,
) -> EvalOptions {
    let options = EvalOptions::dependency(key.clone());
    let Some(ModuleOverride::NonRegistry(non_registry)) = module_override else {
        return options;
    };
    match local_repository_path(non_registry) {
        Some(path) => options.with_include_source(std::rc::Rc::new(
            crate::resolve::WorkspaceIncludeSource::new(path),
        )),
        None => EvalOptions {
            allow_include: true,
            ..options
        },
    }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;
    use crate::eval::EvalOptions;

    /// A [`ModuleFileSource`] that hands out canned `module()`-only text
    /// per name and, along the way, records the highest number of
    /// `module_file` calls it ever saw in flight at once — the thing
    /// buildfiji-mum.24 is actually about. A brief sleep widens the window
    /// a genuinely sequential caller could never produce overlap in.
    struct ConcurrentProbeSource {
        files: BTreeMap<&'static str, &'static str>,
        active: AtomicUsize,
        max_active: Mutex<usize>,
    }

    impl ConcurrentProbeSource {
        fn new(files: BTreeMap<&'static str, &'static str>) -> ConcurrentProbeSource {
            ConcurrentProbeSource {
                files,
                active: AtomicUsize::new(0),
                max_active: Mutex::new(0),
            }
        }

        fn max_concurrent(&self) -> usize {
            *self.max_active.lock().unwrap()
        }
    }

    impl ModuleFileSource for ConcurrentProbeSource {
        fn module_file(
            &self,
            key: &ModuleKey,
            _module_override: Option<&ModuleOverride>,
        ) -> Result<(String, Option<String>)> {
            let now_active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            {
                let mut max_active = self.max_active.lock().unwrap();
                *max_active = (*max_active).max(now_active);
            }
            std::thread::sleep(Duration::from_millis(20));
            self.active.fetch_sub(1, Ordering::SeqCst);

            let text = self
                .files
                .get(key.name.as_str())
                .unwrap_or_else(|| panic!("no fixture module file for {}", key.name));
            Ok((text.to_string(), None))
        }
    }

    #[test]
    fn one_horizons_module_files_are_fetched_concurrently() {
        let root = eval_module_file(
            "MODULE.bazel",
            r#"
module(name = "root", version = "0")
bazel_dep(name = "a", version = "1")
bazel_dep(name = "b", version = "1")
bazel_dep(name = "c", version = "1")
"#,
            &{
                let mut options = EvalOptions::root();
                options.builtin_modules = Vec::new();
                options
            },
        )
        .unwrap();

        let source = ConcurrentProbeSource::new(BTreeMap::from([
            ("a", "module(name = 'a', version = '1')"),
            ("b", "module(name = 'b', version = '1')"),
            ("c", "module(name = 'c', version = '1')"),
            // Every EvalOptions::dependency() implicitly depends on
            // bazel_tools (eval.rs), so a/b/c's own evaluation each need
            // it too — same as any real ModuleFileSource would.
            ("bazel_tools", "module(name = 'bazel_tools')"),
        ]));

        let graph = discover(&root, &Overrides::new(), &source).unwrap();

        assert_eq!(graph.len(), 5); // root + a + b + c + bazel_tools
        assert!(
            source.max_concurrent() > 1,
            "expected a's/b's/c's module_file calls to overlap, but the highest \
             concurrency observed was {}",
            source.max_concurrent()
        );
    }

    fn non_registry_override(
        rule: crate::overrides::RepoRule,
        path: Option<&str>,
    ) -> ModuleOverride {
        use crate::attrs::AttrValue;
        use crate::overrides::{NonRegistryOverride, RepoSpec};
        let attrs = path
            .map(|p| vec![("path".to_owned(), AttrValue::String(p.to_owned()))])
            .unwrap_or_default();
        ModuleOverride::NonRegistry(NonRegistryOverride {
            repo_spec: RepoSpec { rule, attrs },
        })
    }

    /// buildfiji-mum.25: a `local_path_override`'s `include()` resolves
    /// against the override's own path, right away — no fetch needed.
    #[test]
    fn local_path_override_module_can_include() {
        let dir = std::env::temp_dir().join(format!(
            "fjfj-bzlmod-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("extra.MODULE.bazel"),
            "bazel_dep(name = 'c', version = '1')\n",
        )
        .unwrap();

        let key = ModuleKey::new("dep", Version::EMPTY);
        let module_override = non_registry_override(
            crate::overrides::RepoRule::LocalRepository,
            Some(dir.to_str().unwrap()),
        );
        let options = dependency_eval_options(&key, Some(&module_override));
        assert!(options.allow_include);

        let file = eval_module_file(
            "MODULE.bazel",
            "module(name = 'dep', version = '1')\ninclude('//:extra.MODULE.bazel')\n",
            &options,
        )
        .unwrap();
        assert!(
            file.module.deps.iter().any(|d| d.spec.name == "c"),
            "{:?}",
            file.module.deps
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// An `archive_override`/`git_override` module may still *call*
    /// `include()` — Bazel's rule allows it for any non-registry override —
    /// but resolving it needs the fetched checkout (buildfiji-mum.8), which
    /// doesn't exist yet at discovery time.
    #[test]
    fn archive_override_module_can_call_include_but_not_resolve_it_yet() {
        let key = ModuleKey::new("dep", Version::EMPTY);
        let module_override = non_registry_override(crate::overrides::RepoRule::HttpArchive, None);
        let options = dependency_eval_options(&key, Some(&module_override));
        assert!(options.allow_include);
        assert!(options.include_source.is_none());

        let err = eval_module_file(
            "MODULE.bazel",
            "module(name = 'dep', version = '1')\ninclude('//:extra.MODULE.bazel')\n",
            &options,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("no include source is configured"),
            "{err}"
        );
    }
}
