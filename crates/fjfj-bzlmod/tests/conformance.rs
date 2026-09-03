//! Conformance tests: fjfj's module resolution against real Bazel's.
//!
//! Each directory under `tests/fixtures/workspaces` is a workspace with a
//! `MODULE.bazel`, resolved against the local registry in
//! `tests/fixtures/registry`. Alongside it sits either
//! `expected_graph.txt` — the module graph **Bazel 9.2.0 itself
//! resolved**, captured by
//! `bazel run //crates/fjfj-bzlmod/tests/fixtures:refresh_golden` — or
//! `expect_error`, the error Bazel printed. Neither file is written by
//! hand: they are Bazel's output, which is what makes this a conformance
//! test rather than a restatement of the implementation.
//!
//! `bazel mod graph` hides the `bazel_tools` subtree, so the renderer here
//! hides it too; everything else in the graph is compared edge for edge.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fjfj_bzlmod::discovery::RegistrySource;
use fjfj_bzlmod::{Registry, Resolution, ResolveOptions, resolve};

fn fixtures_dir() -> PathBuf {
    // Under `bazel test` the runfiles root is the working directory;
    // under cargo, the crate root is.
    let bazel_path = Path::new("crates/fjfj-bzlmod/tests/fixtures");
    if bazel_path.is_dir() {
        return bazel_path.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn resolve_workspace(workspace: &Path) -> fjfj_bzlmod::Result<Resolution> {
    let source_text = std::fs::read_to_string(workspace.join("MODULE.bazel")).unwrap();
    let registry = Registry::local(
        fixtures_dir()
            .join("registry")
            .canonicalize()
            .expect("fixture registry"),
    );
    let source = RegistrySource::new(vec![registry]);
    resolve(&source_text, &source, &ResolveOptions::default())
}

/// Renders a resolution in the same shape as
/// `tests/fixtures/graph_to_golden.py` renders `bazel mod graph
/// --output=json`: one `<parent key> <apparent name> <child key>` line per
/// edge, sorted and deduplicated.
fn render_graph(resolution: &Resolution) -> String {
    let mut edges: BTreeSet<String> = BTreeSet::new();
    for (key, module) in &resolution.selection.resolved {
        for dep in &module.deps {
            // Built-in modules are implicit; Bazel does not show them.
            if dep.spec.name == "bazel_tools" {
                continue;
            }
            edges.insert(format!(
                "{key} {} {}",
                dep.repo_name,
                dep.spec.to_module_key()
            ));
        }
    }
    edges.into_iter().map(|edge| format!("{edge}\n")).collect()
}

#[test]
fn resolution_matches_bazel() {
    let workspaces = fixtures_dir().join("workspaces");
    let mut checked = 0;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&workspaces)
        .expect("fixture workspaces")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();

    for workspace in entries {
        let name = workspace
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let expected_error = workspace.join("expect_error");
        if expected_error.is_file() {
            let expected = std::fs::read_to_string(&expected_error).unwrap();
            let error = resolve_workspace(&workspace)
                .err()
                .unwrap_or_else(|| panic!("{name}: expected resolution to fail"));
            assert_eq!(
                error.to_string().trim(),
                expected.trim(),
                "{name}: error text differs from Bazel's"
            );
        } else {
            let expected = std::fs::read_to_string(workspace.join("expected_graph.txt"))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            let resolution = resolve_workspace(&workspace)
                .unwrap_or_else(|e| panic!("{name}: resolution failed: {e}"));
            assert_eq!(
                render_graph(&resolution),
                expected,
                "{name}: resolved graph differs from Bazel's"
            );
        }
        checked += 1;
    }
    assert!(
        checked >= 7,
        "expected the fixture workspaces to be present"
    );
}

#[test]
fn selection_prunes_modules_that_lose_their_only_dependent() {
    // c@1.0 depends on d@1.0, but b@1.0 pulls c up to c@2.0, which does
    // not — so d is discovered and then dropped.
    let resolution = resolve_workspace(&fixtures_dir().join("workspaces/mvs")).unwrap();
    let selected: Vec<String> = resolution.selection.keys().map(|k| k.to_string()).collect();
    assert!(selected.contains(&"c@2.0".to_owned()));
    assert!(!selected.iter().any(|k| k.starts_with('d')));
    // It is still in the unpruned graph, which is what `mod` explains
    // resolution from.
    assert!(
        resolution
            .selection
            .unpruned
            .iter()
            .any(|(key, _)| key.name == "d")
    );
}

#[test]
fn multiple_version_override_lets_two_versions_coexist() {
    let resolution =
        resolve_workspace(&fixtures_dir().join("workspaces/multiple_version_override")).unwrap();
    let a_versions: BTreeSet<String> = resolution
        .selection
        .keys()
        .filter(|key| key.name == "a")
        .map(|key| key.version.to_string())
        .collect();
    assert_eq!(
        a_versions,
        BTreeSet::from(["1.0".to_owned(), "2.0".to_owned()])
    );
    // With two versions in the graph, the versioned canonical repo name is
    // the only unique one.
    // Breadth-first from the root, so the root's own dep comes first.
    let keys: Vec<_> = resolution
        .selection
        .keys()
        .filter(|key| key.name == "a")
        .map(|key| key.canonical_repo_name_with_version().unwrap())
        .collect();
    assert_eq!(keys, ["a+2.0", "a+1.0"]);
}

#[test]
fn yanked_versions_are_allowed_when_the_user_says_so() {
    let workspace = fixtures_dir().join("workspaces/yanked");
    let source_text = std::fs::read_to_string(workspace.join("MODULE.bazel")).unwrap();
    let registry = Registry::local(fixtures_dir().join("registry").canonicalize().unwrap());
    let source = RegistrySource::new(vec![registry]);
    let options = ResolveOptions {
        yanked: fjfj_bzlmod::YankedPolicy::AllowAll,
        ..ResolveOptions::default()
    };
    let resolution = resolve(&source_text, &source, &options).unwrap();
    assert!(
        resolution
            .selection
            .keys()
            .any(|key| key.to_string() == "y@2.0")
    );
}

/// Talks to the real Bazel Central Registry.
///
/// Ignored by default: `bazel test` runs sandboxed and offline, and a test
/// whose result depends on someone else's server is not a gate. Run it by
/// hand when the registry client changes:
///
/// ```text
/// bazel test //crates/fjfj-bzlmod:fjfj-bzlmod_conformance_test \
///   --test_arg=--ignored --test_arg=--nocapture --test_output=all
/// ```
#[test]
#[ignore = "reaches the network"]
fn reads_the_bazel_central_registry() {
    let registry = Registry::remote(fjfj_bzlmod::BAZEL_CENTRAL_REGISTRY).unwrap();
    let key =
        fjfj_bzlmod::ModuleKey::new("rules_rust", fjfj_bzlmod::Version::parse("0.74.0").unwrap());

    let module_file = registry
        .module_file(&key)
        .unwrap()
        .expect("rules_rust@0.74.0");
    assert!(module_file.contains("module("), "{module_file}");

    // A version that does not exist reads as absent, not as an error, so
    // the resolver can fall through to the next registry.
    let missing = fjfj_bzlmod::ModuleKey::new(
        "rules_rust",
        fjfj_bzlmod::Version::parse("0.0.0-does-not-exist").unwrap(),
    );
    assert!(registry.module_file(&missing).unwrap().is_none());

    // source.json turns into an http_archive call with a verifiable
    // integrity hash.
    let repo_spec = registry.repo_spec(&key).unwrap();
    assert_eq!(
        repo_spec.rule,
        fjfj_bzlmod::overrides::RepoRule::HttpArchive
    );
    let integrity = repo_spec
        .attrs
        .iter()
        .find(|(name, _)| name == "integrity")
        .and_then(|(_, value)| value.as_str())
        .expect("integrity attribute");
    fjfj_bzlmod::registry::Integrity::parse(integrity).unwrap();

    let metadata = registry.metadata("rules_rust").unwrap().expect("metadata");
    assert!(metadata.versions.iter().any(|v| v == "0.74.0"));
}

/// Resolves this repository's own `MODULE.bazel` against the real registry
/// and checks the result against what the root module asked for.
///
/// Ignored for the same reason as the test above. When this was last run
/// by hand, the selected version of all 29 modules `bazel mod graph`
/// reports for this repository matched Bazel's exactly, resolved from the
/// live Bazel Central Registry.
///
/// To reproduce that comparison, resolve the repo's `MODULE.bazel` with
/// `BAZEL_TOOLS_MODULE` pointing at
/// `$(bazel info install_base)/embedded_tools/MODULE.bazel` — without the
/// real `bazel_tools` module file, its own `bazel_dep`s are missing and
/// several versions come out lower (see
/// `fjfj_bzlmod::discovery::PLACEHOLDER_BAZEL_TOOLS_MODULE` and
/// buildfiji-mum.23).
#[test]
#[ignore = "reaches the network"]
fn resolves_this_repository_against_the_real_registry() {
    let root_module_file =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../MODULE.bazel"))
            .unwrap();
    let registry = Registry::remote(fjfj_bzlmod::BAZEL_CENTRAL_REGISTRY).unwrap();
    let mut source = RegistrySource::new(vec![registry]);
    if let Ok(path) = std::env::var("BAZEL_TOOLS_MODULE") {
        source = source.with_builtin_module("bazel_tools", std::fs::read_to_string(path).unwrap());
    }

    let resolution = resolve(&root_module_file, &source, &ResolveOptions::default()).unwrap();

    // Minimal version selection never picks a version below what was
    // requested, and every direct dep must survive selection.
    for dep in &resolution.root.deps {
        let selected = resolution
            .selection
            .module(&dep.spec.name)
            .unwrap_or_else(|| panic!("{} was not selected", dep.spec.name));
        assert!(
            selected.version >= dep.spec.version,
            "{} resolved to {} but the root asked for {}",
            dep.spec.name,
            selected.version,
            dep.spec.version
        );
    }
    // A transitive graph, not just the direct deps.
    assert!(
        resolution.selection.resolved.len() > 20,
        "expected a transitive graph, got {} modules",
        resolution.selection.resolved.len()
    );
}
