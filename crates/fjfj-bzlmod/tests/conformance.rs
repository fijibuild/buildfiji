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

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fjfj_bzlmod::discovery::RegistrySource;
use fjfj_bzlmod::lockfile::{FileHash, LockSession, Lockfile, LockfileMode};
use fjfj_bzlmod::registry::Fetcher;
use fjfj_bzlmod::{Registry, Resolution, ResolveOptions, WorkspaceIncludeSource, resolve};

fn fixtures_dir() -> PathBuf {
    // Under `bazel test` the runfiles root is the working directory;
    // under cargo, the crate root is.
    let bazel_path = Path::new("crates/fjfj-bzlmod/tests/fixtures");
    if bazel_path.is_dir() {
        return bazel_path.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// The registries the fixtures resolve against, in the order the goldens
/// were captured with: the fixture registry, then the part of the Bazel
/// Central Registry that `bazel_tools`' dependency graph reads
/// (`fixtures/bcr`, vendored by `vendor_bcr.py`, because every module graph
/// includes `bazel_tools`' and so needs those modules).
fn fixture_registries() -> Vec<Registry> {
    vec![
        Registry::local(
            fixtures_dir()
                .join("registry")
                .canonicalize()
                .expect("fixture registry"),
        ),
        Registry::local(
            fixtures_dir()
                .join("bcr")
                .canonicalize()
                .expect("bcr subset"),
        ),
    ]
}

fn resolve_workspace(workspace: &Path) -> fjfj_bzlmod::Result<Resolution> {
    let source_text = std::fs::read_to_string(workspace.join("MODULE.bazel")).unwrap();
    let source = RegistrySource::new(fixture_registries());
    // Harmless for a fixture with no include(): the source is only ever
    // asked to resolve a label if the root MODULE.bazel calls include().
    let options = ResolveOptions {
        include_source: Some(std::rc::Rc::new(WorkspaceIncludeSource::new(workspace))),
        ..ResolveOptions::default()
    };
    resolve(&source_text, &source, &options)
}

/// The selected modules the root reaches without passing through
/// `bazel_tools`.
fn reachable_without_bazel_tools(resolution: &Resolution) -> BTreeSet<fjfj_bzlmod::ModuleKey> {
    let resolved: BTreeMap<_, _> = resolution.selection.resolved.iter().cloned().collect();
    let root = resolution.selection.resolved[0].0.clone();
    let mut seen = BTreeSet::from([root.clone()]);
    let mut queue = vec![root];
    while let Some(key) = queue.pop() {
        for dep in &resolved[&key].deps {
            let child = dep.spec.to_module_key();
            if child.name != "bazel_tools" && seen.insert(child.clone()) {
                queue.push(child);
            }
        }
    }
    seen
}

/// Renders a resolution in the same shape as
/// `tests/fixtures/graph_to_golden.py` renders `bazel mod graph
/// --output=json`: one `<parent key> <apparent name> <child key>` line per
/// edge, sorted and deduplicated. `include_builtin` is `--include_builtin`:
/// without it Bazel shows neither `bazel_tools` nor what it depends on.
fn render_graph(resolution: &Resolution, include_builtin: bool) -> String {
    let mut edges: BTreeSet<String> = BTreeSet::new();
    // Without `--include_builtin`, what is shown is what the workspace's own
    // modules reach without going through `bazel_tools`.
    let shown = reachable_without_bazel_tools(resolution);
    for (key, module) in &resolution.selection.resolved {
        if !include_builtin && !shown.contains(key) {
            continue;
        }
        for dep in &module.deps {
            if !include_builtin && dep.spec.name == "bazel_tools" {
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
                render_graph(&resolution, false),
                expected,
                "{name}: resolved graph differs from Bazel's"
            );
            // With `bazel_tools`' own dependencies, which raise versions.
            let expected = std::fs::read_to_string(workspace.join("expected_graph_builtin.txt"))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                render_graph(&resolution, true),
                expected,
                "{name}: resolved graph with bazel_tools differs from Bazel's"
            );
        }
        checked += 1;
    }
    assert!(
        checked >= 7,
        "expected the fixture workspaces to be present"
    );
}

/// The repo mappings of a resolution as `expected_repo_mapping.txt` holds
/// Bazel's (`dump_mappings.py`): a line per repo, sorted, of its canonical
/// name, a tab, and the mapping as a JSON object in the order it was built.
///
/// `extension_repos` is what running each extension makes, `<repo of the
/// .bzl>+<unique extension name>` and the names of its repos (a workspace's
/// `extension_repos.txt`): running them is not this crate's job.
fn render_repo_mappings(resolution: &Resolution, extension_repos: &str) -> String {
    // `dump_mappings.py` walks the mappings out from the main repo and leaves
    // `@bazel_tools` (and so what only it reaches) out.
    let mut all = resolution.repo_mappings();
    for extension in resolution.extensions() {
        let prefix = format!("{}+{}", extension.bzl_repo, extension.unique_name);
        let Some(line) = extension_repos
            .lines()
            .find(|l| l.split(' ').next() == Some(prefix.as_str()))
        else {
            continue;
        };
        let names: Vec<String> = line.split(' ').skip(1).map(str::to_owned).collect();
        let rows = resolution.extension_repo_mapping(&extension, &names);
        for name in &names {
            all.push((extension.repo_name(name), rows.clone()));
        }
    }
    let by_repo: BTreeMap<&str, &Vec<(String, String)>> = all
        .iter()
        .map(|(repo, rows)| (repo.as_str(), rows))
        .collect();
    let mut shown = BTreeSet::from([""]);
    let mut queue = vec![""];
    while let Some(repo) = queue.pop() {
        for (_, canonical) in by_repo[repo] {
            if canonical != "bazel_tools" && shown.insert(canonical.as_str()) {
                queue.push(canonical);
            }
        }
    }
    let mut lines: Vec<String> = all
        .iter()
        .filter(|(repo, _)| shown.contains(repo.as_str()))
        .map(|(repo, rows)| {
            let body: Vec<String> = rows
                .iter()
                .map(|(apparent, canonical)| format!("\"{apparent}\":\"{canonical}\""))
                .collect();
            format!("{repo}\t{{{}}}\n", body.join(","))
        })
        .collect();
    lines.sort();
    lines.concat()
}

#[test]
fn repo_mappings_match_bazel() {
    let workspaces = fixtures_dir().join("workspaces");
    let mut checked = 0;
    for entry in std::fs::read_dir(&workspaces).expect("fixture workspaces") {
        let workspace = entry.unwrap().path();
        let expected = workspace.join("expected_repo_mapping.txt");
        if !expected.is_file() {
            continue;
        }
        let name = workspace
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let resolution = resolve_workspace(&workspace)
            .unwrap_or_else(|e| panic!("{name}: resolution failed: {e}"));
        let extension_repos =
            std::fs::read_to_string(workspace.join("extension_repos.txt")).unwrap_or_default();
        assert_eq!(
            render_repo_mappings(&resolution, &extension_repos),
            std::fs::read_to_string(&expected).unwrap(),
            "{name}: repo mappings differ from Bazel's"
        );
        checked += 1;
    }
    assert!(checked >= 6, "expected the fixture mappings to be present");
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
    let source = RegistrySource::new(fixture_registries());
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
/// Ignored for the same reason as the test above. Resolves a copy of this
/// repository's own `MODULE.bazel` (`fixtures/repo`) against the live Bazel
/// Central Registry and compares every edge of the graph with the one
/// `bazel mod graph` printed for it, which needs `bazel_tools`' real module
/// file (buildfiji-mum.23: without it `protobuf` came out at 29.1 and not
/// 33.4, among others).
///
/// Refresh the golden with `bazel mod graph --output=json
/// --lockfile_mode=off | python3 tests/fixtures/graph_to_golden.py` run in a
/// directory holding that `MODULE.bazel`.
#[test]
#[ignore = "reaches the network"]
fn resolves_this_repository_against_the_real_registry() {
    let dir = fixtures_dir().join("repo");
    let root_module_file = std::fs::read_to_string(dir.join("MODULE.bazel")).unwrap();
    // `BCR_DIR` replays a directory laid out like the registry (for a
    // machine with Bazel's repository cache and no network).
    let registry = match std::env::var("BCR_DIR") {
        Ok(dir) => Registry::local(dir),
        Err(_) => Registry::remote(fjfj_bzlmod::BAZEL_CENTRAL_REGISTRY).unwrap(),
    };
    let source = RegistrySource::new(vec![registry]);

    let resolution = resolve(&root_module_file, &source, &ResolveOptions::default()).unwrap();

    assert_eq!(
        render_graph(&resolution, false),
        std::fs::read_to_string(dir.join("expected_graph.txt")).unwrap(),
        "the graph differs from `bazel mod graph`'s"
    );
}

/// Serves a directory of registry files under a URL, so the lockfile treats
/// it as the remote registry it locks (`file://` registries are not locked),
/// without a network. With `archive_sources`, every `source.json` is the
/// archive source `lock_hashes.py` served in its place (Bazel refuses a
/// `local_path` module from a remote registry), so the hashes compare.
struct ServedRegistry {
    base: String,
    dir: PathBuf,
    archive_sources: bool,
}

impl Fetcher for ServedRegistry {
    fn fetch(&self, url: &str) -> fjfj_bzlmod::Result<Option<Vec<u8>>> {
        let Some(path) = url.strip_prefix(&self.base) else {
            panic!("asked for {url}, outside {}", self.base);
        };
        let parts: Vec<&str> = path.split('/').collect();
        if self.archive_sources
            && let ["modules", name, version, "source.json"] = parts[..]
            && self.dir.join(path).exists()
        {
            return Ok(Some(
                format!(
                    "{{\"url\":\"https://example.invalid/{name}-{version}.tar.gz\",\"integrity\":\
                     \"sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\",\"strip_prefix\":\"{name}\"}}\n"
                )
                .into_bytes(),
            ));
        }
        Ok(std::fs::read(self.dir.join(path)).ok())
    }
}

const FIXTURE_REGISTRY_URL: &str = "http://127.0.0.1:1";
const BCR_URL: &str = "https://bcr.bazel.build";

/// What a locked resolution of `workspace` recorded, keyed by URL: the
/// registry file hashes and the yanked versions it selected.
fn lock_of(
    workspace: &Path,
    options: ResolveOptions,
) -> (BTreeMap<String, FileHash>, Vec<(String, String)>) {
    let session = LockSession::new(LockfileMode::Update, None).unwrap();
    let served = |base: &str, dir: &str, archive_sources: bool| {
        Registry::new(
            base,
            Box::new(ServedRegistry {
                base: format!("{base}/"),
                dir: fixtures_dir().join(dir),
                archive_sources,
            }),
        )
        .locked(&session)
    };
    let source = RegistrySource::new(vec![
        served(FIXTURE_REGISTRY_URL, "registry", true),
        served(BCR_URL, "bcr", false),
    ]);
    let source_text = std::fs::read_to_string(workspace.join("MODULE.bazel")).unwrap();
    let options = ResolveOptions {
        for_lockfile: true,
        include_source: Some(std::rc::Rc::new(WorkspaceIncludeSource::new(workspace))),
        ..options
    };
    let resolution = resolve(&source_text, &source, &options).unwrap();
    let lock = session.finish(&resolution.selected_yanked);
    (lock.registry_file_hashes, lock.selected_yanked_versions)
}

/// `expected_lock_hashes.txt`: what Bazel's `MODULE.bazel.lock` recorded for
/// the fixture registry, as `lock_hashes.py` captured it, keyed by URL.
fn golden_lock(workspace: &Path) -> (BTreeMap<String, FileHash>, Vec<(String, String)>) {
    let text = std::fs::read_to_string(workspace.join("expected_lock_hashes.txt")).unwrap();
    let mut hashes = BTreeMap::new();
    let mut yanked = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        match fields[..] {
            ["yanked", key, reason] => yanked.push((key.to_owned(), reason.to_owned())),
            [path, hash] => {
                let hash = match hash {
                    "not found" => FileHash::NotFound,
                    hash => FileHash::Sha256(hash.to_owned()),
                };
                hashes.insert(format!("{FIXTURE_REGISTRY_URL}/{path}"), hash);
            }
            _ => panic!("bad golden line {line:?}"),
        }
    }
    // What Bazel read from the Bazel Central Registry is `bazel_tools`' own
    // graph, the same for every fixture (`lock_hashes.py` checks that).
    let real = Lockfile::parse(
        &std::fs::read_to_string(fixtures_dir().join("lockfiles/9.2.0.lock")).unwrap(),
    )
    .unwrap();
    hashes.extend(real.registry_file_hashes);
    (hashes, yanked)
}

/// The lockfile fjfj writes records exactly the registry files Bazel's does,
/// with the same hashes (including the `not found` probes), and the same
/// yanked versions.
#[test]
fn lockfile_registry_hashes_match_bazel() {
    let workspaces = fixtures_dir().join("workspaces");
    let mut checked = 0;
    for entry in std::fs::read_dir(&workspaces).unwrap() {
        let workspace = entry.unwrap().path();
        let name = workspace
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if !workspace.join("expected_lock_hashes.txt").exists() {
            continue;
        }
        let options = if name.starts_with("yanked") {
            ResolveOptions {
                yanked: fjfj_bzlmod::YankedPolicy::AllowAll,
                ..ResolveOptions::default()
            }
        } else {
            ResolveOptions::default()
        };
        let (ours, our_yanked) = lock_of(&workspace, options);
        let (theirs, their_yanked) = golden_lock(&workspace);
        let differences = |a: &BTreeMap<String, FileHash>, b: &BTreeMap<String, FileHash>| {
            a.iter()
                .filter(|(url, hash)| b.get(*url) != Some(*hash))
                .map(|(url, _)| url.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            differences(&ours, &theirs),
            Vec::<String>::new(),
            "{name}: recorded but not (or differently) by Bazel"
        );
        assert_eq!(
            differences(&theirs, &ours),
            Vec::<String>::new(),
            "{name}: recorded by Bazel but not (or differently) by fjfj"
        );
        assert_eq!(our_yanked, their_yanked, "{name}");
        checked += 1;
    }
    assert!(
        checked >= 8,
        "only {checked} workspaces had a golden: {:?}",
        std::fs::read_dir(&workspaces)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>()
    );
}

/// A `MODULE.bazel.lock` exactly as Bazel 9.2.0 wrote it (for a workspace
/// with no extensions of its own, so what is in it is `bazel_tools`' graph:
/// 184 registry hashes and four module extensions' results) reads and writes
/// back byte for byte.
#[test]
fn a_lockfile_bazel_wrote_round_trips_byte_for_byte() {
    let text = std::fs::read_to_string(fixtures_dir().join("lockfiles/9.2.0.lock")).unwrap();
    let lock = Lockfile::parse(&text).unwrap();
    assert_eq!(lock.registry_file_hashes.len(), 184);
    assert!(lock.selected_yanked_versions.is_empty());
    assert_eq!(lock.to_text(), text);
}
