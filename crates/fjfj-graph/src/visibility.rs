//! Target visibility: package specifications, `package_group` contents and
//! the check itself (buildfiji-mum.5).
//!
//! Semantics were established against Bazel 9.2.0:
//!
//! - a target is always visible to its own package;
//! - a target with no `visibility` takes the package's `default_visibility`,
//!   wherever `package()` sits in the file, and a package with neither is
//!   private;
//! - `//visibility:private` grants nothing and may sit beside other entries;
//! - a `package_group` contributes the specs of its own `packages` and,
//!   transitively, of its `includes`;
//! - a `-` spec denies, wherever it is written: order does not matter, and
//!   one in an included group denies for the including group too.

use crate::Label;
use crate::parse::{LabelContext, LabelParseError};
use std::collections::HashSet;

/// The set of packages one specification matches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PackageScope {
    /// `public` or `//visibility:public`: every package of every repo.
    Public,
    /// `//...`: every package of one repo.
    Repo(String),
    /// `//p` or `//p:__pkg__`: exactly one package.
    Package { repo: String, package: String },
    /// `//p/...` or `//p:__subpackages__`: a package and everything below it.
    Subpackages { repo: String, package: String },
}

impl PackageScope {
    pub fn contains(&self, repo: &str, package: &str) -> bool {
        match self {
            PackageScope::Public => true,
            PackageScope::Repo(r) => r == repo,
            PackageScope::Package {
                repo: r,
                package: p,
            } => r == repo && p == package,
            PackageScope::Subpackages {
                repo: r,
                package: p,
            } => {
                r == repo
                    && (p.is_empty()
                        || package == p
                        || package
                            .strip_prefix(p.as_str())
                            .is_some_and(|rest| rest.starts_with('/')))
            }
        }
    }
}

/// One entry of a `package_group`'s `packages`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PackageSpec {
    /// A leading `-`: packages this spec matches are excluded.
    pub negated: bool,
    pub scope: PackageScope,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpecError {
    #[error("invalid package name '{0}': must start with '//', '@', or be 'public' or 'private'")]
    NotAPackageSpec(String),
    #[error("invalid package name '{0}'")]
    Malformed(String),
    /// `//p:q`: a target, where a package was wanted. Bazel builds
    /// `//p:q:__pkg__` from it and reports that name.
    #[error(
        "invalid package name '{spec}': invalid target name '{name}:__pkg__': \
         target names may not contain ':'"
    )]
    NamedTarget { spec: String, name: String },
    #[error(transparent)]
    Label(#[from] LabelParseError),
}

impl PackageSpec {
    /// Parse one string of a `package_group`'s `packages`, written in repo
    /// `repo`. `private` matches nothing and yields `None`.
    pub fn parse(spec: &str, repo: &str) -> Result<Option<PackageSpec>, SpecError> {
        let (negated, body) = match spec.strip_prefix('-') {
            Some(body) => (true, body),
            None => (false, spec),
        };
        let scope = match body {
            "public" => PackageScope::Public,
            "private" => return Ok(None),
            _ if body.starts_with("//") || body.starts_with('@') => {
                parse_scope(body, repo).ok_or_else(|| SpecError::Malformed(spec.to_owned()))??
            }
            _ => return Err(SpecError::NotAPackageSpec(spec.to_owned())),
        };
        Ok(Some(PackageSpec { negated, scope }))
    }
}

/// `//...`, `//p/...`, `//p`, `//p:__pkg__`, `//p:__subpackages__`, each
/// optionally behind `@repo`. `None` is a label that is not one of those.
fn parse_scope(body: &str, repo: &str) -> Option<Result<PackageScope, SpecError>> {
    let ctx = LabelContext { repo, package: "" };
    let parse = |s: &str| Label::parse(s, ctx).map_err(SpecError::from);
    // `//` and `@r//` are the root package of a repo.
    if body.ends_with("//") {
        return Some(parse(&format!("{body}:x")).map(|l| PackageScope::Package {
            repo: l.repo,
            package: String::new(),
        }));
    }
    // `//...` and `//p/...` are not labels: peel the wildcard off, then read
    // what is left as the package it roots.
    if let Some(prefix) = body.strip_suffix("...") {
        let root = if prefix.ends_with("//") {
            prefix
        } else {
            prefix.strip_suffix('/')?
        };
        return Some(parse(&format!("{root}:x")).map(|l| {
            if l.package.is_empty() {
                PackageScope::Repo(l.repo)
            } else {
                PackageScope::Subpackages {
                    repo: l.repo,
                    package: l.package,
                }
            }
        }));
    }
    let named_a_target = body.contains(':');
    // `//p:q:r` never gets as far as being a label: Bazel has already
    // appended `:__pkg__` and found a `:` in the name.
    if let Some((_, name)) = body.split_once(':')
        && name.contains(':')
    {
        return Some(Err(SpecError::NamedTarget {
            spec: body.to_owned(),
            name: name.to_owned(),
        }));
    }
    Some(parse(body).and_then(|l| {
        match l.name.as_str() {
            "__subpackages__" => Ok(PackageScope::Subpackages {
                repo: l.repo,
                package: l.package,
            }),
            // `//p` is `//p:p`, and `//p:__pkg__` names the same package.
            "__pkg__" => Ok(PackageScope::Package {
                repo: l.repo,
                package: l.package,
            }),
            _ if !named_a_target => Ok(PackageScope::Package {
                repo: l.repo,
                package: l.package,
            }),
            _ => Err(SpecError::NamedTarget {
                spec: body.to_owned(),
                name: body
                    .split_once(':')
                    .map_or_else(String::new, |(_, name)| name.to_owned()),
            }),
        }
    }))
}

/// The contents of a `package_group` target.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct PackageGroup {
    pub specs: Vec<PackageSpec>,
    /// Other `package_group` targets whose specs this one includes.
    pub includes: Vec<Label>,
}

/// One entry of a `visibility` attribute.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VisibilityEntry {
    Scope(PackageScope),
    /// A `package_group` target.
    Group(Label),
}

/// A `visibility` attribute. Empty means private.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, serde::Serialize, serde::Deserialize)]
pub struct Visibility {
    pub entries: Vec<VisibilityEntry>,
}

impl Visibility {
    pub fn private() -> Visibility {
        Visibility::default()
    }

    pub fn public() -> Visibility {
        Visibility {
            entries: vec![VisibilityEntry::Scope(PackageScope::Public)],
        }
    }

    /// Parse a `visibility` attribute written in `ctx`'s package.
    pub fn parse<'s>(
        items: impl IntoIterator<Item = &'s str>,
        ctx: LabelContext<'_>,
    ) -> Result<Visibility, LabelParseError> {
        let mut entries = Vec::new();
        for item in items {
            let label = Label::parse(item, ctx)?;
            let scope = match (label.package.as_str(), label.name.as_str()) {
                ("visibility", "public") => PackageScope::Public,
                ("visibility", "private") => continue,
                (_, "__pkg__") => PackageScope::Package {
                    repo: label.repo,
                    package: label.package,
                },
                (_, "__subpackages__") => PackageScope::Subpackages {
                    repo: label.repo,
                    package: label.package,
                },
                _ => {
                    entries.push(VisibilityEntry::Group(label));
                    continue;
                }
            };
            entries.push(VisibilityEntry::Scope(scope));
        }
        Ok(Visibility { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VisibilityError {
    #[error("no such target '{0}': not a package_group")]
    NoSuchGroup(Label),
}

/// Is a target declared in `(declared_repo, declared_package)` with
/// visibility `visibility` visible from package `(repo, package)`?
///
/// `group` resolves a `package_group` label to its contents. Include cycles
/// are cut at the second visit rather than reported.
pub fn is_visible<'g>(
    visibility: &Visibility,
    declared: (&str, &str),
    from: (&str, &str),
    group: &dyn Fn(&Label) -> Option<&'g PackageGroup>,
) -> Result<bool, VisibilityError> {
    if declared == from {
        return Ok(true);
    }
    let mut specs: Vec<&PackageSpec> = Vec::new();
    let mut positive_scopes: Vec<&PackageScope> = Vec::new();
    let mut seen: HashSet<&Label> = HashSet::new();
    let mut pending: Vec<&Label> = Vec::new();
    for entry in &visibility.entries {
        match entry {
            VisibilityEntry::Scope(scope) => positive_scopes.push(scope),
            VisibilityEntry::Group(label) => pending.push(label),
        }
    }
    while let Some(label) = pending.pop() {
        if !seen.insert(label) {
            continue;
        }
        let g = group(label).ok_or_else(|| VisibilityError::NoSuchGroup(label.clone()))?;
        specs.extend(&g.specs);
        pending.extend(&g.includes);
    }
    let (repo, package) = from;
    let denied = specs
        .iter()
        .any(|s| s.negated && s.scope.contains(repo, package));
    let allowed = positive_scopes.iter().any(|s| s.contains(repo, package))
        || specs
            .iter()
            .any(|s| !s.negated && s.scope.contains(repo, package));
    Ok(allowed && !denied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const CTX: LabelContext<'static> = LabelContext {
        repo: "",
        package: "a",
    };

    fn spec(s: &str) -> Option<PackageSpec> {
        PackageSpec::parse(s, "").unwrap()
    }

    fn scope(s: &str) -> PackageScope {
        spec(s).unwrap().scope
    }

    fn pkg(p: &str) -> PackageScope {
        PackageScope::Package {
            repo: "".into(),
            package: p.into(),
        }
    }

    fn sub(p: &str) -> PackageScope {
        PackageScope::Subpackages {
            repo: "".into(),
            package: p.into(),
        }
    }

    #[test]
    fn package_specs() {
        assert_eq!(scope("public"), PackageScope::Public);
        assert_eq!(scope("//..."), PackageScope::Repo("".into()));
        assert_eq!(scope("@//..."), PackageScope::Repo("".into()));
        assert_eq!(scope("@r//..."), PackageScope::Repo("r".into()));
        assert_eq!(scope("//c"), pkg("c"));
        // The root package, probed in a package_group and in load visibility.
        assert_eq!(scope("//"), pkg(""));
        assert_eq!(scope("@//"), pkg(""));
        assert_eq!(
            scope("@r//"),
            PackageScope::Package {
                repo: "r".into(),
                package: "".into()
            }
        );
        assert_eq!(scope("//c/n"), pkg("c/n"));
        assert_eq!(scope("//c:__pkg__"), pkg("c"));
        assert_eq!(scope("//c/..."), sub("c"));
        assert_eq!(scope("//c:__subpackages__"), sub("c"));
        assert!(spec("-//c/n").unwrap().negated);
        // `private` grants and denies nothing.
        assert_eq!(spec("private"), None);
        assert_eq!(
            PackageSpec::parse("c", "").unwrap_err().to_string(),
            "invalid package name 'c': must start with '//', '@', or be 'public' or 'private'"
        );
    }

    /// Bazel reads `//p:q` as `//p:q:__pkg__` and complains about that name.
    #[test]
    fn a_target_is_not_a_package_spec() {
        for (input, name) in [("//a:b", "b"), ("//a:b:c", "b:c"), ("-//a:b", "b")] {
            let shown = input.trim_start_matches('-');
            assert_eq!(
                PackageSpec::parse(input, "").unwrap_err().to_string(),
                format!(
                    "invalid package name '{shown}': invalid target name '{name}:__pkg__': \
                     target names may not contain ':'"
                ),
                "{input}"
            );
        }
    }

    #[test]
    fn a_scope_inside_another_repo_defaults_to_that_repo() {
        assert_eq!(
            PackageSpec::parse("//c/...", "dep+")
                .unwrap()
                .unwrap()
                .scope,
            PackageScope::Subpackages {
                repo: "dep+".into(),
                package: "c".into()
            }
        );
    }

    #[test]
    fn subpackages_match_on_path_boundaries() {
        let s = sub("c");
        assert!(s.contains("", "c"));
        assert!(s.contains("", "c/n"));
        assert!(s.contains("", "c/n/m"));
        assert!(!s.contains("", "cx"));
        assert!(!s.contains("", "d"));
        assert!(!s.contains("r", "c"));
        assert!(sub("").contains("", "anything/at/all"));
    }

    #[test]
    fn visibility_attribute_entries() {
        let v = Visibility::parse(
            [
                "//visibility:public",
                "//visibility:private",
                "//b:__pkg__",
                "//c:__subpackages__",
                ":g",
                "//x:g",
            ],
            CTX,
        )
        .unwrap();
        let group = |p: &str, n: &str| {
            VisibilityEntry::Group(Label {
                repo: "".into(),
                package: p.into(),
                name: n.into(),
            })
        };
        assert_eq!(
            v.entries,
            vec![
                VisibilityEntry::Scope(PackageScope::Public),
                VisibilityEntry::Scope(pkg("b")),
                VisibilityEntry::Scope(sub("c")),
                group("a", "g"),
                group("x", "g"),
            ]
        );
    }

    /// The `package_group`s of a fixture, keyed by target name in package `a`.
    struct Groups(BTreeMap<&'static str, PackageGroup>);

    impl Groups {
        fn new(defs: &[(&'static str, &[&str], &[&str])]) -> Groups {
            Groups(
                defs.iter()
                    .map(|(name, packages, includes)| {
                        (
                            *name,
                            PackageGroup {
                                specs: packages
                                    .iter()
                                    .filter_map(|p| PackageSpec::parse(p, "").unwrap())
                                    .collect(),
                                includes: includes
                                    .iter()
                                    .map(|i| Label::parse(i, CTX).unwrap())
                                    .collect(),
                            },
                        )
                    })
                    .collect(),
            )
        }

        /// Who, out of `c`, `c/n` and `d`, can see a target in `a` that is
        /// visible to `:g`.
        fn seen_by(&self) -> [bool; 3] {
            let vis = Visibility::parse([":g"], CTX).unwrap();
            let lookup = |l: &Label| self.0.get(l.name.as_str());
            ["c", "c/n", "d"].map(|from| is_visible(&vis, ("", "a"), ("", from), &lookup).unwrap())
        }
    }

    // The next four are the vis3/negorder/neginc/negonly probes on 9.2.0.

    #[test]
    fn a_negated_spec_denies_wherever_it_is_written() {
        for packages in [["-//c/n", "//c/..."], ["//c/...", "-//c/n"]] {
            let groups = Groups::new(&[("g", &packages, &[])]);
            assert_eq!(groups.seen_by(), [true, false, false], "{packages:?}");
        }
    }

    #[test]
    fn a_negation_in_an_included_group_denies_for_the_includer() {
        let groups = Groups::new(&[("g", &["//c/..."], &[":h"]), ("h", &["-//c/n"], &[])]);
        assert_eq!(groups.seen_by(), [true, false, false]);
    }

    #[test]
    fn negations_alone_grant_nothing() {
        let groups = Groups::new(&[("g", &["-//c/n"], &[])]);
        assert_eq!(groups.seen_by(), [false, false, false]);
    }

    #[test]
    fn includes_are_followed_and_cycles_are_cut() {
        let groups = Groups::new(&[("g", &["//b/..."], &[":h"]), ("h", &["//c"], &[":g"])]);
        assert_eq!(groups.seen_by(), [true, false, false]);
    }

    #[test]
    fn public_and_private_specs() {
        assert_eq!(
            Groups::new(&[("g", &["public"], &[])]).seen_by(),
            [true, true, true]
        );
        assert_eq!(
            Groups::new(&[("g", &["private"], &[])]).seen_by(),
            [false, false, false]
        );
        assert_eq!(
            Groups::new(&[("g", &["//..."], &[])]).seen_by(),
            [true, true, true]
        );
        assert_eq!(
            Groups::new(&[("g", &[], &[])]).seen_by(),
            [false, false, false]
        );
        // `//c:__pkg__` is `//c`, not `//c/...`.
        assert_eq!(
            Groups::new(&[("g", &["//c:__pkg__"], &[])]).seen_by(),
            [true, false, false]
        );
    }

    #[test]
    fn a_target_is_visible_to_its_own_package_and_private_to_others() {
        let none = |_: &Label| None;
        let private = Visibility::private();
        assert!(is_visible(&private, ("", "a"), ("", "a"), &none).unwrap());
        assert!(!is_visible(&private, ("", "a"), ("", "b"), &none).unwrap());
        // Another repo's `a` is not this repo's `a`.
        assert!(!is_visible(&private, ("", "a"), ("r", "a"), &none).unwrap());
    }

    #[test]
    fn explicit_scopes() {
        let none = |_: &Label| None;
        let vis = Visibility::parse(
            ["//visibility:private", "//b:__pkg__", "//c:__subpackages__"],
            CTX,
        )
        .unwrap();
        let sees = |from: &str| is_visible(&vis, ("", "a"), ("", from), &none).unwrap();
        assert!(sees("b"));
        assert!(!sees("b/x"));
        assert!(sees("c"));
        assert!(sees("c/d"));
        assert!(!sees("e"));
    }

    #[test]
    fn a_missing_group_is_an_error() {
        let none = |_: &Label| None;
        let vis = Visibility::parse([":g"], CTX).unwrap();
        assert!(matches!(
            is_visible(&vis, ("", "a"), ("", "b"), &none),
            Err(VisibilityError::NoSuchGroup(_))
        ));
    }
}
