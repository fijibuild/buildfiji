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

    /// What the BUILD file of a package that failed with errors defined before
    /// it ended: Bazel keeps those targets, with the package marked as in error.
    fn partial(&self, _repo: &str, _package: &str) -> Option<Arc<Package>> {
        None
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
    /// The message of the package error alone, when a package with errors
    /// caused the failure.
    pub package_error: Option<String>,
    /// Whether the pattern is a `...` one.
    pub tree: bool,
    /// Whether the package that failed is one whose BUILD file ran to the end
    /// and reported errors, so that it still defines targets.
    pub defined: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolved {
    /// The targets selected, first selected first, each once.
    pub targets: Vec<Label>,
    /// Those of `targets` a pattern names, not one that selects what is in a
    /// package or below a directory.
    pub explicit: std::collections::BTreeSet<Label>,
    pub failures: Vec<Failure>,
    /// What the patterns select in packages whose BUILD files reported errors
    /// but defined targets: Bazel keeps them, and they fail to analyse.
    pub in_error: Vec<Label>,
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
    ) -> Result<Vec<Label>, Failed> {
        match self.load(repo, package, purpose) {
            Ok(loaded) => self.wildcard_of(&loaded, repo, package, rules_only),
            Err(message) => Err(self.failed(repo, package, message, |partial| {
                self.wildcard_of(partial, repo, package, rules_only).ok()
            })),
        }
    }

    /// `message`, which says a package failed to load, with what the package
    /// defined before it did if it ended with errors (`from`).
    fn failed(
        &self,
        repo: &str,
        package: &str,
        message: String,
        from: impl Fn(&Package) -> Option<Vec<Label>>,
    ) -> Failed {
        let defined = message
            .ends_with("' contains errors")
            .then(|| self.source.partial(repo, package))
            .flatten();
        Failed {
            message,
            partial: defined
                .as_ref()
                .and_then(|partial| from(partial))
                .unwrap_or_default(),
            defined: defined.is_some(),
        }
    }

    fn wildcard_of(
        &self,
        loaded: &Package,
        repo: &str,
        package: &str,
        rules_only: bool,
    ) -> Result<Vec<Label>, Failed> {
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
            let lookup = self.source.lookup(repo).map_err(Failed::plain)?;
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

    fn target(&self, label: &Label) -> Result<Label, Failed> {
        let lookup = self.source.lookup(&label.repo).map_err(Failed::plain)?;
        let loaded = match self.load(&label.repo, &label.package, Purpose::Target) {
            Ok(loaded) => loaded,
            // A package whose file did not run to the end declares no target
            // that can be found, and the errors were said already. One that
            // did declares the targets it got to.
            Err(message) if message.ends_with("' contains errors") => {
                if let Some(partial) = self.source.partial(&label.repo, &label.package)
                    && declared_target(&partial, &lookup, label).is_ok()
                {
                    return Err(Failed {
                        message,
                        partial: vec![label.clone()],
                        defined: true,
                    });
                }
                let build = lookup
                    .build_file(&label.package)
                    .map_err(|e| Failed::plain(e.to_string()))?;
                return Err(Failed::plain(format!(
                    "no such target '{}': target '{}' not declared in package '{}' defined by {}",
                    label_text(label),
                    label.name,
                    label.package,
                    build.display()
                )));
            }
            Err(message) => return Err(Failed::plain(message)),
        };
        declared_target(&loaded, &lookup, label).map_err(Failed::plain)?;
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

    fn select(&self, pattern: &Pattern) -> Selected {
        let mut selected = Selected::default();
        match pattern {
            Pattern::Target(label) => match self.target(label) {
                Ok(label) => selected.targets.push(label),
                Err(failed) => selected.fail(failed),
            },
            Pattern::Path { repo, path } => match self.enclosing(repo, path) {
                Ok(label) => match self.target(&label) {
                    Ok(label) => selected.targets.push(label),
                    Err(failed) => selected.fail(failed),
                },
                Err(message) => selected.fail(Failed::plain(message)),
            },
            Pattern::InPackage {
                repo,
                package,
                rules_only,
            } => match self.wildcard(repo, package, *rules_only, Purpose::Package) {
                Ok(labels) => selected.targets = labels,
                Err(failed) => selected.fail(failed),
            },
            Pattern::Below {
                repo,
                directory,
                rules_only,
            } => {
                let lookup = match self.source.lookup(repo) {
                    Ok(lookup) => lookup,
                    Err(message) => {
                        selected.fail(Failed::plain(message));
                        return selected;
                    }
                };
                let packages = match lookup.packages_under(directory) {
                    Ok(packages) => packages,
                    Err(e) => {
                        selected.fail(Failed::plain(e.to_string()));
                        return selected;
                    }
                };
                if packages.is_empty() {
                    selected.fail(Failed::plain(format!(
                        "no targets found beneath '{directory}'"
                    )));
                    return selected;
                }
                // A package that fails costs the pattern its error, not the
                // targets of the others.
                for package in packages {
                    match self.wildcard(repo, &package, *rules_only, Purpose::Tree) {
                        Ok(labels) => selected.targets.extend(labels),
                        Err(failed) => selected.fail(failed),
                    }
                }
            }
        }
        selected
    }
}

/// Why one package could not be selected from, and what it did define.
struct Failed {
    message: String,
    partial: Vec<Label>,
    defined: bool,
}

impl Failed {
    fn plain(message: String) -> Failed {
        Failed {
            message,
            partial: Vec::new(),
            defined: false,
        }
    }
}

/// What one pattern selected: the targets it got, those of packages with
/// errors, and the first thing that went wrong.
#[derive(Default)]
struct Selected {
    targets: Vec<Label>,
    in_error: Vec<Label>,
    failure: Option<String>,
    defined: bool,
}

impl Selected {
    fn fail(&mut self, failed: Failed) {
        self.in_error.extend(failed.partial);
        if self.failure.is_none() {
            self.failure = Some(failed.message);
            self.defined = failed.defined;
        }
    }
}

/// What Bazel says of a pattern whose package did not load: one whose BUILD
/// file has errors was evaluated for it. One that could not be read says
/// nothing of the pattern for a label, was being parsed for `:all` and `:*`,
/// and is an error under the directory for `...`. Probed on 9.2.0.
fn in_pattern(pattern: &TargetPattern, message: String) -> String {
    let text = &pattern.text;
    if !message.starts_with("error loading package '") {
        message
    } else if message.ends_with("' contains errors") {
        format!("Error evaluating '{text}': {message}")
    } else {
        match &pattern.pattern {
            Pattern::Target(_) | Pattern::Path { .. } => message,
            Pattern::InPackage { .. } => format!("while parsing '{text}': {message}"),
            Pattern::Below { directory, .. } => {
                format!("error loading package under directory '{directory}': {message}")
            }
        }
    }
}

/// A `test_suite` the build could not expand, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteIssue {
    /// The suite that failed: where it is declared is in `location`.
    pub suite: Label,
    /// `file:line:col` of the suite in its package, as targets have it.
    pub location: String,
    pub error: fjfj_graph::suite::SuiteError,
}

/// The suites of a source, as [`fjfj_graph::suite`] reads them.
struct SourceSuites<'a> {
    source: &'a dyn PackageSource,
}

impl SourceSuites<'_> {
    fn strings(attrs: &[(String, AttrValue)], name: &str) -> Vec<String> {
        match attrs.iter().find(|(n, _)| n == name) {
            Some((_, AttrValue::StringList(items))) => items.clone(),
            _ => Vec::new(),
        }
    }

    fn test_of(attrs: &[(String, AttrValue)]) -> fjfj_graph::suite::SuiteTest {
        let size = match attrs.iter().find(|(n, _)| n == "size") {
            Some((_, AttrValue::String(size))) if !size.is_empty() => size.clone(),
            _ => "medium".to_owned(),
        };
        fjfj_graph::suite::SuiteTest {
            tags: Self::strings(attrs, "tags"),
            size,
        }
    }
}

impl fjfj_graph::suite::Suites for SourceSuites<'_> {
    fn member(&self, label: &Label) -> Result<fjfj_graph::suite::Member, String> {
        use fjfj_graph::suite::Member;
        let Ok(package) = self.source.package(&label.repo, &label.package) else {
            return Ok(Member::Other);
        };
        let Some(Target {
            kind: TargetKind::Rule {
                rule_class, attrs, ..
            },
            ..
        }) = package.target(&label.name)
        else {
            return Ok(Member::Other);
        };
        Ok(if rule_class == "test_suite" {
            Member::Suite {
                tests: match attrs.iter().find(|(n, _)| n == "tests") {
                    Some((_, AttrValue::LabelList(labels))) => labels.clone(),
                    _ => Vec::new(),
                },
                tags: Self::strings(attrs, "tags"),
            }
        } else if rule_class.ends_with("_test") {
            Member::Test(Self::test_of(attrs))
        } else {
            Member::Other
        })
    }

    fn package_tests(
        &self,
        label: &Label,
    ) -> Result<Vec<(Label, fjfj_graph::suite::SuiteTest)>, String> {
        let package = self.source.package(&label.repo, &label.package)?;
        Ok(package
            .targets()
            .iter()
            .filter_map(|target| match &target.kind {
                TargetKind::Rule {
                    rule_class, attrs, ..
                } if rule_class.ends_with("_test") => {
                    Some((package.label(target), Self::test_of(attrs)))
                }
                _ => None,
            })
            .collect())
    }
}

/// `--expand_test_suites`: each `test_suite` in `resolved.targets` becomes the
/// tests it stands for, each once, where the suite was. A suite that cannot be
/// expanded is dropped and returned with why.
pub fn expand_test_suites(resolved: &mut Resolved, source: &dyn PackageSource) -> Vec<SuiteIssue> {
    let suites = SourceSuites { source };
    let mut issues = Vec::new();
    let mut out: Vec<Label> = Vec::new();
    let is_suite = |label: &Label| {
        source
            .package(&label.repo, &label.package)
            .ok()
            .and_then(|p| {
                p.target(&label.name).map(|t| {
                    matches!(&t.kind, TargetKind::Rule { rule_class, .. } if rule_class == "test_suite")
                })
            })
            .unwrap_or(false)
    };
    for label in std::mem::take(&mut resolved.targets) {
        if !is_suite(&label) {
            if !out.contains(&label) {
                out.push(label);
            }
            continue;
        }
        match fjfj_graph::suite::expand(&suites, &label, true) {
            Ok(tests) => {
                for test in tests {
                    if !out.contains(&test) {
                        out.push(test);
                    }
                }
            }
            Err(error) => {
                let location = source
                    .package(&label.repo, &label.package)
                    .ok()
                    .and_then(|p| p.target(&label.name).map(|t| t.location.clone()))
                    .unwrap_or_default();
                issues.push(SuiteIssue {
                    suite: label,
                    location,
                    error,
                });
            }
        }
    }
    resolved.targets = out;
    issues
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
    let mut explicit: std::collections::BTreeSet<Label> = std::collections::BTreeSet::new();
    let mut failures = Vec::new();
    let mut in_error: Vec<Label> = Vec::new();
    for pattern in patterns {
        let chosen = resolver.select(&pattern.pattern);
        let labels = chosen.targets;
        if pattern.negative {
            removed.extend(labels);
            removed.extend(chosen.in_error);
        } else {
            if matches!(pattern.pattern, Pattern::Target(_) | Pattern::Path { .. }) {
                explicit.extend(labels.iter().cloned());
            }
            for label in labels {
                if !selected.contains(&label) {
                    selected.push(label);
                }
            }
            for label in chosen.in_error {
                if !in_error.contains(&label) {
                    in_error.push(label);
                }
            }
        }
        if let Some(message) = chosen.failure {
            let package_error = message
                .ends_with("' contains errors")
                .then(|| message.clone());
            failures.push(Failure {
                pattern: pattern.text.clone(),
                message: in_pattern(pattern, message),
                package_error,
                tree: matches!(pattern.pattern, Pattern::Below { .. }),
                defined: chosen.defined,
            });
        }
    }
    selected.retain(|l| !removed.contains(l));
    in_error.retain(|l| !removed.contains(l));
    explicit.retain(|l| selected.contains(l) || in_error.contains(l));
    Resolved {
        targets: selected,
        explicit,
        failures,
        in_error,
    }
}
