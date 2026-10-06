//! `glob()`: the files of one package matching patterns (buildfiji-mum.4).
//!
//! Behaviour checked against Bazel 9.2.0:
//!
//! - a pattern is `/`-separated segments. `*` inside a segment matches any
//!   run of characters; `**` as a whole segment matches zero or more
//!   directories. A dot-file is matched by a segment that is exactly `*` or
//!   that starts with a literal `.`, and by no other segment that starts
//!   with a wildcard: `*` finds `.hidden.txt` but `*.txt` and `*t` do not. `?` is an error in an `include` pattern, and
//!   `[...]` and `{...}` mean nothing: they match literally.
//! - the result is sorted by bytes and has no duplicates. `**` matches the
//!   directory it stands in, so `sub/**` lists `sub` itself when directories
//!   are kept, but never the package's own directory.
//! - a directory that is itself a package (has a BUILD file) is invisible:
//!   not listed, not entered. A `.bazelignore`d directory is listed but not
//!   entered. A `--deleted_packages` package is an ordinary directory again.
//! - symbolic links are followed: one to a directory is entered, and one
//!   that dangles is not there at all. A link that leads back to a directory
//!   being walked fails the whole package, not just the glob.
//! - with `allow_empty = False` each `include` pattern must match something
//!   (directories dropped by `exclude_directories` do not count), and after
//!   `exclude` something must remain.
//! - an `exclude` pattern with no `*` or `?` is compared to whole paths as a
//!   string, so `sub` does not exclude `sub/x`. One with a wildcard is
//!   matched segment by segment, `?` standing for any one character, and is
//!   checked for the errors an `include` pattern gets except `?`.

use crate::PackageLookup;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum GlobError {
    #[error("invalid glob pattern '{pattern}': {reason}")]
    InvalidPattern {
        pattern: String,
        reason: &'static str,
    },
    #[error("{reason} (in glob pattern '{pattern}')")]
    InvalidExclude {
        pattern: String,
        reason: &'static str,
    },
    #[error(
        "glob pattern '{pattern}' didn't match anything, but allow_empty is set to False \
         (the default value of allow_empty can be set with --incompatible_disallow_empty_glob)."
    )]
    NoMatch { pattern: String },
    #[error(
        "all files in the glob have been excluded, but allow_empty is set to False \
         (the default value of allow_empty can be set with --incompatible_disallow_empty_glob)."
    )]
    AllExcluded,
    #[error(
        "subpackages pattern '{pattern}' didn't match anything, but allow_empty is set to False \
         (the default value)"
    )]
    NoSubpackageMatch { pattern: String },
    #[error(
        "all subpackages in subpackages() have been excluded, but allow_empty is set to False "
    )]
    AllSubpackagesExcluded,
    #[error("Symlink issue while evaluating globs: Infinite symlink expansion: {link}- > {target}")]
    SymlinkCycle { link: PathBuf, target: PathBuf },
    #[error("error globbing [{pattern}] op={op}: {path}: {source}", path = path.display())]
    Io {
        pattern: String,
        op: &'static str,
        path: PathBuf,
        source: io::Error,
    },
}

/// The optional parameters of `glob()`.
#[derive(Debug, Clone, Copy)]
pub struct GlobOptions {
    /// `exclude_directories = 1`, the default.
    pub exclude_directories: bool,
    pub allow_empty: bool,
}

impl Default for GlobOptions {
    fn default() -> Self {
        GlobOptions {
            exclude_directories: true,
            allow_empty: false,
        }
    }
}

/// The files of `package` matching `include` and not `exclude`, as paths
/// relative to the package, sorted.
pub fn glob(
    lookup: &PackageLookup,
    package: &str,
    include: &[String],
    exclude: &[String],
    options: GlobOptions,
) -> Result<Vec<String>, GlobError> {
    let _span = tracing::debug_span!("glob", package, patterns = include.len()).entered();
    let mut compiled = Vec::with_capacity(include.len());
    for pattern in include {
        let segments =
            check_pattern(pattern, true).map_err(|reason| GlobError::InvalidPattern {
                pattern: pattern.clone(),
                reason,
            })?;
        compiled.push(segments);
    }
    let excludes = exclude
        .iter()
        .map(|pattern| Exclude::parse(pattern))
        .collect::<Result<Vec<_>, _>>()?;

    let mut found = BTreeSet::new();
    for (pattern, segments) in include.iter().zip(&compiled) {
        let walker = Walker::new(lookup, package, pattern);
        let matched = walker.run(segments)?;
        let mut any = false;
        for (path, is_dir) in matched {
            if is_dir && options.exclude_directories {
                continue;
            }
            any = true;
            found.insert(path);
        }
        if !any && !options.allow_empty {
            return Err(GlobError::NoMatch {
                pattern: pattern.clone(),
            });
        }
    }
    found.retain(|path| !excludes.iter().any(|e| e.matches(path)));
    if found.is_empty() && !options.allow_empty {
        return Err(GlobError::AllExcluded);
    }
    Ok(found.into_iter().collect())
}

/// The packages below `package` that match `include` and not `exclude`, as
/// paths relative to it, sorted: `subpackages()`. The walk does not enter a
/// package, so a package's own subpackages are not listed.
pub fn subpackages(
    lookup: &PackageLookup,
    package: &str,
    include: &[String],
    exclude: &[String],
    allow_empty: bool,
) -> Result<Vec<String>, GlobError> {
    let _span = tracing::debug_span!("subpackages", package, patterns = include.len()).entered();
    let mut compiled = Vec::with_capacity(include.len());
    for pattern in include {
        let segments =
            check_pattern(pattern, true).map_err(|reason| GlobError::InvalidPattern {
                pattern: pattern.clone(),
                reason,
            })?;
        compiled.push(segments);
    }
    let excludes = exclude
        .iter()
        .map(|pattern| Exclude::parse(pattern))
        .collect::<Result<Vec<_>, _>>()?;

    let mut all = Vec::new();
    let mut walked = BTreeSet::new();
    let qualified = |rel: &str| match (package.is_empty(), rel.is_empty()) {
        (_, true) => package.to_owned(),
        (true, false) => rel.to_owned(),
        (false, false) => format!("{package}/{rel}"),
    };
    let mut pending = vec![String::new()];
    while let Some(rel) = pending.pop() {
        let dir = lookup.package_dir(&qualified(&rel));
        let Ok(real) = std::fs::canonicalize(&dir) else {
            continue;
        };
        if !walked.insert(real) {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir()) {
                continue;
            }
            let child = join(&rel, &name);
            let full = qualified(&child);
            if lookup.is_ignored(&full) {
                continue;
            }
            if lookup.is_package(&full) {
                all.push(child);
            } else {
                pending.push(child);
            }
        }
    }
    all.sort();

    let mut found = BTreeSet::new();
    for (pattern, segments) in include.iter().zip(&compiled) {
        let mut any = false;
        for path in &all {
            let parts: Vec<&str> = path.split('/').collect();
            if segments_match_with(segments, &parts, false) {
                any = true;
                found.insert(path.clone());
            }
        }
        if !any && !allow_empty {
            return Err(GlobError::NoSubpackageMatch {
                pattern: pattern.clone(),
            });
        }
    }
    found.retain(|path| !excludes.iter().any(|e| e.matches(path)));
    if found.is_empty() && !allow_empty {
        return Err(GlobError::AllSubpackagesExcluded);
    }
    Ok(found.into_iter().collect())
}

/// One `/`-separated part of a pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    /// `**`
    Recursive,
    /// A name, possibly with `*` in it.
    Name(String),
}

/// Bazel's own reasons, in the order it checks them. `?` is only wrong in an
/// `include` pattern.
fn check_pattern(pattern: &str, forbid_question: bool) -> Result<Vec<Segment>, &'static str> {
    if pattern.is_empty() {
        return Err("pattern cannot be empty");
    }
    if pattern.starts_with('/') {
        return Err("pattern cannot be absolute");
    }
    if forbid_question && pattern.contains('?') {
        return Err("wildcard ? forbidden");
    }
    pattern
        .split('/')
        .map(|segment| match segment {
            "" => Err("empty segment not permitted"),
            "." => Err("segment '.' not permitted"),
            ".." => Err("segment '..' not permitted"),
            "**" => Ok(Segment::Recursive),
            s if s.contains("**") => Err("recursive wildcard must be its own segment"),
            s => Ok(Segment::Name(s.to_owned())),
        })
        .collect()
}

/// One pattern of `REPO.bazel`'s `ignore_directories()`: the segments of a
/// `glob()` pattern over a directory path, with `?` standing for any one
/// character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IgnorePattern(Vec<Segment>);

impl IgnorePattern {
    /// `None` for a pattern that can match no directory: an empty segment
    /// (so a leading or trailing `/`), `.` or `..`.
    pub(crate) fn parse(pattern: &str) -> Option<IgnorePattern> {
        let segments = pattern
            .split('/')
            .map(|segment| match segment {
                "" | "." | ".." => None,
                "**" => Some(Segment::Recursive),
                s => Some(Segment::Name(s.to_owned())),
            })
            .collect::<Option<Vec<_>>>()?;
        Some(IgnorePattern(segments))
    }

    /// Does the directory `path` (relative to the repo root, `""` for the
    /// root) match?
    pub(crate) fn matches(&self, path: &str) -> bool {
        let parts: Vec<&str> = if path.is_empty() {
            Vec::new()
        } else {
            path.split('/').collect()
        };
        segments_match_with(&self.0, &parts, true)
    }
}

/// `*` matches any run of characters within one name, and with `question`
/// set `?` matches any one.
fn wildcard_match_with(pattern: &str, name: &str, question: bool) -> bool {
    if name.starts_with('.') && pattern.starts_with('*') && pattern != "*" {
        return false;
    }
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let (mut pi, mut ni) = (0, 0);
    // Where the last `*` was, and how much of `n` it has swallowed so far.
    let mut star: Option<(usize, usize)> = None;
    while ni < n.len() {
        match p.get(pi) {
            Some('*') => {
                star = Some((pi, ni));
                pi += 1;
            }
            Some(&c) if c == n[ni] || (question && c == '?') => {
                pi += 1;
                ni += 1;
            }
            _ => match star {
                Some((sp, sn)) => {
                    pi = sp + 1;
                    ni = sn + 1;
                    star = Some((sp, sn + 1));
                }
                None => return false,
            },
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

fn wildcard_match(pattern: &str, name: &str) -> bool {
    wildcard_match_with(pattern, name, false)
}

fn has_wildcard(pattern: &str) -> bool {
    pattern.contains('*')
}

/// An `exclude` pattern.
enum Exclude {
    /// No wildcard: equal to the whole path or not.
    Literal(String),
    Pattern(Vec<Segment>),
}

impl Exclude {
    fn parse(pattern: &str) -> Result<Exclude, GlobError> {
        if !pattern.contains('*') && !pattern.contains('?') {
            return Ok(Exclude::Literal(pattern.to_owned()));
        }
        check_pattern(pattern, false)
            .map(Exclude::Pattern)
            .map_err(|reason| GlobError::InvalidExclude {
                pattern: pattern.to_owned(),
                reason,
            })
    }

    fn matches(&self, path: &str) -> bool {
        match self {
            Exclude::Literal(literal) => literal == path,
            Exclude::Pattern(segments) => {
                let parts: Vec<&str> = path.split('/').collect();
                segments_match(segments, &parts)
            }
        }
    }
}

fn segments_match(segments: &[Segment], parts: &[&str]) -> bool {
    segments_match_with(segments, parts, true)
}

fn segments_match_with(segments: &[Segment], parts: &[&str], question: bool) -> bool {
    match segments.split_first() {
        None => parts.is_empty(),
        Some((Segment::Recursive, rest)) => {
            (0..=parts.len()).any(|skip| segments_match_with(rest, &parts[skip..], question))
        }
        Some((Segment::Name(pattern), rest)) => parts.split_first().is_some_and(|(part, tail)| {
            wildcard_match_with(pattern, part, question)
                && segments_match_with(rest, tail, question)
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    File,
    Dir,
}

/// Matches one include pattern against the tree.
struct Walker<'a> {
    lookup: &'a PackageLookup,
    package: &'a str,
    pattern: &'a str,
    /// Real paths of the directories being walked, innermost last.
    ancestors: Vec<PathBuf>,
    found: BTreeMap<String, bool>,
}

impl<'a> Walker<'a> {
    fn new(lookup: &'a PackageLookup, package: &'a str, pattern: &'a str) -> Self {
        Walker {
            lookup,
            package,
            pattern,
            ancestors: Vec::new(),
            found: BTreeMap::new(),
        }
    }

    /// Path (relative to the package) to `(path, is_directory)`.
    fn run(mut self, segments: &[Segment]) -> Result<BTreeMap<String, bool>, GlobError> {
        let root = self.lookup.package_dir(self.package);
        let real = match std::fs::canonicalize(&root) {
            Ok(real) => real,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(self.found),
            Err(e) => return Err(self.io("FILES", &root, e)),
        };
        self.visit("", &real, segments)?;
        Ok(self.found)
    }

    fn io(&self, op: &'static str, path: &Path, source: io::Error) -> GlobError {
        GlobError::Io {
            pattern: self.pattern.to_owned(),
            op,
            path: path.to_owned(),
            source,
        }
    }

    /// The name `rel` has in the repo, for asking the lookup about it.
    fn qualified(&self, rel: &str) -> String {
        match (self.package.is_empty(), rel.is_empty()) {
            (_, true) => self.package.to_owned(),
            (true, false) => rel.to_owned(),
            (false, false) => format!("{}/{rel}", self.package),
        }
    }

    fn is_subpackage(&self, rel: &str) -> bool {
        self.lookup.is_package(&self.qualified(rel))
    }

    /// Match `segments` against what lies under `dir`, a path relative to
    /// the package whose real path is `real`.
    fn visit(&mut self, dir: &str, real: &Path, segments: &[Segment]) -> Result<(), GlobError> {
        let Some((first, rest)) = segments.split_first() else {
            return Ok(());
        };
        // An ignored directory can be named, but nothing in it is seen.
        let ignored = !dir.is_empty() && self.lookup.is_ignored(&self.qualified(dir));
        self.ancestors.push(real.to_owned());
        let result = match first {
            Segment::Recursive if rest.is_empty() => {
                if !dir.is_empty() {
                    self.found.insert(dir.to_owned(), true);
                }
                if ignored {
                    Ok(())
                } else {
                    self.descend_all(dir, real, segments)
                }
            }
            Segment::Recursive => {
                // Zero directories, then one or more.
                let zero = self.visit(dir, real, rest);
                if zero.is_ok() && !ignored {
                    self.descend_all(dir, real, segments)
                } else {
                    zero
                }
            }
            Segment::Name(_) if ignored => Ok(()),
            Segment::Name(pattern) => self.step(dir, real, pattern, rest),
        };
        self.ancestors.pop();
        result
    }

    /// Continue `segments` into every subdirectory of `dir` that is not a
    /// package, and record the files that `**` at the end reaches.
    fn descend_all(
        &mut self,
        dir: &str,
        real: &Path,
        segments: &[Segment],
    ) -> Result<(), GlobError> {
        let last = segments.len() == 1;
        for (name, kind, child_real) in self.children(dir, real, |_| true)? {
            let child = join(dir, &name);
            match kind {
                Kind::File => {
                    if last {
                        self.found.insert(child, false);
                    }
                }
                Kind::Dir => {
                    if !self.is_subpackage(&child) {
                        self.visit(&child, &child_real, segments)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Match one name segment against the entries of `dir`, then carry on
    /// with `rest` inside each directory it picks.
    fn step(
        &mut self,
        dir: &str,
        real: &Path,
        pattern: &str,
        rest: &[Segment],
    ) -> Result<(), GlobError> {
        let entries = if has_wildcard(pattern) {
            self.children(dir, real, |name| wildcard_match(pattern, name))?
        } else {
            self.child(dir, real, pattern)?.into_iter().collect()
        };
        for (name, kind, child_real) in entries {
            let child = join(dir, &name);
            if kind == Kind::Dir && self.is_subpackage(&child) {
                continue;
            }
            if rest.is_empty() {
                self.found.insert(child, kind == Kind::Dir);
            } else if kind == Kind::Dir {
                self.visit(&child, &child_real, rest)?;
            }
        }
        Ok(())
    }

    /// The entries of `dir` whose names satisfy `keep`.
    fn children(
        &self,
        dir: &str,
        real: &Path,
        keep: impl Fn(&str) -> bool,
    ) -> Result<Vec<(String, Kind, PathBuf)>, GlobError> {
        let path = self.lookup.package_dir(&self.qualified(dir));
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            // Not a directory, or not readable: nothing to see.
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(self.io("FILES", &path, e)),
        };
        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| self.io("FILES", &path, e))?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue; // a non-UTF-8 name cannot be in a label
            };
            if !keep(&name) {
                continue;
            }
            if let Some((kind, child_real)) = self.classify(&entry.path(), real, &name)? {
                out.push((name, kind, child_real));
            }
        }
        Ok(out)
    }

    /// The one entry of `dir` called `name`, if there is one.
    fn child(
        &self,
        dir: &str,
        real: &Path,
        name: &str,
    ) -> Result<Option<(String, Kind, PathBuf)>, GlobError> {
        let path = self.lookup.package_dir(&self.qualified(dir)).join(name);
        Ok(self
            .classify(&path, real, name)?
            .map(|(kind, child_real)| (name.to_owned(), kind, child_real)))
    }

    /// What `path` is once symlinks are followed, and where it really is.
    /// `None` for an entry that is not there, which includes a dangling link.
    fn classify(
        &self,
        path: &Path,
        parent_real: &Path,
        name: &str,
    ) -> Result<Option<(Kind, PathBuf)>, GlobError> {
        let meta = match std::fs::symlink_metadata(path) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(self.io("FILES", path, e)),
        };
        if !meta.file_type().is_symlink() {
            let kind = if meta.is_dir() { Kind::Dir } else { Kind::File };
            return Ok(Some((kind, parent_real.join(name))));
        }
        let target = match std::fs::canonicalize(path) {
            Ok(target) => target,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(self.io("FILES", path, e)),
        };
        let kind = match std::fs::metadata(&target) {
            Ok(meta) if meta.is_dir() => Kind::Dir,
            Ok(_) => Kind::File,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(self.io("FILES", path, e)),
        };
        if kind == Kind::Dir && self.ancestors.contains(&target) {
            return Err(GlobError::SymlinkCycle {
                link: path.to_owned(),
                target,
            });
        }
        Ok(Some((kind, target)))
    }
}

fn join(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_owned()
    } else {
        format!("{dir}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// The tree every probe against Bazel 9.2.0 ran over (its own generated
    /// files aside), plus a `.bazelignore` for `ign`.
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for file in [
            "f1.txt",
            "fa.txt",
            ".hidden.txt",
            "B.txt",
            "a b.txt",
            "é.txt",
            "sub/x.txt",
            "sub/deep/y.txt",
            "sub/deep/BUILDX",
            "sub/.dot/q.txt",
            "pk/z.txt",
            "pk/BUILD",
            "sub/pk2/w.txt",
            "sub/pk2/BUILD.bazel",
            "dir.txt/inner",
            "ign/y.txt",
            "ign/deep/z.txt",
            "BUILD.bazel",
            ".bazelignore",
        ] {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "").unwrap();
        }
        fs::write(dir.path().join(".bazelignore"), "ign\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink("sub", dir.path().join("link_to_sub")).unwrap();
            symlink("f1.txt", dir.path().join("link_to_f1")).unwrap();
            symlink("nonexistent", dir.path().join("broken")).unwrap();
        }
        dir
    }

    fn run(
        root: &Path,
        include: &[&str],
        exclude: &[&str],
        options: GlobOptions,
    ) -> Result<Vec<String>, GlobError> {
        let lookup = PackageLookup::new(root).unwrap();
        let own = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        glob(&lookup, "", &own(include), &own(exclude), options)
    }

    fn files(root: &Path, include: &[&str]) -> Vec<String> {
        run(root, include, &[], GlobOptions::default()).unwrap()
    }

    fn with_dirs() -> GlobOptions {
        GlobOptions {
            exclude_directories: false,
            allow_empty: false,
        }
    }

    fn message(r: Result<Vec<String>, GlobError>) -> String {
        r.unwrap_err().to_string()
    }

    // Every expectation below is what `print(glob(...))` gave on Bazel 9.2.0.

    #[test]
    fn star_matches_within_one_segment_and_keeps_dot_files() {
        let d = fixture();
        assert_eq!(
            files(d.path(), &["*.txt"]),
            ["B.txt", "a b.txt", "f1.txt", "fa.txt", "é.txt"]
        );
        assert_eq!(files(d.path(), &["f*"]), ["f1.txt", "fa.txt"]);
        assert_eq!(files(d.path(), &["f*1.txt"]), ["f1.txt"]);
        assert_eq!(files(d.path(), &["*1*"]), ["f1.txt", "link_to_f1"]);
        assert_eq!(
            files(d.path(), &["*/x.txt"]),
            ["link_to_sub/x.txt", "sub/x.txt"]
        );
        assert_eq!(
            files(d.path(), &["sub/*/*.txt"]),
            ["sub/.dot/q.txt", "sub/deep/y.txt"]
        );
        assert_eq!(
            files(d.path(), &["*"]),
            [
                ".bazelignore",
                ".hidden.txt",
                "B.txt",
                "BUILD.bazel",
                "a b.txt",
                "f1.txt",
                "fa.txt",
                "link_to_f1",
                "é.txt"
            ]
        );
    }

    #[test]
    fn a_dot_file_needs_a_lone_star_or_a_leading_dot() {
        let d = fixture();
        assert!(files(d.path(), &["*"]).contains(&".hidden.txt".to_owned()));
        assert!(files(d.path(), &["**/*"]).contains(&"sub/.dot/q.txt".to_owned()));
        assert_eq!(files(d.path(), &[".*"]), [".bazelignore", ".hidden.txt"]);
        assert_eq!(files(d.path(), &[".h*"]), [".hidden.txt"]);
        assert_eq!(files(d.path(), &[".hidden.txt"]), [".hidden.txt"]);
        for pattern in ["*d*", "*hidden*", "*.h*", "*idden.txt", "**/*idden.txt"] {
            assert!(
                message(run(d.path(), &[pattern], &[], GlobOptions::default()))
                    .contains("didn't match anything"),
                "{pattern}"
            );
        }
        // Directories follow the same rule: `.dot` is not `sub/*.*`.
        assert_eq!(
            run(d.path(), &["sub/*.*"], &[], with_dirs()).unwrap(),
            ["sub/x.txt"]
        );
        assert_eq!(
            run(d.path(), &["sub/.*"], &[], with_dirs()).unwrap(),
            ["sub/.dot"]
        );
    }

    #[test]
    fn matching_is_case_sensitive_and_literal_for_other_punctuation() {
        let d = fixture();
        for pattern in ["*.TXT", "f[1a].txt", "{f1,fa}.txt", "\\*", "f1.txt "] {
            assert!(
                message(run(d.path(), &[pattern], &[], GlobOptions::default()))
                    .contains("didn't match anything"),
                "{pattern}"
            );
        }
        assert_eq!(files(d.path(), &["a b.txt"]), ["a b.txt"]);
        assert_eq!(files(d.path(), &["é.txt"]), ["é.txt"]);
    }

    #[test]
    fn double_star_matches_zero_or_more_directories() {
        let d = fixture();
        assert_eq!(
            files(d.path(), &["**/*.txt"]),
            [
                "B.txt",
                "a b.txt",
                "f1.txt",
                "fa.txt",
                "link_to_sub/.dot/q.txt",
                "link_to_sub/deep/y.txt",
                "link_to_sub/x.txt",
                "sub/.dot/q.txt",
                "sub/deep/y.txt",
                "sub/x.txt",
                "é.txt"
            ]
        );
        assert_eq!(files(d.path(), &["sub/**/y.txt"]), ["sub/deep/y.txt"]);
        assert_eq!(
            files(d.path(), &["**/deep/**"]),
            [
                "link_to_sub/deep/BUILDX",
                "link_to_sub/deep/y.txt",
                "sub/deep/BUILDX",
                "sub/deep/y.txt"
            ]
        );
        assert_eq!(
            files(d.path(), &["**/**/x.txt"]),
            files(d.path(), &["**/x.txt"])
        );
    }

    #[test]
    fn a_trailing_double_star_lists_the_directory_itself_only_when_directories_stay() {
        let d = fixture();
        assert_eq!(
            files(d.path(), &["sub/**"]),
            [
                "sub/.dot/q.txt",
                "sub/deep/BUILDX",
                "sub/deep/y.txt",
                "sub/x.txt"
            ]
        );
        assert_eq!(
            run(d.path(), &["sub/**"], &[], with_dirs()).unwrap(),
            [
                "sub",
                "sub/.dot",
                "sub/.dot/q.txt",
                "sub/deep",
                "sub/deep/BUILDX",
                "sub/deep/y.txt",
                "sub/x.txt"
            ]
        );
        // The package's own directory is never in the result.
        assert!(
            !run(d.path(), &["**"], &[], with_dirs())
                .unwrap()
                .contains(&String::new())
        );
    }

    #[test]
    fn directories_are_dropped_unless_asked_for() {
        let d = fixture();
        assert!(
            message(run(d.path(), &["dir.txt"], &[], GlobOptions::default()))
                .contains("'dir.txt' didn't match anything")
        );
        assert_eq!(
            run(d.path(), &["dir.txt"], &[], with_dirs()).unwrap(),
            ["dir.txt"]
        );
        assert_eq!(
            run(d.path(), &["*.txt"], &[], with_dirs()).unwrap(),
            ["B.txt", "a b.txt", "dir.txt", "f1.txt", "fa.txt", "é.txt"]
        );
        assert_eq!(
            run(d.path(), &["sub/*"], &[], with_dirs()).unwrap(),
            ["sub/.dot", "sub/deep", "sub/x.txt"]
        );
    }

    #[test]
    fn a_subpackage_is_invisible() {
        let d = fixture();
        for pattern in ["pk", "pk/*", "pk/z.txt", "sub/pk2/w.txt"] {
            for options in [GlobOptions::default(), with_dirs()] {
                assert!(
                    message(run(d.path(), &[pattern], &[], options))
                        .contains("didn't match anything"),
                    "{pattern}"
                );
            }
        }
        let all = run(d.path(), &["**"], &[], with_dirs()).unwrap();
        assert!(!all.iter().any(|p| p.starts_with("pk") || p.contains("pk2")));
    }

    #[test]
    fn an_ignored_directory_is_listed_but_not_entered() {
        let d = fixture();
        assert_eq!(run(d.path(), &["ign"], &[], with_dirs()).unwrap(), ["ign"]);
        assert_eq!(
            run(d.path(), &["ign/**"], &[], with_dirs()).unwrap(),
            ["ign"]
        );
        for pattern in ["ign/*", "ign/y.txt", "**/z.txt"] {
            assert!(
                message(run(d.path(), &[pattern], &[], GlobOptions::default()))
                    .contains("didn't match anything"),
                "{pattern}"
            );
        }
    }

    #[test]
    fn a_deleted_package_is_an_ordinary_directory() {
        let d = fixture();
        let lookup = PackageLookup::new(d.path())
            .unwrap()
            .with_deleted_packages(["pk"]);
        let got = glob(
            &lookup,
            "",
            &["pk/*".to_owned()],
            &[],
            GlobOptions::default(),
        )
        .unwrap();
        assert_eq!(got, ["pk/BUILD", "pk/z.txt"]);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_followed_and_dangling_ones_are_absent() {
        let d = fixture();
        assert_eq!(files(d.path(), &["link_to_f1"]), ["link_to_f1"]);
        assert_eq!(files(d.path(), &["link_to_sub/*"]), ["link_to_sub/x.txt"]);
        assert_eq!(
            files(d.path(), &["link_to_sub/**"]),
            [
                "link_to_sub/.dot/q.txt",
                "link_to_sub/deep/BUILDX",
                "link_to_sub/deep/y.txt",
                "link_to_sub/x.txt"
            ]
        );
        assert!(
            message(run(d.path(), &["link_to_sub"], &[], GlobOptions::default()))
                .contains("didn't match anything")
        );
        assert_eq!(
            run(d.path(), &["link_to_sub"], &[], with_dirs()).unwrap(),
            ["link_to_sub"]
        );
        for options in [GlobOptions::default(), with_dirs()] {
            assert!(
                message(run(d.path(), &["broken"], &[], options))
                    .contains("'broken' didn't match anything")
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_link_back_to_a_directory_being_walked_is_an_error() {
        let d = fixture();
        std::os::unix::fs::symlink(".", d.path().join("loop")).unwrap();
        for pattern in ["*", "**"] {
            let err = message(run(d.path(), &[pattern], &[], GlobOptions::default()));
            assert!(
                err.starts_with(
                    "Symlink issue while evaluating globs: Infinite symlink expansion:"
                ),
                "{err}"
            );
        }
        // Naming an unrelated entry never looks at the link.
        assert_eq!(files(d.path(), &["f1.txt"]), ["f1.txt"]);
    }

    #[test]
    fn include_patterns_are_checked_in_bazels_words() {
        let d = fixture();
        for (pattern, reason) in [
            ("", "pattern cannot be empty"),
            ("/f1.txt", "pattern cannot be absolute"),
            ("?.txt", "wildcard ? forbidden"),
            ("sub//x.txt", "empty segment not permitted"),
            ("sub/", "empty segment not permitted"),
            ("./f1.txt", "segment '.' not permitted"),
            (".", "segment '.' not permitted"),
            ("sub/.", "segment '.' not permitted"),
            ("sub/../f1.txt", "segment '..' not permitted"),
            ("../x", "segment '..' not permitted"),
            ("a**b", "recursive wildcard must be its own segment"),
            ("***", "recursive wildcard must be its own segment"),
            ("sub/**y", "recursive wildcard must be its own segment"),
            ("sub/x**", "recursive wildcard must be its own segment"),
        ] {
            assert_eq!(
                message(run(d.path(), &[pattern], &[], GlobOptions::default())),
                format!("invalid glob pattern '{pattern}': {reason}"),
                "{pattern:?}"
            );
        }
    }

    #[test]
    fn a_pattern_that_matches_nothing_is_an_error_naming_the_first() {
        let d = fixture();
        let want = |p: &str| {
            format!(
                "glob pattern '{p}' didn't match anything, but allow_empty is set to False \
                 (the default value of allow_empty can be set with \
                 --incompatible_disallow_empty_glob)."
            )
        };
        assert_eq!(
            message(run(
                d.path(),
                &["nope1", "nope2"],
                &[],
                GlobOptions::default()
            )),
            want("nope1")
        );
        assert_eq!(
            message(run(
                d.path(),
                &["f1.txt", "nope.txt"],
                &[],
                GlobOptions::default()
            )),
            want("nope.txt")
        );
        assert_eq!(
            message(run(d.path(), &["nope/**"], &[], GlobOptions::default())),
            want("nope/**")
        );
        let lenient = GlobOptions {
            allow_empty: true,
            ..GlobOptions::default()
        };
        assert_eq!(
            run(d.path(), &["nope*"], &[], lenient).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            run(d.path(), &[], &[], lenient).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn excluding_everything_is_its_own_error() {
        let d = fixture();
        let want = "all files in the glob have been excluded, but allow_empty is set to False \
                    (the default value of allow_empty can be set with \
                    --incompatible_disallow_empty_glob).";
        for (include, exclude) in [
            (&["f1.txt"][..], &["f1.txt"][..]),
            (&["*.txt"], &["*"]),
            (&["*.txt"], &["**"]),
            (&["*.txt"], &["**/*.txt"]),
            (&[], &[]),
        ] {
            assert_eq!(
                message(run(d.path(), include, exclude, GlobOptions::default())),
                want,
                "{include:?} {exclude:?}"
            );
        }
    }

    #[test]
    fn exclude_matches_whole_paths() {
        let d = fixture();
        let with = |exclude: &[&str]| run(d.path(), &["*.txt"], exclude, GlobOptions::default());
        assert_eq!(with(&["f*"]).unwrap(), ["B.txt", "a b.txt", "é.txt"]);
        assert_eq!(with(&["**/f*"]).unwrap(), ["B.txt", "a b.txt", "é.txt"]);
        assert_eq!(with(&["f?.txt"]).unwrap(), ["B.txt", "a b.txt", "é.txt"]);
        assert_eq!(with(&["f*", "*a*"]).unwrap(), ["B.txt", "é.txt"]);
        assert_eq!(
            with(&["**/fa.txt"]).unwrap(),
            ["B.txt", "a b.txt", "f1.txt", "é.txt"]
        );
        // A literal is compared as a string, whatever it looks like.
        for literal in ["/x", "a//b", "", "../x", ".", "sub/", "nope"] {
            assert_eq!(with(&[literal]).unwrap().len(), 5, "{literal:?}");
        }
        let sub = |exclude: &[&str]| run(d.path(), &["sub/**"], exclude, GlobOptions::default());
        assert_eq!(
            sub(&["sub/deep/**"]).unwrap(),
            ["sub/.dot/q.txt", "sub/x.txt"]
        );
        assert_eq!(
            sub(&["**/deep/*"]).unwrap(),
            ["sub/.dot/q.txt", "sub/x.txt"]
        );
        assert_eq!(
            sub(&["sub/deep"]).unwrap().len(),
            4,
            "a literal names one path"
        );
    }

    #[test]
    fn an_exclude_with_a_wildcard_is_checked() {
        let d = fixture();
        for (pattern, reason) in [
            ("a**b", "recursive wildcard must be its own segment"),
            ("f*.txt/", "empty segment not permitted"),
            ("*.txt/", "empty segment not permitted"),
        ] {
            assert_eq!(
                message(run(
                    d.path(),
                    &["*.txt"],
                    &[pattern],
                    GlobOptions::default()
                )),
                format!("{reason} (in glob pattern '{pattern}')")
            );
        }
    }

    #[test]
    fn patterns_overlap_without_repeating_a_file() {
        let d = fixture();
        assert_eq!(files(d.path(), &["f1.txt", "f1.txt"]), ["f1.txt"]);
        assert_eq!(
            files(d.path(), &["*.txt", "f*"]),
            files(d.path(), &["*.txt"])
        );
    }

    #[test]
    fn a_package_below_the_root_globs_relative_to_itself() {
        let d = fixture();
        let lookup = PackageLookup::new(d.path()).unwrap();
        let got = glob(
            &lookup,
            "sub",
            &["**/*.txt".to_owned()],
            &[],
            GlobOptions::default(),
        )
        .unwrap();
        assert_eq!(got, [".dot/q.txt", "deep/y.txt", "x.txt"]);
        // A package inside the walked one is still hidden.
        let hidden = glob(
            &lookup,
            "sub",
            &["pk2/*".to_owned()],
            &[],
            GlobOptions::default(),
        );
        assert!(hidden.is_err());
    }

    /// The tree probed against Bazel 9.2.0's `subpackages()`: packages at
    /// `a/b`, `a/c/d`, `e` and `.h/z`, a plain file `a/x/y`, and `ign`
    /// ignored.
    fn sub_fixture(a_is_package: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let mut files = vec![
            "BUILD",
            "a/b/BUILD",
            "a/c/d/BUILD",
            "e/BUILD",
            ".h/z/BUILD",
            "a/x/y",
            "ign/p/BUILD",
        ];
        if a_is_package {
            files.push("a/BUILD");
        }
        for file in files {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        fs::write(dir.path().join(".bazelignore"), "ign\n").unwrap();
        dir
    }

    fn subs(
        dir: &tempfile::TempDir,
        include: &[&str],
        exclude: &[&str],
        allow_empty: bool,
    ) -> Result<Vec<String>, String> {
        let lookup = PackageLookup::new(dir.path()).unwrap();
        let own = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        subpackages(&lookup, "", &own(include), &own(exclude), allow_empty)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn subpackages_lists_the_outermost_packages_sorted() {
        let dir = sub_fixture(false);
        let want = |l: &[&str]| Ok(l.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(
            subs(&dir, &["**"], &[], false),
            want(&[".h/z", "a/b", "a/c/d", "e"])
        );
        assert_eq!(subs(&dir, &["a/**"], &[], false), want(&["a/b", "a/c/d"]));
        assert_eq!(subs(&dir, &["a/*"], &[], false), want(&["a/b"]));
        assert_eq!(subs(&dir, &["*/*"], &[], false), want(&[".h/z", "a/b"]));
        assert_eq!(subs(&dir, &["*"], &[], false), want(&["e"]));
        assert_eq!(subs(&dir, &["e", "e"], &[], false), want(&["e"]));
        assert_eq!(
            subs(&dir, &["**"], &["a/c/**"], false),
            want(&[".h/z", "a/b", "e"])
        );
        assert_eq!(subs(&dir, &["a/**"], &["a/b"], false), want(&["a/c/d"]));
        assert_eq!(subs(&dir, &["zz"], &[], true), want(&[]));
        // A package hides what is below it.
        let dir = sub_fixture(true);
        assert_eq!(subs(&dir, &["**"], &[], false), want(&[".h/z", "a", "e"]));
        assert_eq!(subs(&dir, &["a/**"], &[], false), want(&["a"]));
        assert!(subs(&dir, &["a/b"], &[], false).is_err());
    }

    #[test]
    fn subpackages_errors_are_bazels() {
        let dir = sub_fixture(false);
        let err = |i: &[&str], x: &[&str]| subs(&dir, i, x, false).unwrap_err();
        assert_eq!(
            err(&["zz"], &[]),
            "subpackages pattern 'zz' didn't match anything, but allow_empty is set to False \
             (the default value)"
        );
        assert_eq!(
            err(&["a/x"], &[]),
            "subpackages pattern 'a/x' didn't match anything, but allow_empty is set to False \
             (the default value)"
        );
        assert_eq!(
            err(&["e"], &["e"]),
            "all subpackages in subpackages() have been excluded, but allow_empty is set to False "
        );
        assert_eq!(
            err(&["/a"], &[]),
            "invalid glob pattern '/a': pattern cannot be absolute"
        );
        assert_eq!(
            err(&["a/.."], &[]),
            "invalid glob pattern 'a/..': segment '..' not permitted"
        );
        assert_eq!(
            err(&["**x"], &[]),
            "invalid glob pattern '**x': recursive wildcard must be its own segment"
        );
        assert_eq!(
            err(&[""], &[]),
            "invalid glob pattern '': pattern cannot be empty"
        );
        assert_eq!(
            err(&["?"], &[]),
            "invalid glob pattern '?': wildcard ? forbidden"
        );
    }
}
