//! Files of a build (buildfiji-136.13): a source file, or one an action
//! creates.
//!
//! An artifact is named the way Bazel names it. Its *exec path* is where it is
//! in the execroot: `pkg/f.txt` for a source in the main repository,
//! `external/repo/pkg/f.txt` for one in another, and
//! `bazel-out/k8-fastbuild/bin/pkg/f.txt` for an output. Its root-relative
//! path is what follows the root.

use std::collections::BTreeSet;
use std::sync::Arc;

/// The directory of the execroot an artifact's path is relative to.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Root {
    /// From the execroot: empty for the main repository's sources.
    pub prefix: String,
    /// A file of the source tree rather than one an action makes.
    pub source: bool,
}

impl Root {
    /// The source root of a repository: the execroot itself for the main
    /// one, `external/<repo>` for the others.
    pub fn source_of(repo: &str) -> Root {
        Root {
            prefix: if repo.is_empty() {
                String::new()
            } else {
                format!("external/{repo}")
            },
            source: true,
        }
    }

    /// An output root such as `bazel-out/k8-fastbuild/bin`.
    pub fn derived(prefix: impl Into<String>) -> Root {
        Root {
            prefix: prefix.into(),
            source: false,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Artifact {
    pub root: Root,
    /// Relative to the root, `/` separated.
    pub path: String,
}

impl Artifact {
    pub fn source(repo: &str, package: &str, name: &str) -> Artifact {
        Artifact {
            root: Root::source_of(repo),
            path: join(package, name),
        }
    }

    /// An output of a rule of `package` in `repo`, under `bin_dir` (`bazel-out/<config>/bin`).
    pub fn derived(bin_dir: &str, repo: &str, package: &str, name: &str) -> Artifact {
        let path = join(package, name);
        Artifact {
            root: Root::derived(bin_dir),
            path: if repo.is_empty() {
                path
            } else {
                format!("external/{repo}/{path}")
            },
        }
    }

    pub fn is_source(&self) -> bool {
        self.root.source
    }

    /// Where it is in the execroot.
    pub fn exec_path(&self) -> String {
        join(&self.root.prefix, &self.path)
    }
}

impl std::fmt::Display for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.exec_path())
    }
}

fn join(dir: &str, name: &str) -> String {
    match (dir.is_empty(), name.is_empty()) {
        (true, _) => name.to_owned(),
        (_, true) => dir.to_owned(),
        _ => format!("{dir}/{name}"),
    }
}

/// Files in the order a rule gave them, without repeats, sharing what its
/// dependencies already hold: Bazel's `NestedSet` and Starlark's `depset`.
/// Immutable, so a dependent shares a dependency's set instead of copying it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct NestedSet<T> {
    direct: Vec<T>,
    transitive: Vec<Arc<NestedSet<T>>>,
}

impl<T: Clone + Ord> NestedSet<T> {
    pub fn empty() -> NestedSet<T> {
        NestedSet {
            direct: Vec::new(),
            transitive: Vec::new(),
        }
    }

    pub fn new(direct: Vec<T>, transitive: Vec<Arc<NestedSet<T>>>) -> NestedSet<T> {
        NestedSet { direct, transitive }
    }

    pub fn of(direct: Vec<T>) -> NestedSet<T> {
        NestedSet::new(direct, Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.direct.is_empty() && self.transitive.iter().all(|t| t.is_empty())
    }

    /// Every element once, the elements of a nested set before the set's own
    /// (`postorder`, the default order of a Starlark `depset`'s `to_list`
    /// being `default`, which Bazel makes the same).
    pub fn to_vec(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        let mut visited: Vec<*const NestedSet<T>> = Vec::new();
        self.collect(&mut out, &mut seen, &mut visited);
        out
    }

    fn collect(
        &self,
        out: &mut Vec<T>,
        seen: &mut BTreeSet<T>,
        visited: &mut Vec<*const NestedSet<T>>,
    ) {
        for nested in &self.transitive {
            let at = Arc::as_ptr(nested);
            if visited.contains(&at) {
                continue;
            }
            visited.push(at);
            nested.collect(out, seen, visited);
        }
        for item in &self.direct {
            if seen.insert(item.clone()) {
                out.push(item.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_paths_are_where_bazel_puts_files() {
        assert_eq!(Artifact::source("", "a", "x.txt").exec_path(), "a/x.txt");
        assert_eq!(Artifact::source("", "", "x.txt").exec_path(), "x.txt");
        assert_eq!(
            Artifact::source("dep+", "p", "x.txt").exec_path(),
            "external/dep+/p/x.txt"
        );
        let out = Artifact::derived("bazel-out/k8-fastbuild/bin", "", "a", "o.txt");
        assert_eq!(out.exec_path(), "bazel-out/k8-fastbuild/bin/a/o.txt");
        assert!(!out.is_source());
        assert_eq!(
            Artifact::derived("bazel-out/k8-fastbuild/bin", "dep+", "p", "o").exec_path(),
            "bazel-out/k8-fastbuild/bin/external/dep+/p/o"
        );
    }

    #[test]
    fn a_nested_set_lists_each_element_once_dependencies_first() {
        let shared = Arc::new(NestedSet::of(vec![1, 2]));
        let left = Arc::new(NestedSet::new(vec![3], vec![shared.clone()]));
        let right = Arc::new(NestedSet::new(vec![4, 2], vec![shared.clone()]));
        let top = NestedSet::new(vec![5, 1], vec![left, right]);
        assert_eq!(top.to_vec(), vec![1, 2, 3, 4, 5]);
        assert!(NestedSet::<u32>::empty().is_empty());
        assert!(!top.is_empty());
    }
}
