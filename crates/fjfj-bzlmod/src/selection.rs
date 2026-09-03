//! Minimal Version Selection: turning the discovered dependency graph into
//! the one the build actually uses.
//!
//! Ported from Bazel 9.2.0's `Selection.java`. The shape of the algorithm:
//!
//! 1. Every module falls into a *selection group*. Normally that is just
//!    its name, so one version per module name survives — the highest one
//!    requested by anybody.
//! 2. A `multiple_version_override` splits a name into several groups, one
//!    per allowed version, and each module "snaps up" to the lowest
//!    allowed version that is not below it.
//! 3. Every dependency edge is rewritten to the version its group
//!    selected, and the graph is walked from the root. Whatever is not
//!    reachable after the rewrite is dropped — removing one module can
//!    strand others.
//!
//! **Compatibility levels are absent on purpose.** Bazel 9.2.0 parses
//! `compatibility_level` and `max_compatibility_level`, warns that they
//! are no-ops, and hard-codes every module's level to 0
//! (`ModuleFileGlobals.module`). With one level, a `DepSpec` always has
//! exactly one candidate version, so Bazel's search over combinations of
//! candidates — its `enumerateStrategies` cartesian product — collapses to
//! a single strategy, and this port evaluates that one strategy directly.
//! Reintroducing levels means reintroducing the search; see
//! `docs/design/starlark-and-loading.md`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::error::{BzlmodError, Result};
use crate::module::{DepSpec, Module, ModuleKey};
use crate::overrides::ModuleOverride;
use crate::version::Version;

/// The discovered graph: every module whose file was read, keyed by the
/// version it was requested at.
pub type DepGraph = BTreeMap<ModuleKey, Module>;

/// Root-module overrides, by module name.
pub type Overrides = BTreeMap<String, ModuleOverride>;

/// What selection produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    /// The modules the build uses, in breadth-first order from the root,
    /// with every dep edge pointing at a selected version.
    pub resolved: Vec<(ModuleKey, Module)>,
    /// Every discovered module, also with edges rewritten, including the
    /// ones selection dropped. `fjfj mod` needs this to explain why a
    /// version lost.
    pub unpruned: Vec<(ModuleKey, Module)>,
}

impl Selection {
    /// The selected module for a name, if one survived.
    pub fn module(&self, name: &str) -> Option<&Module> {
        self.resolved
            .iter()
            .find(|(key, _)| key.name == name)
            .map(|(_, module)| module)
    }

    /// The selected keys, in breadth-first order.
    pub fn keys(&self) -> impl Iterator<Item = &ModuleKey> {
        self.resolved.iter().map(|(key, _)| key)
    }
}

/// A module's selection group: its name plus, when a
/// `multiple_version_override` is in play, the allowed version it snaps up
/// to. `Version::EMPTY` as the target means "no override" — and, because
/// the empty version sorts above every real one, it also means "every
/// dependency edge on this name may resolve here", which is exactly the
/// no-override behaviour.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SelectionGroup {
    module_name: String,
    target_allowed_version: Version,
}

/// Runs version selection.
pub fn run(dep_graph: &DepGraph, overrides: &Overrides) -> Result<Selection> {
    let allowed_version_sets = compute_allowed_version_sets(overrides, dep_graph)?;

    let selection_groups: BTreeMap<ModuleKey, SelectionGroup> = dep_graph
        .keys()
        .map(|key| {
            let group = compute_selection_group(key, &allowed_version_sets);
            (key.clone(), group)
        })
        .collect();

    // The heart of MVS: each group keeps the highest version anyone asked
    // for.
    let mut selected_versions: BTreeMap<&SelectionGroup, Version> = BTreeMap::new();
    for (key, group) in &selection_groups {
        selected_versions
            .entry(group)
            .and_modify(|selected| {
                if key.version > *selected {
                    *selected = key.version.clone();
                }
            })
            .or_insert_with(|| key.version.clone());
    }

    let resolve = |dep: &DepSpec| -> Version {
        resolve_dep(dep, &selected_versions).unwrap_or_else(|| dep.version.clone())
    };

    // Two walks. The first honours nodep edges, so that a module reachable
    // only through one still gets its conflicts checked; the second
    // ignores them, so that a module reachable *only* through a nodep edge
    // does not end up in the final graph. Bazel notes the second walk
    // cannot fail once the first has succeeded.
    walk(dep_graph, overrides, &selection_groups, &resolve, false)?;
    let resolved = walk(dep_graph, overrides, &selection_groups, &resolve, true)?;

    let unpruned = dep_graph
        .iter()
        .map(|(key, module)| {
            (
                key.clone(),
                module.with_deps_transformed(|dep| dep.with_version(resolve(dep))),
            )
        })
        .collect();

    Ok(Selection { resolved, unpruned })
}

/// For each module under a `multiple_version_override`, the set of
/// versions that may coexist. Seeded with `Version::EMPTY` so that a
/// module above every allowed version gets a target of EMPTY, which the
/// walk then reports — but only if it is still reachable, since an
/// unreferenced module is not an error.
fn compute_allowed_version_sets(
    overrides: &Overrides,
    dep_graph: &DepGraph,
) -> Result<BTreeMap<String, BTreeSet<Version>>> {
    let mut sets: BTreeMap<String, BTreeSet<Version>> = BTreeMap::new();
    for (module_name, module_override) in overrides {
        let ModuleOverride::MultipleVersion(mvo) = module_override else {
            continue;
        };
        for allowed_version in &mvo.versions {
            let key = ModuleKey::new(module_name.clone(), allowed_version.clone());
            if !dep_graph.contains_key(&key) {
                return Err(BzlmodError::resolution(format!(
                    "multiple_version_override for module {module_name} contains version \
                     {allowed_version}, but it doesn't exist in the dependency graph"
                )));
            }
            sets.entry(module_name.clone())
                .or_insert_with(|| BTreeSet::from([Version::EMPTY]))
                .insert(allowed_version.clone());
        }
    }
    Ok(sets)
}

fn compute_selection_group(
    key: &ModuleKey,
    allowed_version_sets: &BTreeMap<String, BTreeSet<Version>>,
) -> SelectionGroup {
    let target_allowed_version = match allowed_version_sets.get(&key.name) {
        None => Version::EMPTY,
        // The lowest allowed version that is not below this module's own.
        Some(allowed) => allowed
            .range(key.version.clone()..)
            .next()
            .cloned()
            .unwrap_or(Version::EMPTY),
    };
    SelectionGroup {
        module_name: key.name.clone(),
        target_allowed_version,
    }
}

/// The version a dependency edge resolves to: among the groups for that
/// module name whose target allows the requested version, the lowest
/// selected version — "upgrade to the nearest allowed version", not "jump
/// to the newest".
fn resolve_dep(
    dep: &DepSpec,
    selected_versions: &BTreeMap<&SelectionGroup, Version>,
) -> Option<Version> {
    selected_versions
        .iter()
        .filter(|(group, _)| {
            group.module_name == dep.name && group.target_allowed_version >= dep.version
        })
        .map(|(_, version)| version.clone())
        .min()
}

/// Breadth-first from the root, rewriting edges as it goes and checking
/// that what it reaches is consistent.
fn walk(
    dep_graph: &DepGraph,
    overrides: &Overrides,
    selection_groups: &BTreeMap<ModuleKey, SelectionGroup>,
    resolve: &impl Fn(&DepSpec) -> Version,
    ignore_nodeps: bool,
) -> Result<Vec<(ModuleKey, Module)>> {
    let mut result: Vec<(ModuleKey, Module)> = Vec::new();
    let mut module_by_name: BTreeMap<String, (ModuleKey, Option<ModuleKey>)> = BTreeMap::new();
    let mut known: BTreeSet<ModuleKey> = BTreeSet::new();
    let mut to_visit: VecDeque<(ModuleKey, Option<ModuleKey>)> = VecDeque::new();

    let root = ModuleKey::root();
    known.insert(root.clone());
    to_visit.push_back((root, None));

    while let Some((key, dependent)) = to_visit.pop_front() {
        let module = dep_graph.get(&key).ok_or_else(|| {
            BzlmodError::resolution(format!(
                "selected module {key} is not in the dependency graph"
            ))
        })?;
        let module = module.with_deps_transformed(|dep| dep.with_version(resolve(dep)));

        visit(
            &key,
            &module,
            dependent.as_ref(),
            overrides,
            selection_groups,
            &mut module_by_name,
        )?;

        let edges = module
            .deps
            .iter()
            .map(|d| &d.spec)
            .chain(module.nodep_deps.iter().filter(|_| !ignore_nodeps));
        for dep in edges {
            let dep_key = dep.to_module_key();
            if known.insert(dep_key.clone()) {
                to_visit.push_back((dep_key, Some(key.clone())));
            }
        }
        result.push((key, module));
    }
    Ok(result)
}

fn visit(
    key: &ModuleKey,
    module: &Module,
    dependent: Option<&ModuleKey>,
    overrides: &Overrides,
    selection_groups: &BTreeMap<ModuleKey, SelectionGroup>,
    module_by_name: &mut BTreeMap<String, (ModuleKey, Option<ModuleKey>)>,
) -> Result<()> {
    match overrides.get(&key.name) {
        Some(ModuleOverride::MultipleVersion(mvo)) => {
            if selection_groups
                .get(key)
                .is_some_and(|group| group.target_allowed_version.is_empty())
            {
                // Reachable, and above every version the override allows.
                let from = dependent.map(ModuleKey::to_string).unwrap_or_default();
                let allowed = mvo
                    .versions
                    .iter()
                    .map(Version::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(BzlmodError::resolution(format!(
                    "{from} depends on {key} which is not allowed by the \
                     multiple_version_override on {}, which allows only [{allowed}]",
                    key.name
                )));
            }
        }
        _ => {
            // Without a multiple-version override, one version per name.
            // Unreachable while every module's compatibility level is 0:
            // one selection group per name means one selected version per
            // name. Kept because it is the invariant the walk relies on,
            // and it is what would catch a mistake in `resolve_dep`.
            if let Some((existing_key, existing_from)) =
                module_by_name.insert(module.name.clone(), (key.clone(), dependent.cloned()))
                && existing_key != *key
            {
                return Err(BzlmodError::resolution(format!(
                    "{} depends on {key} with compatibility level 0, but {} depends on \
                     {existing_key} with compatibility level 0 which is different",
                    dependent.map(ModuleKey::to_string).unwrap_or_default(),
                    existing_from.map(|k| k.to_string()).unwrap_or_default(),
                )));
            }
        }
    }

    // Two `bazel_dep`s that ended up on the same module are almost always
    // a mistake: the author asked for two versions and selection gave them
    // one, so one of the two repo names now silently aliases the other.
    let mut seen: BTreeMap<ModuleKey, &str> = BTreeMap::new();
    for dep in &module.deps {
        let dep_key = dep.spec.to_module_key();
        if let Some(previous) = seen.insert(dep_key.clone(), &dep.repo_name) {
            return Err(BzlmodError::resolution(format!(
                "{key} depends on {dep_key} at least twice (with repo names {} and {previous}). \
                 Consider adding a multiple_version_override if you want to depend on multiple \
                 versions of {} simultaneously",
                dep.repo_name, dep.spec.name
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Dep;
    use crate::overrides::{MultipleVersionOverride, SingleVersionOverride};

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    /// Builds a module with the given `name@version -> [dep@version]`
    /// shape. The root module is the one with an empty name.
    fn module(name: &str, version: &str, deps: &[(&str, &str)]) -> (ModuleKey, Module) {
        let key = if name.is_empty() {
            ModuleKey::root()
        } else {
            ModuleKey::new(name, v(version))
        };
        let module = Module {
            key: key.clone(),
            name: if name.is_empty() {
                "root".to_owned()
            } else {
                name.to_owned()
            },
            version: key.version.clone(),
            repo_name: name.to_owned(),
            deps: deps
                .iter()
                .map(|(dep_name, dep_version)| Dep {
                    repo_name: (*dep_name).to_owned(),
                    spec: DepSpec::new(*dep_name, v(dep_version)),
                })
                .collect(),
            nodep_deps: Vec::new(),
            registry: None,
            bazel_compatibility: Vec::new(),
            toolchains_to_register: Vec::new(),
            execution_platforms_to_register: Vec::new(),
            extension_usages: Vec::new(),
            flag_aliases: Vec::new(),
        };
        (key, module)
    }

    fn graph(modules: Vec<(ModuleKey, Module)>) -> DepGraph {
        modules.into_iter().collect()
    }

    fn selected(selection: &Selection) -> Vec<String> {
        selection.keys().map(ModuleKey::to_string).collect()
    }

    #[test]
    fn picks_the_highest_requested_version() {
        let dep_graph = graph(vec![
            module("", "", &[("a", "1.0"), ("b", "1.0")]),
            module("a", "1.0", &[]),
            module("a", "2.0", &[]),
            module("b", "1.0", &[("a", "2.0")]),
        ]);
        let selection = run(&dep_graph, &Overrides::new()).unwrap();
        assert_eq!(selected(&selection), ["<root>", "a@2.0", "b@1.0"]);
    }

    #[test]
    fn drops_modules_that_lose_their_only_dependent() {
        // c@1.0 brings d, but b upgrades c past it.
        let dep_graph = graph(vec![
            module("", "", &[("b", "1.0"), ("c", "1.0")]),
            module("b", "1.0", &[("c", "2.0")]),
            module("c", "1.0", &[("d", "1.0")]),
            module("c", "2.0", &[]),
            module("d", "1.0", &[]),
        ]);
        let selection = run(&dep_graph, &Overrides::new()).unwrap();
        assert_eq!(selected(&selection), ["<root>", "b@1.0", "c@2.0"]);
        // The unpruned graph keeps everything discovered, with edges
        // rewritten, so `mod explain` can say why d went away.
        assert_eq!(selection.unpruned.len(), 5);
    }

    #[test]
    fn a_module_may_not_depend_on_one_module_twice() {
        // Two bazel_deps on `a` under different repo names collapse onto
        // one version after selection.
        let mut root = module("", "", &[("a", "1.0")]).1;
        root.deps.push(Dep {
            repo_name: "a_new".to_owned(),
            spec: DepSpec::new("a", v("2.0")),
        });
        let dep_graph = graph(vec![
            (ModuleKey::root(), root),
            module("a", "1.0", &[]),
            module("a", "2.0", &[]),
        ]);
        let err = run(&dep_graph, &Overrides::new()).unwrap_err();
        assert!(err.to_string().contains("at least twice"), "{err}");
    }

    #[test]
    fn multiple_version_override_snaps_up_to_the_nearest_allowed_version() {
        // 1.0 and 3.0 are allowed; a request for 2.0 lands on 3.0, and a
        // request for 1.0 stays at 1.0.
        let dep_graph = graph(vec![
            module("", "", &[("a", "1.0"), ("b", "1.0")]),
            module("a", "1.0", &[]),
            module("a", "2.0", &[]),
            module("a", "3.0", &[]),
            module("b", "1.0", &[("a", "2.0")]),
        ]);
        let overrides = Overrides::from([(
            "a".to_owned(),
            ModuleOverride::MultipleVersion(MultipleVersionOverride {
                versions: vec![v("1.0"), v("3.0")],
                registry: None,
            }),
        )]);
        let selection = run(&dep_graph, &overrides).unwrap();
        assert_eq!(selected(&selection), ["<root>", "a@1.0", "b@1.0", "a@3.0"]);
    }

    #[test]
    fn multiple_version_override_rejects_a_version_above_every_allowed_one() {
        let dep_graph = graph(vec![
            module("", "", &[("a", "1.0"), ("b", "1.0")]),
            module("a", "1.0", &[]),
            module("a", "2.0", &[]),
            module("a", "9.0", &[]),
            module("b", "1.0", &[("a", "9.0")]),
        ]);
        let overrides = Overrides::from([(
            "a".to_owned(),
            ModuleOverride::MultipleVersion(MultipleVersionOverride {
                versions: vec![v("1.0"), v("2.0")],
                registry: None,
            }),
        )]);
        let err = run(&dep_graph, &overrides).unwrap_err();
        assert!(
            err.to_string()
                .contains("not allowed by the multiple_version_override"),
            "{err}"
        );
    }

    #[test]
    fn multiple_version_override_versions_must_exist_in_the_graph() {
        let dep_graph = graph(vec![
            module("", "", &[("a", "1.0")]),
            module("a", "1.0", &[]),
        ]);
        let overrides = Overrides::from([(
            "a".to_owned(),
            ModuleOverride::MultipleVersion(MultipleVersionOverride {
                versions: vec![v("1.0"), v("5.0")],
                registry: None,
            }),
        )]);
        let err = run(&dep_graph, &overrides).unwrap_err();
        assert!(err.to_string().contains("doesn't exist"), "{err}");
    }

    #[test]
    fn a_version_above_the_allowed_set_is_fine_once_unreachable() {
        // a@9.0 is not allowed, but nothing reaches it after selection, so
        // it is simply dropped rather than reported.
        let dep_graph = graph(vec![
            module("", "", &[("a", "1.0"), ("b", "1.0")]),
            module("a", "1.0", &[]),
            module("a", "2.0", &[]),
            module("a", "9.0", &[]),
            module("b", "1.0", &[("a", "2.0")]),
        ]);
        let overrides = Overrides::from([(
            "a".to_owned(),
            ModuleOverride::MultipleVersion(MultipleVersionOverride {
                versions: vec![v("1.0"), v("2.0")],
                registry: None,
            }),
        )]);
        let selection = run(&dep_graph, &overrides).unwrap();
        assert!(!selected(&selection).contains(&"a@9.0".to_owned()));
    }

    #[test]
    fn a_single_version_override_is_applied_before_selection() {
        // The override rewrote the edge during discovery, so by the time
        // selection runs there is only one version to choose.
        let dep_graph = graph(vec![
            module("", "", &[("a", "2.0")]),
            module("a", "2.0", &[]),
        ]);
        let overrides = Overrides::from([(
            "a".to_owned(),
            ModuleOverride::SingleVersion(SingleVersionOverride {
                version: v("2.0"),
                registry: None,
                patches: Vec::new(),
                patch_cmds: Vec::new(),
                patch_strip: 0,
            }),
        )]);
        let selection = run(&dep_graph, &overrides).unwrap();
        assert_eq!(selected(&selection), ["<root>", "a@2.0"]);
    }

    #[test]
    fn nodep_edges_do_not_by_themselves_keep_a_module_in_the_graph() {
        // b is reachable only through a nodep edge, so the pruning walk
        // leaves it out even though the strict walk visited it.
        let (root_key, root) = module("", "", &[("a", "1.0")]);
        let (a_key, mut a) = module("a", "1.0", &[]);
        a.nodep_deps.push(DepSpec::new("b", v("1.0")));
        let dep_graph = graph(vec![(root_key, root), (a_key, a), module("b", "1.0", &[])]);
        let selection = run(&dep_graph, &Overrides::new()).unwrap();
        assert_eq!(selected(&selection), ["<root>", "a@1.0"]);
    }
}
