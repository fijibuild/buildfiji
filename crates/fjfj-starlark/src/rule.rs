//! `rule()` (buildfiji-mum.3.5): a rule class a `.bzl` defines, callable from
//! a BUILD file once it is bound to a top-level name.
//!
//! What Bazel 9.2.0 does, all of it read off probes:
//!
//! - `rule(implementation, *, test, attrs, outputs, executable, ...)`
//!   returns a `rule`, which prints as `<rule NAME>` once named and `<rule>`
//!   before, has no members and is equal only to itself. Its keyword
//!   arguments are checked as they are bound, in the order the call writes
//!   them, and the wording is the signature's (`in call to rule(), parameter
//!   'test' got value of type 'int', want 'bool'`); what is only checked
//!   afterwards (the names of `attrs`, `provides`, `toolchains` ...) is
//!   Bazel's own.
//! - The rule is named as a provider is, by the first top-level name it is
//!   bound to ([`crate::exports`]), and a rule that is not named cannot be
//!   called: `Invalid rule class hasn't been exported by a bzl file`. A test
//!   rule must be named `*_test`, and no other may be.
//! - A call takes what [`RuleSchema::starlark`] says: the attributes every
//!   rule has, an executable's and a test's, and the rule's own, none of
//!   which may be redeclared. Everything else about a call is
//!   [`crate::instantiate`], which is also what `filegroup` and `alias` use.
//!
//! What `rule()` keeps that the schema cannot hold (the implementation, the
//! attribute descriptors, an `outputs` function, `provides`) stays on the
//! value for the analysis beads.

use crate::args::fatal;
use crate::attr::{sequence, view as attribute_view};
use crate::decl::{
    all_of_type, build_setting_attrs, build_setting_of, check_exec_groups, check_subrules_exported,
    is_transition,
};
use crate::exports::{Kind, Named, is_exported, name_at_assignment, next_id, resolve_name};
use crate::instantiate::call_rule;
use crate::label::{evaluating_bzl, label_of_value, parse_in_caller};
use crate::provider::is_provider;
use allocative::Allocative;
use fjfj_graph::rule::{AttrType, suggest_keyword};
use fjfj_graph::schema::{RuleSchema, SchemaAttr};
use starlark::collections::StarlarkHasher;
use starlark::environment::GlobalsBuilder;
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::starlark_module;
use starlark::values::dict::DictRef;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;
use std::hash::Hash;
use std::sync::{Arc, OnceLock};

/// A rule class, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct RuleGen<V> {
    /// What a rule is, across freezing.
    #[trace(static)]
    id: u64,
    #[trace(static)]
    #[allocative(skip)]
    schema: Arc<RuleSchema>,
    /// The `implementation` function, which analysis calls.
    #[allow(dead_code)]
    implementation: V,
    /// The rule's own attribute descriptors, as declared.
    #[allow(dead_code)]
    attrs: Vec<V>,
    /// The names of `attrs`, in the same order.
    #[trace(static)]
    #[allocative(skip)]
    attr_names: Vec<String>,
    /// `outputs` when it is a function (at most one).
    outputs: Vec<V>,
    /// `provides`, as given.
    #[allow(dead_code)]
    provides: Vec<V>,
    /// `cfg`, `build_setting`, `exec_groups` and `subrules`, as given, for
    /// the parameters that were.
    #[trace(static)]
    #[allocative(skip)]
    #[allow(dead_code)]
    declared_names: Vec<&'static str>,
    #[allow(dead_code)]
    declared: Vec<V>,
    #[trace(static)]
    #[allocative(skip)]
    #[allow(dead_code)]
    doc: Option<String>,
    /// The top-level name it is bound to, once it is.
    #[trace(static)]
    #[allocative(skip)]
    name: OnceLock<String>,
}

starlark_complex_value!(pub(crate) Rule);

impl<'v> Freeze for Rule<'v> {
    type Frozen = FrozenRule;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenRule> {
        let all = |values: Vec<Value<'v>>| -> FreezeResult<Vec<FrozenValue>> {
            values.into_iter().map(|v| v.freeze(freezer)).collect()
        };
        Ok(RuleGen {
            id: self.id,
            schema: self.schema,
            implementation: self.implementation.freeze(freezer)?,
            attrs: all(self.attrs)?,
            attr_names: self.attr_names,
            outputs: all(self.outputs)?,
            provides: all(self.provides)?,
            declared_names: self.declared_names,
            declared: all(self.declared)?,
            doc: self.doc,
            name: self.name,
        })
    }
}

impl<V> fmt::Display for RuleGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name.get() {
            Some(name) => write!(f, "<rule {name}>"),
            None => write!(f, "<rule>"),
        }
    }
}

fn identity<'v>(value: Value<'v>) -> Option<(u64, &'v OnceLock<String>, bool)> {
    fn of<'v, V: ValueLike<'v>>(r: &'v RuleGen<V>) -> (u64, &'v OnceLock<String>, bool) {
        (r.id, &r.name, r.schema.test)
    }
    if let Some(live) = value.downcast_ref::<Rule<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenRule>().map(of)
    }
}

/// The schema of the rule `value`, if it is one.
pub(crate) fn schema_of(value: Value<'_>) -> Option<Arc<RuleSchema>> {
    fn of<V>(rule: &RuleGen<V>) -> Arc<RuleSchema> {
        rule.schema.clone()
    }
    if let Some(live) = value.downcast_ref::<Rule<'_>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenRule>().map(of)
    }
}

/// The `implementation` of the rule `value`, if it is one.
pub(crate) fn implementation_of<'v>(value: Value<'v>) -> Option<Value<'v>> {
    if let Some(live) = value.downcast_ref::<Rule<'v>>() {
        Some(live.implementation)
    } else {
        value
            .downcast_ref::<FrozenRule>()
            .map(|frozen| frozen.implementation.to_value())
    }
}

/// The aspects the attribute `attr` of the rule `value` asks for.
pub(crate) fn aspects_of<'v>(value: Value<'v>, attr: &str) -> Vec<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(rule: &RuleGen<V>, attr: &str) -> Vec<Value<'v>> {
        rule.attr_names
            .iter()
            .position(|n| n == attr)
            .and_then(|at| crate::attr::view(rule.attrs[at].to_value()))
            .map(|v| v.aspects)
            .unwrap_or_default()
    }
    if let Some(live) = value.downcast_ref::<Rule<'v>>() {
        of(live, attr)
    } else {
        value
            .downcast_ref::<FrozenRule>()
            .map(|f| of(f, attr))
            .unwrap_or_default()
    }
}

/// The names of the attributes the rule `value` declares.
pub(crate) fn declared_attr_names<'v>(value: Value<'v>) -> Vec<String> {
    if let Some(live) = value.downcast_ref::<Rule<'v>>() {
        live.attr_names.clone()
    } else {
        value
            .downcast_ref::<FrozenRule>()
            .map(|frozen| frozen.attr_names.clone())
            .unwrap_or_default()
    }
}

/// The function that computes the default of the attribute `attr` of the rule
/// `value`, if its default is one.
pub(crate) fn computed_default_of<'v>(value: Value<'v>, attr: &str) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(rule: &RuleGen<V>, attr: &str) -> Option<Value<'v>> {
        let at = rule.attr_names.iter().position(|n| n == attr)?;
        crate::attr::view(rule.attrs[at].to_value())?.computed
    }
    if let Some(live) = value.downcast_ref::<Rule<'v>>() {
        of(live, attr)
    } else {
        value.downcast_ref::<FrozenRule>().and_then(|f| of(f, attr))
    }
}

/// The transition on the edge to the attribute `attr` of the rule `value`, or
/// with `None` the one the rule applies to itself (`rule(cfg = ...)`).
pub(crate) fn transition_of<'v>(value: Value<'v>, attr: Option<&str>) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(rule: &RuleGen<V>, attr: Option<&str>) -> Option<Value<'v>> {
        match attr {
            None => rule
                .declared_names
                .iter()
                .position(|n| *n == "cfg")
                .map(|at| rule.declared[at].to_value()),
            Some(name) => {
                let at = rule.attr_names.iter().position(|n| n == name)?;
                crate::attr::view(rule.attrs[at].to_value())?.transition
            }
        }
    }
    if let Some(live) = value.downcast_ref::<Rule<'v>>() {
        of(live, attr)
    } else {
        value.downcast_ref::<FrozenRule>().and_then(|f| of(f, attr))
    }
}

/// A rule, for [`crate::exports`] to name.
pub(crate) fn named<'v>(value: Value<'v>) -> Option<Named<'v>> {
    identity(value).map(|(id, name, test)| Named {
        id,
        name,
        kind: Kind::Rule { test },
    })
}

#[starlark_value(type = "rule")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for RuleGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let Some(name) = resolve_name(me, eval)? else {
            return Err(fatal(
                "Invalid rule class hasn't been exported by a bzl file",
            ));
        };
        let outputs = self.outputs.first().map(|f| f.to_value());
        call_rule(&self.schema, &name, outputs, args, eval)?;
        Ok(Value::new_none())
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        self.id.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(identity(other).is_some_and(|(id, _, _)| id == self.id))
    }
}

// ---- rule() -------------------------------------------------------------------------

/// The parameters of `rule()`, in Bazel's order: `implementation` may be
/// given by position, the rest by name only.
const KEYWORDS: &[&str] = &[
    "implementation",
    "test",
    "attrs",
    "outputs",
    "executable",
    "output_to_genfiles",
    "fragments",
    "host_fragments",
    "_skylark_testable",
    "toolchains",
    "doc",
    "provides",
    "dependency_resolution_rule",
    "exec_compatible_with",
    "analysis_test",
    "build_setting",
    "cfg",
    "exec_groups",
    "initializer",
    "parent",
    "extendable",
    "subrules",
];

fn wrong(keyword: &str, value: Value<'_>, want: &str) -> starlark::Error {
    fatal(format!(
        "in call to rule(), parameter '{keyword}' got value of type '{}', want '{want}'",
        value.get_type()
    ))
}

/// What is callable, as `initializer` must be.
fn is_callable(value: Value<'_>) -> bool {
    matches!(
        value.get_type(),
        "function" | "builtin_function_or_method" | "Provider" | "RawConstructor" | "rule"
    )
}

/// The first entry of `dict` that is not a `string: <want>` one, as Bazel
/// writes the type of a dict that has one.
fn first_wrong_entry<'v>(
    dict: &DictRef<'v>,
    want: impl Fn(Value<'v>) -> bool,
) -> Option<(Value<'v>, Value<'v>)> {
    dict.iter()
        .find(|(k, v)| k.unpack_str().is_none() || !want(*v))
}

/// Check what the signature says of one argument, as it is bound.
fn check_argument<'v>(keyword: &str, value: Value<'v>, heap: Heap<'v>) -> starlark::Result<()> {
    let is_bool = value.unpack_bool().is_some();
    match keyword {
        "implementation" => {
            if value.get_type() != "function" {
                return Err(wrong(keyword, value, "function"));
            }
        }
        "test"
        | "executable"
        | "output_to_genfiles"
        | "_skylark_testable"
        | "dependency_resolution_rule"
        | "analysis_test" => {
            if !is_bool {
                return Err(wrong(keyword, value, "bool"));
            }
        }
        "attrs" => {
            let Some(dict) = DictRef::from_value(value) else {
                return Err(wrong(keyword, value, "dict"));
            };
            if let Some((k, v)) = first_wrong_entry(&dict, |v| attribute_view(v).is_some()) {
                return Err(fatal(format!(
                    "got dict<{}, {}> for 'attrs', want dict<string, Attribute>",
                    k.get_type(),
                    v.get_type()
                )));
            }
        }
        "outputs" => {
            let ok = value.is_none()
                || DictRef::from_value(value).is_some()
                || value.get_type() == "function";
            if !ok {
                return Err(wrong(keyword, value, "dict, NoneType, or function"));
            }
        }
        "fragments"
        | "host_fragments"
        | "toolchains"
        | "provides"
        | "exec_compatible_with"
        | "subrules" => {
            if sequence(value, heap).is_none() {
                return Err(wrong(keyword, value, "sequence"));
            }
        }
        "doc" => {
            if !value.is_none() && value.unpack_str().is_none() {
                return Err(wrong(keyword, value, "string or NoneType"));
            }
        }
        "build_setting" => {
            if !value.is_none() && value.get_type() != "BuildSetting" {
                return Err(wrong(keyword, value, "BuildSetting or NoneType"));
            }
        }
        "exec_groups" => {
            if !value.is_none() && DictRef::from_value(value).is_none() {
                return Err(wrong(keyword, value, "dict or NoneType"));
            }
        }
        "initializer" => {
            if !value.is_none() && !is_callable(value) {
                return Err(wrong(keyword, value, "callable or NoneType"));
            }
        }
        "extendable" => {
            let ok = value.is_none()
                || is_bool
                || value.unpack_str().is_some()
                || label_of_value(value).is_some();
            if !ok {
                return Err(wrong(keyword, value, "bool, Label, string, or NoneType"));
            }
        }
        _ => {}
    }
    Ok(())
}

/// The name of the Java class Bazel words a wrong `toolchains` element with.
fn java_name(value: Value<'_>) -> String {
    match value.get_type() {
        "int" => "Int32".to_owned(),
        "Attribute" => "Descriptor".to_owned(),
        "bool" => "Boolean".to_owned(),
        other => other.to_owned(),
    }
}

/// The strings of a sequence, or the error for the first element that is
/// not one.
fn strings_of<'v>(
    keyword: &str,
    value: Value<'v>,
    heap: Heap<'v>,
) -> starlark::Result<Vec<String>> {
    sequence(value, heap)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            item.unpack_str().map(str::to_owned).ok_or_else(|| {
                fatal(format!(
                    "at index {i} of {keyword}, got element of type {}, want string",
                    item.get_type()
                ))
            })
        })
        .collect()
}

/// The `values` of an attribute, as the message that lists them writes them.
pub(crate) fn value_strings(ty: AttrType, values: &[Value<'_>]) -> Vec<String> {
    values
        .iter()
        .filter_map(|v| match ty {
            AttrType::String => v.unpack_str().map(str::to_owned),
            AttrType::Int => v.unpack_i32().map(|i| i.to_string()),
            _ => None,
        })
        .collect()
}

fn make_rule<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    if !evaluating_bzl(eval) {
        return Err(fatal(
            "rule() can only be used during .bzl initialization (top-level evaluation)",
        ));
    }
    let heap = eval.heap();
    let positions: Vec<Value<'v>> = args.positions(heap)?.collect();
    let mut given: Vec<Option<Value<'v>>> = vec![None; KEYWORDS.len()];
    if let Some(first) = positions.first() {
        check_argument("implementation", *first, heap)?;
        given[0] = Some(*first);
    }
    for (key, value) in args.names_map()?.iter() {
        let Some(i) = KEYWORDS.iter().position(|k| *k == key.as_str()) else {
            let hint = suggest_keyword(key.as_str(), KEYWORDS.iter().copied())
                .map(|s| format!(" (did you mean '{s}'?)"))
                .unwrap_or_default();
            return Err(fatal(format!(
                "rule() got unexpected keyword argument '{}'{hint}",
                key.as_str()
            )));
        };
        check_argument(KEYWORDS[i], *value, heap)?;
        if given[i].is_some() {
            return Err(fatal(format!(
                "rule() got multiple values for argument '{}'",
                key.as_str()
            )));
        }
        given[i] = Some(*value);
    }
    if positions.len() > 1 {
        return Err(fatal(format!(
            "rule() accepts no more than 1 positional argument but got {}",
            positions.len()
        )));
    }
    let arg = |keyword: &str| {
        let i = KEYWORDS.iter().position(|k| *k == keyword).expect("known");
        given[i].filter(|v| !v.is_none())
    };
    let Some(implementation) = given[0] else {
        return Err(fatal(
            "rule() missing 1 required positional argument: implementation",
        ));
    };
    let flag = |keyword: &str| arg(keyword).and_then(|v| v.unpack_bool());
    // An analysis test is a test: its class name must say so.
    let test = flag("test").unwrap_or(false) || flag("analysis_test").unwrap_or(false);
    let executable = flag("executable").unwrap_or(false);

    // The rule's own attributes.
    let mut own: Vec<SchemaAttr> = Vec::new();
    let mut descriptors: Vec<Value<'v>> = Vec::new();
    let mut attr_names: Vec<String> = Vec::new();
    let mut add_attrs = |attrs: DictRef<'v>, skip_existing: bool| -> starlark::Result<()> {
        for (name, descriptor) in attrs.iter() {
            let view = attribute_view(descriptor).expect("checked as bound");
            let name = name.unpack_str().expect("checked as bound");
            if skip_existing && attr_names.iter().any(|n| n == name) {
                continue;
            }
            // `configurable` is for the attributes of built-in rules only.
            if view.def.configurable.is_some() {
                return Err(fatal(format!(
                    "attribute '{name}' has the 'configurable' argument set, which is not \
                     allowed in rule definitions"
                )));
            }
            own.push(SchemaAttr {
                info: fjfj_graph::schema::AttrInfo {
                    values_repr: view.values.iter().map(|v| v.to_repr()).collect(),
                    providers: view
                        .providers
                        .iter()
                        .map(|alternative| {
                            alternative
                                .iter()
                                .filter_map(|p| crate::provider::origin(*p))
                                .collect()
                        })
                        .collect(),
                },
                name: name.to_owned(),
                def: view.def.clone(),
                values: value_strings(view.def.ty, &view.values),
                hidden: false,
                configurable: !matches!(view.def.ty, AttrType::Output | AttrType::OutputList),
                set: false,
            });
            descriptors.push(descriptor);
            attr_names.push(name.to_owned());
        }
        Ok(())
    };
    if let Some(attrs) = arg("attrs").and_then(DictRef::from_value) {
        add_attrs(attrs, false)?;
    }
    // The attributes of the subrules (and of theirs) are the rule's own.
    if let Some(subrules) = arg("subrules") {
        let mut pending = all_of_type("subrules", subrules, heap, "Subrule", "Subrule")?;
        let mut seen = 0;
        while seen < pending.len() {
            let subrule = pending[seen];
            seen += 1;
            if let Some(attrs) =
                crate::decl::subrule_arg(subrule, "attrs").and_then(DictRef::from_value)
            {
                add_attrs(attrs, true)?;
            }
            if let Some(nested) = crate::decl::subrule_arg(subrule, "subrules") {
                pending.extend(all_of_type("subrules", nested, heap, "Subrule", "Subrule")?);
            }
        }
    }

    // `outputs`: templates, or a function of the attributes.
    let mut templates: Vec<(String, String)> = Vec::new();
    let mut outputs_fn: Vec<Value<'v>> = Vec::new();
    match arg("outputs") {
        Some(f) if DictRef::from_value(f).is_none() => outputs_fn.push(f),
        Some(dict) => {
            let dict = DictRef::from_value(dict).expect("a dict");
            if let Some((k, v)) = first_wrong_entry(&dict, |v| v.unpack_str().is_some()) {
                return Err(fatal(format!(
                    "got dict<{}, {}> for 'implicit outputs of the rule class', want \
                     dict<string, string>",
                    k.get_type(),
                    v.get_type()
                )));
            }
            for (k, v) in dict.iter() {
                templates.push((
                    k.unpack_str().expect("checked").to_owned(),
                    v.unpack_str().expect("checked").to_owned(),
                ));
            }
        }
        None => {}
    }
    if let Some(setting) = arg("build_setting").and_then(build_setting_of) {
        for reserved in ["build_setting_default", "help"] {
            if own.iter().any(|a| a.name == reserved) {
                return Err(fatal(format!(
                    "There is already a built-in attribute '{reserved}' which cannot be overridden."
                )));
            }
        }
        own.extend(build_setting_attrs(&setting));
    }
    let mut schema =
        RuleSchema::starlark(own, test, executable, templates).map_err(|e| fatal(e.to_string()))?;
    schema.defined_in = crate::label::evaluating_file(eval);
    schema.definition_stack = crate::native::call_frames(eval);
    schema.incoming_transition = arg("cfg").is_some_and(crate::decl::is_defined_transition);
    schema.build_setting = arg("build_setting")
        .and_then(build_setting_of)
        .map(|s| s.spec());

    if let Some(fragments) = arg("fragments") {
        schema.fragments = strings_of("fragments", fragments, heap)?;
    }
    schema.doc = arg("doc").and_then(|d| d.unpack_str()).map(str::to_owned);
    if let Some(toolchains) = arg("toolchains") {
        for item in sequence(toolchains, heap).unwrap_or_default() {
            if let Some(label) = label_of_value(item) {
                schema.toolchains.push((label, true));
                continue;
            }
            if let Some(requirement) = item.downcast_ref::<crate::decl::ToolchainTypeRequirement>()
            {
                schema
                    .toolchains
                    .push((requirement.label.clone(), requirement.mandatory));
                continue;
            }
            let Some(text) = item.unpack_str() else {
                return Err(fatal(format!(
                    "'toolchains' takes a toolchain_type, Label, or String, but instead got a {}",
                    java_name(item)
                )));
            };
            match parse_in_caller(eval, "rule", text)? {
                Ok(label) => schema.toolchains.push((label, true)),
                Err(e) => {
                    return Err(fatal(format!(
                        "Unable to parse toolchain_type label '{text}': {e}"
                    )));
                }
            }
        }
    }
    let mut provides: Vec<Value<'v>> = Vec::new();
    if let Some(list) = arg("provides") {
        for (i, item) in sequence(list, heap).unwrap_or_default().iter().enumerate() {
            if item.get_type() != "Provider" {
                return Err(fatal(format!(
                    "at index {i} of provides, got element of type {}, want Provider",
                    item.get_type()
                )));
            }
            if is_provider(*item) && !is_exported(*item, eval) {
                return Err(fatal(
                    "Providers should be top-level values in extension files that define them.",
                ));
            }
            provides.push(*item);
        }
    }
    schema.provides = provides
        .iter()
        .filter_map(|p| crate::provider::origin(*p))
        .collect();
    if let Some(list) = arg("exec_compatible_with") {
        for text in strings_of("exec_compatible_with", list, heap)? {
            if let Err(e) = parse_in_caller(eval, "rule", &text)? {
                return Err(fatal(format!(
                    "Unable to parse label '{text}' in attribute 'exec_compatible_with': {e}"
                )));
            }
        }
    }
    let mut declared: Vec<(&'static str, Value<'v>)> = Vec::new();
    let mut exec_groups = Vec::new();
    if let Some(subrules) = arg("subrules") {
        let items = all_of_type("subrules", subrules, heap, "Subrule", "Subrule")?;
        check_subrules_exported(&items, eval)?;
        declared.push(("subrules", subrules));
    }
    if let Some(groups) = arg("exec_groups").and_then(DictRef::from_value) {
        check_exec_groups(&groups)?;
        declared.push(("exec_groups", arg("exec_groups").expect("given")));
        exec_groups = groups
            .iter()
            .filter_map(|(name, group)| {
                let group = group.downcast_ref::<crate::decl::ExecGroup>()?;
                Some(fjfj_graph::schema::ExecGroupSchema {
                    name: name.unpack_str()?.to_owned(),
                    exec_compatible_with: group.exec_compatible_with.clone(),
                    toolchains: group.toolchains.clone(),
                })
            })
            .collect();
    }
    if let Some(cfg) = arg("cfg") {
        if !is_transition(cfg) {
            return Err(fatal(
                "`cfg` must be set to a transition object initialized by the transition() function.",
            ));
        }
        if arg("build_setting").is_some() {
            return Err(fatal(
                "Build setting rules cannot use the `cfg` param to apply transitions to themselves.",
            ));
        }
        declared.push(("cfg", cfg));
    }
    if let Some(setting) = arg("build_setting") {
        declared.push(("build_setting", setting));
    }
    if let Some(parent) = arg("parent") {
        return Err(fatal(format!(
            "Parent needs to be a Starlark rule, was {}",
            parent.get_type()
        )));
    }
    if let Some(text) = arg("extendable").and_then(|v| v.unpack_str())
        && let Err(e) = parse_in_caller(eval, "rule", text)?
    {
        return Err(fatal(format!("Unable to parse label '{text}': {e}")));
    }

    schema.exec_groups = exec_groups;
    let doc = arg("doc").and_then(|d| d.unpack_str()).map(str::to_owned);
    let name = OnceLock::new();
    name_at_assignment(eval, Kind::Rule { test }, &name)?;
    Ok(heap.alloc_complex(RuleGen {
        id: next_id(),
        schema: Arc::new(schema),
        implementation,
        attrs: descriptors,
        attr_names,
        outputs: outputs_fn,
        provides,
        declared_names: declared.iter().map(|(n, _)| *n).collect(),
        declared: declared.iter().map(|(_, v)| *v).collect(),
        doc,
        name,
    }))
}

#[starlark_module]
pub(crate) fn rule_globals(builder: &mut GlobalsBuilder) {
    /// `rule(implementation, *, test, attrs, outputs, executable, ...)`.
    fn rule<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_rule(args, eval)
    }
}
