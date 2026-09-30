//! The values that only declare (buildfiji-mum.3.7): `aspect()`,
//! `transition()`, `analysis_test_transition()`, `exec_group()`,
//! `configuration_field()`, `subrule()` and the `config` namespace
//! (`config.bool()`, `config.exec()`, ...).
//!
//! Each checks its arguments with Bazel 9.2.0's words and returns an inert,
//! freezable value that keeps what a later bead reads (analysis, transitions,
//! toolchain resolution): the validated arguments are kept by parameter name
//! ([`aspect_arg`], [`subrule_arg`]), and what compares by value is kept in a
//! form that does.
//!
//! What Bazel does, all read off probes:
//!
//! - **Order of checks.** Type errors of the parameters come first, in the
//!   order the arguments were written (positional ones first), then the
//!   contents of the arguments. For `aspect()` the contents are checked in
//!   this order: the shape of `attrs`, `subrules`, what `attrs` says of its
//!   attributes, `exec_compatible_with`, `exec_groups`, `toolchains`,
//!   `attr_aspects`, `toolchains_aspects`, `required_providers`,
//!   `required_aspect_providers`, `provides`, `requires` and `fragments`.
//! - **Naming.** An aspect and a subrule are named by the top-level name they
//!   are bound to, like a rule. An aspect always prints `<aspect>`; a subrule
//!   prints `<subrule NAME>`, or `<subrule unexported subrule>` if it has
//!   none. Only `aspect()` and `configuration_field()` refuse to run outside
//!   `.bzl` initialization.
//! - **Transitions.** `transition()` checks its settings (`//command_line_option:x`
//!   or a label), `and_then` joins two, and every transition has the type
//!   `transition`; `config.exec()` has type `ExecTransitionFactory`.
//!   Two transitions are equal when they have the same implementation and
//!   the same settings, in any order, as written; a `config.target()` equals
//!   any other, and so does a `config.none()`; nothing else is equal but
//!   itself.

use crate::args::{Param, Wording, bind, fatal, param, positional_only};
use crate::attr::{provider_alternatives, sequence, view as attribute_view};
use crate::exports::{Kind, Named, name_at_assignment, next_id, resolve_name};
use crate::label::{display_label, evaluating_bzl, label_of_value, parse_in_caller};
use allocative::Allocative;
use fjfj_graph::Label;
use fjfj_graph::rule::{AttrDef, AttrFlag, AttrType, AttrValue};
use fjfj_graph::schema::SchemaAttr;
use starlark::environment::{GlobalsBuilder, Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::DictRef;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;
use std::hash::Hash;
use std::sync::OnceLock;

// ---- checking arguments ---------------------------------------------------------------

/// One parameter of a declaration builtin.
#[derive(Clone, Copy)]
pub(crate) struct P {
    pub(crate) name: &'static str,
    /// Can be given by position (in the order listed).
    pub(crate) positional: bool,
    pub(crate) required: bool,
    /// How the signature words the type it takes.
    pub(crate) want: &'static str,
    pub(crate) ok: fn(Value<'_>) -> bool,
    /// The flag that turns an experimental parameter on; while it is off
    /// the parameter is refused whatever it is given.
    pub(crate) experimental: Option<&'static str>,
    /// Can only be given by position.
    pub(crate) positional_only: bool,
}

impl P {
    /// This parameter can only be given by position.
    pub(crate) const fn positional_only(mut self) -> P {
        self.positional_only = true;
        self
    }

    /// This parameter is experimental, behind `flag`.
    pub(crate) const fn experimental(mut self, flag: &'static str) -> P {
        self.experimental = Some(flag);
        self
    }
}

pub(crate) const fn p(
    name: &'static str,
    positional: bool,
    required: bool,
    want: &'static str,
    ok: fn(Value<'_>) -> bool,
) -> P {
    P {
        name,
        positional,
        required,
        want,
        ok,
        experimental: None,
        positional_only: false,
    }
}

pub(crate) fn is_sequence(v: Value<'_>) -> bool {
    matches!(v.get_type(), "list" | "tuple" | "range")
}
pub(crate) fn is_function(v: Value<'_>) -> bool {
    v.get_type() == "function"
}
fn is_callable(v: Value<'_>) -> bool {
    matches!(
        v.get_type(),
        "function" | "Provider" | "RawConstructor" | "rule"
    )
}
pub(crate) fn is_dict(v: Value<'_>) -> bool {
    DictRef::from_value(v).is_some()
}
pub(crate) fn is_bool(v: Value<'_>) -> bool {
    v.unpack_bool().is_some()
}
fn is_string(v: Value<'_>) -> bool {
    v.unpack_str().is_some()
}
fn is_sequence_or_function(v: Value<'_>) -> bool {
    is_sequence(v) || is_function(v)
}
fn is_function_or_none(v: Value<'_>) -> bool {
    v.is_none() || is_function(v)
}
pub(crate) fn is_string_or_none(v: Value<'_>) -> bool {
    v.is_none() || is_string(v)
}
fn is_dict_or_none(v: Value<'_>) -> bool {
    v.is_none() || is_dict(v)
}

/// Bind `args` to `params` as Bazel does and check the type of each as it
/// was given, in the order it was given. Returns the values by parameter.
pub(crate) fn bind_checked<'v>(
    function: &str,
    params: &[P],
    args: &Arguments<'v, '_>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<Option<Value<'v>>>> {
    // Bazel converts each argument to its parameter's type as it reads it,
    // before it finds that a parameter was given twice or too many were
    // given: the positional ones, then the named, in the order written.
    let wrong = |p: &P, value: Value<'_>| match p.experimental {
        Some(flag) => fatal(format!(
            "in call to {function}(), parameter '{}' is experimental and thus unavailable with \
             the current flags. It may be enabled by setting {flag}",
            p.name
        )),
        None => fatal(format!(
            "in call to {function}(), parameter '{}' got value of type '{}', want '{}'",
            p.name,
            value.get_type(),
            p.want
        )),
    };
    let slots: Vec<usize> = (0..params.len())
        .filter(|&i| params[i].positional)
        .collect();
    for (value, &slot) in args.positions(eval.heap())?.zip(&slots) {
        if params[slot].experimental.is_some() || !(params[slot].ok)(value) {
            return Err(wrong(&params[slot], value));
        }
    }
    for (key, value) in args.names_map()?.iter() {
        if let Some(p) = params.iter().find(|p| p.name == key.as_str())
            && (p.experimental.is_some() || !(p.ok)(*value))
        {
            return Err(wrong(p, *value));
        }
    }
    let bind_params: Vec<Param> = params
        .iter()
        .map(|p| {
            if p.positional_only {
                positional_only(p.name, p.required)
            } else {
                param(p.name, p.positional, p.required)
            }
        })
        .collect();
    bind(function, Wording::Signature, &bind_params, args, eval)
}

fn not_initializing(function: &str) -> starlark::Error {
    fatal(format!(
        "{function}() can only be used during .bzl initialization (top-level evaluation)"
    ))
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

/// The labels of `exec_compatible_with`: strings, read in the `.bzl` making
/// the call.
fn exec_constraints<'v>(
    function: &str,
    value: Value<'v>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<Label>> {
    let mut labels = Vec::new();
    for text in strings_of("exec_compatible_with", value, eval.heap())? {
        match parse_in_caller(eval, function, &text)? {
            Ok(label) => labels.push(label),
            Err(e) => {
                return Err(fatal(format!(
                    "Unable to parse label '{text}' in attribute 'exec_compatible_with': {e}"
                )));
            }
        }
    }
    Ok(labels)
}

/// The Java class Bazel words a wrong `toolchains` element with.
fn java_name(value: Value<'_>) -> String {
    match value.get_type() {
        "int" => "Int32".to_owned(),
        "Attribute" => "Descriptor".to_owned(),
        "bool" => "Boolean".to_owned(),
        other => other.to_owned(),
    }
}

/// The labels of `toolchains`: strings or `Label`s.
fn toolchain_types<'v>(
    function: &str,
    value: Value<'v>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<Label>> {
    let mut labels = Vec::new();
    for item in sequence(value, eval.heap()).unwrap_or_default() {
        if let Some(label) = label_of_value(item) {
            labels.push(label);
            continue;
        }
        let Some(text) = item.unpack_str() else {
            return Err(fatal(format!(
                "'toolchains' takes a toolchain_type, Label, or String, but instead got a {}",
                java_name(item)
            )));
        };
        match parse_in_caller(eval, function, text)? {
            Ok(label) => labels.push(label),
            Err(e) => {
                return Err(fatal(format!(
                    "Unable to parse toolchain_type label '{text}': {e}"
                )));
            }
        }
    }
    Ok(labels)
}

/// `fragments`, whose elements are strings.
fn fragment_names<'v>(value: Value<'v>, heap: Heap<'v>) -> starlark::Result<Vec<String>> {
    strings_of("fragments", value, heap)
}

/// An exec group's name must be an identifier.
fn valid_exec_group_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A dict of `exec_group()`s, as `exec_groups` takes.
pub(crate) fn check_exec_groups<'v>(dict: &DictRef<'v>) -> starlark::Result<()> {
    if let Some((k, v)) = dict
        .iter()
        .find(|(k, v)| k.unpack_str().is_none() || v.get_type() != "exec_group")
    {
        return Err(fatal(format!(
            "got dict<{}, {}> for 'exec_group', want dict<string, exec_group>",
            k.get_type(),
            v.get_type()
        )));
    }
    for (k, _) in dict.iter() {
        let name = k.unpack_str().expect("checked");
        if !valid_exec_group_name(name) {
            return Err(fatal(format!(
                "Exec group name '{name}' is not a valid name."
            )));
        }
    }
    Ok(())
}

/// Every element of `value` must be of `ty`, named `what` in the error.
pub(crate) fn all_of_type<'v>(
    keyword: &str,
    value: Value<'v>,
    heap: Heap<'v>,
    ty: &str,
    what: &str,
) -> starlark::Result<Vec<Value<'v>>> {
    let items = sequence(value, heap).unwrap_or_default();
    for (i, item) in items.iter().enumerate() {
        if item.get_type() != ty {
            return Err(fatal(format!(
                "at index {i} of {keyword}, got element of type {}, want {what}",
                item.get_type()
            )));
        }
    }
    Ok(items)
}

/// A subrule must have been bound to a name by the file that made it.
pub(crate) fn check_subrules_exported<'v>(
    subrules: &[Value<'v>],
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<()> {
    for subrule in subrules {
        if resolve_name(*subrule, eval)?.is_none() {
            return Err(fatal("Invalid subrule hasn't been exported by a bzl file"));
        }
    }
    Ok(())
}

/// What `attrs` of an aspect or subrule may hold, checked one attribute at a
/// time; `check` is given the attribute's name and view.
fn attribute_names<'v>(dict: &DictRef<'v>) -> starlark::Result<()> {
    if let Some((k, v)) = dict
        .iter()
        .find(|(k, v)| k.unpack_str().is_none() || attribute_view(*v).is_none())
    {
        return Err(fatal(format!(
            "got dict<{}, {}> for 'attrs', want dict<string, Attribute>",
            k.get_type(),
            v.get_type()
        )));
    }
    for (name, _) in dict.iter() {
        let name = name.unpack_str().expect("checked");
        if !is_identifier(name) {
            return Err(fatal(format!(
                "attribute name `{name}` is not a valid identifier."
            )));
        }
    }
    Ok(())
}

pub(crate) fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ---- aspect ---------------------------------------------------------------------------

/// An aspect, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct AspectGen<V> {
    #[trace(static)]
    id: u64,
    /// The arguments it was made with, by parameter name.
    #[trace(static)]
    #[allocative(skip)]
    names: Vec<&'static str>,
    args: Vec<V>,
    #[trace(static)]
    #[allocative(skip)]
    name: OnceLock<String>,
}

starlark_complex_value!(pub(crate) Aspect);

impl<'v> Freeze for Aspect<'v> {
    type Frozen = FrozenAspect;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenAspect> {
        Ok(AspectGen {
            id: self.id,
            names: self.names,
            args: self
                .args
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
            name: self.name,
        })
    }
}

impl<V> fmt::Display for AspectGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<aspect>")
    }
}

fn aspect_identity<'v>(value: Value<'v>) -> Option<(u64, &'v OnceLock<String>)> {
    fn of<'v, V: ValueLike<'v>>(a: &'v AspectGen<V>) -> (u64, &'v OnceLock<String>) {
        (a.id, &a.name)
    }
    if let Some(live) = value.downcast_ref::<Aspect<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenAspect>().map(of)
    }
}

/// What the aspect `value` was made with for the parameter `name`: `None` if
/// it was not given.
#[allow(dead_code)]
pub(crate) fn aspect_arg<'v>(value: Value<'v>, name: &str) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(a: &AspectGen<V>, name: &str) -> Option<Value<'v>> {
        let at = a.names.iter().position(|n| *n == name)?;
        Some(a.args[at].to_value())
    }
    if let Some(live) = value.downcast_ref::<Aspect<'v>>() {
        of(live, name)
    } else {
        value
            .downcast_ref::<FrozenAspect>()
            .and_then(|a| of(a, name))
    }
}

#[starlark_value(type = "Aspect")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for AspectGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        self.id.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(aspect_identity(other).is_some_and(|(id, _)| id == self.id))
    }
}

const ASPECT_PARAMS: &[P] = &[
    p("implementation", true, true, "function", is_function),
    p(
        "attr_aspects",
        true,
        false,
        "sequence or function",
        is_sequence_or_function,
    ),
    p(
        "toolchains_aspects",
        true,
        false,
        "sequence or function",
        is_sequence_or_function,
    ),
    p("attrs", true, false, "dict", is_dict),
    p("required_providers", true, false, "sequence", is_sequence),
    p(
        "required_aspect_providers",
        true,
        false,
        "sequence",
        is_sequence,
    ),
    p("provides", true, false, "sequence", is_sequence),
    p("requires", true, false, "sequence", is_sequence),
    p(
        "propagation_predicate",
        true,
        false,
        "function or NoneType",
        is_function_or_none,
    ),
    p("fragments", true, false, "sequence", is_sequence),
    p("host_fragments", true, false, "sequence", is_sequence),
    p("toolchains", true, false, "sequence", is_sequence),
    p("doc", true, false, "string or NoneType", is_string_or_none),
    p(
        "exec_compatible_with",
        false,
        false,
        "sequence",
        is_sequence,
    ),
    p(
        "exec_groups",
        false,
        false,
        "dict or NoneType",
        is_dict_or_none,
    ),
    p("subrules", false, false, "sequence", is_sequence),
    p("apply_to_generating_rules", false, false, "bool", is_bool),
];

/// Whether an aspect or subrule's own attribute `name` is acceptable, which
/// differs between the two only in the words (and in what they allow).
fn check_aspect_attrs<'v>(dict: &DictRef<'v>) -> starlark::Result<()> {
    for (name, descriptor) in dict.iter() {
        let name = name.unpack_str().expect("checked");
        let view = attribute_view(descriptor).expect("checked");
        if name.starts_with('_') {
            // A private attribute is the aspect's own, and must have a value.
            let valued = view.def.default.is_some()
                || view.def.ty.zero().is_some()
                || view.computed.is_some();
            if !valued {
                return Err(fatal(format!(
                    "Aspect attribute '{name}' has no default value."
                )));
            }
        } else if !matches!(
            view.def.ty,
            fjfj_graph::rule::AttrType::Bool
                | fjfj_graph::rule::AttrType::Int
                | fjfj_graph::rule::AttrType::String
        ) {
            return Err(fatal(format!(
                "Aspect parameter attribute '{name}' must have type 'bool', 'int' or 'string'."
            )));
        }
    }
    Ok(())
}

fn make_aspect<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    if !evaluating_bzl(eval) {
        return Err(not_initializing("aspect"));
    }
    let bound = bind_checked("aspect", ASPECT_PARAMS, args, eval)?;
    let heap = eval.heap();
    let arg = |name: &str| {
        let i = ASPECT_PARAMS
            .iter()
            .position(|p| p.name == name)
            .expect("known");
        bound[i]
    };
    if let Some(attrs) = arg("attrs").and_then(DictRef::from_value) {
        attribute_names(&attrs)?;
    }
    if let Some(subrules) = arg("subrules") {
        let items = all_of_type("subrules", subrules, heap, "Subrule", "Subrule")?;
        check_subrules_exported(&items, eval)?;
    }
    if let Some(attrs) = arg("attrs").and_then(DictRef::from_value) {
        check_aspect_attrs(&attrs)?;
    }
    if let Some(value) = arg("exec_compatible_with") {
        exec_constraints("aspect", value, eval)?;
    }
    if let Some(groups) = arg("exec_groups").and_then(DictRef::from_value) {
        check_exec_groups(&groups)?;
    }
    if let Some(value) = arg("toolchains") {
        toolchain_types("aspect", value, eval)?;
    }
    for keyword in ["attr_aspects", "toolchains_aspects"] {
        if let Some(value) = arg(keyword).filter(|v| is_sequence(*v)) {
            strings_of(keyword, value, heap)?;
        }
    }
    for keyword in ["required_providers", "required_aspect_providers"] {
        if let Some(value) = arg(keyword) {
            provider_alternatives(keyword, value, eval)?;
        }
    }
    if let Some(value) = arg("provides") {
        let items = all_of_type("provides", value, heap, "Provider", "Provider")?;
        crate::attr::check_exported(&items, eval)?;
    }
    if let Some(value) = arg("requires") {
        all_of_type("requires", value, heap, "Aspect", "Aspect")?;
    }
    if let Some(value) = arg("fragments") {
        fragment_names(value, heap)?;
    }
    let mut names = Vec::new();
    let mut values = Vec::new();
    for (p, value) in ASPECT_PARAMS.iter().zip(&bound) {
        if let Some(value) = value {
            names.push(p.name);
            values.push(*value);
        }
    }
    let name = OnceLock::new();
    name_at_assignment(eval, Kind::Aspect, &name)?;
    Ok(heap.alloc_complex(AspectGen {
        id: next_id(),
        names,
        args: values,
        name,
    }))
}

// ---- subrule --------------------------------------------------------------------------

/// A subrule, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct SubruleGen<V> {
    #[trace(static)]
    id: u64,
    #[trace(static)]
    #[allocative(skip)]
    names: Vec<&'static str>,
    args: Vec<V>,
    #[trace(static)]
    #[allocative(skip)]
    name: OnceLock<String>,
}

starlark_complex_value!(pub(crate) Subrule);

impl<'v> Freeze for Subrule<'v> {
    type Frozen = FrozenSubrule;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenSubrule> {
        Ok(SubruleGen {
            id: self.id,
            names: self.names,
            args: self
                .args
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
            name: self.name,
        })
    }
}

impl<V> fmt::Display for SubruleGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<subrule {}>",
            self.name.get().map_or("unexported subrule", String::as_str)
        )
    }
}

fn subrule_identity<'v>(value: Value<'v>) -> Option<(u64, &'v OnceLock<String>)> {
    fn of<'v, V: ValueLike<'v>>(a: &'v SubruleGen<V>) -> (u64, &'v OnceLock<String>) {
        (a.id, &a.name)
    }
    if let Some(live) = value.downcast_ref::<Subrule<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenSubrule>().map(of)
    }
}

/// What the subrule `value` was made with for the parameter `name`.
#[allow(dead_code)]
pub(crate) fn subrule_arg<'v>(value: Value<'v>, name: &str) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(a: &SubruleGen<V>, name: &str) -> Option<Value<'v>> {
        let at = a.names.iter().position(|n| *n == name)?;
        Some(a.args[at].to_value())
    }
    if let Some(live) = value.downcast_ref::<Subrule<'v>>() {
        of(live, name)
    } else {
        value
            .downcast_ref::<FrozenSubrule>()
            .and_then(|a| of(a, name))
    }
}

#[starlark_value(type = "Subrule")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for SubruleGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        me: Value<'v>,
        _args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let name = resolve_name(me, eval)?.unwrap_or_else(|| "unexported subrule".to_owned());
        Err(fatal(format!(
            "{name} can only be called from a rule or aspect implementation"
        )))
    }

    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        self.id.hash(hasher);
        Err(fatal("unhashable type: 'Subrule'"))
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(subrule_identity(other).is_some_and(|(id, _)| id == self.id))
    }
}

const SUBRULE_PARAMS: &[P] = &[
    p("implementation", false, true, "function", is_function),
    p("attrs", false, false, "dict", is_dict),
    p("toolchains", false, false, "sequence", is_sequence),
    p("fragments", false, false, "sequence", is_sequence),
    p("subrules", false, false, "sequence", is_sequence),
];

fn make_subrule<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind_checked("subrule", SUBRULE_PARAMS, args, eval)?;
    let heap = eval.heap();
    let arg = |name: &str| {
        let i = SUBRULE_PARAMS
            .iter()
            .position(|p| p.name == name)
            .expect("known");
        bound[i]
    };
    if let Some(attrs) = arg("attrs").and_then(DictRef::from_value) {
        attribute_names(&attrs)?;
        for (name, descriptor) in attrs.iter() {
            let name = name.unpack_str().expect("checked");
            if !name.starts_with('_') {
                return Err(fatal(format!(
                    "illegal attribute name '{name}': subrules may only define private \
                     attributes (whose names begin with '_')."
                )));
            }
            let view = attribute_view(descriptor).expect("checked");
            let valued = view.def.default.is_some()
                || view.def.ty.zero().is_some()
                || view.computed.is_some();
            if !valued {
                return Err(fatal(format!(
                    "for attribute '{name}': no default value specified"
                )));
            }
            use fjfj_graph::rule::AttrType::{Label, LabelList};
            if !matches!(view.def.ty, Label | LabelList) {
                return Err(fatal(format!(
                    "bad type for attribute '{name}': subrule attributes may only be label or \
                     lists of labels."
                )));
            }
        }
    }
    if let Some(value) = arg("toolchains") {
        toolchain_types("subrule", value, eval)?;
    }
    if let Some(value) = arg("fragments") {
        fragment_names(value, heap)?;
    }
    if let Some(value) = arg("subrules") {
        all_of_type("subrules", value, heap, "Subrule", "Subrule")?;
    }
    let mut names = Vec::new();
    let mut values = Vec::new();
    for (p, value) in SUBRULE_PARAMS.iter().zip(&bound) {
        if let Some(value) = value {
            names.push(p.name);
            values.push(*value);
        }
    }
    let name = OnceLock::new();
    name_at_assignment(eval, Kind::Subrule, &name)?;
    Ok(heap.alloc_complex(SubruleGen {
        id: next_id(),
        names,
        args: values,
        name,
    }))
}

// ---- exec_group, configuration_field, build settings ----------------------------------

/// An `exec_group()`: constraints and toolchain types, as labels. Two are
/// equal when they name the same sets of labels.
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ExecGroup {
    #[allocative(skip)]
    pub(crate) exec_compatible_with: Vec<Label>,
    #[allocative(skip)]
    pub(crate) toolchains: Vec<Label>,
}

starlark_simple_value!(ExecGroup);

impl fmt::Display for ExecGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"
        )
    }
}

fn label_set(labels: &[Label]) -> Vec<String> {
    let mut set: Vec<String> = labels.iter().map(display_label).collect();
    set.sort();
    set.dedup();
    set
}

#[starlark_value(type = "exec_group")]
impl<'v> StarlarkValue<'v> for ExecGroup {
    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        let Some(other) = other.downcast_ref::<ExecGroup>() else {
            return Ok(false);
        };
        Ok(
            label_set(&self.exec_compatible_with) == label_set(&other.exec_compatible_with)
                && label_set(&self.toolchains) == label_set(&other.toolchains),
        )
    }
}

const EXEC_GROUP_PARAMS: &[P] = &[
    p(
        "exec_compatible_with",
        false,
        false,
        "sequence",
        is_sequence,
    ),
    p("toolchains", false, false, "sequence", is_sequence),
];

fn make_exec_group<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind_checked("exec_group", EXEC_GROUP_PARAMS, args, eval)?;
    let constraints = match bound[0] {
        Some(v) => exec_constraints("exec_group", v, eval)?,
        None => Vec::new(),
    };
    let toolchains = match bound[1] {
        Some(v) => toolchain_types("exec_group", v, eval)?,
        None => Vec::new(),
    };
    Ok(eval.heap().alloc(ExecGroup {
        exec_compatible_with: constraints,
        toolchains,
    }))
}

/// The fragments, and the late-bound fields each has, that Bazel 9.2.0
/// lets `configuration_field` name.
const CONFIGURATION_FIELDS: &[(&str, &[&str])] = &[
    ("apple", &["xcode_config_label"]),
    ("bazel_py", &[]),
    ("coverage", &["output_generator"]),
    (
        "cpp",
        &[
            "zipper",
            "libc_top",
            "fdo_profile",
            "fdo_prefetch_hints",
            "memprof_profile",
            "propeller_optimize",
            "custom_malloc",
            "cs_fdo_profile",
        ],
    ),
    ("j2objc", &[]),
    (
        "java",
        &[
            "bytecode_optimizer",
            "local_java_optimization_configuration",
            "launcher",
        ],
    ),
    ("objc", &[]),
    ("platform", &[]),
    (
        "proto",
        &[
            "proto_compiler",
            "proto_toolchain_for_java",
            "proto_toolchain_for_cc",
            "proto_toolchain_for_java_lite",
        ],
    ),
    ("py", &["native_rules_allowlist"]),
    ("android", &["legacy_main_dex_list_generator"]),
    ("bazel_android", &[]),
];

/// A `configuration_field()`: a label attribute's default that is read from
/// the configuration.
#[derive(Debug, Clone, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct LateBoundDefault {
    pub(crate) fragment: String,
    pub(crate) name: String,
}

starlark_simple_value!(LateBoundDefault);

impl fmt::Display for LateBoundDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<late-bound default>")
    }
}

#[starlark_value(type = "LateBoundDefault")]
impl<'v> StarlarkValue<'v> for LateBoundDefault {
    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other.downcast_ref::<LateBoundDefault>() == Some(self))
    }
}

const CONFIGURATION_FIELD_PARAMS: &[P] = &[
    p("fragment", true, true, "string", is_string),
    p("name", true, true, "string", is_string),
];

fn make_configuration_field<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    if !evaluating_bzl(eval) {
        return Err(not_initializing("configuration_field"));
    }
    let bound = bind_checked(
        "configuration_field",
        CONFIGURATION_FIELD_PARAMS,
        args,
        eval,
    )?;
    let fragment = bound[0].and_then(|v| v.unpack_str()).expect("required");
    let name = bound[1].and_then(|v| v.unpack_str()).expect("required");
    let Some((_, fields)) = CONFIGURATION_FIELDS.iter().find(|(f, _)| *f == fragment) else {
        return Err(fatal(format!(
            "invalid configuration fragment name '{fragment}'"
        )));
    };
    if !fields.contains(&name) {
        return Err(fatal(format!(
            "invalid configuration field name '{name}' on fragment '{fragment}'"
        )));
    }
    Ok(eval.heap().alloc(LateBoundDefault {
        fragment: fragment.to_owned(),
        name: name.to_owned(),
    }))
}

/// What `config.bool()` and its kind make: the type of a build setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingType {
    Bool,
    Int,
    String,
    StringList,
    StringSet,
}

/// A build setting, as `rule(build_setting = ...)` takes one.
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct BuildSetting {
    id: u64,
    #[allocative(skip)]
    pub(crate) ty: SettingType,
    pub(crate) flag: bool,
    /// `allow_multiple` of a string, `repeatable` of a list or set.
    pub(crate) multiple: bool,
}

starlark_simple_value!(BuildSetting);

impl fmt::Display for BuildSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<build_setting.{}>",
            match self.ty {
                SettingType::Bool => "boolean",
                SettingType::Int => "int",
                SettingType::String => "string",
                SettingType::StringList => "list(string)",
                SettingType::StringSet => "set(string)",
            }
        )
    }
}

#[starlark_value(type = "BuildSetting")]
impl<'v> StarlarkValue<'v> for BuildSetting {
    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<BuildSetting>()
            .is_some_and(|o| o.id == self.id))
    }
}

/// The two attributes a build setting rule has after its own: its default
/// (mandatory, of the setting's type) and `help`.
pub(crate) fn build_setting_attrs(setting: &BuildSetting) -> Vec<SchemaAttr> {
    let ty = match setting.ty {
        SettingType::Bool => AttrType::Bool,
        SettingType::Int => AttrType::Int,
        SettingType::String => AttrType::String,
        SettingType::StringList | SettingType::StringSet => AttrType::StringList,
    };
    let mut def = AttrDef::new(ty);
    def.flags.insert(AttrFlag::Mandatory);
    let mut default = SchemaAttr::new("build_setting_default", def);
    if setting.ty == SettingType::StringSet {
        // What `existing_rule` shows of a set is nothing.
        default.set = true;
        default.hidden = true;
    }
    let mut help = AttrDef::new(AttrType::String);
    help.default = Some(AttrValue::String(String::new()));
    vec![default, SchemaAttr::new("help", help)]
}

/// The build setting `value` is, if it is one.
pub(crate) fn build_setting_of(value: Value<'_>) -> Option<BuildSetting> {
    value.downcast_ref::<BuildSetting>().cloned()
}

fn make_setting<'v>(
    function: &'static str,
    ty: SettingType,
    second: Option<&'static str>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let mut params = vec![p("flag", false, false, "bool", is_bool)];
    if let Some(name) = second {
        params.push(p(name, false, false, "bool", is_bool));
    }
    let bound = bind_checked(function, &params, args, eval)?;
    let flag = bound[0].and_then(|v| v.unpack_bool()).unwrap_or(false);
    let multiple = bound
        .get(1)
        .copied()
        .flatten()
        .and_then(|v| v.unpack_bool())
        .unwrap_or(false);
    if multiple && !flag && matches!(ty, SettingType::StringList | SettingType::StringSet) {
        return Err(fatal(
            "'repeatable' can only be set for a setting with 'flag = True'",
        ));
    }
    Ok(eval.heap().alloc(BuildSetting {
        id: next_id(),
        ty,
        flag,
        multiple,
    }))
}

// ---- transitions ----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionKind {
    /// `transition(implementation, inputs, outputs)`.
    Defined,
    /// `analysis_test_transition(settings)`.
    AnalysisTest,
    /// `a.and_then(b)`.
    Composed,
    /// `config.target()`.
    Target,
    /// `config.none()`.
    None,
}

/// A transition, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct TransitionGen<V> {
    #[trace(static)]
    id: u64,
    #[trace(static)]
    #[allocative(skip)]
    kind: TransitionKind,
    /// The implementation of a defined one, the two parts of a composed one,
    /// or the values of an analysis test's settings.
    values: Vec<V>,
    /// The settings as written: inputs of a defined transition, and its
    /// outputs or the keys of an analysis test's settings.
    #[trace(static)]
    #[allocative(skip)]
    inputs: Vec<String>,
    #[trace(static)]
    #[allocative(skip)]
    outputs: Vec<String>,
}

starlark_complex_value!(pub(crate) Transition);

impl<'v> Freeze for Transition<'v> {
    type Frozen = FrozenTransition;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenTransition> {
        Ok(TransitionGen {
            id: self.id,
            kind: self.kind,
            values: self
                .values
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
            inputs: self.inputs,
            outputs: self.outputs,
        })
    }
}

impl<V> fmt::Display for TransitionGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TransitionKind::Defined | TransitionKind::Composed => write!(f, "<transition object>"),
            TransitionKind::AnalysisTest => write!(f, "<analysis_test_transition object>"),
            TransitionKind::Target => write!(
                f,
                "<unknown object \
                 com.google.devtools.build.lib.analysis.config.transitions.NoTransition$Factory>"
            ),
            TransitionKind::None => write!(
                f,
                "<unknown object \
                 com.google.devtools.build.lib.analysis.config.transitions.NoConfigTransition$Factory>"
            ),
        }
    }
}

/// A transition's parts as a reader sees them.
struct TransitionView<'v> {
    id: u64,
    kind: TransitionKind,
    values: Vec<Value<'v>>,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

fn transition_view<'v>(value: Value<'v>) -> Option<TransitionView<'v>> {
    fn of<'v, V: ValueLike<'v>>(t: &TransitionGen<V>) -> TransitionView<'v> {
        TransitionView {
            id: t.id,
            kind: t.kind,
            values: t.values.iter().map(|v| v.to_value()).collect(),
            inputs: t.inputs.clone(),
            outputs: t.outputs.clone(),
        }
    }
    if let Some(live) = value.downcast_ref::<Transition<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenTransition>().map(of)
    }
}

/// Whether `value` is a transition (of any kind but `config.exec()`).
pub(crate) fn is_transition(value: Value<'_>) -> bool {
    transition_view(value).is_some()
}

/// Whether `value` is `config.target()`.
pub(crate) fn is_target_transition(value: Value<'_>) -> bool {
    transition_view(value).is_some_and(|t| t.kind == TransitionKind::Target)
}

/// Whether two attributes' `cfg` are the same one. Only what has no state of
/// its own is: `config.none()` is `config.none()`; a transition of the
/// `.bzl` is never the same as another, nor as itself, and a
/// `config.exec()` only as itself.
pub(crate) fn same_transition_in_attribute<'v>(
    x: Value<'v>,
    y: Value<'v>,
) -> starlark::Result<bool> {
    match (transition_view(x), transition_view(y)) {
        (Some(a), Some(b)) => Ok(a.kind == b.kind && a.kind == TransitionKind::None),
        (Some(_), None) | (None, Some(_)) => Ok(false),
        (None, None) => x.equals(y),
    }
}

fn sorted(items: &[String]) -> Vec<&String> {
    let mut v: Vec<&String> = items.iter().collect();
    v.sort();
    v
}

#[starlark_value(type = "transition")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for TransitionGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("transition", transition_methods);
        Some(RES.methods())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        let Some(them) = transition_view(other) else {
            return Ok(false);
        };
        if them.kind != self.kind {
            return Ok(false);
        }
        match self.kind {
            TransitionKind::Target | TransitionKind::None => Ok(true),
            TransitionKind::Composed => Ok(them.id == self.id),
            TransitionKind::Defined => Ok(self.values[0].to_value().equals(them.values[0])?
                && sorted(&self.inputs) == sorted(&them.inputs)
                && sorted(&self.outputs) == sorted(&them.outputs)),
            TransitionKind::AnalysisTest => {
                if sorted(&self.outputs) != sorted(&them.outputs) {
                    return Ok(false);
                }
                for (k, v) in self.outputs.iter().zip(&self.values) {
                    let at = them.outputs.iter().position(|o| o == k).expect("same keys");
                    if !v.to_value().equals(them.values[at])? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }
}

#[starlark_module]
fn transition_methods(builder: &mut MethodsBuilder) {
    /// `and_then(transition)`: this transition, then that one.
    fn and_then<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "and_then",
            Wording::Signature,
            &[param("transition", true, true)],
            args,
            eval,
        )?;
        let next = bound[0].expect("required");
        if !is_transition(next) && next.get_type() != "ExecTransitionFactory" {
            return Err(fatal(format!(
                "in call to and_then(), parameter 'transition' got value of type '{}', want \
                 'transition'",
                next.get_type()
            )));
        }
        Ok(eval.heap().alloc_complex(TransitionGen {
            id: next_id(),
            kind: TransitionKind::Composed,
            values: vec![this, next],
            inputs: Vec::new(),
            outputs: Vec::new(),
        }))
    }
}

/// `//command_line_option:x` is a native option; anything else is a label.
/// `what` is `INPUTS` or `OUTPUTS`, and `noun` `input` or `output`. Returns
/// the canonical text of the setting.
fn check_setting<'v>(
    text: &str,
    what: &str,
    noun: &str,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<String> {
    const NATIVE: &str = "//command_line_option:";
    if let Some(option) = text.strip_prefix(NATIVE) {
        if option.is_empty() || option.contains(':') {
            return Err(fatal(format!(
                "Malformed label in transition {what} parameter: '{text}'"
            )));
        }
        return Ok(text.to_owned());
    }
    let invalid = |e: &dyn fmt::Display| {
        fatal(format!(
            "invalid transition {noun} '{text}'. If this is intended as a native option, it \
             must begin with //command_line_option: {e}"
        ))
    };
    // Only an absolute label is one here; what is wrong with the name comes
    // first when the name is empty.
    if !text.starts_with('@') && !text.starts_with("//") {
        let name = text.rsplit_once(':').map_or(text, |(_, n)| n);
        if name.is_empty() {
            let e = Label::parse(
                "//x:",
                fjfj_graph::LabelContext {
                    repo: "",
                    package: "",
                },
            )
            .expect_err("empty name");
            return Err(invalid(&e));
        }
        return Err(invalid(&fjfj_graph::LabelParseError::NotAbsolute(
            text.to_owned(),
        )));
    }
    match parse_in_caller(eval, "transition", text)? {
        Ok(label) if label.repo.starts_with('[') => Err(fatal(format!(
            "invalid transition {noun} '{}': no repo visible as @{} from main repository",
            display_label(&label),
            text.trim_start_matches('@')
                .split("//")
                .next()
                .unwrap_or_default()
        ))),
        Ok(label) => Ok(display_label(&label)),
        Err(e) => Err(invalid(&e)),
    }
}

/// The settings of one list, checked one at a time: each must be a setting
/// Bazel can read, and none may appear twice.
fn check_settings<'v>(
    texts: &[String],
    what: &str,
    noun: &str,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<()> {
    let mut seen: Vec<(String, String)> = Vec::new();
    for text in texts {
        if seen.iter().any(|(_, written)| written == text) {
            return Err(fatal(format!("duplicate transition {noun} '{text}'")));
        }
        let canonical = check_setting(text, what, noun, eval)?;
        if let Some((_, earlier)) = seen.iter().find(|(c, _)| *c == canonical) {
            return Err(fatal(format!(
                "Transition declares duplicate build setting '{canonical}' in {what} (specified \
                 as '{text}' and '{earlier}')"
            )));
        }
        seen.push((canonical, text.clone()));
    }
    Ok(())
}

const TRANSITION_PARAMS: &[P] = &[
    p("implementation", false, true, "callable", is_callable),
    p("inputs", false, true, "sequence", is_sequence),
    p("outputs", false, true, "sequence", is_sequence),
];

fn make_transition<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind_checked("transition", TRANSITION_PARAMS, args, eval)?;
    let heap = eval.heap();
    let inputs = strings_of("inputs", bound[1].expect("required"), heap)?;
    let outputs = strings_of("outputs", bound[2].expect("required"), heap)?;
    check_settings(&inputs, "INPUTS", "input", eval)?;
    check_settings(&outputs, "OUTPUTS", "output", eval)?;
    Ok(heap.alloc_complex(TransitionGen {
        id: next_id(),
        kind: TransitionKind::Defined,
        values: vec![bound[0].expect("required")],
        inputs,
        outputs,
    }))
}

fn make_analysis_test_transition<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let params = [p("settings", false, true, "dict", is_dict)];
    let bound = bind_checked("analysis_test_transition", &params, args, eval)?;
    let dict = DictRef::from_value(bound[0].expect("required")).expect("checked");
    if let Some((k, v)) = dict.iter().find(|(k, _)| k.unpack_str().is_none()) {
        return Err(fatal(format!(
            "got dict<{}, {}> for 'changed_settings dict', want dict<string, unknown>",
            k.get_type(),
            v.get_type()
        )));
    }
    let keys: Vec<String> = dict
        .iter()
        .map(|(k, _)| k.unpack_str().expect("checked").to_owned())
        .collect();
    let values: Vec<Value<'v>> = dict.iter().map(|(_, v)| v).collect();
    check_settings(&keys, "OUTPUTS", "output", eval)?;
    Ok(eval.heap().alloc_complex(TransitionGen {
        id: next_id(),
        kind: TransitionKind::AnalysisTest,
        values,
        inputs: Vec::new(),
        outputs: keys,
    }))
}

/// `config.exec(exec_group = None)`.
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ExecTransitionFactory {
    id: u64,
    pub(crate) exec_group: Option<String>,
}

starlark_simple_value!(ExecTransitionFactory);

impl fmt::Display for ExecTransitionFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory>"
        )
    }
}

#[starlark_value(type = "ExecTransitionFactory")]
impl<'v> StarlarkValue<'v> for ExecTransitionFactory {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("transition", transition_methods);
        Some(RES.methods())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<ExecTransitionFactory>()
            .is_some_and(|o| o.id == self.id))
    }
}

fn singleton<'v>(kind: TransitionKind, heap: Heap<'v>) -> Value<'v> {
    heap.alloc_complex(TransitionGen::<Value<'v>> {
        id: next_id(),
        kind,
        values: Vec::new(),
        inputs: Vec::new(),
        outputs: Vec::new(),
    })
}

// ---- globals --------------------------------------------------------------------------

#[starlark_module]
pub(crate) fn decl_globals(builder: &mut GlobalsBuilder) {
    /// `aspect(implementation, attr_aspects, ...)`.
    fn aspect<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_aspect(args, eval)
    }

    /// `subrule(*, implementation, attrs, toolchains, fragments, subrules)`.
    fn subrule<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_subrule(args, eval)
    }

    /// `exec_group(*, exec_compatible_with, toolchains)`.
    fn exec_group<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_exec_group(args, eval)
    }

    /// `configuration_field(fragment, name)`.
    fn configuration_field<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_configuration_field(args, eval)
    }

    /// `transition(*, implementation, inputs, outputs)`.
    fn transition<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_transition(args, eval)
    }

    /// `analysis_test_transition(*, settings)`.
    fn analysis_test_transition<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_analysis_test_transition(args, eval)
    }

    /// The `config` namespace.
    const config: StarlarkConfig = StarlarkConfig;
}

/// The `config` namespace, whose members build settings and transitions.
#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
struct StarlarkConfig;

starlark_simple_value!(StarlarkConfig);

impl fmt::Display for StarlarkConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<config>")
    }
}

#[starlark_value(type = "config")]
impl<'v> StarlarkValue<'v> for StarlarkConfig {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("config", config_members);
        Some(RES.methods())
    }
}

#[starlark_module]
fn config_members(builder: &mut MethodsBuilder) {
    /// `config.exec(exec_group = None)`.
    fn exec<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let params = [p(
            "exec_group",
            true,
            false,
            "string or NoneType",
            is_string_or_none,
        )];
        let bound = bind_checked("exec", &params, args, eval)?;
        let exec_group = bound[0].and_then(|v| v.unpack_str()).map(str::to_owned);
        Ok(eval.heap().alloc(ExecTransitionFactory {
            id: next_id(),
            exec_group,
        }))
    }

    /// `config.target()`.
    fn target<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        bind_checked("target", &[], args, eval)?;
        Ok(singleton(TransitionKind::Target, eval.heap()))
    }

    /// `config.none()`.
    fn none<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        bind_checked("none", &[], args, eval)?;
        Ok(singleton(TransitionKind::None, eval.heap()))
    }

    /// `config.bool(*, flag = False)`.
    fn bool<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_setting("bool", SettingType::Bool, None, args, eval)
    }

    /// `config.int(*, flag = False)`.
    fn int<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_setting("int", SettingType::Int, None, args, eval)
    }

    /// `config.string(*, flag = False, allow_multiple = False)`.
    fn string<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_setting(
            "string",
            SettingType::String,
            Some("allow_multiple"),
            args,
            eval,
        )
    }

    /// `config.string_list(*, flag = False, repeatable = False)`.
    fn string_list<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_setting(
            "string_list",
            SettingType::StringList,
            Some("repeatable"),
            args,
            eval,
        )
    }

    /// `config.string_set(*, flag = False, repeatable = False)`.
    fn string_set<'v>(
        #[starlark(this)] _this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_setting(
            "string_set",
            SettingType::StringSet,
            Some("repeatable"),
            args,
            eval,
        )
    }
}

// ---- naming ---------------------------------------------------------------------------

/// An aspect or a subrule, for [`crate::exports`] to name.
pub(crate) fn named<'v>(value: Value<'v>) -> Option<Named<'v>> {
    if let Some((id, name)) = aspect_identity(value) {
        return Some(Named {
            id,
            name,
            kind: Kind::Aspect,
        });
    }
    subrule_identity(value).map(|(id, name)| Named {
        id,
        name,
        kind: Kind::Subrule,
    })
}
