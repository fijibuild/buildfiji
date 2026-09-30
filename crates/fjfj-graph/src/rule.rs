//! Attribute values, and the attribute schema of the rules Bazel implements
//! natively that BUILD files call directly (buildfiji-mum.4).
//!
//! Only rules with no Starlark definition are here: `filegroup` and `alias`.
//! Everything else a BUILD file calls is a `rule()` from a `.bzl`, which
//! brings its own schema. The lists below were read off Bazel 9.2.0:
//! which attributes each rule accepts, their types, and what
//! `native.existing_rule` shows for one that was never set.

use crate::Label;

/// A value stored in a rule attribute.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttrValue {
    Bool(bool),
    String(String),
    Label(Label),
    StringList(Vec<String>),
    LabelList(Vec<Label>),
}

/// What an attribute holds, and so how a value is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    Bool,
    String,
    StringList,
    Label,
    LabelList,
}

impl AttrType {
    /// The name Bazel's type errors use.
    pub fn name(self) -> &'static str {
        match self {
            AttrType::Bool => "bool",
            AttrType::String => "string",
            AttrType::StringList => "list(string)",
            // A label is written as a string.
            AttrType::Label => "string",
            AttrType::LabelList => "list(label)",
        }
    }
}

/// What an attribute is when the BUILD file does not say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrDefault {
    /// Nothing: `native.existing_rule` leaves the key out.
    Unset,
    False,
    EmptyString,
    EmptyList,
}

impl AttrDefault {
    /// The value this stands for, `None` for [`AttrDefault::Unset`].
    pub fn value(self, ty: AttrType) -> Option<AttrValue> {
        match (self, ty) {
            (AttrDefault::Unset, _) => None,
            (AttrDefault::False, _) => Some(AttrValue::Bool(false)),
            (AttrDefault::EmptyString, _) => Some(AttrValue::String(String::new())),
            (AttrDefault::EmptyList, AttrType::LabelList) => Some(AttrValue::LabelList(vec![])),
            (AttrDefault::EmptyList, _) => Some(AttrValue::StringList(vec![])),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AttrSpec {
    pub name: &'static str,
    pub ty: AttrType,
    /// A rule without it is an error.
    pub mandatory: bool,
    pub default: AttrDefault,
}

/// A natively implemented rule. `attrs` is in the order `existing_rule`
/// lists them, after `name` and `kind`.
#[derive(Debug, Clone, Copy)]
pub struct RuleClass {
    pub name: &'static str,
    pub attrs: &'static [AttrSpec],
}

impl RuleClass {
    pub fn attr(&self, name: &str) -> Option<&AttrSpec> {
        self.attrs.iter().find(|a| a.name == name)
    }

    /// Bazel's "did you mean" for an attribute it does not know: the nearest
    /// name if it is close enough.
    pub fn suggest(&self, given: &str) -> Option<&'static str> {
        suggest(given, self.attrs.iter().map(|a| a.name))
    }
}

/// The native rule called `name`, if there is one.
pub fn native_rule(name: &str) -> Option<&'static RuleClass> {
    match name {
        "filegroup" => Some(&FILEGROUP),
        "alias" => Some(&ALIAS),
        _ => None,
    }
}

const fn attr(name: &'static str, ty: AttrType, default: AttrDefault) -> AttrSpec {
    AttrSpec {
        name,
        ty,
        mandatory: false,
        default,
    }
}

use AttrDefault::{EmptyList, EmptyString, Unset};
use AttrType::{Bool, Label as LabelTy, LabelList, String as Str, StringList};

pub static FILEGROUP: RuleClass = RuleClass {
    name: "filegroup",
    attrs: &[
        attr("visibility", LabelList, EmptyList),
        attr("transitive_configs", StringList, EmptyList),
        attr("deprecation", Str, Unset),
        attr("tags", StringList, EmptyList),
        attr("generator_name", Str, EmptyString),
        attr("generator_function", Str, EmptyString),
        attr("generator_location", Str, EmptyString),
        // Shown by `existing_rule` only when the rule sets it, like
        // `deprecation`: a package's `default_testonly` may stand in.
        attr("testonly", Bool, Unset),
        attr("features", StringList, EmptyList),
        attr("compatible_with", LabelList, EmptyList),
        attr("restricted_to", LabelList, EmptyList),
        attr("aspect_hints", LabelList, EmptyList),
        attr("distribs", StringList, EmptyList),
        attr("target_compatible_with", LabelList, EmptyList),
        attr("srcs", LabelList, EmptyList),
        attr("output_group", Str, EmptyString),
        attr("data", LabelList, EmptyList),
        attr("output_licenses", StringList, EmptyList),
        attr("licenses", StringList, Unset),
        attr("package_metadata", LabelList, Unset),
        attr("applicable_licenses", LabelList, Unset),
    ],
};

pub static ALIAS: RuleClass = RuleClass {
    name: "alias",
    attrs: &[
        attr("visibility", LabelList, EmptyList),
        attr("transitive_configs", StringList, EmptyList),
        attr("deprecation", Str, Unset),
        attr("tags", StringList, EmptyList),
        attr("generator_name", Str, EmptyString),
        attr("generator_function", Str, EmptyString),
        attr("generator_location", Str, EmptyString),
        // `alias` takes its `testonly` from `actual` unless told otherwise.
        attr("testonly", Bool, Unset),
        attr("features", StringList, EmptyList),
        attr("compatible_with", LabelList, EmptyList),
        attr("restricted_to", LabelList, EmptyList),
        attr("aspect_hints", LabelList, EmptyList),
        attr("target_compatible_with", LabelList, EmptyList),
        AttrSpec {
            name: "actual",
            ty: LabelTy,
            mandatory: true,
            default: Unset,
        },
        attr("package_metadata", LabelList, Unset),
        attr("applicable_licenses", LabelList, Unset),
    ],
};

/// Bazel's `SpellChecker`, as far as it could be told apart from outside:
/// the candidate with the smallest edit distance, if that distance is at
/// most a third of the word's length (rounded up).
pub fn suggest<'a>(given: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (given.chars().count() + 1) / 3;
    candidates
        .into_iter()
        .map(|c| (edit_distance(given, c), c))
        .filter(|&(d, _)| d <= limit)
        .min_by_key(|&(d, _)| d)
        .map(|(_, c)| c)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = if ca == cb {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[j])
            };
            diagonal = above;
        }
    }
    row[b.len()]
}

/// How `native.existing_rule` writes a label: `:x` inside the package,
/// `//p:x` elsewhere in the repo, `@@repo//p:x` in another repo.
pub fn label_relative_to(label: &Label, repo: &str, package: &str) -> String {
    if label.repo != repo {
        format!("@@{}//{}:{}", label.repo, label.package, label.name)
    } else if label.package == package {
        format!(":{}", label.name)
    } else {
        format!("//{}:{}", label.package, label.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each row is a misspelling and what Bazel 9.2.0 suggested for it on a
    /// `filegroup` (an empty string: nothing).
    #[test]
    fn suggestions_match_bazels() {
        for (given, want) in [
            ("src", "srcs"),
            ("srcz", "srcs"),
            ("dat", "data"),
            ("tag", "tags"),
            ("tagz", "tags"),
            ("testonl", "testonly"),
            ("feature", "features"),
            ("deprecate", "deprecation"),
            ("outputgroup", "output_group"),
            ("compatible_with_", "compatible_with"),
            ("deprecated_", "deprecation"),
            ("output_group_", "output_group"),
            ("visibility_", "visibility"),
            ("sr", ""),
            ("da", ""),
            ("s", ""),
            ("visib", ""),
            ("compat", ""),
            ("tools", ""),
            ("actual", ""),
            ("toolchains", ""),
            ("srcs_version", ""),
            ("exec_properties", ""),
            ("default_visibility", ""),
        ] {
            assert_eq!(FILEGROUP.suggest(given).unwrap_or(""), want, "{given}");
        }
        // `alias` has no `srcs` to suggest.
        assert_eq!(ALIAS.suggest("src"), None);
        assert_eq!(ALIAS.suggest("tag"), Some("tags"));
    }

    #[test]
    fn the_two_rules_accept_what_bazel_accepts() {
        for name in [
            "srcs",
            "data",
            "output_group",
            "visibility",
            "deprecation",
            "tags",
            "testonly",
            "features",
            "compatible_with",
            "restricted_to",
            "target_compatible_with",
            "licenses",
            "distribs",
            "output_licenses",
            "transitive_configs",
            "aspect_hints",
            "package_metadata",
            "applicable_licenses",
            "generator_name",
        ] {
            assert!(FILEGROUP.attr(name).is_some(), "filegroup {name}");
        }
        for name in [
            "exec_compatible_with",
            "exec_properties",
            "toolchains",
            "actual",
            "tools",
            "srcs_version",
        ] {
            assert!(FILEGROUP.attr(name).is_none(), "filegroup {name}");
        }
        for name in [
            "actual",
            "visibility",
            "deprecation",
            "tags",
            "testonly",
            "features",
            "compatible_with",
            "restricted_to",
            "target_compatible_with",
            "transitive_configs",
            "aspect_hints",
            "package_metadata",
            "applicable_licenses",
        ] {
            assert!(ALIAS.attr(name).is_some(), "alias {name}");
        }
        for name in [
            "srcs",
            "data",
            "output_group",
            "exec_compatible_with",
            "exec_properties",
            "licenses",
            "distribs",
            "output_licenses",
            "toolchains",
            "tools",
        ] {
            assert!(ALIAS.attr(name).is_none(), "alias {name}");
        }
        assert!(ALIAS.attr("actual").unwrap().mandatory);
    }

    #[test]
    fn labels_are_written_relative_to_where_they_are_read() {
        let l = |repo: &str, package: &str, name: &str| Label {
            repo: repo.into(),
            package: package.into(),
            name: name.into(),
        };
        assert_eq!(label_relative_to(&l("", "a", "x"), "", "a"), ":x");
        assert_eq!(label_relative_to(&l("", "b", "x"), "", "a"), "//b:x");
        assert_eq!(label_relative_to(&l("r", "b", "x"), "", "a"), "@@r//b:x");
        assert_eq!(label_relative_to(&l("", "b", "x"), "r", "b"), "@@//b:x");
    }
}
