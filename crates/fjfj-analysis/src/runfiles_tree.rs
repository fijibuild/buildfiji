//! The runfiles tree of an executable target (buildfiji-136.9), as Bazel 9.2.0
//! builds it on Linux:
//!
//! - `bin.runfiles/<repo>/<path>` links to every runfile, `_main` for the
//!   main repository, `MANIFEST` and `_repo_mapping` link to
//!   `bin.runfiles_manifest` and `bin.repo_mapping`;
//! - the manifest lists `<path> <where it is>` for each, then `_repo_mapping`;
//! - the repository mapping lists `<source repo>,<apparent name>,<repo>` for
//!   each repository that has runfiles (the main one always) and each name it
//!   has for one that has runfiles.

use crate::target::ConfiguredTarget;
use fjfj_graph::{Action, ActionKind, Artifact, Runfiles};
use fjfj_starlark::RepoMappings;
use std::collections::BTreeSet;

/// Where a runfile is under the tree, from its repository's directory.
fn tree_path(artifact: &Artifact, main_name: &str) -> String {
    let (repo, rest) = if artifact.is_source() {
        match artifact.root.prefix.strip_prefix("external/") {
            Some(repo) => (repo.to_owned(), artifact.path.clone()),
            None => (main_name.to_owned(), artifact.path.clone()),
        }
    } else {
        match artifact.path.strip_prefix("external/") {
            Some(under) => match under.split_once('/') {
                Some((repo, rest)) => (repo.to_owned(), rest.to_owned()),
                None => (under.to_owned(), String::new()),
            },
            None => (main_name.to_owned(), artifact.path.clone()),
        }
    };
    format!("{repo}/{rest}")
}

/// The repository a tree path starts in.
fn repo_of(path: &str) -> &str {
    path.split('/').next().unwrap_or_default()
}

/// Add the runfiles tree actions of `target`, which has an executable.
pub(crate) fn register(
    target: &mut ConfiguredTarget,
    main_name: &str,
    mappings: &RepoMappings,
    repo_names: &dyn Fn(&str) -> String,
) {
    let Some(exe) = target.executable.clone() else {
        return;
    };
    let Runfiles {
        files,
        symlinks,
        root_symlinks,
        python_inits,
    } = target.runfiles.clone();
    let mut entries: Vec<(String, Artifact)> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut add = |path: String, artifact: Artifact| {
        if seen.insert(path.clone()) {
            entries.push((path, artifact));
        }
    };
    add(tree_path(&exe, main_name), exe.clone());
    for file in &files {
        add(tree_path(file, main_name), file.clone());
    }
    for (path, file) in &symlinks {
        add(format!("{main_name}/{path}"), file.clone());
    }
    for (path, file) in &root_symlinks {
        add(path.clone(), file.clone());
    }
    let empty_files = if python_inits {
        self::python_inits(main_name, &seen, &entries)
    } else {
        Vec::new()
    };

    // The repositories with runfiles: canonical names, the main one as "".
    let canonical = |tree_repo: &str| {
        if tree_repo == main_name {
            String::new()
        } else {
            tree_repo.to_owned()
        }
    };
    let with_runfiles: BTreeSet<String> = entries
        .iter()
        .map(|(p, _)| repo_of(p))
        .map(canonical)
        .chain(std::iter::once(String::new()))
        .collect();
    let mut lines: Vec<(String, String, String)> = Vec::new();
    for source in &*target.transitive_repos {
        for (apparent, target_repo) in mappings.entries(source) {
            if apparent.is_empty() || !with_runfiles.contains(&target_repo) {
                continue;
            }
            let shown = if target_repo.is_empty() {
                main_name.to_owned()
            } else {
                repo_names(&target_repo)
            };
            lines.push((source.clone(), apparent, shown));
        }
    }
    lines.sort();
    lines.dedup();
    let repo_mapping_contents: String = lines
        .iter()
        .map(|(s, a, t)| format!("{s},{a},{t}\n"))
        .collect();

    let at = |suffix: &str| Artifact {
        root: exe.root.clone(),
        path: format!("{}{suffix}", exe.path),
        tree: false,
    };
    let (dir, manifest, repo_mapping) = (
        at(".runfiles"),
        at(".runfiles_manifest"),
        at(".repo_mapping"),
    );
    // The four actions Bazel registers: what each reads is what Bazel lists,
    // which is less than it needs: the tree only links what its manifest
    // names, so the `RunfilesTree` is what waits for the files themselves.
    let tree_manifest = Artifact {
        root: exe.root.clone(),
        path: format!("{}/MANIFEST", dir.path),
        tree: false,
    };
    let sources: String = entries
        .iter()
        .map(|(path, file)| format!("{path} {}\n", file.exec_path()))
        .collect();
    let label = fjfj_graph::expand::label_text(&target.label);
    let made = |mnemonic: &str,
                message: String,
                kind: ActionKind,
                inputs: Vec<Artifact>,
                outputs: Vec<Artifact>| Action {
        owner: target.label.clone(),
        owner_kind: target.rule_class.clone().unwrap_or_default(),
        location: String::new(),
        configuration: target.configuration.mnemonic(),
        mnemonic: mnemonic.to_owned(),
        progress_message: Some(message),
        kind,
        inputs,
        input_set: None,
        outputs,
    };
    let mut linked: Vec<Artifact> = entries.iter().map(|(_, a)| a.clone()).collect();
    linked.push(tree_manifest.clone());
    linked.push(repo_mapping.clone());
    let actions = [
        made(
            "RepoMappingManifest",
            format!("Writing repo mapping manifest for {label}"),
            ActionKind::WriteFile {
                contents: repo_mapping_contents.into_bytes(),
                executable: false,
            },
            Vec::new(),
            vec![repo_mapping.clone()],
        ),
        made(
            "SourceSymlinkManifest",
            format!("Creating source manifest for {label}"),
            ActionKind::WriteFile {
                contents: sources.into_bytes(),
                executable: false,
            },
            Vec::new(),
            vec![manifest.clone()],
        ),
        made(
            "SymlinkTree",
            format!("Creating runfiles tree {}", dir.exec_path()),
            ActionKind::SymlinkTree {
                dir: dir.exec_path(),
                repo_mapping: repo_mapping.exec_path(),
                entries: entries.clone(),
                empty_files,
            },
            vec![manifest.clone()],
            vec![tree_manifest],
        ),
        made(
            "RunfilesTree",
            format!("runfiles for {label}"),
            ActionKind::RunfilesTree,
            linked,
            vec![dir.clone()],
        ),
    ];
    target.actions.extend(actions);
    target.extra_outputs = vec![dir, manifest, repo_mapping];
}

/// The paths of an empty `__init__.py` in the runfiles root and in every
/// directory above a Python file (`.py`, `.pyc` or `.so`) that has none, except
/// the main repository's own directory: python's `legacy_create_init`.
fn python_inits(
    main_name: &str,
    seen: &BTreeSet<String>,
    entries: &[(String, Artifact)],
) -> Vec<String> {
    let mut dirs: BTreeSet<String> = BTreeSet::new();
    for (path, _) in entries {
        if !(path.ends_with(".py") || path.ends_with(".pyc") || path.ends_with(".so")) {
            continue;
        }
        let parts: Vec<&str> = path.split('/').collect();
        dirs.insert(String::new());
        for i in 1..parts.len() {
            dirs.insert(parts[..i].join("/"));
        }
    }
    dirs.remove(main_name);
    dirs.iter()
        .map(|dir| {
            if dir.is_empty() {
                "__init__.py".to_owned()
            } else {
                format!("{dir}/__init__.py")
            }
        })
        .filter(|init| !seen.contains(init) && !seen.contains(&format!("{init}c")))
        .collect()
}
