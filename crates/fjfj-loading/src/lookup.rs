//! Finding a repo's packages on disk.
//!
//! Behaviour checked against Bazel 9.2.0:
//!
//! - a package is a directory holding a `BUILD.bazel` or `BUILD` *file*;
//!   `BUILD.bazel` wins when both exist. A directory of either name, or a
//!   symlink that does not lead to a file, does not count, and the search
//!   falls through to the other name. A symlink to a file does count.
//! - `.bazelignore` lists directories, one per line, that hold no packages
//!   at all, subtree included. Blank lines and lines starting with `#` are
//!   skipped, nothing else is trimmed, and there are no wildcards. `c/`,
//!   `./c` and `c//d` are normalised; an entry with `..` in it matches
//!   nothing; an absolute path is an error.
//! - `REPO.bazel`'s `ignore_directories()` is `.bazelignore` with patterns:
//!   `glob()`'s segments over a directory's path, `?` standing for any one
//!   character. `["c", "**/gen", "x/*"]` removes `c`, any `gen`, and every
//!   child of `x` with its subtree, but not `x`. A pattern that is absolute,
//!   ends in `/` or has an empty, `.` or `..` segment matches nothing.
//! - `--deleted_packages=a/sub` removes exactly that package.
//! - an ignored or deleted package is reported as deleted, in the same words
//!   for both.

use crate::glob::IgnorePattern;
use fjfj_graph::LabelError;
use fjfj_graph::label::validate_package_name;
use std::collections::BTreeSet;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    #[error(
        "no such package '{package}': BUILD file not found in any of the following \
         directories. Add a BUILD file to a directory to mark it as a package.\n - {package}"
    )]
    NotFound { package: String },
    #[error("no such package '{package}': Package is considered deleted due to --deleted_packages")]
    Deleted { package: String },
    #[error("invalid package name '{package}': {source}")]
    InvalidName { package: String, source: LabelError },
    #[error("Invalid path in {file}: '{entry}': cannot be an absolute path")]
    AbsoluteIgnoreEntry { file: PathBuf, entry: String },
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

/// Which directories of one repo are packages.
#[derive(Debug, Clone)]
pub struct PackageLookup {
    root: PathBuf,
    ignored: BTreeSet<String>,
    /// `REPO.bazel`'s `ignore_directories()`.
    ignore_patterns: Vec<IgnorePattern>,
    deleted: BTreeSet<String>,
    /// Directories directly under the root that a `//...` walk skips: the
    /// `bazel-*` convenience symlinks, which lead into the output base.
    skipped_at_root: BTreeSet<String>,
}

impl PackageLookup {
    /// A lookup over the repo rooted at `root`, reading its `.bazelignore`
    /// if it has one.
    pub fn new(root: impl Into<PathBuf>) -> Result<PackageLookup, LookupError> {
        let root = root.into();
        let file = root.join(".bazelignore");
        let ignored = match std::fs::read_to_string(&file) {
            Ok(text) => parse_bazelignore(&file, &text)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => BTreeSet::new(),
            Err(source) => return Err(LookupError::Io { path: file, source }),
        };
        Ok(PackageLookup {
            root,
            ignored,
            ignore_patterns: Vec::new(),
            deleted: BTreeSet::new(),
            skipped_at_root: BTreeSet::new(),
        })
    }

    /// `REPO.bazel`'s `ignore_directories(patterns)`: a directory a pattern
    /// matches is ignored as a `.bazelignore` entry is, subtree and all.
    /// Patterns are `glob()`'s over the directory's path from the repo root,
    /// `?` included; one with an empty, `.` or `..` segment matches nothing.
    pub fn with_ignore_directories<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.ignore_patterns.extend(
            patterns
                .into_iter()
                .filter_map(|p| IgnorePattern::parse(p.as_ref())),
        );
        self
    }

    /// `--deleted_packages`: each named package stops being one.
    pub fn with_deleted_packages<I, S>(mut self, packages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.deleted.extend(packages.into_iter().map(Into::into));
        self
    }

    /// Top-level directory names a `//...` walk must not enter.
    pub fn with_skipped_root_dirs<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.skipped_at_root
            .extend(names.into_iter().map(Into::into));
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The directory a package name maps to, whether or not it exists.
    pub fn package_dir(&self, package: &str) -> PathBuf {
        if package.is_empty() {
            self.root.clone()
        } else {
            self.root.join(package)
        }
    }

    /// The BUILD file that makes `package` a package.
    pub fn build_file(&self, package: &str) -> Result<PathBuf, LookupError> {
        validate_package_name(package).map_err(|source| LookupError::InvalidName {
            package: package.to_owned(),
            source,
        })?;
        if self.is_removed(package) {
            return Err(LookupError::Deleted {
                package: package.to_owned(),
            });
        }
        let dir = self.package_dir(package);
        ["BUILD.bazel", "BUILD"]
            .into_iter()
            .map(|name| dir.join(name))
            // `metadata` follows symlinks, so a link to a file passes and a
            // dangling one, or a directory, does not.
            .find(|path| std::fs::metadata(path).is_ok_and(|m| m.is_file()))
            .ok_or_else(|| LookupError::NotFound {
                package: package.to_owned(),
            })
    }

    /// Whether `package` is a package. Suits
    /// `fjfj_graph::package::PackageBuilder`'s subpackage check.
    pub fn is_package(&self, package: &str) -> bool {
        self.build_file(package).is_ok()
    }

    /// Every package at or below `package`, sorted by name: the universe of
    /// `//package/...`. A directory that does not exist has none.
    pub fn packages_under(&self, package: &str) -> Result<Vec<String>, LookupError> {
        let _span = tracing::debug_span!("packages_under", package).entered();
        validate_package_name(package).map_err(|source| LookupError::InvalidName {
            package: package.to_owned(),
            source,
        })?;
        let mut found = Vec::new();
        // Real paths already walked, so a symlink loop ends.
        let mut walked = BTreeSet::new();
        let mut pending = vec![package.to_owned()];
        while let Some(name) = pending.pop() {
            if self.is_ignored(&name) {
                continue;
            }
            let dir = self.package_dir(&name);
            let real = match std::fs::canonicalize(&dir) {
                Ok(real) => real,
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(source) => return Err(LookupError::Io { path: dir, source }),
            };
            if !walked.insert(real) {
                continue;
            }
            if !self.deleted.contains(&name) && self.build_file(&name).is_ok() {
                found.push(name.clone());
            }
            let entries = match std::fs::read_dir(&dir) {
                Ok(entries) => entries,
                // A directory that is not one (a file, via a symlink) or that
                // cannot be read holds no packages.
                Err(_) => continue,
            };
            for entry in entries {
                let entry = entry.map_err(|source| LookupError::Io {
                    path: dir.clone(),
                    source,
                })?;
                let Some(child) = entry.file_name().to_str().map(str::to_owned) else {
                    continue; // a non-UTF-8 name cannot be in a package name
                };
                if name.is_empty() && self.skipped_at_root.contains(&child) {
                    continue;
                }
                if !std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir()) {
                    continue;
                }
                let child_name = if name.is_empty() {
                    child
                } else {
                    format!("{name}/{child}")
                };
                // A directory whose name is no legal package component
                // (`..` cannot occur, but `a:b` can) holds no package.
                if validate_package_name(&child_name).is_ok() {
                    pending.push(child_name);
                }
            }
        }
        found.sort();
        Ok(found)
    }

    fn is_removed(&self, package: &str) -> bool {
        self.deleted.contains(package) || self.is_ignored(package)
    }

    /// Is `package` at or below a `.bazelignore` directory?
    pub fn is_ignored(&self, package: &str) -> bool {
        if self.ignored.is_empty() && self.ignore_patterns.is_empty() {
            return false;
        }
        let matches = |path: &str| {
            self.ignored.contains(path) || self.ignore_patterns.iter().any(|p| p.matches(path))
        };
        // The root is a directory too: `**` matches it.
        if !self.ignore_patterns.is_empty() && matches("") {
            return true;
        }
        let mut prefix = String::new();
        for component in package.split('/').filter(|c| !c.is_empty()) {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            if matches(&prefix) {
                return true;
            }
        }
        false
    }
}

fn parse_bazelignore(file: &Path, text: &str) -> Result<BTreeSet<String>, LookupError> {
    let mut ignored = BTreeSet::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let path = Path::new(line);
        if path.is_absolute() {
            return Err(LookupError::AbsoluteIgnoreEntry {
                file: file.to_owned(),
                entry: line.to_owned(),
            });
        }
        let mut parts = Vec::new();
        let mut escapes = false;
        for component in path.components() {
            match component {
                Component::Normal(c) => match c.to_str() {
                    Some(c) => parts.push(c),
                    None => escapes = true,
                },
                Component::ParentDir => escapes = true,
                _ => {}
            }
        }
        if !escapes && !parts.is_empty() {
            ignored.insert(parts.join("/"));
        }
    }
    Ok(ignored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A repo with the given files (contents are irrelevant to lookup).
    fn repo(files: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for file in files {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "").unwrap();
        }
        dir
    }

    fn write(dir: &tempfile::TempDir, file: &str, text: &str) {
        fs::write(dir.path().join(file), text).unwrap();
    }

    fn packages(lookup: &PackageLookup) -> Vec<String> {
        lookup.packages_under("").unwrap()
    }

    #[test]
    fn build_bazel_wins_over_build() {
        let dir = repo(&["a/BUILD", "a/BUILD.bazel", "b/BUILD", "c/BUILD.bazel"]);
        let l = PackageLookup::new(dir.path()).unwrap();
        assert!(l.build_file("a").unwrap().ends_with("a/BUILD.bazel"));
        assert!(l.build_file("b").unwrap().ends_with("b/BUILD"));
        assert!(l.build_file("c").unwrap().ends_with("c/BUILD.bazel"));
    }

    #[test]
    fn a_directory_named_build_is_not_a_build_file() {
        let dir = repo(&["c/BUILD/x", "d/BUILD.bazel/x", "e/BUILD.bazel/x", "e/BUILD"]);
        let l = PackageLookup::new(dir.path()).unwrap();
        assert!(!l.is_package("c"));
        assert!(!l.is_package("d"));
        // A directory called BUILD.bazel falls through to the BUILD file.
        assert!(l.build_file("e").unwrap().ends_with("e/BUILD"));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_build_files() {
        use std::os::unix::fs::symlink;
        let dir = repo(&["a/BUILD"]);
        fs::create_dir(dir.path().join("b")).unwrap();
        symlink("../a/BUILD", dir.path().join("b/BUILD")).unwrap();
        fs::create_dir(dir.path().join("c")).unwrap();
        symlink("../nonexistent", dir.path().join("c/BUILD")).unwrap();
        let l = PackageLookup::new(dir.path()).unwrap();
        assert!(l.is_package("b"), "a link to a file is a BUILD file");
        assert!(!l.is_package("c"), "a dangling link is not");
    }

    #[test]
    fn a_missing_package_says_where_it_looked() {
        let dir = repo(&["a/BUILD", "a/nope/f.txt"]);
        let l = PackageLookup::new(dir.path()).unwrap();
        for (name, dir) in [("nope", "nope"), ("a/nope", "a/nope")] {
            assert_eq!(
                l.build_file(name).unwrap_err().to_string(),
                format!(
                    "no such package '{name}': BUILD file not found in any of the following \
                     directories. Add a BUILD file to a directory to mark it as a package.\n - {dir}"
                )
            );
        }
        assert_eq!(
            l.build_file("").unwrap_err().to_string(),
            "no such package '': BUILD file not found in any of the following directories. \
             Add a BUILD file to a directory to mark it as a package.\n - "
        );
    }

    #[test]
    fn a_bad_package_name_is_rejected_before_touching_the_disk() {
        let dir = repo(&[]);
        let l = PackageLookup::new(dir.path()).unwrap();
        assert!(matches!(
            l.build_file("../x"),
            Err(LookupError::InvalidName { .. })
        ));
        assert!(matches!(
            l.build_file("/etc"),
            Err(LookupError::InvalidName { .. })
        ));
    }

    #[test]
    fn deleted_packages_are_exactly_those_named() {
        let dir = repo(&["a/BUILD", "a/sub/BUILD", "a/sub/x/BUILD"]);
        let l = PackageLookup::new(dir.path())
            .unwrap()
            .with_deleted_packages(["a/sub"]);
        assert_eq!(
            l.build_file("a/sub").unwrap_err().to_string(),
            "no such package 'a/sub': Package is considered deleted due to --deleted_packages"
        );
        assert!(!l.is_package("a/sub"));
        assert!(l.is_package("a"));
        assert!(l.is_package("a/sub/x"));
        assert_eq!(packages(&l), ["a", "a/sub/x"]);
    }

    #[test]
    fn bazelignore_removes_a_directory_and_everything_below() {
        let dir = repo(&[
            "a/BUILD",
            "c/BUILD",
            "c/sub/BUILD",
            "d/BUILD",
            "d/e/BUILD",
            "d/e/f/BUILD",
        ]);
        write(&dir, ".bazelignore", "c\nd/e\n# a comment\n\n");
        let l = PackageLookup::new(dir.path()).unwrap();
        assert_eq!(packages(&l), ["a", "d"]);
        // Reported as deleted, in --deleted_packages' words.
        assert!(matches!(
            l.build_file("c"),
            Err(LookupError::Deleted { .. })
        ));
        assert!(matches!(
            l.build_file("d/e/f"),
            Err(LookupError::Deleted { .. })
        ));
        assert!(!l.is_package("c/sub"));
    }

    #[test]
    fn bazelignore_entries_are_normalised_but_not_trimmed_or_globbed() {
        let dir = repo(&[
            "a/BUILD",
            "c/BUILD",
            "c1/BUILD",
            "d/BUILD",
            "e/f/BUILD",
            "g/BUILD",
            "h/BUILD",
        ]);
        // `c/`, `./d` and `e//f` normalise; `  g  ` and `c*` match nothing;
        // `../up` matches nothing and is not an error.
        write(&dir, ".bazelignore", "c/\n./d\ne//f\n  g  \nh*\n../up\n");
        let l = PackageLookup::new(dir.path()).unwrap();
        assert_eq!(packages(&l), ["a", "c1", "g", "h"]);
    }

    #[test]
    fn an_absolute_bazelignore_entry_is_an_error() {
        let dir = repo(&[]);
        write(&dir, ".bazelignore", "ok\n/abs\n");
        let err = PackageLookup::new(dir.path()).unwrap_err().to_string();
        assert!(
            err.ends_with(".bazelignore: '/abs': cannot be an absolute path")
                && err.starts_with("Invalid path in "),
            "{err}"
        );
    }

    #[test]
    fn packages_under_a_prefix_include_the_prefix() {
        let dir = repo(&[
            "BUILD",
            "a/BUILD",
            "a/b/BUILD",
            "a/b/c/BUILD",
            "z/BUILD",
            "a/x/f",
        ]);
        let l = PackageLookup::new(dir.path()).unwrap();
        assert_eq!(packages(&l), ["", "a", "a/b", "a/b/c", "z"]);
        assert_eq!(l.packages_under("a/b").unwrap(), ["a/b", "a/b/c"]);
        // A directory with no BUILD file holds no packages.
        assert_eq!(l.packages_under("a/x").unwrap(), Vec::<String>::new());
        assert_eq!(l.packages_under("nope").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn root_dirs_can_be_skipped() {
        let dir = repo(&["a/BUILD", "bazel-out/BUILD", "a/bazel-out/BUILD"]);
        let l = PackageLookup::new(dir.path())
            .unwrap()
            .with_skipped_root_dirs(["bazel-out"]);
        // Only the top-level one.
        assert_eq!(packages(&l), ["a", "a/bazel-out"]);
    }

    /// The lookup is what tells a `PackageBuilder` where the packages are.
    #[test]
    fn the_lookup_drives_the_subpackage_check() {
        let dir = repo(&["a/BUILD", "a/sub/BUILD", "a/plain/f.txt"]);
        let l = PackageLookup::new(dir.path()).unwrap();
        let is_package = |p: &str| l.is_package(p);
        let mut b = fjfj_graph::package::PackageBuilder::new("", "a", &is_package);
        b.export_file("plain/f.txt", None, "a/BUILD:1:1").unwrap();
        let err = b
            .export_file("sub/f.txt", None, "a/BUILD:2:1")
            .unwrap_err()
            .to_string();
        assert!(err.contains("'a/sub' is a subpackage"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_loop_ends() {
        use std::os::unix::fs::symlink;
        let dir = repo(&["a/BUILD"]);
        symlink("..", dir.path().join("a/up")).unwrap();
        let l = PackageLookup::new(dir.path()).unwrap();
        assert_eq!(packages(&l), ["a"]);
    }

    #[test]
    fn ignore_directories_patterns_remove_what_they_match_and_below() {
        let files = [
            "BUILD",
            "c/BUILD",
            "c/sub/BUILD",
            "x/BUILD",
            "x/k/BUILD",
            "x/k/deep/BUILD",
            "gen/BUILD",
            "a/gen/BUILD",
            "a/b/gen/BUILD",
            "keep/BUILD",
            "a/keep/BUILD",
        ];
        let packages = |patterns: &[&str]| {
            let dir = repo(&files);
            PackageLookup::new(dir.path())
                .unwrap()
                .with_ignore_directories(patterns)
                .packages_under("")
                .unwrap()
        };
        assert_eq!(
            packages(&["c", "**/gen", "x/*"]),
            ["", "a/keep", "keep", "x"]
        );
        assert_eq!(packages(&["?"]), ["", "gen", "keep"]);
        assert_eq!(packages(&["x/k"]).len(), 9);
        assert_eq!(packages(&["c/sub"]).len(), 10);
        // Nothing to match: written wrong, or no pattern at all.
        for none in [&["/abs"][..], &["a/../b"], &["c/"], &["./c"], &[""], &[]] {
            assert_eq!(packages(none).len(), 11, "{none:?}");
        }
        assert_eq!(packages(&["*"]), [""]);
        assert_eq!(packages(&["**"]), Vec::<String>::new());
        let dir = repo(&files);
        let l = PackageLookup::new(dir.path())
            .unwrap()
            .with_ignore_directories(["c"]);
        assert!(
            l.build_file("c/sub")
                .unwrap_err()
                .to_string()
                .ends_with("Package is considered deleted due to --deleted_packages")
        );
    }
}
