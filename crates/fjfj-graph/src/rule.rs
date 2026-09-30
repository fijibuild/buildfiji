//! Attribute values, and the attribute schema of the rules Bazel implements
//! natively that BUILD files call directly (buildfiji-mum.4).
//!
//! Only rules with no Starlark definition are here: `filegroup` and `alias`.
//! Everything else a BUILD file calls is a `rule()` from a `.bzl`, which
//! brings its own schema. The lists below were read off Bazel 9.2.0:
//! which attributes each rule accepts, their types, and what
//! `native.existing_rule` shows for one that was never set.
//!
//! [`AttrDef`] is one attribute as a schema states it, whoever wrote the
//! schema: `attr.*` in a `.bzl` builds one, and an [`AttrSpec`] of a native
//! rule converts to one. It holds only data; what needs a Starlark value
//! (providers, aspects, transitions, a default that is a function) stays with
//! the `.bzl` that made it.

use crate::Label;
use std::collections::BTreeSet;

/// A value stored in a rule attribute.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttrValue {
    Bool(bool),
    /// A signed 32-bit integer, all Bazel's `int` attributes can hold.
    Int(i32),
    String(String),
    Label(Label),
    StringList(Vec<String>),
    IntList(Vec<i32>),
    LabelList(Vec<Label>),
    /// The dict kinds keep the order the rule wrote them in.
    StringDict(Vec<(String, String)>),
    StringListDict(Vec<(String, Vec<String>)>),
    LabelKeyedStringDict(Vec<(Label, String)>),
    StringKeyedLabelDict(Vec<(String, Label)>),
    LabelListDict(Vec<(String, Vec<Label>)>),
}

/// What an attribute holds, and so how a value is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    Bool,
    Int,
    String,
    StringList,
    IntList,
    Label,
    LabelList,
    StringDict,
    StringListDict,
    LabelKeyedStringDict,
    StringKeyedLabelDict,
    LabelListDict,
    /// A file the rule creates, written as a string.
    Output,
    OutputList,
}

impl AttrType {
    /// The name Bazel's type errors use.
    pub fn name(self) -> &'static str {
        match self {
            AttrType::Bool => "bool",
            AttrType::Int => "int",
            AttrType::String => "string",
            AttrType::StringList => "list(string)",
            AttrType::IntList => "list(int)",
            // A label or an output is written as a string.
            AttrType::Label | AttrType::Output => "string",
            AttrType::LabelList => "list(label)",
            AttrType::StringDict => "dict(string, string)",
            AttrType::StringListDict => "dict(string, list(string))",
            AttrType::LabelKeyedStringDict => "dict(label, string)",
            AttrType::StringKeyedLabelDict => "dict(string, label)",
            AttrType::LabelListDict => "dict(string, list(label))",
            AttrType::OutputList => "list(output)",
        }
    }

    /// What an attribute of this type is when nothing says otherwise:
    /// `None` for the kinds that have no value (a label, an output).
    pub fn zero(self) -> Option<AttrValue> {
        Some(match self {
            AttrType::Bool => AttrValue::Bool(false),
            AttrType::Int => AttrValue::Int(0),
            AttrType::String => AttrValue::String(String::new()),
            AttrType::StringList => AttrValue::StringList(vec![]),
            AttrType::IntList => AttrValue::IntList(vec![]),
            AttrType::LabelList | AttrType::OutputList => AttrValue::LabelList(vec![]),
            AttrType::StringDict => AttrValue::StringDict(vec![]),
            AttrType::StringListDict => AttrValue::StringListDict(vec![]),
            AttrType::LabelKeyedStringDict => AttrValue::LabelKeyedStringDict(vec![]),
            AttrType::StringKeyedLabelDict => AttrValue::StringKeyedLabelDict(vec![]),
            AttrType::LabelListDict => AttrValue::LabelListDict(vec![]),
            AttrType::Label | AttrType::Output => return None,
        })
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

/// The property flags of `attr.*(flags = [...])`, which are the ones Bazel's
/// `Attribute` carries. `mandatory`, `allow_empty = False`, `executable` and
/// `allow_single_file` set their flag too, so a flag and its keyword are the
/// same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttrFlag {
    Mandatory,
    OrderIndependent,
    DirectCompileTimeInput,
    NonEmpty,
    SingleArtifact,
    SkipAnalysisTimeFiletypeCheck,
    Undocumented,
    Executable,
    SkipConstraintsOverride,
    OutputLicenses,
    /// Set on every attribute a `.bzl` declares.
    StarlarkDefined,
}

impl AttrFlag {
    const ALL: [AttrFlag; 11] = [
        AttrFlag::Mandatory,
        AttrFlag::OrderIndependent,
        AttrFlag::DirectCompileTimeInput,
        AttrFlag::NonEmpty,
        AttrFlag::SingleArtifact,
        AttrFlag::SkipAnalysisTimeFiletypeCheck,
        AttrFlag::Undocumented,
        AttrFlag::Executable,
        AttrFlag::SkipConstraintsOverride,
        AttrFlag::OutputLicenses,
        AttrFlag::StarlarkDefined,
    ];

    /// The flag `attr.*` calls `name`.
    pub fn parse(name: &str) -> Option<AttrFlag> {
        AttrFlag::ALL.into_iter().find(|f| f.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            AttrFlag::Mandatory => "MANDATORY",
            AttrFlag::OrderIndependent => "ORDER_INDEPENDENT",
            AttrFlag::DirectCompileTimeInput => "DIRECT_COMPILE_TIME_INPUT",
            AttrFlag::NonEmpty => "NON_EMPTY",
            AttrFlag::SingleArtifact => "SINGLE_ARTIFACT",
            AttrFlag::SkipAnalysisTimeFiletypeCheck => "SKIP_ANALYSIS_TIME_FILETYPE_CHECK",
            AttrFlag::Undocumented => "UNDOCUMENTED",
            AttrFlag::Executable => "EXECUTABLE",
            AttrFlag::SkipConstraintsOverride => "SKIP_CONSTRAINTS_OVERRIDE",
            AttrFlag::OutputLicenses => "OUTPUT_LICENSES",
            AttrFlag::StarlarkDefined => "STARLARK_DEFINED",
        }
    }
}

/// Which source files a label attribute takes (`allow_files`,
/// `allow_single_file`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTypes {
    /// Rule targets only.
    None,
    Any,
    /// Files with one of these suffixes.
    Suffixes(Vec<String>),
}

/// The configuration a dependency is built in (`cfg`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cfg {
    Target,
    Exec,
    /// `"host"`, which Bazel 9.2.0 still accepts.
    Host,
    /// A transition a `.bzl` supplied, which it keeps.
    Transition,
}

/// One attribute as a schema states it. See the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrDef {
    pub ty: AttrType,
    /// The default the schema wrote, `None` if it wrote none.
    pub default: Option<AttrValue>,
    /// The default is a function, kept with the `.bzl` that made it.
    pub computed_default: bool,
    pub doc: Option<String>,
    pub flags: BTreeSet<AttrFlag>,
    pub files: FileTypes,
    /// Rule kinds a label may name; `None` is any.
    pub allow_rules: Option<BTreeSet<String>>,
    pub cfg: Cfg,
    /// `configurable = ...` if it was given.
    pub configurable: Option<bool>,
    pub skip_validations: bool,
    /// The attribute has no value until a rule sets one, though its type has
    /// a zero (`deprecation`, which `native.existing_rule` leaves out).
    pub no_default: bool,
}

impl AttrDef {
    pub fn new(ty: AttrType) -> AttrDef {
        AttrDef {
            ty,
            default: None,
            computed_default: false,
            doc: None,
            flags: BTreeSet::new(),
            files: FileTypes::None,
            allow_rules: None,
            cfg: Cfg::Target,
            configurable: None,
            skip_validations: false,
            no_default: false,
        }
    }

    pub fn mandatory(&self) -> bool {
        self.flags.contains(&AttrFlag::Mandatory)
    }

    /// A list or dict attribute may be empty unless flagged non-empty.
    pub fn allow_empty(&self) -> bool {
        !self.flags.contains(&AttrFlag::NonEmpty)
    }

    pub fn executable(&self) -> bool {
        self.flags.contains(&AttrFlag::Executable)
    }

    pub fn single_file(&self) -> bool {
        self.flags.contains(&AttrFlag::SingleArtifact)
    }

    /// The value the attribute has when a rule sets none. `None` if it has
    /// no value, or if the default is computed.
    pub fn default_value(&self) -> Option<AttrValue> {
        if self.computed_default || (self.no_default && self.default.is_none()) {
            return None;
        }
        self.default.clone().or_else(|| self.ty.zero())
    }
}

/// A native rule's attribute, as a schema would state it.
impl From<&AttrSpec> for AttrDef {
    fn from(spec: &AttrSpec) -> AttrDef {
        let mut def = AttrDef::new(spec.ty);
        def.default = spec.default.value(spec.ty);
        def.no_default = spec.default == AttrDefault::Unset;
        if spec.mandatory {
            def.flags.insert(AttrFlag::Mandatory);
        }
        def
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
        attr("transitive_configs", LabelList, EmptyList),
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
        attr("transitive_configs", LabelList, EmptyList),
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

/// Bazel's "did you mean" for an unknown repository or rule attribute, as far
/// as it could be told apart from outside: the candidate with the smallest
/// edit distance, if that distance is at most a third of the word's length
/// (rounded up).
pub fn suggest<'a>(given: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (given.chars().count() + 1) / 3;
    candidates
        .into_iter()
        .map(|c| (edit_distance(given, c), c))
        .filter(|&(d, _)| d <= limit)
        .min_by_key(|&(d, _)| d)
        .map(|(_, c)| c)
}

/// The same for a keyword argument a builtin function does not take. It is
/// not the rule of [`suggest`]: `non_empty` gets `allow_empty` (four edits,
/// for a word of nine), which that rule refuses. The rule that fits every
/// suggestion and refusal Bazel 9.2.0 made for a misspelt keyword of the
/// `attr.*` builders (and of `glob`) is a distance of at most nine
/// twentieths of the shorter of the two words. It is the only ratio that
/// does, and no probe separates it from a nearby formula.
pub fn suggest_keyword<'a>(
    given: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let given_len = given.chars().count();
    candidates
        .into_iter()
        .map(|c| (edit_distance(given, c), c))
        .filter(|&(d, c)| d * 20 <= 9 * given_len.min(c.chars().count()))
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

    /// Keyword suggestions, each a misspelling and what Bazel 9.2.0 offered
    /// for it among the keywords of `attr.string_list` (an empty string:
    /// nothing).
    #[test]
    fn keyword_suggestions_match_bazels() {
        let keywords = ["mandatory", "allow_empty", "default", "doc", "configurable"];
        for (given, want) in [
            ("non_empty", "allow_empty"),
            ("defualt", "default"),
            ("mandatry", "mandatory"),
            ("allow_empt", "allow_empty"),
            ("nonempty", ""),
            ("allowlist", ""),
            ("order", ""),
            ("name", ""),
            ("values", ""),
            ("visibility", ""),
            ("default_provider", ""),
            ("allow_files", ""),
            ("do", ""),
        ] {
            assert_eq!(
                suggest_keyword(given, keywords).unwrap_or(""),
                want,
                "{given}"
            );
        }
        // The keywords of `attr.label`, where `tags` is two edits from
        // `flags` and is refused.
        let label = ["default", "flags", "cfg", "doc", "allow_single_file"];
        assert_eq!(suggest_keyword("tags", label), None);
        assert_eq!(suggest_keyword("single_file", label), None);
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

    /// The error text `expected value of type '...' for attribute` uses for
    /// each kind, from instantiating a `rule()` with each `attr.*` in Bazel
    /// 9.2.0. `bool` is not in it: Bazel words that one differently.
    #[test]
    fn every_attr_kind_has_bazels_name() {
        for (ty, want) in [
            (AttrType::Int, "int"),
            (AttrType::IntList, "list(int)"),
            (AttrType::Label, "string"),
            (AttrType::LabelKeyedStringDict, "dict(label, string)"),
            (AttrType::LabelList, "list(label)"),
            (AttrType::LabelListDict, "dict(string, list(label))"),
            (AttrType::Output, "string"),
            (AttrType::OutputList, "list(output)"),
            (AttrType::String, "string"),
            (AttrType::StringDict, "dict(string, string)"),
            (AttrType::StringKeyedLabelDict, "dict(string, label)"),
            (AttrType::StringList, "list(string)"),
            (AttrType::StringListDict, "dict(string, list(string))"),
        ] {
            assert_eq!(ty.name(), want);
        }
    }

    #[test]
    fn a_kind_with_no_value_has_no_zero() {
        assert_eq!(AttrType::Label.zero(), None);
        assert_eq!(AttrType::Output.zero(), None);
        assert_eq!(AttrType::Int.zero(), Some(AttrValue::Int(0)));
        assert_eq!(
            AttrType::LabelListDict.zero(),
            Some(AttrValue::LabelListDict(vec![]))
        );
    }

    #[test]
    fn flags_are_read_by_the_names_attr_uses() {
        for f in AttrFlag::ALL {
            assert_eq!(AttrFlag::parse(f.name()), Some(f));
        }
        assert_eq!(AttrFlag::parse("single_artifact"), None);
        assert_eq!(AttrFlag::parse("HIDDEN"), None);
    }

    #[test]
    fn a_native_attribute_states_itself_as_a_definition() {
        let actual = AttrDef::from(ALIAS.attr("actual").unwrap());
        assert_eq!(actual.ty, AttrType::Label);
        assert!(actual.mandatory());
        assert_eq!(actual.default_value(), None);
        let srcs = AttrDef::from(FILEGROUP.attr("srcs").unwrap());
        assert!(!srcs.mandatory());
        assert_eq!(srcs.default_value(), Some(AttrValue::LabelList(vec![])));
        // A native attribute is not a Starlark-defined one.
        assert!(!srcs.flags.contains(&AttrFlag::StarlarkDefined));
    }

    #[test]
    fn a_computed_default_is_not_a_value() {
        let mut def = AttrDef::new(AttrType::Label);
        def.computed_default = true;
        assert_eq!(def.default_value(), None);
        let mut list = AttrDef::new(AttrType::StringList);
        assert_eq!(list.default_value(), Some(AttrValue::StringList(vec![])));
        list.default = Some(AttrValue::StringList(vec!["a".into()]));
        assert_eq!(
            list.default_value(),
            Some(AttrValue::StringList(vec!["a".into()]))
        );
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
