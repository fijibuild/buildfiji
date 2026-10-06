//! From target patterns to targets (buildfiji-gwl.3's loading half).
//!
//! Behaviour checked against Bazel 9.2.0 with `bazel build --nobuild`:
//!
//! - `:all` selects a package's rules, `:*` and `:all-targets` every target:
//!   rules, package groups, generated files, exported files, the BUILD file
//!   and the files its rules name. `//d/...` is `:all` in every package below
//!   `d`. A rule tagged `manual` is left out of all three, and still selected
//!   when named.
//! - a pattern with no `:` that is not `//p` is a path: a package directory
//!   names that package's target of the directory's name, and any other path
//!   is a target of the nearest package above it.
//! - a negative pattern removes what it selects from what the positive ones
//!   selected, wherever it is written.
//! - a pattern that fails is skipped, and the rest are still resolved.

use crate::{LookupError, PackageLookup};
use fjfj_graph::Label;
use fjfj_graph::package::{Package, Target, TargetKind};
use fjfj_graph::pattern::{Pattern, TargetPattern};
use fjfj_graph::rule::{AttrDefault, AttrValue, native_rule, suggest};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Where packages come from; the repo is made first if it must be.
pub trait PackageSource: Send + Sync {
    /// The files of repo `repo` (canonical), or why it could not be had.
    fn lookup(&self, repo: &str) -> Result<Arc<PackageLookup>, String>;

    /// `package` of `repo` with its BUILD file evaluated, or Bazel's
    /// message for what went wrong.
    fn package(&self, repo: &str, package: &str) -> Result<Arc<Package>, String>;

    /// [`PackageSource::package`] for `purpose`, which decides what a
    /// failure says besides the error of the package.
    fn package_for(
        &self,
        repo: &str,
        package: &str,
        _purpose: Purpose,
    ) -> Result<Arc<Package>, String> {
        self.package(repo, package)
    }
}

/// Why a package is being loaded, as Bazel's events for a failure differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// Its targets by `:all`, a dependency or any other way.
    Package,
    /// One target of it, by name: no `package contains errors` event, the
    /// package was not evaluated as a whole.
    Target,
    /// A directory tree it is under, `//...`: an extra event for the package
    /// with no name comes first.
    Tree,
}

/// A pattern that selected nothing, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The pattern as given.
    pub pattern: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolved {
    /// The targets selected, first selected first, each once.
    pub targets: Vec<Label>,
    pub failures: Vec<Failure>,
}

fn label_text(label: &Label) -> String {
    if label.repo.is_empty() {
        format!("//{}:{}", label.package, label.name)
    } else {
        format!("@@{}//{}:{}", label.repo, label.package, label.name)
    }
}

/// Whether the rule is tagged `manual`, by the BUILD file or by default (a
/// `toolchain` or `config_setting` is, unless it says otherwise).
fn is_manual(target: &Target) -> bool {
    let TargetKind::Rule {
        rule_class, attrs, ..
    } = &target.kind
    else {
        return false;
    };
    match attrs.iter().find(|(name, _)| name == "tags") {
        Some((_, AttrValue::StringList(tags))) => tags.iter().any(|t| t == "manual"),
        Some(_) => false,
        None => native_rule(rule_class)
            .and_then(|class| class.attr("tags"))
            .is_some_and(
                |tags| matches!(tags.default, AttrDefault::Strs(list) if list.contains(&"manual")),
            ),
    }
}

/// Whether `label` names a target of `loaded` (its package, in `lookup`'s
/// repo): a declared one, a file a rule names, or the BUILD file. If not, the
/// words Bazel says it in.
pub fn declared_target(
    loaded: &Package,
    lookup: &PackageLookup,
    label: &Label,
) -> Result<(), String> {
    if loaded.target(&label.name).is_some() || loaded.input_files().contains(&label.name) {
        return Ok(());
    }
    let build = lookup
        .build_file(&label.package)
        .map_err(|e| e.to_string())?;
    if build.file_name().and_then(|n| n.to_str()) == Some(label.name.as_str()) {
        return Ok(());
    }
    let build_rel = match label.package.as_str() {
        "" => build.file_name().map(|n| n.to_string_lossy().into_owned()),
        p => build
            .file_name()
            .map(|n| format!("{p}/{}", n.to_string_lossy())),
    }
    .unwrap_or_default();
    let on_disk = lookup.package_dir(&label.package).join(&label.name);
    let hint = if on_disk.is_dir() {
        format!(
            "; however, a source directory of this name exists.  (Perhaps add 'exports_files([\"{}\"])' to {build_rel}, or define a filegroup?)",
            label.name
        )
    } else if on_disk.is_file() {
        format!(
            "; however, a source file of this name exists.  (Perhaps add 'exports_files([\"{}\"])' to {build_rel}?)",
            label.name
        )
    } else {
        // Bazel's suggestion ignores case: `A` finds `a`.
        let lower = label.name.to_lowercase();
        let names: Vec<&str> = loaded.targets().iter().map(|t| t.name.as_str()).collect();
        let lowered: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        match suggest(&lower, lowered.iter().map(String::as_str)) {
            Some(found) => {
                let original = names
                    .iter()
                    .zip(&lowered)
                    .find(|(_, l)| l.as_str() == found)
                    .map(|(n, _)| *n)
                    .unwrap_or(found);
                format!(" (did you mean {original}?)")
            }
            None => String::new(),
        }
    };
    Err(format!(
        "no such target '{}': target '{}' not declared in package '{}' defined by {}{hint}",
        label_text(label),
        label.name,
        label.package,
        build.display()
    ))
}

/// A package as loaded, or what went wrong.
type Loaded = Result<Arc<Package>, String>;

struct Resolver<'a> {
    include_manual: bool,
    source: &'a dyn PackageSource,
    packages: RefCell<HashMap<(String, String), Loaded>>,
}

impl Resolver<'_> {
    fn load(&self, repo: &str, package: &str, purpose: Purpose) -> Loaded {
        let key = (repo.to_owned(), package.to_owned());
        if let Some(done) = self.packages.borrow().get(&key) {
            return done.clone();
        }
        let loaded = self.source.package_for(repo, package, purpose);
        self.packages.borrow_mut().insert(key, loaded.clone());
        loaded
    }

    /// The targets of one package a wildcard selects.
    fn wildcard(
        &self,
        repo: &str,
        package: &str,
        rules_only: bool,
        purpose: Purpose,
    ) -> Result<Vec<Label>, String> {
        let loaded = self.load(repo, package, purpose)?;
        let mut out = Vec::new();
        let label = |name: &str| Label {
            repo: repo.to_owned(),
            package: package.to_owned(),
            name: name.to_owned(),
        };
        for target in loaded.targets() {
            let is_rule = matches!(target.kind, TargetKind::Rule { .. });
            if (rules_only && !is_rule) || (!self.include_manual && is_manual(target)) {
                continue;
            }
            out.push(label(&target.name));
        }
        if !rules_only {
            let lookup = self.source.lookup(repo)?;
            if let Some(name) = lookup
                .build_file(package)
                .ok()
                .and_then(|b| b.file_name().and_then(|n| n.to_str()).map(str::to_owned))
            {
                out.push(label(&name));
            }
            out.extend(loaded.input_files().iter().map(|n| label(n)));
        }
        Ok(out)
    }

    fn target(&self, label: &Label) -> Result<Label, String> {
        let lookup = self.source.lookup(&label.repo)?;
        let loaded = match self.load(&label.repo, &label.package, Purpose::Target) {
            Ok(loaded) => loaded,
            // A package with errors declares no target that can be found, and
            // the errors were said already.
            Err(message) if message.ends_with("' contains errors") => {
                let build = lookup
                    .build_file(&label.package)
                    .map_err(|e| e.to_string())?;
                return Err(format!(
                    "no such target '{}': target '{}' not declared in package '{}' defined by {}",
                    label_text(label),
                    label.name,
                    label.package,
                    build.display()
                ));
            }
            Err(message) => return Err(message),
        };
        declared_target(&loaded, &lookup, label)?;
        Ok(label.clone())
    }

    /// The package that holds `path`: the path itself if it is one, else the
    /// nearest directory above it that is, with the rest as the target name.
    fn enclosing(&self, repo: &str, path: &str) -> Result<Label, String> {
        let lookup = self.source.lookup(repo)?;
        if !path.is_empty() && lookup.is_package(path) {
            let name = path.rsplit('/').next().unwrap_or(path);
            return Ok(Label {
                repo: repo.to_owned(),
                package: path.to_owned(),
                name: name.to_owned(),
            });
        }
        let mut package = path;
        loop {
            let (parent, _) = package.rsplit_once('/').unwrap_or(("", package));
            package = parent;
            if lookup.is_package(package) || package.is_empty() {
                break;
            }
        }
        let name = path
            .strip_prefix(package)
            .map(|rest| rest.trim_start_matches('/'))
            .unwrap_or(path);
        if !lookup.is_package(package) {
            return Err(lookup
                .build_file(package)
                .err()
                .map(|e: LookupError| e.to_string())
                .unwrap_or_default());
        }
        Ok(Label {
            repo: repo.to_owned(),
            package: package.to_owned(),
            name: name.to_owned(),
        })
    }

    fn select(&self, pattern: &Pattern) -> Result<Vec<Label>, String> {
        match pattern {
            Pattern::Target(label) => Ok(vec![self.target(label)?]),
            Pattern::Path { repo, path } => {
                let label = self.enclosing(repo, path)?;
                Ok(vec![self.target(&label)?])
            }
            Pattern::InPackage {
                repo,
                package,
                rules_only,
            } => self.wildcard(repo, package, *rules_only, Purpose::Package),
            Pattern::Below {
                repo,
                directory,
                rules_only,
            } => {
                let lookup = self.source.lookup(repo)?;
                let packages = lookup
                    .packages_under(directory)
                    .map_err(|e| e.to_string())?;
                if packages.is_empty() {
                    return Err(format!("no targets found beneath '{directory}'"));
                }
                let mut out = Vec::new();
                for package in packages {
                    out.extend(self.wildcard(repo, &package, *rules_only, Purpose::Tree)?);
                }
                Ok(out)
            }
        }
    }
}

/// What Bazel says of a pattern whose package did not load: one whose BUILD
/// file has errors was evaluated for it, one that could not be read was
/// being parsed.
fn in_pattern(pattern: &str, message: String) -> String {
    if !message.starts_with("error loading package '") {
        message
    } else if message.ends_with("' contains errors") {
        format!("Error evaluating '{pattern}': {message}")
    } else {
        format!("while parsing '{pattern}': {message}")
    }
}

/// The targets `patterns` select.
pub fn resolve(patterns: &[TargetPattern], source: &dyn PackageSource) -> Resolved {
    resolve_with(patterns, source, false)
}

/// [`resolve`], with the targets tagged `manual` that a wildcard leaves out
/// kept in when `include_manual`: what `register_toolchains` selects.
pub fn resolve_with(
    patterns: &[TargetPattern],
    source: &dyn PackageSource,
    include_manual: bool,
) -> Resolved {
    let resolver = Resolver {
        include_manual,
        source,
        packages: RefCell::new(HashMap::new()),
    };
    let mut selected: Vec<Label> = Vec::new();
    let mut removed: HashSet<Label> = HashSet::new();
    let mut failures = Vec::new();
    for pattern in patterns {
        match resolver.select(&pattern.pattern) {
            Ok(labels) if pattern.negative => removed.extend(labels),
            Ok(labels) => {
                for label in labels {
                    if !selected.contains(&label) {
                        selected.push(label);
                    }
                }
            }
            Err(message) => failures.push(Failure {
                pattern: pattern.text.clone(),
                message: in_pattern(&pattern.text, message),
            }),
        }
    }
    selected.retain(|l| !removed.contains(l));
    Resolved {
        targets: selected,
        failures,
    }
}
