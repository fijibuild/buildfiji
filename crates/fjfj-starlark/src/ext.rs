//! The values a `.bzl` declares module extensions and repository rules with
//! (buildfiji-mum.8.1): `repository_rule()`, `module_extension()` and
//! `tag_class()`.
//!
//! Each checks its arguments with Bazel 9.2.0's words and returns an inert,
//! freezable value that keeps what the beads that run them read
//! ([`repository_rule_arg`], [`module_extension_arg`], [`tag_class_arg`]).
//! Nothing here runs an implementation function: that is `repository_ctx`
//! (buildfiji-mum.8.2) and `module_ctx` (buildfiji-mum.8.4).
//!
//! What Bazel does, all read off probes:
//!
//! - **Where.** The three are globals of a `.bzl` file only; a BUILD file
//!   does not have them, and neither does `native`.
//! - **Checks.** Each parameter's type is checked as the argument is read,
//!   positional ones first and then the named, in the order written
//!   (`implementation` may be given by position, and for `tag_class()` so
//!   may `attrs`; nothing else). What the arguments hold is checked after
//!   that: for `repository_rule()` the elements of `environ` (worded `at
//!   index 0 of repository_rule`), then the shape of `attrs`, then that it
//!   does not redeclare `name`; for `module_extension()` the shape of
//!   `tag_classes`, then the elements of `environ` (worded `at index 0 of
//!   environ`). An attribute's own name is not checked: a repository rule's
//!   or a tag class's attributes may be called anything. `remotable=` is
//!   refused whatever it says, as an experimental flag.
//! - **Naming.** A repository rule is named by the top-level name it is
//!   bound to and prints `<starlark repository rule @@//:file.bzl%NAME>`, or
//!   `<anonymous starlark repository rule>`. A module extension and a tag
//!   class print as unknown Java objects, `<unknown object
//!   com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>` and `...
//!   TagClass>`.
//! - **Equality.** A repository rule and a module extension are equal to
//!   themselves only. Two tag classes are equal when their attributes and
//!   docs are.
//! - **Calling.** A repository rule may only be called from a module
//!   extension's implementation; called anywhere else it says so (or, given
//!   positional arguments, `unexpected positional arguments`). A module
//!   extension and a tag class cannot be called at all. None of the three
//!   has fields or encodes as JSON, and a module extension and a tag class
//!   cannot be hashed.

use crate::args::fatal;
use crate::attr::{sequence, view as attribute_view};
use crate::decl::{P, bind_checked, is_bool, is_dict, is_sequence, is_string_or_none, p};
use crate::exports::{Kind, Named, name_at_assignment, next_id};
use crate::label::{bzl_name, caller};
use allocative::Allocative;
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
use std::sync::OnceLock;

fn is_callable(v: Value<'_>) -> bool {
    matches!(
        v.get_type(),
        "function"
            | "builtin_function_or_method"
            | "Provider"
            | "RawConstructor"
            | "rule"
            | "macro"
            | "Subrule"
    )
}

fn is_dict_or_none(v: Value<'_>) -> bool {
    v.is_none() || is_dict(v)
}

/// The value of the parameter `name` of the call `params` describes.
fn by_name<'v>(params: &[P], bound: &[Option<Value<'v>>], name: &str) -> Option<Value<'v>> {
    let at = params.iter().position(|p| p.name == name).expect("known");
    bound[at]
}

/// The parameters that were given, by name, with their values.
fn given<'v>(params: &[P], bound: &[Option<Value<'v>>]) -> (Vec<&'static str>, Vec<Value<'v>>) {
    let mut names = Vec::new();
    let mut values = Vec::new();
    for (p, value) in params.iter().zip(bound) {
        if let Some(value) = value {
            names.push(p.name);
            values.push(*value);
        }
    }
    (names, values)
}

/// The error for an element of `environ` that is not a string, worded the
/// way `function` words it.
fn environ_strings<'v>(
    wording: &str,
    value: Value<'v>,
    heap: Heap<'v>,
) -> starlark::Result<Vec<String>> {
    let mut out = Vec::new();
    for (i, item) in sequence(value, heap).unwrap_or_default().iter().enumerate() {
        match item.unpack_str() {
            Some(s) => out.push(s.to_owned()),
            None => {
                return Err(fatal(format!(
                    "at index {i} of {wording}, got element of type {}, want string",
                    item.get_type()
                )));
            }
        }
    }
    Ok(out)
}

/// A dict of `keyword` must map strings to `want` values, which `is` says.
fn dict_of<'v>(
    keyword: &str,
    dict: &DictRef<'v>,
    want: &str,
    is: impl Fn(Value<'v>) -> bool,
) -> starlark::Result<()> {
    match dict
        .iter()
        .find(|(k, v)| k.unpack_str().is_none() || !is(*v))
    {
        Some((k, v)) => Err(fatal(format!(
            "got dict<{}, {}> for '{keyword}', want dict<string, {want}>",
            k.get_type(),
            v.get_type()
        ))),
        None => Ok(()),
    }
}

fn attributes<'v>(keyword: &str, dict: &DictRef<'v>) -> starlark::Result<()> {
    dict_of(keyword, dict, "Attribute", |v| attribute_view(v).is_some())
}

// ---- repository_rule ------------------------------------------------------------------

/// A `repository_rule()`, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct RepositoryRuleGen<V> {
    #[trace(static)]
    id: u64,
    /// The `.bzl` it was declared in, as `@@repo//pkg:file.bzl`.
    #[trace(static)]
    #[allocative(skip)]
    file: String,
    /// The arguments it was made with, by parameter name.
    #[trace(static)]
    #[allocative(skip)]
    names: Vec<&'static str>,
    args: Vec<V>,
    #[trace(static)]
    #[allocative(skip)]
    name: OnceLock<String>,
}

starlark_complex_value!(pub(crate) RepositoryRule);

impl<'v> Freeze for RepositoryRule<'v> {
    type Frozen = FrozenRepositoryRule;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenRepositoryRule> {
        Ok(RepositoryRuleGen {
            id: self.id,
            file: self.file,
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

impl<V> fmt::Display for RepositoryRuleGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name.get() {
            Some(name) => write!(f, "<starlark repository rule {}%{name}>", self.file),
            None => write!(f, "<anonymous starlark repository rule>"),
        }
    }
}

fn repository_rule_identity<'v>(value: Value<'v>) -> Option<(u64, &'v OnceLock<String>)> {
    fn of<'v, V: ValueLike<'v>>(a: &'v RepositoryRuleGen<V>) -> (u64, &'v OnceLock<String>) {
        (a.id, &a.name)
    }
    if let Some(live) = value.downcast_ref::<RepositoryRule<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenRepositoryRule>().map(of)
    }
}

/// What the repository rule `value` was made with for the parameter `name`.
pub(crate) fn repository_rule_arg<'v>(value: Value<'v>, name: &str) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(a: &RepositoryRuleGen<V>, name: &str) -> Option<Value<'v>> {
        let at = a.names.iter().position(|n| *n == name)?;
        Some(a.args[at].to_value())
    }
    if let Some(live) = value.downcast_ref::<RepositoryRule<'v>>() {
        of(live, name)
    } else {
        value
            .downcast_ref::<FrozenRepositoryRule>()
            .and_then(|a| of(a, name))
    }
}

#[starlark_value(type = "repository_rule")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for RepositoryRuleGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        if crate::module_ctx::extension_running(eval) {
            let Some(name) = self.name.get() else {
                return Err(fatal(
                    "attempting to instantiate a non-exported repository rule",
                ));
            };
            let id = format!("{}%{name}", self.file);
            if let Some(made) = crate::module_ctx::call_repository_rule(me, id, name, args, eval)? {
                return Ok(made);
            }
        }
        if args.positions(eval.heap())?.next().is_some() {
            return Err(fatal("unexpected positional arguments"));
        }
        Err(fatal(
            "repo rules can only be called from within module extension impl functions",
        ))
    }

    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        self.id.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(repository_rule_identity(other).is_some_and(|(id, _)| id == self.id))
    }
}

const REPOSITORY_RULE_PARAMS: &[P] = &[
    p("implementation", true, true, "callable", is_callable),
    p("attrs", false, false, "dict or NoneType", is_dict_or_none),
    p("local", false, false, "bool", is_bool),
    p("environ", false, false, "sequence", is_sequence),
    p("configure", false, false, "bool", is_bool),
    p("remotable", false, false, "bool", is_bool).experimental("--experimental_repo_remote_exec"),
    p("doc", false, false, "string or NoneType", is_string_or_none),
];

fn make_repository_rule<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind_checked("repository_rule", REPOSITORY_RULE_PARAMS, args, eval)?;
    let heap = eval.heap();
    let arg = |name: &str| by_name(REPOSITORY_RULE_PARAMS, &bound, name);
    if let Some(environ) = arg("environ") {
        environ_strings("repository_rule", environ, heap)?;
    }
    if let Some(attrs) = arg("attrs").and_then(DictRef::from_value) {
        attributes("attrs", &attrs)?;
        if attrs.iter().any(|(k, _)| k.unpack_str() == Some("name")) {
            return Err(fatal(
                "There is already a built-in attribute 'name' which cannot be overridden",
            ));
        }
    }
    let (file, _) = caller(eval, "repository_rule")?;
    let (names, values) = given(REPOSITORY_RULE_PARAMS, &bound);
    let name = OnceLock::new();
    name_at_assignment(eval, Kind::RepositoryRule, &name)?;
    Ok(heap.alloc_complex(RepositoryRuleGen {
        id: next_id(),
        file: bzl_name(&file),
        names,
        args: values,
        name,
    }))
}

// ---- module_extension -----------------------------------------------------------------

/// A `module_extension()`, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct ModuleExtensionGen<V> {
    #[trace(static)]
    id: u64,
    /// Where `module_extension()` was called, `ext.bzl:7:23`.
    #[trace(static)]
    #[allocative(skip)]
    location: String,
    #[trace(static)]
    #[allocative(skip)]
    names: Vec<&'static str>,
    args: Vec<V>,
}

starlark_complex_value!(pub(crate) ModuleExtension);

impl<'v> Freeze for ModuleExtension<'v> {
    type Frozen = FrozenModuleExtension;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenModuleExtension> {
        Ok(ModuleExtensionGen {
            id: self.id,
            location: self.location,
            names: self.names,
            args: self
                .args
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
        })
    }
}

impl<V> fmt::Display for ModuleExtensionGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"
        )
    }
}

fn module_extension_id(value: Value<'_>) -> Option<u64> {
    if let Some(live) = value.downcast_ref::<ModuleExtension<'_>>() {
        Some(live.id)
    } else {
        value
            .downcast_ref::<FrozenModuleExtension>()
            .map(|frozen| frozen.id)
    }
}

/// Where `module_extension()` was called to make `value`.
pub(crate) fn module_extension_def_location(value: Value<'_>) -> Option<String> {
    if let Some(live) = value.downcast_ref::<ModuleExtension<'_>>() {
        Some(live.location.clone())
    } else {
        value
            .downcast_ref::<FrozenModuleExtension>()
            .map(|frozen| frozen.location.clone())
    }
}

/// What the module extension `value` was made with for the parameter `name`.
pub(crate) fn module_extension_arg<'v>(value: Value<'v>, name: &str) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(a: &ModuleExtensionGen<V>, name: &str) -> Option<Value<'v>> {
        let at = a.names.iter().position(|n| *n == name)?;
        Some(a.args[at].to_value())
    }
    if let Some(live) = value.downcast_ref::<ModuleExtension<'v>>() {
        of(live, name)
    } else {
        value
            .downcast_ref::<FrozenModuleExtension>()
            .and_then(|a| of(a, name))
    }
}

#[starlark_value(type = "ModuleExtension")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for ModuleExtensionGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        _me: Value<'v>,
        _args: &Arguments<'v, '_>,
        _eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        Err(fatal("'ModuleExtension' object is not callable"))
    }

    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        self.id.hash(hasher);
        Err(fatal("unhashable type: 'ModuleExtension'"))
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(module_extension_id(other) == Some(self.id))
    }
}

const MODULE_EXTENSION_PARAMS: &[P] = &[
    p("implementation", true, true, "callable", is_callable),
    p("tag_classes", false, false, "dict", is_dict),
    p("doc", false, false, "string or NoneType", is_string_or_none),
    p("environ", false, false, "sequence", is_sequence),
    p("os_dependent", false, false, "bool", is_bool),
    p("arch_dependent", false, false, "bool", is_bool),
];

fn make_module_extension<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind_checked("module_extension", MODULE_EXTENSION_PARAMS, args, eval)?;
    let heap = eval.heap();
    let arg = |name: &str| by_name(MODULE_EXTENSION_PARAMS, &bound, name);
    if let Some(classes) = arg("tag_classes").and_then(DictRef::from_value) {
        dict_of("tag_classes", &classes, "tag_class", |v| {
            v.get_type() == "tag_class"
        })?;
    }
    if let Some(environ) = arg("environ") {
        environ_strings("environ", environ, heap)?;
    }
    let (names, values) = given(MODULE_EXTENSION_PARAMS, &bound);
    let location = crate::exports::paren_location(eval);
    Ok(heap.alloc_complex(ModuleExtensionGen {
        id: next_id(),
        location,
        names,
        args: values,
    }))
}

// ---- tag_class ------------------------------------------------------------------------

/// A `tag_class()`, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct TagClassGen<V> {
    #[trace(static)]
    #[allocative(skip)]
    names: Vec<&'static str>,
    args: Vec<V>,
}

starlark_complex_value!(pub(crate) TagClass);

impl<'v> Freeze for TagClass<'v> {
    type Frozen = FrozenTagClass;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenTagClass> {
        Ok(TagClassGen {
            names: self.names,
            args: self
                .args
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
        })
    }
}

impl<V> fmt::Display for TagClassGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"
        )
    }
}

/// What the tag class `value` was made with for the parameter `name`.
pub(crate) fn tag_class_arg<'v>(value: Value<'v>, name: &str) -> Option<Value<'v>> {
    fn of<'v, V: ValueLike<'v>>(a: &TagClassGen<V>, name: &str) -> Option<Value<'v>> {
        let at = a.names.iter().position(|n| *n == name)?;
        Some(a.args[at].to_value())
    }
    if let Some(live) = value.downcast_ref::<TagClass<'v>>() {
        of(live, name)
    } else {
        value
            .downcast_ref::<FrozenTagClass>()
            .and_then(|a| of(a, name))
    }
}

#[starlark_value(type = "tag_class")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for TagClassGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        _me: Value<'v>,
        _args: &Arguments<'v, '_>,
        _eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        Err(fatal("'tag_class' object is not callable"))
    }

    fn write_hash(
        &self,
        _hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        Err(fatal("unhashable type: 'tag_class'"))
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        if other.get_type() != "tag_class" {
            return Ok(false);
        }
        for name in ["attrs", "doc"] {
            let mine = self
                .names
                .iter()
                .position(|n| *n == name)
                .map(|at| self.args[at].to_value());
            let theirs = tag_class_arg(other, name);
            let same = match (mine, theirs) {
                (None, None) => true,
                (Some(a), Some(b)) => a.equals(b)?,
                // An absent `doc` is the same as `doc = None`, and an absent `attrs` as `{}`.
                (Some(v), None) | (None, Some(v)) => {
                    v.is_none() || v.length().is_ok_and(|n| n == 0)
                }
            };
            if !same {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

const TAG_CLASS_PARAMS: &[P] = &[
    p("attrs", true, false, "dict", is_dict),
    p("doc", false, false, "string or NoneType", is_string_or_none),
];

fn make_tag_class<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind_checked("tag_class", TAG_CLASS_PARAMS, args, eval)?;
    if let Some(attrs) = by_name(TAG_CLASS_PARAMS, &bound, "attrs").and_then(DictRef::from_value) {
        attributes("attrs", &attrs)?;
    }
    let (names, values) = given(TAG_CLASS_PARAMS, &bound);
    Ok(eval.heap().alloc_complex(TagClassGen {
        names,
        args: values,
    }))
}

// ---- the globals ----------------------------------------------------------------------

pub(crate) fn named<'v>(value: Value<'v>) -> Option<Named<'v>> {
    repository_rule_identity(value).map(|(id, name)| Named {
        id,
        name,
        kind: Kind::RepositoryRule,
    })
}

#[starlark_module]
pub(crate) fn ext_globals(builder: &mut GlobalsBuilder) {
    /// `repository_rule(implementation, *, attrs, local, environ, configure, remotable, doc)`.
    fn repository_rule<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_repository_rule(args, eval)
    }

    /// `module_extension(implementation, *, tag_classes, doc, environ, os_dependent, arch_dependent)`.
    fn module_extension<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_module_extension(args, eval)
    }

    /// `tag_class(attrs, *, doc)`.
    fn tag_class<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_tag_class(args, eval)
    }
}
