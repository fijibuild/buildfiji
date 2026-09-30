//! A loaded package: its targets, their visibility, and the rules a BUILD
//! file must follow while declaring them (buildfiji-mum.5).
//!
//! Pure data. What is on disk (which directories are packages) reaches this
//! module as a predicate, so `fjfj-graph` stays free of I/O. The Starlark
//! bindings that call [`PackageBuilder`] are in `fjfj-starlark`
//! (buildfiji-mum.4).

use crate::Label;
use crate::label::{self, LabelError};
use crate::rule::AttrValue;
use crate::visibility::{PackageGroup, Visibility};
use std::collections::BTreeMap;

/// What a target is.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TargetKind {
    Rule {
        rule_class: String,
        /// The attributes the BUILD file set, in the order it wrote them,
        /// `name` and `visibility` excluded. What it did not set is the rule
        /// class's default, which the class knows.
        attrs: Vec<(String, AttrValue)>,
    },
    /// A source file exported with `exports_files`. Other files in the
    /// package are not targets.
    SourceFile,
    PackageGroup(PackageGroup),
    /// A file a rule declares it creates: an `attr.output`, or one of the
    /// `outputs` of a `rule()`.
    GeneratedFile {
        rule: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Target {
    pub name: String,
    pub kind: TargetKind,
    /// The `visibility` attribute as written; `None` takes the package's
    /// `default_visibility`.
    pub visibility: Option<Visibility>,
    /// Where the BUILD file declared it, as `file:line:col`, for errors.
    pub location: String,
}

impl Target {
    /// How Bazel names this target in a conflict message.
    fn describe(&self) -> String {
        match &self.kind {
            TargetKind::Rule { rule_class, .. } => format!("{rule_class} rule"),
            TargetKind::SourceFile => "source file".to_owned(),
            TargetKind::PackageGroup(_) => "package group".to_owned(),
            TargetKind::GeneratedFile { rule } => format!("generated file from rule '{rule}'"),
        }
    }
}

/// What `package()` sets besides `default_visibility`. Rules do not see
/// these until analysis: `native.existing_rule` shows only what a rule set
/// itself.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct PackageDefaults {
    pub testonly: bool,
    pub deprecation: Option<String>,
    pub compatible_with: Vec<Label>,
    pub restricted_to: Vec<Label>,
    pub features: Vec<String>,
    pub hdrs_check: Option<String>,
    pub licenses: Vec<String>,
    /// `default_package_metadata`, or the deprecated
    /// `default_applicable_licenses`; giving both is an error.
    pub package_metadata: Vec<Label>,
}

/// Every argument of one `package()` call.
#[derive(Debug, Clone, Default)]
pub struct PackageSettings {
    pub default_visibility: Option<Visibility>,
    pub defaults: PackageDefaults,
}

/// A package, once its BUILD file has been evaluated.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub repo: String,
    pub name: String,
    /// From `package(default_visibility = ...)`; private if never given.
    pub default_visibility: Visibility,
    /// The other `package()` settings; empty if it was never called.
    pub defaults: PackageDefaults,
    targets: Vec<Target>,
    index: BTreeMap<String, usize>,
}

impl Package {
    /// Targets in declaration order.
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    pub fn target(&self, name: &str) -> Option<&Target> {
        self.index.get(name).map(|&i| &self.targets[i])
    }

    pub fn label(&self, target: &Target) -> Label {
        Label {
            repo: self.repo.clone(),
            package: self.name.clone(),
            name: target.name.clone(),
        }
    }

    /// The visibility that applies to `target`: its own, else the package's
    /// default, however late in the file `package()` came.
    pub fn visibility_of<'a>(&'a self, target: &'a Target) -> &'a Visibility {
        target
            .visibility
            .as_ref()
            .unwrap_or(&self.default_visibility)
    }
}

/// Everything a BUILD file can do wrong to a package, in Bazel's words.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PackageError {
    #[error("illegal rule name: {name}: invalid target name '{name}': {source}")]
    IllegalRuleName { name: String, source: LabelError },
    /// `exports_files` reports the bare validation error.
    #[error("{0}")]
    IllegalFileName(LabelError),
    #[error("{0}")]
    Subpackage(Box<SubpackageError>),
    #[error("{what} conflicts with existing {existing}, defined at {location}")]
    Conflict {
        what: String,
        existing: String,
        location: String,
    },
    #[error(
        "illegal output file name '{name}' in rule {rule} due to: invalid target name '{name}': {source}"
    )]
    IllegalOutputName {
        name: String,
        rule: String,
        source: LabelError,
    },
    #[error("rule '{rule}' has more than one generated file named '{name}'")]
    DuplicateOutput { rule: String, name: String },
    #[error("'package' can only be used once per BUILD file")]
    PackageCalledTwice,
    #[error("visibility for exported file '{0}' declared twice")]
    VisibilityDeclaredTwice(String),
}

/// A label inside a package that reaches into one of its subpackages.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "Label '{label}' is invalid because '{subpackage}' is a subpackage; \
     perhaps you meant to put the colon here: '{suggestion}'?"
)]
pub struct SubpackageError {
    pub label: Label,
    /// The nearest enclosing package inside this one, e.g. `a/sub`.
    pub subpackage: String,
    pub suggestion: Label,
}

/// A target's name may not walk into a subpackage: `//a:sub/f.txt` is an
/// error when `a/sub` is a package. Bazel reports the deepest such package.
///
/// Only names inside the package being loaded are checked. `//a:sub/f.txt`
/// written in package `b` loads fine and fails later, as a missing target.
/// `is_package` answers for a package name in this repo.
pub fn check_subpackage_crossing(
    repo: &str,
    package: &str,
    name: &str,
    is_package: &dyn Fn(&str) -> bool,
) -> Result<(), Box<SubpackageError>> {
    let mut dir = name;
    while let Some((parent, _)) = dir.rsplit_once('/') {
        dir = parent;
        let candidate = if package.is_empty() {
            dir.to_owned()
        } else {
            format!("{package}/{dir}")
        };
        if is_package(&candidate) {
            let rest = &name[dir.len() + 1..];
            return Err(Box::new(SubpackageError {
                label: Label {
                    repo: repo.into(),
                    package: package.into(),
                    name: name.into(),
                },
                suggestion: Label {
                    repo: repo.into(),
                    package: candidate.clone(),
                    name: rest.into(),
                },
                subpackage: candidate,
            }));
        }
    }
    Ok(())
}

/// Accumulates the targets a BUILD file declares, enforcing what Bazel
/// enforces as each is added.
pub struct PackageBuilder<'a> {
    repo: String,
    name: String,
    is_package: &'a dyn Fn(&str) -> bool,
    package_called: bool,
    default_visibility: Visibility,
    defaults: PackageDefaults,
    targets: Vec<Target>,
    index: BTreeMap<String, usize>,
}

impl<'a> PackageBuilder<'a> {
    /// `is_package` says whether a package name (in this repo) has a BUILD
    /// file; it drives the subpackage-crossing check.
    pub fn new(
        repo: impl Into<String>,
        name: impl Into<String>,
        is_package: &'a dyn Fn(&str) -> bool,
    ) -> Self {
        PackageBuilder {
            repo: repo.into(),
            name: name.into(),
            is_package,
            package_called: false,
            default_visibility: Visibility::private(),
            defaults: PackageDefaults::default(),
            targets: Vec::new(),
            index: BTreeMap::new(),
        }
    }

    /// `package(...)`: allowed once, before or after any target.
    pub fn call_package(&mut self, settings: PackageSettings) -> Result<(), PackageError> {
        if self.package_called {
            return Err(PackageError::PackageCalledTwice);
        }
        self.package_called = true;
        if let Some(v) = settings.default_visibility {
            self.default_visibility = v;
        }
        self.defaults = settings.defaults;
        Ok(())
    }

    /// Declare a rule instance whose attributes the caller does not record.
    /// `location` is `file:line:col`.
    pub fn add_rule(
        &mut self,
        name: &str,
        rule_class: &str,
        visibility: Option<Visibility>,
        location: &str,
    ) -> Result<(), PackageError> {
        match self.add_rule_with(name, rule_class, Vec::new(), visibility, location)? {
            None => Ok(()),
            Some(crossing) => Err(crossing),
        }
    }

    /// [`add_rule`](Self::add_rule), recording the attributes the BUILD file
    /// set (see [`TargetKind::Rule`]). A name that enters a subpackage does
    /// not stop the rule being added, as it does not in Bazel: it comes back
    /// as `Ok(Some(error))`, for the caller to report.
    pub fn add_rule_with(
        &mut self,
        name: &str,
        rule_class: &str,
        attrs: Vec<(String, AttrValue)>,
        visibility: Option<Visibility>,
        location: &str,
    ) -> Result<Option<PackageError>, PackageError> {
        label::validate_target_name(name).map_err(|source| PackageError::IllegalRuleName {
            name: name.to_owned(),
            source,
        })?;
        let crossing = self.check_crossing(name).err();
        self.check_conflict(name, &format!("{rule_class} rule '{name}'"))?;
        self.push(Target {
            name: name.to_owned(),
            kind: TargetKind::Rule {
                rule_class: rule_class.to_owned(),
                attrs,
            },
            visibility,
            location: location.to_owned(),
        });
        Ok(crossing)
    }

    /// A file the rule called `rule` (whose label is `rule_label`, for
    /// the error) creates. A file that is the rule's own name is not another
    /// target, and one created twice by the rule is an error.
    pub fn add_generated_file(
        &mut self,
        name: &str,
        rule: &str,
        rule_label: &str,
        location: &str,
    ) -> Result<(), PackageError> {
        label::validate_target_name(name).map_err(|source| PackageError::IllegalOutputName {
            name: name.to_owned(),
            rule: rule_label.to_owned(),
            source,
        })?;
        if name == rule {
            return Ok(());
        }
        if let Some(&i) = self.index.get(name)
            && let TargetKind::GeneratedFile { rule: made_by } = &self.targets[i].kind
            && made_by == rule
        {
            return Err(PackageError::DuplicateOutput {
                rule: rule.to_owned(),
                name: name.to_owned(),
            });
        }
        self.check_conflict(name, &format!("generated file '{name}' in rule '{rule}'"))?;
        self.push(Target {
            name: name.to_owned(),
            kind: TargetKind::GeneratedFile {
                rule: rule.to_owned(),
            },
            visibility: None,
            location: location.to_owned(),
        });
        Ok(())
    }

    /// `package_group(name, packages, includes)`.
    pub fn add_package_group(
        &mut self,
        name: &str,
        group: PackageGroup,
        location: &str,
    ) -> Result<(), PackageError> {
        label::validate_target_name(name).map_err(|source| PackageError::IllegalRuleName {
            name: name.to_owned(),
            source,
        })?;
        self.check_crossing(name)?;
        self.check_conflict(name, &format!("package group '{name}'"))?;
        self.push(Target {
            name: name.to_owned(),
            kind: TargetKind::PackageGroup(group),
            visibility: None,
            location: location.to_owned(),
        });
        Ok(())
    }

    /// `exports_files([name], visibility = ...)`. With no `visibility` the
    /// file is public; giving one twice, or after an export without one,
    /// is an error, and exporting a file again with none is not.
    pub fn export_file(
        &mut self,
        name: &str,
        visibility: Option<Visibility>,
        location: &str,
    ) -> Result<(), PackageError> {
        label::validate_target_name(name).map_err(PackageError::IllegalFileName)?;
        self.check_crossing(name)?;
        if let Some(&i) = self.index.get(name) {
            let existing = &self.targets[i];
            if existing.kind == TargetKind::SourceFile {
                return match visibility {
                    None => Ok(()),
                    Some(_) => Err(PackageError::VisibilityDeclaredTwice(name.to_owned())),
                };
            }
        }
        self.check_conflict(name, &format!("source file '{name}'"))?;
        self.push(Target {
            name: name.to_owned(),
            kind: TargetKind::SourceFile,
            visibility: Some(visibility.unwrap_or_else(Visibility::public)),
            location: location.to_owned(),
        });
        Ok(())
    }

    /// Whether a target of any kind is called `name`.
    pub fn has_target(&self, name: &str) -> bool {
        self.index.contains_key(name)
    }

    /// The rule called `name`, if one has been declared. Exported files and
    /// package groups are targets but not rules.
    pub fn rule(&self, name: &str) -> Option<&Target> {
        self.index
            .get(name)
            .map(|&i| &self.targets[i])
            .filter(|t| matches!(t.kind, TargetKind::Rule { .. }))
    }

    /// The rules declared so far, in declaration order.
    pub fn rules(&self) -> impl Iterator<Item = &Target> {
        self.targets
            .iter()
            .filter(|t| matches!(t.kind, TargetKind::Rule { .. }))
    }

    pub fn build(self) -> Package {
        Package {
            repo: self.repo,
            name: self.name,
            default_visibility: self.default_visibility,
            defaults: self.defaults,
            targets: self.targets,
            index: self.index,
        }
    }

    fn check_crossing(&self, name: &str) -> Result<(), PackageError> {
        check_subpackage_crossing(&self.repo, &self.name, name, self.is_package)
            .map_err(PackageError::Subpackage)
    }

    fn check_conflict(&self, name: &str, what: &str) -> Result<(), PackageError> {
        match self.index.get(name).map(|&i| &self.targets[i]) {
            Some(existing) => Err(PackageError::Conflict {
                what: what.to_owned(),
                existing: existing.describe(),
                location: existing.location.clone(),
            }),
            None => Ok(()),
        }
    }

    fn push(&mut self, target: Target) {
        self.index.insert(target.name.clone(), self.targets.len());
        self.targets.push(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packages(known: &'static [&'static str]) -> impl Fn(&str) -> bool {
        move |p| known.contains(&p)
    }

    fn message(r: Result<(), PackageError>) -> String {
        r.unwrap_err().to_string()
    }

    // Every message below is what Bazel 9.2.0 printed for the same BUILD file.

    #[test]
    fn a_name_that_enters_a_subpackage_is_an_error() {
        let is_pkg = packages(&["a/sub"]);
        let mut b = PackageBuilder::new("", "a", &is_pkg);
        assert_eq!(
            message(b.add_rule("sub/f.txt", "filegroup", None, "a/BUILD:1:10")),
            "Label '//a:sub/f.txt' is invalid because 'a/sub' is a subpackage; \
             perhaps you meant to put the colon here: '//a/sub:f.txt'?"
        );
        assert_eq!(
            message(b.add_rule("sub/x", "filegroup", None, "a/BUILD:1:10")),
            "Label '//a:sub/x' is invalid because 'a/sub' is a subpackage; \
             perhaps you meant to put the colon here: '//a/sub:x'?"
        );
        // A sibling that merely shares a prefix is not a subpackage.
        b.add_rule("subway", "filegroup", None, "a/BUILD:2:10")
            .unwrap();
        b.add_rule("s/f", "filegroup", None, "a/BUILD:3:10")
            .unwrap();
    }

    #[test]
    fn the_deepest_enclosing_subpackage_is_the_one_reported() {
        let none = |_: &str| false;
        assert_eq!(check_subpackage_crossing("", "a", "sub/x/f", &none), Ok(()));
        let is_pkg = packages(&["a/sub", "a/sub/x"]);
        let err = check_subpackage_crossing("", "a", "sub/x/f.txt", &is_pkg).unwrap_err();
        assert_eq!(err.subpackage, "a/sub/x");
        assert_eq!(
            err.to_string(),
            "Label '//a:sub/x/f.txt' is invalid because 'a/sub/x' is a subpackage; \
             perhaps you meant to put the colon here: '//a/sub/x:f.txt'?"
        );
    }

    #[test]
    fn the_root_package_has_subpackages_too() {
        let is_pkg = packages(&["x"]);
        let err = check_subpackage_crossing("", "", "x/f.txt", &is_pkg).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Label '//:x/f.txt' is invalid because 'x' is a subpackage; \
             perhaps you meant to put the colon here: '//x:f.txt'?"
        );
    }

    #[test]
    fn illegal_names() {
        let is_pkg = packages(&[]);
        let mut b = PackageBuilder::new("", "a", &is_pkg);
        assert_eq!(
            message(b.add_rule("../x", "filegroup", None, "l")),
            "illegal rule name: ../x: invalid target name '../x': \
             target names may not contain up-level references '..'"
        );
        // A space is legal in a target name.
        b.add_rule("a b", "filegroup", None, "l").unwrap();
        assert_eq!(
            message(b.export_file("/x", None, "l")),
            "target names may not start with '/'"
        );
    }

    #[test]
    fn declaring_a_name_twice() {
        let is_pkg = packages(&[]);
        let mut b = PackageBuilder::new("", "a", &is_pkg);
        b.add_rule("x", "filegroup", None, "a/BUILD:1:10").unwrap();
        assert_eq!(
            message(b.add_rule("x", "filegroup", None, "a/BUILD:2:10")),
            "filegroup rule 'x' conflicts with existing filegroup rule, defined at a/BUILD:1:10"
        );
        assert_eq!(
            message(b.export_file("x", None, "a/BUILD:2:14")),
            "source file 'x' conflicts with existing filegroup rule, defined at a/BUILD:1:10"
        );
        assert_eq!(
            message(b.add_package_group("x", PackageGroup::default(), "a/BUILD:3:10")),
            "package group 'x' conflicts with existing filegroup rule, defined at a/BUILD:1:10"
        );
        b.export_file("f", None, "a/BUILD:4:14").unwrap();
        assert_eq!(
            message(b.add_rule("f", "filegroup", None, "a/BUILD:5:10")),
            "filegroup rule 'f' conflicts with existing source file, defined at a/BUILD:4:14"
        );
        b.add_package_group("g", PackageGroup::default(), "a/BUILD:6:14")
            .unwrap();
        assert_eq!(
            message(b.add_package_group("g", PackageGroup::default(), "a/BUILD:7:14")),
            "package group 'g' conflicts with existing package group, defined at a/BUILD:6:14"
        );
    }

    #[test]
    fn exports_files_twice() {
        let is_pkg = packages(&[]);
        let mut b = PackageBuilder::new("", "a", &is_pkg);
        b.export_file("f", None, "l").unwrap();
        b.export_file("f", None, "l").unwrap();
        let private = Some(Visibility::private());
        assert_eq!(
            message(b.export_file("f", private.clone(), "l")),
            "visibility for exported file 'f' declared twice"
        );
        b.export_file("g", private.clone(), "l").unwrap();
        assert_eq!(
            message(b.export_file("g", private, "l")),
            "visibility for exported file 'g' declared twice"
        );
    }

    #[test]
    fn package_may_be_called_once_and_at_any_point() {
        let is_pkg = packages(&[]);
        let mut b = PackageBuilder::new("", "a", &is_pkg);
        b.add_rule("before", "filegroup", None, "l").unwrap();
        b.call_package(PackageSettings {
            default_visibility: Some(Visibility::public()),
            ..PackageSettings::default()
        })
        .unwrap();
        b.add_rule("after", "filegroup", None, "l").unwrap();
        assert_eq!(
            message(b.call_package(PackageSettings::default())),
            "'package' can only be used once per BUILD file"
        );
        let p = b.build();
        // `package()` came after `before`, and still covers it.
        for name in ["before", "after"] {
            let t = p.target(name).unwrap();
            assert_eq!(p.visibility_of(t), &Visibility::public(), "{name}");
        }
    }

    #[test]
    fn visibility_defaults_to_private_and_targets_may_override() {
        let is_pkg = packages(&[]);
        let mut b = PackageBuilder::new("r", "a", &is_pkg);
        b.add_rule("x", "filegroup", None, "l").unwrap();
        b.add_rule("y", "filegroup", Some(Visibility::public()), "l")
            .unwrap();
        b.export_file("f", None, "l").unwrap();
        let p = b.build();
        assert_eq!(
            p.visibility_of(p.target("x").unwrap()),
            &Visibility::private()
        );
        assert_eq!(
            p.visibility_of(p.target("y").unwrap()),
            &Visibility::public()
        );
        // An exported file with no `visibility` is public.
        assert_eq!(
            p.visibility_of(p.target("f").unwrap()),
            &Visibility::public()
        );
        assert_eq!(p.label(p.target("x").unwrap()).to_string(), "@r//a:x");
        let order: Vec<_> = p.targets().iter().map(|t| t.name.as_str()).collect();
        assert_eq!(order, ["x", "y", "f"]);
        assert!(p.target("nope").is_none());
    }
}
