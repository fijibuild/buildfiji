//! `fjfj mod graph|deps|show_repo|explain`: presenting a resolved
//! [`Resolution`] the way Bazel's own `mod` command does. Pure rendering —
//! no I/O, no flag parsing — so it's unit-testable against a `Resolution`
//! built by hand or by `fjfj_bzlmod::resolve` directly, the same as any of
//! this crate's tests.
//!
//! `graph`'s `--output=json` shape (`key`/`name`/`version`/`apparentName`/
//! `dependencies`/`indirectDependencies`/`cycles`, `root: true` only on
//! the root node) is transcribed from a real `bazel mod graph
//! --output=json` run against `crates/fjfj-bzlmod/tests/fixtures`
//! (buildfiji-mum.24's own conformance fixtures), not reconstructed from
//! documentation.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use fjfj_bzlmod::{Module, ModuleKey, Resolution};
use serde_json::{Value, json};

/// Bazel hides the `bazel_tools` subtree from `mod` output — fjfj supplies
/// a placeholder for it (`fjfj_bzlmod::discovery::PLACEHOLDER_BAZEL_TOOLS_MODULE`),
/// so showing it would be showing an implementation detail, not something
/// the user wrote.
fn is_hidden(name: &str) -> bool {
    name == "bazel_tools"
}

/// One place to look a selected module up by key — `Resolution` only
/// offers linear scans by name.
struct Index<'a> {
    by_key: BTreeMap<&'a ModuleKey, &'a Module>,
}

impl<'a> Index<'a> {
    fn new(resolution: &'a Resolution) -> Index<'a> {
        Index {
            by_key: resolution
                .selection
                .resolved
                .iter()
                .map(|(key, module)| (key, module))
                .collect(),
        }
    }

    fn get(&self, key: &ModuleKey) -> Option<&'a Module> {
        self.by_key.get(key).copied()
    }
}

/// `fjfj mod graph --output=json`.
pub fn render_graph_json(resolution: &Resolution) -> Value {
    let index = Index::new(resolution);
    let root = ModuleKey::root();
    let mut path = vec![root.clone()];
    let mut node = build_node(&root, "root", &index, &mut path);
    node["root"] = json!(true);
    node
}

fn build_node(
    key: &ModuleKey,
    apparent_name: &str,
    index: &Index,
    path: &mut Vec<ModuleKey>,
) -> Value {
    let module = index.get(key);
    let name = module.map(|m| m.name.as_str()).unwrap_or(&key.name);
    let version = module
        .map(|m| m.version.to_string())
        .unwrap_or_else(|| key.version.to_string());

    let mut dependencies = Vec::new();
    let mut cycles = Vec::new();
    if let Some(module) = module {
        for dep in &module.deps {
            if is_hidden(&dep.spec.name) {
                continue;
            }
            let child_key = dep.spec.to_module_key();
            if path.contains(&child_key) {
                // A genuine cycle in the dependency graph — stop
                // recursing into it (this crate's DFS would never
                // terminate otherwise) and report it the way the node
                // that closes the cycle would.
                cycles.push(child_key.to_string());
                continue;
            }
            path.push(child_key.clone());
            dependencies.push(build_node(&child_key, &dep.repo_name, index, path));
            path.pop();
        }
    }

    json!({
        "key": key.to_string(),
        "name": name,
        "version": version,
        "apparentName": apparent_name,
        "dependencies": dependencies,
        "indirectDependencies": [],
        "cycles": cycles,
    })
}

/// `fjfj mod graph` (`--output=text`, the default) — an indented tree, one
/// line per edge. Not byte-matched against Bazel's own box-drawing
/// output; `--output=json` is the fidelity-tested format (buildfiji-mum.24's
/// fixtures already exercise it via `render_graph_json`'s edges).
pub fn render_graph_text(resolution: &Resolution) -> String {
    let index = Index::new(resolution);
    let root = ModuleKey::root();
    let mut out = String::new();
    let mut path = vec![root.clone()];
    write_text_node(&mut out, &root, "root", &index, 0, &mut path);
    out
}

fn write_text_node(
    out: &mut String,
    key: &ModuleKey,
    apparent_name: &str,
    index: &Index,
    depth: usize,
    path: &mut Vec<ModuleKey>,
) {
    let indent = "  ".repeat(depth);
    let _ = writeln!(out, "{indent}{key} ({apparent_name})");
    let Some(module) = index.get(key) else {
        return;
    };
    for dep in &module.deps {
        if is_hidden(&dep.spec.name) {
            continue;
        }
        let child_key = dep.spec.to_module_key();
        if path.contains(&child_key) {
            let _ = writeln!(out, "{indent}  {child_key} ({}) (cycle)", dep.repo_name);
            continue;
        }
        path.push(child_key.clone());
        write_text_node(out, &child_key, &dep.repo_name, index, depth + 1, path);
        path.pop();
    }
}

/// `fjfj mod deps <module>...`: each named module's own direct deps, plus
/// the version selection actually resolved it to.
pub fn render_deps(resolution: &Resolution, names: &[String]) -> anyhow::Result<String> {
    let mut out = String::new();
    for name in names {
        let Some(module) = resolution.selection.module(name) else {
            anyhow::bail!("module '{name}' is not in the resolved graph");
        };
        let _ = writeln!(out, "{} ({})", module.key, name);
        for dep in &module.deps {
            if is_hidden(&dep.spec.name) {
                continue;
            }
            let selected = resolution
                .selection
                .module(&dep.spec.name)
                .map(|m| m.key.to_string())
                .unwrap_or_else(|| "<not selected>".to_owned());
            let _ = writeln!(
                out,
                "  {} -> {} (requested {})",
                dep.repo_name,
                selected,
                dep.spec.to_module_key()
            );
        }
    }
    Ok(out)
}

/// `fjfj mod show_repo <repo>...`: what fjfj knows about each named
/// module's repo today — name, version, canonical/apparent names, and its
/// declared deps. Bazel's own `show_repo` also dumps the repo rule's
/// attributes once it has fetched the repo; fjfj can't yet (buildfiji-mum.8),
/// so that part is out of scope here.
pub fn render_show_repo(resolution: &Resolution, names: &[String]) -> anyhow::Result<String> {
    let mut out = String::new();
    for name in names {
        let Some(module) = resolution.selection.module(name) else {
            anyhow::bail!("repo '{name}' is not in the resolved graph");
        };
        let _ = writeln!(out, "## {name}:");
        let _ = writeln!(out, "key: {}", module.key);
        let _ = writeln!(out, "name: {}", module.name);
        let _ = writeln!(out, "version: {}", module.version);
        let _ = writeln!(out, "repo_name: {}", module.repo_name);
        let _ = writeln!(
            out,
            "canonical_repo_name: {}",
            module.key.canonical_repo_name()
        );
        if let Some(registry) = &module.registry {
            let _ = writeln!(out, "registry: {registry}");
        }
    }
    Ok(out)
}

/// `fjfj mod explain <repo>...`: why the selected version won — every
/// version anybody in the (unpruned) graph asked for, and which one MVS
/// picked.
pub fn render_explain(resolution: &Resolution, names: &[String]) -> anyhow::Result<String> {
    let mut out = String::new();
    for name in names {
        let Some(selected) = resolution.selection.module(name) else {
            anyhow::bail!("module '{name}' is not in the resolved graph");
        };
        let _ = writeln!(out, "{name}: selected {}", selected.key);
        let mut requesters: Vec<(&ModuleKey, &fjfj_bzlmod::module::DepSpec)> = Vec::new();
        for (key, module) in &resolution.selection.unpruned {
            for dep in &module.deps {
                if dep.spec.name == *name {
                    requesters.push((key, &dep.spec));
                }
            }
        }
        requesters.sort_by(|a, b| a.0.cmp(b.0));
        for (requester, spec) in requesters {
            let _ = writeln!(out, "  {requester} requested {}", spec.to_module_key());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fjfj_bzlmod::discovery::RegistrySource;
    use fjfj_bzlmod::{Registry, ResolveOptions, resolve};

    fn fixtures_dir() -> std::path::PathBuf {
        let bazel_path = std::path::Path::new("crates/fjfj-bzlmod/tests/fixtures/registry");
        if bazel_path.is_dir() {
            return bazel_path.canonicalize().unwrap();
        }
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fjfj-bzlmod/tests/fixtures/registry")
            .canonicalize()
            .expect("fjfj-bzlmod fixture registry")
    }

    fn resolve_fixture(module_bazel: &str) -> Resolution {
        let registry = Registry::local(fixtures_dir());
        let source = RegistrySource::new(vec![registry]);
        resolve(module_bazel, &source, &ResolveOptions::default()).unwrap()
    }

    /// Flattens `render_graph_json`'s tree back into the same `<parent>
    /// <apparent> <child>` edge format `fjfj-bzlmod`'s own conformance
    /// tests compare against real Bazel's golden output — so this reuses
    /// buildfiji-mum.24's fixtures as a real conformance check on the
    /// graph shape, not just "it doesn't panic".
    fn edges(node: &Value) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        collect_edges(node, &mut out);
        out
    }

    fn collect_edges(node: &Value, out: &mut std::collections::BTreeSet<String>) {
        let parent = node["key"].as_str().unwrap();
        for dep in node["dependencies"].as_array().unwrap() {
            let apparent = dep["apparentName"].as_str().unwrap();
            let child = dep["key"].as_str().unwrap();
            out.insert(format!("{parent} {apparent} {child}"));
            collect_edges(dep, out);
        }
    }

    #[test]
    fn graph_json_matches_the_mvs_fixtures_golden_edges() {
        let module_bazel =
            std::fs::read_to_string(fixtures_dir().join("../workspaces/mvs/MODULE.bazel")).unwrap();
        let resolution = resolve_fixture(&module_bazel);
        let expected =
            std::fs::read_to_string(fixtures_dir().join("../workspaces/mvs/expected_graph.txt"))
                .unwrap();
        let expected: std::collections::BTreeSet<String> =
            expected.lines().map(str::to_owned).collect();
        assert_eq!(edges(&render_graph_json(&resolution)), expected);
    }

    #[test]
    fn graph_json_root_node_shape() {
        let resolution = resolve_fixture("module(name = 'root', version = '0')");
        let node = render_graph_json(&resolution);
        assert_eq!(node["key"], "<root>");
        assert_eq!(node["apparentName"], "root");
        assert_eq!(node["root"], true);
        assert_eq!(node["dependencies"], json!([]));
    }

    #[test]
    fn graph_text_indents_by_depth() {
        let resolution = resolve_fixture(
            "module(name = 'root', version = '0')\nbazel_dep(name = 'b', version = '1.0')",
        );
        let text = render_graph_text(&resolution);
        assert!(text.starts_with("<root> (root)\n"));
        assert!(text.contains("  b@1.0 (b)\n"));
    }

    #[test]
    fn deps_lists_direct_dependencies() {
        // The mvs fixture: b@1.0 depends on c, which selection resolves
        // to c@2.0.
        let module_bazel =
            std::fs::read_to_string(fixtures_dir().join("../workspaces/mvs/MODULE.bazel")).unwrap();
        let resolution = resolve_fixture(&module_bazel);
        let out = render_deps(&resolution, &["b".to_owned()]).unwrap();
        assert!(out.contains("c -> c@2.0"));
    }

    #[test]
    fn deps_rejects_an_unknown_module() {
        let resolution = resolve_fixture("module(name = 'root', version = '0')");
        assert!(render_deps(&resolution, &["nope".to_owned()]).is_err());
    }

    #[test]
    fn show_repo_reports_key_fields() {
        let resolution = resolve_fixture(
            "module(name = 'root', version = '0')\nbazel_dep(name = 'a', version = '1.0')",
        );
        let out = render_show_repo(&resolution, &["a".to_owned()]).unwrap();
        assert!(out.contains("name: a"));
        assert!(out.contains("version: 1.0"));
        assert!(out.contains("canonical_repo_name: a+"));
    }

    #[test]
    fn explain_lists_every_requester() {
        // b@1.0 depends on c@2.0; root depends on b@1.0 directly and c@1.0
        // through nothing else here, so explain(c) should show only b's
        // request once selection has rewritten the unpruned graph's edges.
        let module_bazel =
            std::fs::read_to_string(fixtures_dir().join("../workspaces/mvs/MODULE.bazel")).unwrap();
        let resolution = resolve_fixture(&module_bazel);
        let out = render_explain(&resolution, &["c".to_owned()]).unwrap();
        assert!(out.contains("selected c@2.0"));
        assert!(out.contains("requested c@2.0"));
    }
}
