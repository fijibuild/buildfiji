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
    /// A value that depends on the configuration: `select()`s, and plain
    /// values between them. Resolving one is buildfiji-136.5's.
    Select(SelectorList),
}

/// The label of the branch a `select()` takes when no other matches.
pub fn default_condition() -> Label {
    Label {
        repo: String::new(),
        package: "conditions".to_owned(),
        name: "default".to_owned(),
    }
}

/// One `select()` (or a plain value between selects, which is one with
/// only the default branch).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Selector {
    /// The conditions in the order the `select()` named them, each with the
    /// value it gives. `None` is a `None` branch of a label attribute.
    pub branches: Vec<(Label, Option<AttrValue>)>,
    pub no_match_error: String,
    /// Written as a plain value, or as `select({"//conditions:default": x})`:
    /// `existing_rule` shows `x` itself when nothing else is in the list.
    pub unconditional: bool,
}

/// Selectors joined by `+` (or `|` for dicts). A list that is all
/// unconditional is a plain value and never becomes one of these.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelectorList {
    pub elements: Vec<Selector>,
    pub pipe: bool,
}

impl AttrValue {
    /// Every label the value holds, in the order it was written, those of a
    /// `select()` included.
    pub fn labels<'a>(&'a self, out: &mut Vec<&'a Label>) {
        use AttrValue::*;
        match self {
            Label(l) => out.push(l),
            LabelList(ls) => out.extend(ls),
            LabelKeyedStringDict(d) => out.extend(d.iter().map(|(l, _)| l)),
            StringKeyedLabelDict(d) => out.extend(d.iter().map(|(_, l)| l)),
            LabelListDict(d) => out.extend(d.iter().flat_map(|(_, ls)| ls)),
            Select(list) => {
                for selector in &list.elements {
                    for (condition, value) in &selector.branches {
                        out.push(condition);
                        if let Some(value) = value {
                            value.labels(out);
                        }
                    }
                }
            }
            Bool(_) | Int(_) | String(_) | StringList(_) | IntList(_) | StringDict(_)
            | StringListDict(_) => {}
        }
    }

    /// `a` followed by `b`, as `+` joins two lists, two strings or two ints
    /// and `|` two dicts (the later entry of a key wins). `None` for values
    /// of different kinds, and kinds that do not join (a bool, a label).
    pub fn concat(a: &AttrValue, b: &AttrValue) -> Option<AttrValue> {
        use AttrValue::*;
        fn union<K: Clone + PartialEq, V: Clone>(a: &[(K, V)], b: &[(K, V)]) -> Vec<(K, V)> {
            let mut out: Vec<(K, V)> = a.to_vec();
            for (k, v) in b {
                match out.iter_mut().find(|(ok, _)| ok == k) {
                    Some(entry) => entry.1 = v.clone(),
                    None => out.push((k.clone(), v.clone())),
                }
            }
            out
        }
        Some(match (a, b) {
            (String(x), String(y)) => String(format!("{x}{y}")),
            (Int(x), Int(y)) => Int(x.wrapping_add(*y)),
            (StringList(x), StringList(y)) => StringList([x.as_slice(), y].concat()),
            (IntList(x), IntList(y)) => IntList([x.as_slice(), y].concat()),
            (LabelList(x), LabelList(y)) => LabelList([x.as_slice(), y].concat()),
            (StringDict(x), StringDict(y)) => StringDict(union(x, y)),
            (StringListDict(x), StringListDict(y)) => StringListDict(union(x, y)),
            (LabelKeyedStringDict(x), LabelKeyedStringDict(y)) => LabelKeyedStringDict(union(x, y)),
            (StringKeyedLabelDict(x), StringKeyedLabelDict(y)) => StringKeyedLabelDict(union(x, y)),
            (LabelListDict(x), LabelListDict(y)) => LabelListDict(union(x, y)),
            _ => return None,
        })
    }
}

impl SelectorList {
    /// What the list is if nothing in it depends on the configuration: its
    /// elements joined. `None` when any does, or a `None` branch is left.
    pub fn flatten(&self) -> Option<AttrValue> {
        let mut values = self.elements.iter().map(|e| match e.branches.as_slice() {
            [(_, value)] if e.unconditional => value.as_ref(),
            _ => None,
        });
        let first = values.next()??.clone();
        values.try_fold(first, |acc, next| AttrValue::concat(&acc, next?))
    }
}

/// What an attribute holds, and so how a value is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    Bool,
    Int,
    /// An int that is -1, 0 or 1, or a bool (`genrule`'s `stamp`).
    Tristate,
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
            AttrType::Int | AttrType::Tristate => "int",
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
            AttrType::Int | AttrType::Tristate => AttrValue::Int(0),
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
    True,
    EmptyString,
    EmptyList,
    EmptyDict,
    Int(i32),
    Str(&'static str),
    Strs(&'static [&'static str]),
}

impl AttrDefault {
    /// The value this stands for, `None` for [`AttrDefault::Unset`].
    pub fn value(self, ty: AttrType) -> Option<AttrValue> {
        match (self, ty) {
            (AttrDefault::Unset, _) => None,
            (AttrDefault::False, _) => Some(AttrValue::Bool(false)),
            (AttrDefault::True, _) => Some(AttrValue::Bool(true)),
            (AttrDefault::EmptyString, _) => Some(AttrValue::String(String::new())),
            (AttrDefault::EmptyList, AttrType::LabelList) => Some(AttrValue::LabelList(vec![])),
            (AttrDefault::EmptyList, _) => Some(AttrValue::StringList(vec![])),
            (AttrDefault::EmptyDict, _) => ty.zero(),
            (AttrDefault::Int(i), _) => Some(AttrValue::Int(i)),
            (AttrDefault::Str(s), _) => Some(AttrValue::String(s.to_owned())),
            (AttrDefault::Strs(list), _) => Some(AttrValue::StringList(
                list.iter().map(|s| (*s).to_owned()).collect(),
            )),
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
    /// name if it is close enough, by [`suggest_attribute`].
    pub fn suggest(&self, given: &str) -> Option<&'static str> {
        suggest_attribute(given, self.attrs.iter().map(|a| a.name))
    }
}

/// The native rule called `name`, if there is one.
pub fn native_rule(name: &str) -> Option<&'static RuleClass> {
    match name {
        "filegroup" => Some(&FILEGROUP),
        "alias" => Some(&ALIAS),
        _ => crate::native_rules::CLASSES
            .iter()
            .find(|class| class.name == name)
            .copied(),
    }
}

/// An optional attribute of a native rule class.
pub(crate) const fn a(name: &'static str, ty: AttrType, default: AttrDefault) -> AttrSpec {
    attr(name, ty, default)
}

/// A mandatory attribute of a native rule class.
pub(crate) const fn m(name: &'static str, ty: AttrType) -> AttrSpec {
    AttrSpec {
        name,
        ty,
        mandatory: true,
        default: AttrDefault::Unset,
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

/// The same for an attribute a rule does not have: at most `(n - 1) / 2` edits
/// for a word of `n` characters, and never more than four (probed on Bazel
/// 9.2.0 for words of three to twenty; the length is the given word's).
pub fn suggest_attribute<'a>(
    given: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    let limit = (given.chars().count().saturating_sub(1) / 2).min(4);
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
        // The most edits it takes, by the length of the word (probed on 9.2.0).
        for (len, most) in [
            (3, 1),
            (4, 1),
            (5, 2),
            (6, 2),
            (7, 3),
            (8, 3),
            (9, 4),
            (12, 4),
            (20, 4),
        ] {
            let word: String = "abcdefghijklmnopqrst".chars().take(len).collect();
            let near = |edits: usize| -> String {
                word.chars()
                    .take(len - edits)
                    .chain(std::iter::repeat_n('1', edits))
                    .collect()
            };
            assert_eq!(
                suggest_attribute(&near(most), [word.as_str()]),
                Some(word.as_str()),
                "{len}"
            );
            assert_eq!(
                suggest_attribute(&near(most + 1), [word.as_str()]),
                None,
                "{len}"
            );
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

    fn one(value: Option<AttrValue>, unconditional: bool) -> Selector {
        Selector {
            branches: vec![(default_condition(), value)],
            no_match_error: String::new(),
            unconditional,
        }
    }

    fn strings(items: &[&str]) -> AttrValue {
        AttrValue::StringList(items.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn values_join_as_their_type_does() {
        use AttrValue as V;
        let join = |a: V, b: V| V::concat(&a, &b);
        let text = |s: &str| V::String(s.to_owned());
        assert_eq!(
            join(strings(&["a"]), strings(&["b"])),
            Some(strings(&["a", "b"]))
        );
        assert_eq!(join(text("a"), text("b")), Some(text("ab")));
        assert_eq!(join(V::Int(2), V::Int(3)), Some(V::Int(5)));
        // A dict joins by key, the later value winning and the earlier key's place kept.
        let entries = |rows: &[(&str, &str)]| {
            V::StringDict(
                rows.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            )
        };
        assert_eq!(
            join(
                entries(&[("a", "1"), ("b", "2")]),
                entries(&[("a", "3"), ("c", "4")])
            ),
            Some(entries(&[("a", "3"), ("b", "2"), ("c", "4")]))
        );
        // Bools and labels do not join, and neither do unlike kinds.
        assert_eq!(join(V::Bool(true), V::Bool(false)), None);
        let label = Label {
            repo: String::new(),
            package: String::new(),
            name: "a".into(),
        };
        assert_eq!(join(V::Label(label.clone()), V::Label(label)), None);
        assert_eq!(join(V::Int(1), text("a")), None);
    }

    #[test]
    fn a_selector_list_that_no_configuration_changes_flattens() {
        let list = |elements| SelectorList {
            elements,
            pipe: false,
        };
        assert_eq!(
            list(vec![
                one(Some(strings(&["a"])), true),
                one(Some(strings(&["b"])), true)
            ])
            .flatten(),
            Some(strings(&["a", "b"]))
        );
        // One conditional element, or a `None` branch, leaves it as it is.
        assert_eq!(
            list(vec![
                one(Some(strings(&["a"])), true),
                one(Some(strings(&["b"])), false)
            ])
            .flatten(),
            None
        );
        assert_eq!(list(vec![one(None, true)]).flatten(), None);
        // The default condition is a label of the main repo.
        assert_eq!(default_condition().to_string(), "//conditions:default");
    }
}
