//! `provider()` (buildfiji-mum.3.4): a callable symbol whose calls make
//! instances, in `.bzl` files only.
//!
//! What Bazel 9.2.0 does, all of it read off probes:
//!
//! - `provider(doc, *, fields, init)` returns a `Provider`: it prints as
//!   `<provider>`, has no members, is hashable and is equal only to itself.
//!   With `init` it returns the pair `(provider, raw_constructor)` instead,
//!   and the raw constructor (`RawConstructor`) makes an instance without
//!   calling `init`.
//! - Calling a provider makes an instance, which is a `struct` in every way
//!   the language can see (`type`, `repr`, `dir`, `json.encode`): see
//!   [`crate::structs`], which holds the reference to the provider. It is
//!   immutable, and takes keyword arguments only. `fields`, a list, tuple or
//!   dict of names, restricts which; without it any name goes. A field left
//!   out is absent, not `None`.
//! - `init` is called with the arguments of the call and must return a
//!   dict with string keys, whose entries are the fields.
//! - A provider is named when a `.bzl` binds it to a top-level name, at the
//!   assignment, and the first name wins: `Q = provider()` then `P = Q` is
//!   `Q`. The name is what an error says (`got unexpected field 'b' in call
//!   to instantiate provider P`) and what makes it usable in `providers` and
//!   `provides`; one that is never bound, or is only in a list or a dict,
//!   has none (`<no name>`). Naming is by value, so `P = make()` names what
//!   `make()` returned.
//!
//! The `starlark` crate has no hook on an assignment and will not show a
//! native function a private (`_P`) name, so a provider looks for its name
//! among the public names of the module being evaluated when it needs it,
//! and [`export_providers`] names the rest when the module is done. A
//! provider bound only to a `_private` name is therefore anonymous until its
//! module ends, and [`is_exported`] gives it the benefit of the doubt in a
//! module that binds any.

use crate::args::fatal;
use crate::attr::sequence;
use crate::exports::{Kind, Named, name_at_assignment, next_id, resolve_name};
use crate::structs::new_instance;
use allocative::Allocative;
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
use std::sync::OnceLock;

/// What an unnamed provider is called in an error.
const NO_NAME: &str = "<no name>";

/// What Bazel prints for a raw constructor, which is a Java class it has no
/// printing for.
const RAW_CONSTRUCTOR: &str =
    "<unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>";

/// A provider, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct ProviderGen<V> {
    /// What a provider is, across freezing: two are the same if these are.
    #[trace(static)]
    id: u64,
    /// The names an instance may have; none if any.
    #[trace(static)]
    fields: Option<Vec<String>>,
    /// Kept for documentation, which nothing reads yet.
    #[trace(static)]
    #[allow(dead_code)]
    doc: Option<String>,
    /// `init`, if it was given (at most one).
    init: Vec<V>,
    /// The top-level name it is bound to, once it is.
    #[trace(static)]
    #[allocative(skip)]
    name: OnceLock<String>,
    /// The `.bzl` that made it, `None` for a built-in one.
    #[trace(static)]
    #[allocative(skip)]
    file: Option<String>,
}

starlark_complex_value!(pub(crate) Provider);

impl<'v> Freeze for Provider<'v> {
    type Frozen = FrozenProvider;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenProvider> {
        Ok(ProviderGen {
            id: self.id,
            fields: self.fields,
            doc: self.doc,
            init: self
                .init
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
            name: self.name,
            file: self.file,
        })
    }
}

impl<V> fmt::Display for ProviderGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<provider>")
    }
}

/// A provider's raw constructor: the same instances without `init`.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct RawConstructorGen<V> {
    provider: V,
}

starlark_complex_value!(pub(crate) RawConstructor);

impl<'v> Freeze for RawConstructor<'v> {
    type Frozen = FrozenRawConstructor;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenRawConstructor> {
        Ok(RawConstructorGen {
            provider: self.provider.freeze(freezer)?,
        })
    }
}

impl<V> fmt::Display for RawConstructorGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{RAW_CONSTRUCTOR}")
    }
}

/// What a provider value says about itself, live or frozen.
struct View<'v> {
    id: u64,
    fields: Option<&'v [String]>,
    name: &'v OnceLock<String>,
    file: Option<&'v str>,
}

fn view<'v>(value: Value<'v>) -> Option<View<'v>> {
    fn of<'v, V: ValueLike<'v>>(p: &'v ProviderGen<V>) -> View<'v> {
        View {
            id: p.id,
            fields: p.fields.as_deref(),
            name: &p.name,
            file: p.file.as_deref(),
        }
    }
    if let Some(live) = value.downcast_ref::<Provider<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenProvider>().map(of)
    }
}

/// Whether `value` is a provider made by `provider()`, as opposed to a
/// built-in one.
pub(crate) fn is_provider(value: Value<'_>) -> bool {
    view(value).is_some()
}

/// A provider as a rule class lists it: its name and the file that defined it.
pub(crate) fn origin(value: Value<'_>) -> Option<fjfj_graph::schema::ProviderRef> {
    let p = view(value)?;
    Some(fjfj_graph::schema::ProviderRef {
        name: p.name.get()?.clone(),
        file: p.file.unwrap_or("<native>").to_owned(),
    })
}

/// A provider, for [`crate::exports`] to name.
pub(crate) fn named<'v>(value: Value<'v>) -> Option<Named<'v>> {
    view(value).map(|p| Named {
        id: p.id,
        name: p.name,
        kind: Kind::Provider,
    })
}

/// The name of the provider of an instance, for an error: `struct` for what
/// `struct()` made.
pub(crate) fn instance_of(provider: Option<Value<'_>>) -> String {
    match provider {
        None => "struct".to_owned(),
        Some(p) => view(p)
            .and_then(|v| v.name.get().cloned())
            .unwrap_or_else(|| NO_NAME.to_owned()),
    }
}

/// What an error calls an instance of `provider`: the name it was exported
/// under, or `struct` while it has none.
pub(crate) fn instance_type_in_errors(provider: Value<'_>) -> String {
    view(provider)
        .and_then(|v| v.name.get().cloned())
        .unwrap_or_else(|| "struct".to_owned())
}

/// Whether two instances are of one provider (or both of `struct`).
pub(crate) fn same_provider(a: Option<Value<'_>>, b: Option<Value<'_>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => match (view(a), view(b)) {
            (Some(a), Some(b)) => a.id == b.id,
            _ => false,
        },
        _ => false,
    }
}

/// Bazel's word for the type of the first entry of `dict` that is not a
/// `string: string` one, when only those are wanted.
fn wrong_entry(key: Value<'_>, value: Value<'_>) -> String {
    format!("dict<{}, {}>", key.get_type(), value.get_type())
}

/// The fields and values a call gives, and how many positional arguments it
/// had.
struct Call<'v> {
    positional: usize,
    fields: Vec<(String, Value<'v>)>,
}

fn call_of<'v>(args: &Arguments<'v, '_>, heap: Heap<'v>) -> starlark::Result<Call<'v>> {
    Ok(Call {
        positional: args.positions(heap)?.count(),
        fields: args
            .names_map()?
            .iter()
            .map(|(name, value)| (name.as_str().to_owned(), *value))
            .collect(),
    })
}

/// Make an instance of `provider` from what a call gave, or say why not; `who`
/// is what the call is named in the error about positional arguments.
fn instantiate<'v>(
    provider: Value<'v>,
    who: &str,
    call: Call<'v>,
    name: Option<&str>,
    heap: Heap<'v>,
) -> starlark::Result<Value<'v>> {
    if call.positional > 0 {
        return Err(fatal(format!("{who}: unexpected positional arguments")));
    }
    let me = view(provider).expect("a provider");
    if let Some(allowed) = me.fields {
        let unexpected: Vec<String> = call
            .fields
            .iter()
            .filter(|(name, _)| !allowed.contains(name))
            .map(|(name, _)| format!("'{name}'"))
            .collect();
        if !unexpected.is_empty() {
            return Err(fatal(format!(
                "got unexpected field{} {} in call to instantiate provider {}",
                if unexpected.len() == 1 { "" } else { "s" },
                unexpected.join(", "),
                name.unwrap_or(NO_NAME)
            )));
        }
    }
    Ok(new_instance(heap, provider, call.fields))
}

#[starlark_value(type = "Provider")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for ProviderGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let name = resolve_name(me, eval)?;
        let heap = eval.heap();
        let who = name.as_deref().unwrap_or(NO_NAME);
        let Some(init) = self.init.first() else {
            return instantiate(me, who, call_of(args, heap)?, name.as_deref(), heap);
        };
        // `init` sees the arguments as they were written, and says what is
        // wrong with them before anything here does.
        let made = init.to_value().invoke(args, eval)?;
        let Some(dict) = DictRef::from_value(made) else {
            return Err(fatal(format!(
                "got {} for 'return value of provider init()', want dict",
                made.get_type()
            )));
        };
        let mut fields = Vec::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let Some(text) = key.unpack_str() else {
                return Err(fatal(format!(
                    "got {} for 'return value of provider init()', want dict<string, unknown>",
                    wrong_entry(key, value)
                )));
            };
            fields.push((text.to_owned(), value));
        }
        let call = Call {
            positional: 0,
            fields,
        };
        instantiate(me, who, call, name.as_deref(), heap)
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        self.id.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(view(other).is_some_and(|other| other.id == self.id))
    }
}

#[starlark_value(type = "RawConstructor")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for RawConstructorGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        _me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let provider = self.provider.to_value();
        let name = resolve_name(provider, eval)?;
        let who = match &name {
            Some(name) => format!("<raw constructor for {name}>"),
            None => "<raw constructor>".to_owned(),
        };
        let heap = eval.heap();
        instantiate(provider, &who, call_of(args, heap)?, name.as_deref(), heap)
    }
}

/// What `provider()` accepts is checked as each argument is bound, and the
/// positional argument that is one too many is reported after all the rest.
fn check_doc<'v>(value: Value<'v>, _heap: Heap<'v>) -> starlark::Result<()> {
    if value.is_none() || value.unpack_str().is_some() {
        return Ok(());
    }
    Err(fatal(format!(
        "in call to provider(), parameter 'doc' got value of type '{}', want 'string or NoneType'",
        value.get_type()
    )))
}

fn check_fields<'v>(value: Value<'v>, heap: Heap<'v>) -> starlark::Result<()> {
    if value.is_none() || DictRef::from_value(value).is_some() {
        return Ok(());
    }
    if sequence(value, heap).is_some() {
        return Ok(());
    }
    Err(fatal(format!(
        "in call to provider(), parameter 'fields' got value of type '{}', want 'sequence, dict, or NoneType'",
        value.get_type()
    )))
}

/// The types Bazel calls callable that a `.bzl` can have to give as `init`.
fn is_callable(value: Value<'_>) -> bool {
    matches!(
        value.get_type(),
        "function" | "builtin_function_or_method" | "Provider" | "RawConstructor"
    )
}

fn check_init<'v>(value: Value<'v>, _heap: Heap<'v>) -> starlark::Result<()> {
    if value.is_none() || is_callable(value) {
        return Ok(());
    }
    Err(fatal(format!(
        "in call to provider(), parameter 'init' got value of type '{}', want 'callable or NoneType'",
        value.get_type()
    )))
}

/// The names in `fields`, a list of them or a dict of them to documentation.
fn field_names<'v>(value: Value<'v>, heap: Heap<'v>) -> starlark::Result<Option<Vec<String>>> {
    if value.is_none() {
        return Ok(None);
    }
    let mut names = Vec::new();
    if let Some(dict) = DictRef::from_value(value) {
        for (key, doc) in dict.iter() {
            match (key.unpack_str(), doc.unpack_str()) {
                (Some(name), Some(_)) => names.push(name.to_owned()),
                _ => {
                    return Err(fatal(format!(
                        "got {} for 'fields', want dict<string, string>",
                        wrong_entry(key, doc)
                    )));
                }
            }
        }
    } else {
        for (i, item) in sequence(value, heap).unwrap_or_default().iter().enumerate() {
            let Some(name) = item.unpack_str() else {
                return Err(fatal(format!(
                    "at index {i} of fields, got element of type {}, want string",
                    item.get_type()
                )));
            };
            names.push(name.to_owned());
        }
    }
    if let Some(dup) = names
        .iter()
        .enumerate()
        .find_map(|(i, n)| names[..i].contains(n).then_some(n))
    {
        // Bazel throws an exception here and prints no message.
        return Err(fatal(format!("duplicate field '{dup}' in fields")));
    }
    Ok(Some(names))
}

type Check = for<'a> fn(Value<'a>, Heap<'a>) -> starlark::Result<()>;

fn make_provider<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let heap = eval.heap();
    let positions: Vec<Value<'v>> = args.positions(heap)?.collect();
    let (mut doc, mut fields, mut init): (Option<Value>, Option<Value>, Option<Value>) =
        (None, None, None);
    if let Some(first) = positions.first() {
        check_doc(*first, heap)?;
        doc = Some(*first);
    }
    for (key, value) in args.names_map()?.iter() {
        let (slot, check): (&mut Option<Value>, Check) = match key.as_str() {
            "doc" => (&mut doc, check_doc),
            "fields" => (&mut fields, check_fields),
            "init" => (&mut init, check_init),
            other => {
                let hint = fjfj_graph::rule::suggest_keyword(other, ["doc", "fields", "init"])
                    .map(|s| format!(" (did you mean '{s}'?)"))
                    .unwrap_or_default();
                return Err(fatal(format!(
                    "provider() got unexpected keyword argument '{other}'{hint}"
                )));
            }
        };
        check(*value, heap)?;
        if slot.is_some() {
            return Err(fatal(format!(
                "provider() got multiple values for argument '{}'",
                key.as_str()
            )));
        }
        *slot = Some(*value);
    }
    if positions.len() > 1 {
        return Err(fatal(format!(
            "provider() accepts no more than 1 positional argument but got {}",
            positions.len()
        )));
    }

    let fields = match fields {
        Some(value) => field_names(value, heap)?,
        None => None,
    };
    let init: Vec<Value<'v>> = init.filter(|i| !i.is_none()).into_iter().collect();
    let has_init = !init.is_empty();
    let name = OnceLock::new();
    // `P = provider(...)` names it at once; `P, R = provider(init = ...)` is
    // named when the module is done.
    name_at_assignment(eval, Kind::Provider, &name)?;
    let provider = heap.alloc_complex(ProviderGen {
        id: next_id(),
        fields,
        doc: doc.and_then(|d| d.unpack_str()).map(str::to_owned),
        init,
        name,
        // What the builtins define is Bazel's own.
        file: crate::label::evaluating_file(eval)
            .filter(|l| l.repo != "_builtins")
            .map(|l| fjfj_graph::expand::label_text(&l)),
    });
    if !has_init {
        return Ok(provider);
    }
    let raw = heap.alloc_complex(RawConstructorGen { provider });
    Ok(heap.alloc((provider, raw)))
}

#[starlark_module]
pub(crate) fn provider_globals(builder: &mut GlobalsBuilder) {
    /// `provider(doc, *, fields, init)`.
    fn provider<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_provider(args, eval)
    }
}
