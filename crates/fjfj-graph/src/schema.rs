//! The schema of a rule class, whoever wrote it (buildfiji-mum.3.5): the
//! attributes a call to it may set, their types and defaults, and what the
//! class adds of its own.
//!
//! A native rule (`filegroup`, `alias`) states its schema in a static
//! [`RuleClass`]; a `rule()` in a `.bzl` builds one at run time from its
//! `attrs`. Both become a [`RuleSchema`], and instantiating a rule reads
//! nothing else, so one code path serves both.
//!
//! What every Starlark rule has, and in which order `native.existing_rule`
//! lists it, was read off Bazel 9.2.0:
//!
//! - the *universal* attributes (`name`, `visibility`, `tags`, ...);
//! - for `executable = True`, `args` and `output_licenses`;
//! - for `test = True` (whose class name must end in `_test`, and whose
//!   `testonly` defaults to true), `size`, `timeout`, `flaky`,
//!   `shard_count`, `local` and `args`;
//! - the rule's own, in the order it declared them.
//!
//! None of those names can be redeclared, and neither can a private one
//! (`_x`) be set by a call, or be seen in `existing_rule`.

use crate::rule::{AttrDef, AttrFlag, AttrType, AttrValue, RuleClass};

/// One attribute of a rule, as its schema states it.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaAttr {
    pub name: String,
    pub def: AttrDef,
    /// `values = [...]` of a string or int attribute, as the error that
    /// names them writes each (`'a'`, `'1'`); empty if any value is allowed.
    pub values: Vec<String>,
    /// `native.existing_rule` never lists it, even when it is set.
    pub hidden: bool,
    /// Whether a `select()` may be its value. Outputs never can, and
    /// Bazel fixes a few of the attributes every rule has (`visibility`,
    /// `tags`, ...); all others can.
    pub configurable: bool,
    /// A list of strings written as a `set` (the default of a string-set
    /// build setting); a value that is not a set is refused.
    pub set: bool,
}

impl SchemaAttr {
    pub fn new(name: &str, def: AttrDef) -> SchemaAttr {
        SchemaAttr {
            name: name.to_owned(),
            values: Vec::new(),
            hidden: false,
            configurable: !matches!(def.ty, AttrType::Output | AttrType::OutputList),
            set: false,
            def,
        }
    }

    /// A private attribute (`_x`) is the rule's own: a call cannot set it,
    /// and `existing_rule` does not show it.
    pub fn private(&self) -> bool {
        self.name.starts_with('_')
    }

    /// Whether a call may name it.
    pub fn settable(&self) -> bool {
        !self.private()
    }

    /// Whether `native.existing_rule` may list it.
    pub fn shown(&self) -> bool {
        !self.private() && !self.hidden
    }
}

/// A rule class's schema. See the module docs.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSchema {
    /// Every attribute, in the order `existing_rule` lists them, `name`
    /// first (and printed with `kind`, which is not an attribute).
    pub attrs: Vec<SchemaAttr>,
    pub test: bool,
    pub executable: bool,
    /// A `.bzl` rule, whose call words a few errors its own way.
    pub starlark: bool,
    /// The `.bzl` that made the class (a `rule()`'s), for analysis to find its
    /// implementation; `None` for a native class.
    pub defined_in: Option<crate::Label>,
    /// The toolchain types the class asks for and whether each must resolve.
    pub toolchains: Vec<(crate::Label, bool)>,
    /// Implicit outputs given as templates: the output's name in the
    /// rule's `outputs` and its template (`%{name}.txt`).
    pub outputs: Vec<(String, String)>,
    /// `build_setting = config.string(flag = True)`: the class is a build
    /// setting, and this is its type.
    pub build_setting: Option<BuildSettingSpec>,
    /// `rule(cfg = transition(...))`: the target is built in the configuration
    /// the transition makes of the one it was asked for.
    pub incoming_transition: bool,
    /// For a `rule()`: the calls that led to it, outermost first, which
    /// `query --output=build` shows as where the class is defined.
    pub definition_stack: Vec<crate::package::StackFrame>,
    /// `rule(fragments = ...)`: the configuration fragments the class reads,
    /// as written.
    pub fragments: Vec<String>,
}

/// The type of a build setting and whether the command line may set it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildSettingSpec {
    pub kind: SettingKind,
    pub flag: bool,
    pub multiple: bool,
}

/// What a build setting holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
    Bool,
    Int,
    String,
    StringList,
    StringSet,
}

/// What is wrong with the attributes a `rule()` declares.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemaError {
    #[error("attribute name `{0}` is not a valid identifier.")]
    NotAnIdentifier(String),
    #[error("attribute `{0}`: built-in attributes cannot be overridden.")]
    BuiltIn(String),
}

impl RuleSchema {
    /// The attribute called `name`.
    pub fn attr(&self, name: &str) -> Option<&SchemaAttr> {
        self.attrs.iter().find(|a| a.name == name)
    }

    /// Bazel's "did you mean" for an attribute a call names and the class
    /// does not have.
    pub fn suggest(&self, given: &str) -> Option<&str> {
        crate::rule::suggest(
            given,
            self.attrs
                .iter()
                .filter(|a| a.settable())
                .map(|a| a.name.as_str()),
        )
    }

    /// A native rule's schema.
    pub fn native(class: &RuleClass) -> RuleSchema {
        let mut name = AttrDef::new(AttrType::String);
        name.flags.insert(AttrFlag::Mandatory);
        let mut attrs = vec![SchemaAttr::new("name", name)];
        attrs.extend(
            class
                .attrs
                .iter()
                .map(|spec| SchemaAttr::new(spec.name, AttrDef::from(spec))),
        );
        fixed(&mut attrs, FIXED_UNIVERSAL);
        RuleSchema {
            attrs,
            test: false,
            executable: false,
            starlark: false,
            defined_in: None,
            toolchains: Vec::new(),
            outputs: Vec::new(),
            build_setting: None,
            incoming_transition: false,
            definition_stack: Vec::new(),
            fragments: Vec::new(),
        }
    }

    /// The schema of a `rule()` with these attributes of its own.
    pub fn starlark(
        own: Vec<SchemaAttr>,
        test: bool,
        executable: bool,
        outputs: Vec<(String, String)>,
    ) -> Result<RuleSchema, SchemaError> {
        let mut attrs = universal(test);
        fixed(&mut attrs, FIXED_UNIVERSAL);
        if test {
            let mut own_test = test_attrs();
            fixed(&mut own_test, FIXED_TEST);
            attrs.extend(own_test);
        } else if executable {
            attrs.extend(executable_attrs());
        }
        for attr in &own {
            if !is_identifier(&attr.name) {
                return Err(SchemaError::NotAnIdentifier(attr.name.clone()));
            }
            if attrs.iter().any(|a| a.name == attr.name) {
                return Err(SchemaError::BuiltIn(attr.name.clone()));
            }
        }
        let declares_licenses = own.iter().any(|a| a.name == "applicable_licenses");
        attrs.extend(own);
        // Every rule takes `applicable_licenses` unless it declares its own.
        if !declares_licenses {
            let mut applicable = without("applicable_licenses", AttrType::LabelList);
            applicable.hidden = true;
            applicable.configurable = false;
            attrs.push(applicable);
        }
        Ok(RuleSchema {
            attrs,
            test,
            executable: executable || test,
            starlark: true,
            defined_in: None,
            toolchains: Vec::new(),
            outputs,
            build_setting: None,
            incoming_transition: false,
            definition_stack: Vec::new(),
            fragments: Vec::new(),
        })
    }
}

/// The attributes every rule has that a `select()` may not set.
const FIXED_UNIVERSAL: &[&str] = &[
    "name",
    "visibility",
    "transitive_configs",
    "deprecation",
    "tags",
    "generator_name",
    "generator_function",
    "generator_location",
    "testonly",
    "compatible_with",
    "restricted_to",
    "package_metadata",
    "applicable_licenses",
    "exec_compatible_with",
    "exec_group_compatible_with",
];

/// The same for a test's own.
const FIXED_TEST: &[&str] = &["size", "timeout", "flaky", "local"];

fn fixed(attrs: &mut [SchemaAttr], names: &[&str]) {
    for attr in attrs {
        if names.contains(&attr.name.as_str()) {
            attr.configurable = false;
        }
    }
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// An attribute of `ty` whose value is `default` if a call sets none.
fn with(name: &str, ty: AttrType, default: Option<AttrValue>) -> SchemaAttr {
    let mut def = AttrDef::new(ty);
    def.default = default;
    SchemaAttr::new(name, def)
}

/// An attribute with no value until a call sets one.
fn without(name: &str, ty: AttrType) -> SchemaAttr {
    let mut attr = SchemaAttr::new(name, AttrDef::new(ty));
    attr.def.no_default = true;
    attr
}

/// What every rule has, in the order `existing_rule` lists it.
fn universal(test: bool) -> Vec<SchemaAttr> {
    use AttrType::{Bool, LabelList, LabelListDict, String as Str, StringDict, StringList};
    let mut name = AttrDef::new(Str);
    name.flags.insert(AttrFlag::Mandatory);
    let mut testonly = without("testonly", Bool);
    if test {
        testonly = with("testonly", Bool, Some(AttrValue::Bool(true)));
    }
    vec![
        SchemaAttr::new("name", name),
        with("expect_failure", Str, None),
        with("visibility", LabelList, None),
        with("transitive_configs", LabelList, None),
        without("deprecation", Str),
        with("tags", StringList, None),
        with("generator_name", Str, None),
        with("generator_function", Str, None),
        with("generator_location", Str, None),
        testonly,
        with("features", StringList, None),
        with("compatible_with", LabelList, None),
        with("restricted_to", LabelList, None),
        without("package_metadata", LabelList),
        with("aspect_hints", LabelList, None),
        with("toolchains", LabelList, None),
        with("exec_properties", StringDict, None),
        with("exec_compatible_with", LabelList, None),
        with("exec_group_compatible_with", LabelListDict, None),
        with("target_compatible_with", LabelList, None),
    ]
}

fn executable_attrs() -> Vec<SchemaAttr> {
    vec![
        with("args", AttrType::StringList, None),
        with("output_licenses", AttrType::StringList, None),
    ]
}

fn test_attrs() -> Vec<SchemaAttr> {
    vec![
        with(
            "size",
            AttrType::String,
            Some(AttrValue::String("medium".to_owned())),
        ),
        without("timeout", AttrType::String),
        with("flaky", AttrType::Bool, Some(AttrValue::Bool(false))),
        with("shard_count", AttrType::Int, Some(AttrValue::Int(-1))),
        with("local", AttrType::Bool, Some(AttrValue::Bool(false))),
        with("args", AttrType::StringList, None),
    ]
}

/// The sizes a test may have, and the timeouts.
pub const TEST_SIZES: &[&str] = &["small", "medium", "large", "enormous"];
pub const TEST_TIMEOUTS: &[&str] = &["short", "moderate", "long", "eternal"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{ALIAS, FILEGROUP};

    fn names(schema: &RuleSchema) -> Vec<&str> {
        schema
            .attrs
            .iter()
            .filter(|a| a.shown())
            .map(|a| a.name.as_str())
            .collect()
    }

    #[test]
    fn a_native_rule_has_a_name_and_its_own_attributes() {
        let filegroup = RuleSchema::native(&FILEGROUP);
        assert_eq!(filegroup.attrs[0].name, "name");
        assert!(filegroup.attr("srcs").is_some());
        assert!(!filegroup.starlark);
        let alias = RuleSchema::native(&ALIAS);
        assert!(alias.attr("actual").unwrap().def.mandatory());
        assert_eq!(alias.suggest("nme"), Some("name"));
    }

    #[test]
    fn a_starlark_rule_lists_the_attributes_bazel_lists() {
        let plain = RuleSchema::starlark(vec![], false, false, vec![]).unwrap();
        assert_eq!(
            names(&plain),
            [
                "name",
                "expect_failure",
                "visibility",
                "transitive_configs",
                "deprecation",
                "tags",
                "generator_name",
                "generator_function",
                "generator_location",
                "testonly",
                "features",
                "compatible_with",
                "restricted_to",
                "package_metadata",
                "aspect_hints",
                "toolchains",
                "exec_properties",
                "exec_compatible_with",
                "exec_group_compatible_with",
                "target_compatible_with",
            ]
        );
        let exe = RuleSchema::starlark(vec![], false, true, vec![]).unwrap();
        assert_eq!(&names(&exe)[20..], ["args", "output_licenses"]);
        let test = RuleSchema::starlark(vec![], true, false, vec![]).unwrap();
        assert_eq!(
            &names(&test)[20..],
            ["size", "timeout", "flaky", "shard_count", "local", "args"]
        );
        assert!(test.executable && test.test);
        assert_eq!(
            test.attr("testonly").unwrap().def.default_value(),
            Some(AttrValue::Bool(true))
        );
        assert_eq!(plain.attr("testonly").unwrap().def.default_value(), None);
    }

    #[test]
    fn the_built_in_names_cannot_be_redeclared() {
        let own = |name: &str| vec![SchemaAttr::new(name, AttrDef::new(AttrType::String))];
        for name in ["name", "tags", "deprecation", "toolchains"] {
            assert_eq!(
                RuleSchema::starlark(own(name), false, false, vec![]),
                Err(SchemaError::BuiltIn(name.to_owned())),
                "{name}"
            );
        }
        // `args` is the executable's, and the test's, and nobody else's.
        assert!(RuleSchema::starlark(own("args"), false, false, vec![]).is_ok());
        assert!(RuleSchema::starlark(own("args"), false, true, vec![]).is_err());
        assert!(RuleSchema::starlark(own("size"), true, false, vec![]).is_err());
        assert!(RuleSchema::starlark(own("size"), false, false, vec![]).is_ok());
        // `licenses` and `distribs` are a native rule's, not a universal one,
        // and a rule may declare its own `applicable_licenses`.
        assert!(RuleSchema::starlark(own("applicable_licenses"), false, false, vec![]).is_ok());
        assert!(RuleSchema::starlark(own("licenses"), false, false, vec![]).is_ok());
        assert_eq!(
            RuleSchema::starlark(own("x y"), false, false, vec![]),
            Err(SchemaError::NotAnIdentifier("x y".to_owned()))
        );
        assert!(RuleSchema::starlark(own(""), false, false, vec![]).is_err());
        assert!(RuleSchema::starlark(own("1x"), false, false, vec![]).is_err());
    }

    #[test]
    fn a_private_attribute_is_not_settable_or_shown() {
        let own = vec![SchemaAttr::new("_x", AttrDef::new(AttrType::String))];
        let schema = RuleSchema::starlark(own, false, false, vec![]).unwrap();
        let x = schema.attr("_x").unwrap();
        assert!(!x.settable() && !x.shown());
        assert!(schema.attr("name").unwrap().settable());
    }

    #[test]
    fn outputs_and_a_few_of_the_universal_attributes_are_not_configurable() {
        let plain = RuleSchema::starlark(vec![], false, false, vec![]).unwrap();
        let fixed: Vec<&str> = plain
            .attrs
            .iter()
            .filter(|a| !a.configurable)
            .map(|a| a.name.as_str())
            .collect();
        assert_eq!(
            fixed,
            [
                "name",
                "visibility",
                "transitive_configs",
                "deprecation",
                "tags",
                "generator_name",
                "generator_function",
                "generator_location",
                "testonly",
                "compatible_with",
                "restricted_to",
                "package_metadata",
                "exec_compatible_with",
                "exec_group_compatible_with",
                "applicable_licenses",
            ]
        );
        let test = RuleSchema::starlark(vec![], true, false, vec![]).unwrap();
        for name in ["size", "timeout", "flaky", "local"] {
            assert!(!test.attr(name).unwrap().configurable, "{name}");
        }
        for name in ["shard_count", "args", "features", "target_compatible_with"] {
            assert!(test.attr(name).unwrap().configurable, "{name}");
        }
        let own = vec![
            SchemaAttr::new("o", AttrDef::new(AttrType::Output)),
            SchemaAttr::new("s", AttrDef::new(AttrType::String)),
            // Only a test's own `size` is fixed.
            SchemaAttr::new("size", AttrDef::new(AttrType::String)),
        ];
        let schema = RuleSchema::starlark(own, false, false, vec![]).unwrap();
        assert!(!schema.attr("o").unwrap().configurable);
        assert!(schema.attr("s").unwrap().configurable);
        assert!(schema.attr("size").unwrap().configurable);
        let filegroup = RuleSchema::native(&FILEGROUP);
        assert!(filegroup.attr("srcs").unwrap().configurable);
        assert!(!filegroup.attr("visibility").unwrap().configurable);
    }
}
