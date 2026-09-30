//! `attr.*` (buildfiji-mum.3.3): the fourteen attribute-schema builders of a
//! `.bzl` file, each returning an `Attribute` descriptor.
//!
//! What Bazel 9.2.0 does, all of it read off probes:
//!
//! - a descriptor prints as `<attr.string>`, has no members, is not
//!   hashable and is equal to another when everything it holds is;
//! - arguments are checked as they are bound, in the order the call writes
//!   them (positional ones first), and only then is anything converted: the
//!   `default` (a label is read in the `.bzl` that makes the call, so `:x` is
//!   in its package), `flags`, `executable` against `cfg`, `allow_files` and
//!   `allow_single_file`, `allow_rules`, `providers`, `cfg` and `aspects`, in
//!   that order;
//! - `mandatory`, `allow_empty = False`, `executable` and `allow_single_file`
//!   set the flag of the same meaning, so `flags = ["MANDATORY"]` and
//!   `mandatory = True` are one thing;
//! - `values`, `providers` and `aspects` are kept as given, and a `default`
//!   that is a function is not called until the rule is instantiated.
//!
//! The data an [`AttrDef`] can hold is in `fjfj-graph`; what needs a
//! Starlark value stays here.

use crate::args::{describe, fatal};
use crate::depset::{depset_to_list, is_depset};
use crate::exports::is_exported;
use crate::label::{display_label, label_of_value, parse_in_caller};
use allocative::Allocative;
use fjfj_graph::Label;
use fjfj_graph::rule::{
    AttrDef, AttrFlag, AttrType, AttrValue, Cfg as Config, FileTypes, suggest_keyword,
};
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
use std::collections::BTreeSet;
use std::fmt;

/// The `attr` value: a namespace of the builders.
#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct AttrModule;

starlark_simple_value!(AttrModule);

impl fmt::Display for AttrModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<attr>")
    }
}

#[starlark_value(type = "attr")]
impl<'v> StarlarkValue<'v> for AttrModule {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("attr", attr_methods);
        Some(RES.methods())
    }
}

/// `attr`, which `.bzl` files have and BUILD files do not.
pub(crate) fn attr_globals(builder: &mut GlobalsBuilder) {
    builder.set("attr", AttrModule);
}

/// A keyword of a builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kw {
    Default,
    Doc,
    Mandatory,
    Values,
    Configurable,
    AllowEmpty,
    AllowFiles,
    AllowSingleFile,
    AllowRules,
    Providers,
    Flags,
    Cfg,
    Aspects,
    Executable,
    SkipValidations,
    ForDependencyResolution,
    Materializer,
}

impl Kw {
    fn name(self) -> &'static str {
        match self {
            Kw::Default => "default",
            Kw::Doc => "doc",
            Kw::Mandatory => "mandatory",
            Kw::Values => "values",
            Kw::Configurable => "configurable",
            Kw::AllowEmpty => "allow_empty",
            Kw::AllowFiles => "allow_files",
            Kw::AllowSingleFile => "allow_single_file",
            Kw::AllowRules => "allow_rules",
            Kw::Providers => "providers",
            Kw::Flags => "flags",
            Kw::Cfg => "cfg",
            Kw::Aspects => "aspects",
            Kw::Executable => "executable",
            Kw::SkipValidations => "skip_validations",
            Kw::ForDependencyResolution => "for_dependency_resolution",
            Kw::Materializer => "materializer",
        }
    }
}

/// What one builder takes: `positional` in order, then `keywords`, which
/// name every parameter, the positional ones included.
struct Builder {
    name: &'static str,
    ty: AttrType,
    positional: &'static [Kw],
    keywords: &'static [Kw],
}

use Kw::*;

const BOOL: Builder = Builder {
    name: "bool",
    ty: AttrType::Bool,
    positional: &[],
    keywords: &[Default, Doc, Mandatory, Configurable],
};
const INT: Builder = Builder {
    name: "int",
    ty: AttrType::Int,
    positional: &[],
    keywords: &[Default, Doc, Mandatory, Values, Configurable],
};
const INT_LIST: Builder = Builder {
    name: "int_list",
    ty: AttrType::IntList,
    positional: &[Mandatory, AllowEmpty],
    keywords: &[Mandatory, AllowEmpty, Default, Doc, Configurable],
};
const LABEL: Builder = Builder {
    name: "label",
    ty: AttrType::Label,
    positional: &[],
    keywords: &[
        Default,
        Doc,
        Executable,
        AllowFiles,
        AllowSingleFile,
        Mandatory,
        SkipValidations,
        Providers,
        ForDependencyResolution,
        AllowRules,
        Cfg,
        Aspects,
        Flags,
        Configurable,
        Materializer,
    ],
};
const LABEL_KEYED_STRING_DICT: Builder = Builder {
    name: "label_keyed_string_dict",
    ty: AttrType::LabelKeyedStringDict,
    positional: &[AllowEmpty],
    keywords: &[
        AllowEmpty,
        Default,
        Doc,
        AllowFiles,
        AllowRules,
        Providers,
        Flags,
        Mandatory,
        SkipValidations,
        Cfg,
        Aspects,
        Configurable,
        ForDependencyResolution,
    ],
};
const LABEL_LIST: Builder = Builder {
    name: "label_list",
    ty: AttrType::LabelList,
    positional: &[AllowEmpty],
    keywords: &[
        AllowEmpty,
        Default,
        Doc,
        AllowFiles,
        AllowRules,
        Providers,
        Flags,
        Mandatory,
        SkipValidations,
        Cfg,
        Aspects,
        Configurable,
        ForDependencyResolution,
        Materializer,
    ],
};
const LABEL_LIST_DICT: Builder = Builder {
    name: "label_list_dict",
    ty: AttrType::LabelListDict,
    positional: &[AllowEmpty],
    keywords: &[
        AllowEmpty,
        Default,
        Doc,
        AllowFiles,
        AllowRules,
        Providers,
        Flags,
        Mandatory,
        SkipValidations,
        Cfg,
        Aspects,
        Configurable,
        ForDependencyResolution,
    ],
};
const OUTPUT: Builder = Builder {
    name: "output",
    ty: AttrType::Output,
    positional: &[],
    keywords: &[Doc, Mandatory],
};
const OUTPUT_LIST: Builder = Builder {
    name: "output_list",
    ty: AttrType::OutputList,
    positional: &[AllowEmpty],
    keywords: &[AllowEmpty, Doc, Mandatory],
};
const STRING: Builder = Builder {
    name: "string",
    ty: AttrType::String,
    positional: &[],
    keywords: &[Default, Doc, Mandatory, Values, Configurable],
};
const STRING_DICT: Builder = Builder {
    name: "string_dict",
    ty: AttrType::StringDict,
    positional: &[AllowEmpty],
    keywords: &[AllowEmpty, Default, Doc, Mandatory, Configurable],
};
const STRING_KEYED_LABEL_DICT: Builder = Builder {
    name: "string_keyed_label_dict",
    ty: AttrType::StringKeyedLabelDict,
    positional: &[AllowEmpty],
    keywords: &[
        AllowEmpty,
        Default,
        Doc,
        AllowFiles,
        AllowRules,
        Providers,
        Flags,
        Mandatory,
        Cfg,
        Aspects,
        Configurable,
        ForDependencyResolution,
    ],
};
const STRING_LIST: Builder = Builder {
    name: "string_list",
    ty: AttrType::StringList,
    positional: &[Mandatory, AllowEmpty],
    keywords: &[Mandatory, AllowEmpty, Default, Doc, Configurable],
};
const STRING_LIST_DICT: Builder = Builder {
    name: "string_list_dict",
    ty: AttrType::StringListDict,
    positional: &[AllowEmpty],
    keywords: &[AllowEmpty, Default, Doc, Mandatory, Configurable],
};

impl Builder {
    /// The builders that take labels also take the file, rule, provider and
    /// aspect constraints on them.
    fn labels(&self) -> bool {
        matches!(
            self.ty,
            AttrType::Label
                | AttrType::LabelList
                | AttrType::LabelKeyedStringDict
                | AttrType::StringKeyedLabelDict
                | AttrType::LabelListDict
        )
    }

    /// What Bazel calls the attribute in a message about its default: a
    /// label builder names itself, the others leave it blank.
    fn attribute(&self) -> &'static str {
        if self.labels() { self.name } else { "" }
    }
}

/// What the call gave, in the order it wrote it.
struct Given<'v>(Vec<(Kw, Value<'v>)>);

impl<'v> Given<'v> {
    fn get(&self, kw: Kw) -> Option<Value<'v>> {
        self.0.iter().find(|(k, _)| *k == kw).map(|(_, v)| *v)
    }

    /// The value, unless it was left out or is `None`.
    fn some(&self, kw: Kw) -> Option<Value<'v>> {
        self.get(kw).filter(|v| !v.is_none())
    }

    fn flag(&self, kw: Kw) -> bool {
        self.get(kw).and_then(|v| v.unpack_bool()).unwrap_or(false)
    }
}

/// A list, tuple or range: what Bazel takes for a `sequence`.
pub(crate) fn sequence<'v>(value: Value<'v>, heap: Heap<'v>) -> Option<Vec<Value<'v>>> {
    if matches!(value.get_type(), "list" | "tuple" | "range") {
        value.iterate(heap).ok().map(|items| items.collect())
    } else {
        None
    }
}

fn is_function(value: Value<'_>) -> bool {
    value.get_type() == "function"
}

/// A `LateBoundDefault` (`configuration_field()`), a label attribute's
/// default that is read from the configuration.
fn is_late_bound(value: Value<'_>) -> bool {
    value.get_type() == "LateBoundDefault"
}

/// What a keyword takes, in Bazel's words, or `None` if the signature takes
/// anything.
fn wanted<'v>(builder: &Builder, kw: Kw, value: Value<'v>, heap: Heap<'v>) -> Option<&'static str> {
    let sequence_ok = |v| sequence(v, heap).is_some();
    match kw {
        Mandatory | AllowEmpty | Executable | SkipValidations => {
            (value.unpack_bool().is_none()).then_some("bool")
        }
        Doc => (!value.is_none() && value.unpack_str().is_none()).then_some("string or NoneType"),
        Configurable => (value.unpack_bool().is_none()).then_some("bool or unbound"),
        Values | Providers | Flags | Aspects => (!sequence_ok(value)).then_some("sequence"),
        AllowFiles => (value.unpack_bool().is_none() && !value.is_none() && !sequence_ok(value))
            .then_some("bool, sequence, or NoneType"),
        AllowRules => (!value.is_none() && !sequence_ok(value)).then_some("sequence or NoneType"),
        AllowSingleFile | Cfg | ForDependencyResolution | Materializer => None,
        Default => match builder.ty {
            AttrType::Bool => value.unpack_bool().is_none().then_some("bool"),
            AttrType::Int => (value.get_type() != "int").then_some("int"),
            AttrType::String => value.unpack_str().is_none().then_some("string"),
            AttrType::IntList | AttrType::StringList => (!sequence_ok(value)).then_some("sequence"),
            AttrType::Label => (!value.is_none()
                && value.unpack_str().is_none()
                && label_of_value(value).is_none()
                && !is_late_bound(value)
                && !is_function(value))
            .then_some("Label, string, LateBoundDefault, function, or NoneType"),
            AttrType::LabelList => {
                (!sequence_ok(value) && !is_function(value)).then_some("sequence or function")
            }
            AttrType::LabelKeyedStringDict | AttrType::StringKeyedLabelDict => {
                (DictRef::from_value(value).is_none() && !is_function(value))
                    .then_some("dict or function")
            }
            AttrType::StringDict | AttrType::StringListDict | AttrType::LabelListDict => {
                DictRef::from_value(value).is_none().then_some("dict")
            }
            // Outputs have no default.
            AttrType::Output | AttrType::OutputList => None,
        },
    }
}

/// Check one argument against what its keyword takes, and keep it.
fn accept<'v>(
    builder: &Builder,
    heap: Heap<'v>,
    given: &mut Vec<(Kw, Value<'v>)>,
    kw: Kw,
    value: Value<'v>,
) -> starlark::Result<()> {
    if kw == Materializer {
        return Err(fatal(format!(
            "in call to {}(), parameter 'materializer' is experimental and thus unavailable with \
             the current flags. It may be enabled by setting --experimental_dormant_deps",
            builder.name
        )));
    }
    if let Some(want) = wanted(builder, kw, value, heap) {
        return Err(fatal(format!(
            "in call to {}(), parameter '{}' got value of type '{}', want '{want}'",
            builder.name,
            kw.name(),
            value.get_type()
        )));
    }
    given.push((kw, value));
    Ok(())
}

/// Bind the call's arguments, checking each as it is bound.
fn bind_attr<'v>(
    builder: &Builder,
    args: &Arguments<'v, '_>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Given<'v>> {
    let heap = eval.heap();
    let mut given: Vec<(Kw, Value<'v>)> = Vec::new();
    let positions: Vec<Value<'v>> = args.positions(heap)?.collect();
    for (value, &kw) in positions.iter().zip(builder.positional) {
        accept(builder, heap, &mut given, kw, *value)?;
    }
    for (key, value) in args.names_map()?.iter() {
        let name = key.as_str();
        let Some(kw) = builder.keywords.iter().copied().find(|k| k.name() == name) else {
            return Err(fatal(format!(
                "{}() got unexpected keyword argument '{name}'{}",
                builder.name,
                suggest_keyword(name, builder.keywords.iter().map(|k| k.name()))
                    .map(|s| format!(" (did you mean '{s}'?)"))
                    .unwrap_or_default()
            )));
        };
        if given.iter().any(|(k, _)| *k == kw) {
            return Err(fatal(format!(
                "{}() got multiple values for argument '{name}'",
                builder.name
            )));
        }
        accept(builder, heap, &mut given, kw, *value)?;
    }
    if positions.len() > builder.positional.len() {
        return Err(fatal(if builder.positional.is_empty() {
            format!("{}() got unexpected positional argument", builder.name)
        } else {
            format!(
                "{}() accepts no more than {} positional argument{} but got {}",
                builder.name,
                builder.positional.len(),
                if builder.positional.len() == 1 {
                    ""
                } else {
                    "s"
                },
                positions.len()
            )
        }));
    }
    Ok(Given(given))
}

/// What a descriptor keeps that a graph [`AttrDef`] cannot: Starlark values.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct AttributeGen<V> {
    #[trace(static)]
    #[allocative(skip)]
    builder: &'static str,
    #[trace(static)]
    #[allocative(skip)]
    def: AttrDef,
    /// `values`, as given.
    values: Vec<V>,
    /// `providers`, as alternatives: a flat list is one of them.
    providers: Vec<Vec<V>>,
    aspects: Vec<V>,
    /// `cfg`, when it is a transition and not a name (at most one).
    transition: Vec<V>,
    /// A `default` that is a function or a late-bound default (at most one).
    computed: Vec<V>,
    /// `for_dependency_resolution`, if it was given and is not `None` (at
    /// most one).
    for_dependency_resolution: Vec<V>,
}

starlark_complex_value!(pub(crate) Attribute);

impl<'v> Freeze for Attribute<'v> {
    type Frozen = FrozenAttribute;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenAttribute> {
        let all = |values: Vec<Value<'v>>| -> FreezeResult<Vec<FrozenValue>> {
            values.into_iter().map(|v| v.freeze(freezer)).collect()
        };
        Ok(AttributeGen {
            builder: self.builder,
            def: self.def,
            values: all(self.values)?,
            providers: self
                .providers
                .into_iter()
                .map(all)
                .collect::<FreezeResult<_>>()?,
            aspects: all(self.aspects)?,
            transition: all(self.transition)?,
            computed: all(self.computed)?,
            for_dependency_resolution: all(self.for_dependency_resolution)?,
        })
    }
}

/// A descriptor, live or frozen, as plain borrowed data.
pub(crate) struct AttributeView<'a, 'v> {
    pub(crate) builder: &'static str,
    pub(crate) def: &'a AttrDef,
    pub(crate) values: Vec<Value<'v>>,
    pub(crate) providers: Vec<Vec<Value<'v>>>,
    pub(crate) aspects: Vec<Value<'v>>,
    pub(crate) transition: Option<Value<'v>>,
    /// Read by `rule()` (buildfiji-mum.3.5), when it calls a computed default.
    #[allow(dead_code)]
    pub(crate) computed: Option<Value<'v>>,
    pub(crate) for_dependency_resolution: Option<Value<'v>>,
}

fn view_of<'a, 'v, V: ValueLike<'v>>(a: &'a AttributeGen<V>) -> AttributeView<'a, 'v> {
    let all = |values: &[V]| values.iter().map(|v| v.to_value()).collect();
    AttributeView {
        builder: a.builder,
        def: &a.def,
        values: all(&a.values),
        providers: a.providers.iter().map(|p| all(p)).collect(),
        aspects: all(&a.aspects),
        transition: a.transition.first().map(|v| v.to_value()),
        computed: a.computed.first().map(|v| v.to_value()),
        for_dependency_resolution: a.for_dependency_resolution.first().map(|v| v.to_value()),
    }
}

/// The descriptor `value` is, if it is one.
pub(crate) fn view<'v>(value: Value<'v>) -> Option<AttributeView<'v, 'v>> {
    if let Some(live) = value.downcast_ref::<Attribute<'v>>() {
        Some(view_of(live))
    } else {
        value.downcast_ref::<FrozenAttribute>().map(view_of)
    }
}

impl<V> fmt::Display for AttributeGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<attr.{}>", self.builder)
    }
}

/// `value` with the entries of a dict sorted by key, which is how two dicts
/// that differ only in order compare equal.
fn unordered_dicts(value: AttrValue) -> AttrValue {
    fn by_label<V>(entries: &mut [(Label, V)]) {
        entries.sort_by(|x, y| {
            (&x.0.repo, &x.0.package, &x.0.name).cmp(&(&y.0.repo, &y.0.package, &y.0.name))
        });
    }
    match value {
        AttrValue::StringDict(mut e) => {
            e.sort_by(|x, y| x.0.cmp(&y.0));
            AttrValue::StringDict(e)
        }
        AttrValue::StringListDict(mut e) => {
            e.sort_by(|x, y| x.0.cmp(&y.0));
            AttrValue::StringListDict(e)
        }
        AttrValue::StringKeyedLabelDict(mut e) => {
            e.sort_by(|x, y| x.0.cmp(&y.0));
            AttrValue::StringKeyedLabelDict(e)
        }
        AttrValue::LabelListDict(mut e) => {
            e.sort_by(|x, y| x.0.cmp(&y.0));
            AttrValue::LabelListDict(e)
        }
        AttrValue::LabelKeyedStringDict(mut e) => {
            by_label(&mut e);
            AttrValue::LabelKeyedStringDict(e)
        }
        other => other,
    }
}

/// Two values are the same thing to Bazel's `Attribute.equals`, which
/// compares some parts by identity: a value list, a suffix list, a
/// configuration other than the target's, and a computed default are never
/// equal to another descriptor's.
fn same_attribute<'v>(
    a: &AttributeView<'_, 'v>,
    b: &AttributeView<'_, 'v>,
) -> starlark::Result<bool> {
    // A transition is compared by `same_transition_in_attribute`, and a
    // late-bound default by value.
    let opaque = |v: &AttributeView<'_, 'v>| {
        !v.values.is_empty()
            || matches!(v.def.files, FileTypes::Suffixes(_))
            || (v.def.computed_default && !v.computed.is_some_and(is_late_bound))
            || matches!(v.def.cfg, Config::Exec | Config::Host)
    };
    if opaque(a) || opaque(b) {
        return Ok(false);
    }
    if a.builder != b.builder {
        return Ok(false);
    }
    // A default left out is the type's zero, and a dict is the same in any
    // order.
    let comparable = |v: &AttributeView<'_, 'v>| {
        let mut def = v.def.clone();
        def.default = def
            .default
            .take()
            .or_else(|| def.ty.zero())
            .map(unordered_dicts);
        def
    };
    if comparable(a) != comparable(b) {
        return Ok(false);
    }
    let same_list = |x: &[Value<'v>], y: &[Value<'v>]| -> starlark::Result<bool> {
        if x.len() != y.len() {
            return Ok(false);
        }
        for (p, q) in x.iter().zip(y) {
            if !p.equals(*q)? {
                return Ok(false);
            }
        }
        Ok(true)
    };
    // An alternative is a set of providers: order does not matter in it.
    let same_set = |x: &[Value<'v>], y: &[Value<'v>]| -> starlark::Result<bool> {
        for (from, to) in [(x, y), (y, x)] {
            for p in from {
                let mut found = false;
                for q in to {
                    if p.equals(*q)? {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    };
    if a.providers.len() != b.providers.len() {
        return Ok(false);
    }
    for (x, y) in a.providers.iter().zip(&b.providers) {
        if !same_set(x, y)? {
            return Ok(false);
        }
    }
    if !same_list(&a.aspects, &b.aspects)? {
        return Ok(false);
    }
    if let (Some(x), Some(y)) = (a.computed, b.computed)
        && !x.equals(y)?
    {
        return Ok(false);
    }
    let same_option = |x: Option<Value<'v>>, y: Option<Value<'v>>| match (x, y) {
        (None, None) => Ok(true),
        (Some(x), Some(y)) => x.equals(y),
        _ => Ok(false),
    };
    let same_transition = |x: Option<Value<'v>>, y: Option<Value<'v>>| match (x, y) {
        (None, None) => Ok(true),
        (Some(x), Some(y)) => crate::decl::same_transition_in_attribute(x, y),
        _ => Ok(false),
    };
    Ok(same_transition(a.transition, b.transition)?
        && same_option(a.for_dependency_resolution, b.for_dependency_resolution)?)
}

#[starlark_value(type = "Attribute")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for AttributeGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        match view(other) {
            Some(theirs) => same_attribute(&view_of(self), &theirs),
            None => Ok(false),
        }
    }
}

#[starlark_module]
fn attr_methods(builder: &mut MethodsBuilder) {
    /// `attr.bool(...)`.
    fn bool<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&BOOL, args, eval)
    }

    /// `attr.int(...)`.
    fn int<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&INT, args, eval)
    }

    /// `attr.int_list(...)`.
    fn int_list<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&INT_LIST, args, eval)
    }

    /// `attr.label(...)`.
    fn label<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&LABEL, args, eval)
    }

    /// `attr.label_keyed_string_dict(...)`.
    fn label_keyed_string_dict<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&LABEL_KEYED_STRING_DICT, args, eval)
    }

    /// `attr.label_list(...)`.
    fn label_list<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&LABEL_LIST, args, eval)
    }

    /// `attr.label_list_dict(...)`.
    fn label_list_dict<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&LABEL_LIST_DICT, args, eval)
    }

    /// `attr.output(...)`.
    fn output<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&OUTPUT, args, eval)
    }

    /// `attr.output_list(...)`.
    fn output_list<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&OUTPUT_LIST, args, eval)
    }

    /// `attr.string(...)`.
    fn string<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&STRING, args, eval)
    }

    /// `attr.string_dict(...)`.
    fn string_dict<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&STRING_DICT, args, eval)
    }

    /// `attr.string_keyed_label_dict(...)`.
    fn string_keyed_label_dict<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&STRING_KEYED_LABEL_DICT, args, eval)
    }

    /// `attr.string_list(...)`.
    fn string_list<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&STRING_LIST, args, eval)
    }

    /// `attr.string_list_dict(...)`.
    fn string_list_dict<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        build_attr(&STRING_LIST_DICT, args, eval)
    }
}

/// What the conversions of one call collect that a graph [`AttrDef`] does
/// not hold.
#[derive(Default)]
struct Kept<'v> {
    values: Vec<Value<'v>>,
    providers: Vec<Vec<Value<'v>>>,
    aspects: Vec<Value<'v>>,
    transition: Option<Value<'v>>,
    computed: Option<Value<'v>>,
    for_dependency_resolution: Option<Value<'v>>,
}

fn build_attr<'v>(
    builder: &Builder,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let given = bind_attr(builder, args, eval)?;
    let heap = eval.heap();
    let mut def = AttrDef::new(builder.ty);
    def.flags.insert(AttrFlag::StarlarkDefined);
    let mut kept = Kept::default();

    def.doc = given
        .some(Doc)
        .and_then(|d| d.unpack_str())
        .map(str::to_owned);
    if given.flag(Mandatory) {
        def.flags.insert(AttrFlag::Mandatory);
    }
    if given.get(AllowEmpty).is_some() && !given.flag(AllowEmpty) {
        def.flags.insert(AttrFlag::NonEmpty);
    }
    def.configurable = given.get(Configurable).and_then(|v| v.unpack_bool());
    def.skip_validations = given.flag(SkipValidations);
    if let Some(values) = given.get(Values) {
        kept.values = sequence(values, heap).unwrap_or_default();
    }
    kept.for_dependency_resolution = given.some(ForDependencyResolution);

    if let Some(default) = given.get(Default) {
        convert_default(builder, default, eval, &mut def, &mut kept)?;
    }
    if builder.labels() {
        constrain_labels(&given, eval, &mut def, &mut kept)?;
    }

    Ok(heap.alloc_complex(AttributeGen {
        builder: builder.name,
        def,
        values: kept.values,
        providers: kept.providers,
        aspects: kept.aspects,
        transition: kept.transition.into_iter().collect(),
        computed: kept.computed.into_iter().collect(),
        for_dependency_resolution: kept.for_dependency_resolution.into_iter().collect(),
    }))
}

/// `default`, converted to what the attribute holds. A function is kept
/// as it is.
fn convert_default<'v>(
    builder: &Builder,
    default: Value<'v>,
    eval: &Evaluator<'v, '_, '_>,
    def: &mut AttrDef,
    kept: &mut Kept<'v>,
) -> starlark::Result<()> {
    let heap = eval.heap();
    let attribute = builder.attribute();
    let param = format!("parameter 'default' of attribute '{attribute}'");
    let element =
        |i: usize| format!("element {i} of parameter 'default' of attribute '{attribute}'");
    if builder.labels() && !default.is_none() && (is_function(default) || is_late_bound(default)) {
        kept.computed = Some(default);
        def.computed_default = true;
        return Ok(());
    }
    let value = match builder.ty {
        AttrType::Bool => AttrValue::Bool(default.unpack_bool().unwrap_or(false)),
        AttrType::Int => AttrValue::Int(int32(default, &param)?),
        AttrType::String => AttrValue::String(default.unpack_str().unwrap_or_default().to_owned()),
        AttrType::IntList => {
            let items = sequence(default, heap).unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.into_iter().enumerate() {
                if item.get_type() != "int" {
                    return Err(wrong_element("int", &element(i), item));
                }
                out.push(int32(item, &element(i))?);
            }
            AttrValue::IntList(out)
        }
        AttrType::StringList => {
            let items = sequence(default, heap).unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.into_iter().enumerate() {
                out.push(string_of(item, &element(i))?);
            }
            AttrValue::StringList(out)
        }
        AttrType::Label => {
            if default.is_none() {
                return Ok(());
            }
            AttrValue::Label(label_of(eval, builder, default, &param)?)
        }
        AttrType::LabelList => {
            let items = sequence(default, heap).unwrap_or_default();
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.into_iter().enumerate() {
                out.push(label_of(eval, builder, item, &element(i))?);
            }
            AttrValue::LabelList(out)
        }
        AttrType::StringDict => {
            let mut out = Vec::new();
            for (k, v) in DictRef::from_value(default).expect("checked").iter() {
                out.push((
                    string_of(k, "dict key element")?,
                    string_of(v, "dict value element")?,
                ));
            }
            AttrValue::StringDict(out)
        }
        AttrType::StringListDict => {
            let mut out = Vec::new();
            for (k, v) in DictRef::from_value(default).expect("checked").iter() {
                let key = string_of(k, "dict key element")?;
                let items = list_value(v, "list(string)", heap)?;
                let mut list = Vec::with_capacity(items.len());
                for (i, item) in items.into_iter().enumerate() {
                    list.push(string_of(
                        item,
                        &format!("element {i} of dict value element"),
                    )?);
                }
                out.push((key, list));
            }
            AttrValue::StringListDict(out)
        }
        AttrType::LabelListDict => {
            let mut out = Vec::new();
            for (k, v) in DictRef::from_value(default).expect("checked").iter() {
                let key = string_of(k, "dict key element")?;
                let items = list_value(v, "list(label)", heap)?;
                let mut list = Vec::with_capacity(items.len());
                for (i, item) in items.into_iter().enumerate() {
                    list.push(label_of(
                        eval,
                        builder,
                        item,
                        &format!("element {i} of dict value element"),
                    )?);
                }
                out.push((key, list));
            }
            AttrValue::LabelListDict(out)
        }
        AttrType::LabelKeyedStringDict => {
            let mut out: Vec<(Label, String)> = Vec::new();
            let mut written: Vec<Value<'v>> = Vec::new();
            for (k, v) in DictRef::from_value(default).expect("checked").iter() {
                let key = label_of(eval, builder, k, "dict key element")?;
                out.push((key, string_of(v, "dict value element")?));
                written.push(k);
            }
            for (i, (label, _)) in out.iter().enumerate() {
                let same: Vec<String> = out
                    .iter()
                    .enumerate()
                    .filter(|(_, (other, _))| other == label)
                    .map(|(j, _)| written[j].to_repr())
                    .collect();
                if same.len() > 1 && out[..i].iter().all(|(other, _)| other != label) {
                    return Err(fatal(format!(
                        "duplicate labels in parameter 'default' of attribute '{}': {} (as [{}])",
                        builder.name,
                        display_label(label),
                        same.join(", ")
                    )));
                }
            }
            AttrValue::LabelKeyedStringDict(out)
        }
        AttrType::StringKeyedLabelDict => {
            let mut out = Vec::new();
            for (k, v) in DictRef::from_value(default).expect("checked").iter() {
                let key = string_of(k, "dict key element")?;
                out.push((key, label_of(eval, builder, v, "dict value element")?));
            }
            AttrValue::StringKeyedLabelDict(out)
        }
        AttrType::Output | AttrType::OutputList => return Ok(()),
    };
    def.default = Some(value);
    Ok(())
}

fn wrong_element(want: &str, place: &str, item: Value<'_>) -> starlark::Error {
    fatal(format!(
        "expected value of type '{want}' for {place}, but got {}",
        describe(item)
    ))
}

fn string_of(item: Value<'_>, place: &str) -> starlark::Result<String> {
    item.unpack_str()
        .map(str::to_owned)
        .ok_or_else(|| wrong_element("string", place, item))
}

fn int32(item: Value<'_>, place: &str) -> starlark::Result<i32> {
    item.unpack_i32().ok_or_else(|| {
        fatal(format!(
            "for {place}, got {}, want value in signed 32-bit range",
            item.to_repr()
        ))
    })
}

/// A dict value that is a list of something: a sequence, or a depset.
fn list_value<'v>(
    value: Value<'v>,
    want: &str,
    heap: Heap<'v>,
) -> starlark::Result<Vec<Value<'v>>> {
    if let Some(items) = sequence(value, heap) {
        Ok(items)
    } else if is_depset(value) {
        depset_to_list(value).expect("a depset")
    } else {
        Err(wrong_element(want, "dict value element", value))
    }
}

/// `item` as a label written in the `.bzl` making the call: a label is
/// itself, a string is read there. `place` says where it was, for the error.
fn label_of<'v>(
    eval: &Evaluator<'v, '_, '_>,
    builder: &Builder,
    item: Value<'v>,
    place: &str,
) -> starlark::Result<Label> {
    if let Some(label) = label_of_value(item) {
        return Ok(label);
    }
    let Some(text) = item.unpack_str() else {
        return Err(wrong_element("string", place, item));
    };
    let function = format!("attr.{}", builder.name);
    parse_in_caller(eval, &function, text)?
        .map_err(|e| fatal(format!("invalid label '{text}' in {place}: {e}")))
}

/// The constraints only a label attribute has, checked in Bazel's order.
fn constrain_labels<'v>(
    given: &Given<'v>,
    eval: &Evaluator<'v, '_, '_>,
    def: &mut AttrDef,
    kept: &mut Kept<'v>,
) -> starlark::Result<()> {
    let heap = eval.heap();
    if let Some(flags) = given.get(Flags) {
        let items = sequence(flags, heap).unwrap_or_default();
        let mut names = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            let Some(name) = item.unpack_str() else {
                return Err(fatal(format!(
                    "at index {i} of flags, got element of type {}, want string",
                    item.get_type()
                )));
            };
            names.push(name);
        }
        for name in names {
            match AttrFlag::parse(name) {
                Some(flag) => {
                    def.flags.insert(flag);
                }
                None => return Err(fatal(format!("unknown attribute flag '{name}'"))),
            }
        }
    }

    let cfg = given.some(Cfg);
    if given.flag(Executable) {
        if cfg.is_none() {
            return Err(fatal(
                "cfg parameter is mandatory when executable=True is provided. Please see \
                 https://bazel.build/extending/rules#configurations for more details.",
            ));
        }
        def.flags.insert(AttrFlag::Executable);
    }

    let files = given.some(AllowFiles);
    let single = given.some(AllowSingleFile);
    if files.is_some() && single.is_some() {
        return Err(fatal(
            "Cannot specify both allow_files and allow_single_file",
        ));
    }
    if let Some(files) = files {
        def.files = file_types(files, heap, false)?;
    }
    if let Some(single) = single {
        def.files = file_types(single, heap, true)?;
        def.flags.insert(AttrFlag::SingleArtifact);
    }

    if let Some(rules) = given.some(AllowRules) {
        let items = sequence(rules, heap).unwrap_or_default();
        let mut names = BTreeSet::new();
        for (i, item) in items.iter().enumerate() {
            let Some(name) = item.unpack_str() else {
                return Err(fatal(format!(
                    "at index {i} of allowed rule classes for attribute definition, got element \
                     of type {}, want string",
                    item.get_type()
                )));
            };
            names.insert(name.to_owned());
        }
        def.allow_rules = Some(names);
    }

    if let Some(providers) = given.get(Providers) {
        kept.providers = provider_alternatives("providers", providers, eval)?;
    }

    if let Some(cfg) = cfg {
        match cfg.unpack_str() {
            Some("target") => def.cfg = Config::Target,
            Some("exec") => def.cfg = Config::Exec,
            Some("host") => def.cfg = Config::Host,
            Some(_) => return Err(bad_cfg()),
            // `config.target()` is the target configuration by another name.
            None if crate::decl::is_target_transition(cfg) => def.cfg = Config::Target,
            None if matches!(cfg.get_type(), "transition" | "ExecTransitionFactory") => {
                def.cfg = Config::Transition;
                kept.transition = Some(cfg);
            }
            None => return Err(bad_cfg()),
        }
    }

    if let Some(aspects) = given.get(Aspects) {
        let items = sequence(aspects, heap).unwrap_or_default();
        for (i, item) in items.iter().enumerate() {
            if item.get_type() != "Aspect" {
                return Err(fatal(format!(
                    "at index {i} of aspects, got element of type {}, want Aspect",
                    item.get_type()
                )));
            }
        }
        for item in &items {
            if !is_exported(*item, eval) {
                return Err(fatal(
                    "Aspects should be top-level values in extension files that define them.",
                ));
            }
            kept.aspects.push(*item);
        }
    }
    Ok(())
}

fn bad_cfg() -> starlark::Error {
    fatal(
        "cfg must be either 'target', 'exec' or a starlark defined transition defined by the \
         exec() or transition() functions.",
    )
}

/// `allow_files` or `allow_single_file`. Only the second checks what it is
/// given here: the first was checked as an argument.
fn file_types<'v>(value: Value<'v>, heap: Heap<'v>, single: bool) -> starlark::Result<FileTypes> {
    if let Some(any) = value.unpack_bool() {
        return Ok(if any { FileTypes::Any } else { FileTypes::None });
    }
    let Some(items) = sequence(value, heap) else {
        debug_assert!(single);
        return Err(fatal(
            "allow_single_file should be a boolean or a string list",
        ));
    };
    let mut suffixes = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let Some(suffix) = item.unpack_str() else {
            return Err(fatal(format!(
                "at index {i} of allow_files argument, got element of type {}, want string",
                item.get_type()
            )));
        };
        suffixes.push(suffix.to_owned());
    }
    Ok(FileTypes::Suffixes(suffixes))
}

/// A provider can only be required by a name a `.bzl` gave it.
pub(crate) fn check_exported<'v>(
    providers: &[Value<'v>],
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<()> {
    for provider in providers {
        if crate::provider::is_provider(*provider) && !is_exported(*provider, eval) {
            return Err(fatal(
                "Providers should be top-level values in extension files that define them.",
            ));
        }
    }
    Ok(())
}

/// `providers`: a list of providers is one alternative, and a list of lists
/// is several.
pub(crate) fn provider_alternatives<'v>(
    keyword: &str,
    value: Value<'v>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<Vec<Value<'v>>>> {
    let heap = eval.heap();
    let items = sequence(value, heap).unwrap_or_default();
    if !items.is_empty() && items.iter().all(|p| p.get_type() == "Provider") {
        check_exported(&items, eval)?;
        return Ok(vec![items]);
    }
    let mut lists = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let Some(list) = sequence(*item, heap) else {
            return Err(fatal(format!(
                "at index {i} of {keyword}, got element of type {}, want sequence",
                item.get_type()
            )));
        };
        lists.push(list);
    }
    for list in &lists {
        for (j, provider) in list.iter().enumerate() {
            if provider.get_type() != "Provider" {
                return Err(fatal(format!(
                    "at index {j} of {keyword}, got element of type {}, want Provider",
                    provider.get_type()
                )));
            }
        }
    }
    for list in &lists {
        check_exported(list, eval)?;
    }
    // One empty alternative is no constraint, as an empty list is.
    if lists.len() == 1 && lists[0].is_empty() {
        lists.clear();
    }
    Ok(lists)
}
