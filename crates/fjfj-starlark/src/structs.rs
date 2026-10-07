//! `struct` (buildfiji-mum.3.1): an immutable record of named fields, in
//! `.bzl` files only.
//!
//! What Bazel 9.2.0 does, which differs from the `starlark` crate's own
//! struct in the ways that show:
//!
//! - fields are kept sorted by name, so `struct(b = 1, a = 2)` prints
//!   `struct(a = 2, b = 1)` and encodes to JSON in that order;
//! - `repr` is `struct(a = 1)`, and a struct has nothing but its fields:
//!   `dir` lists them and nothing else, and `to_json` / `to_proto` are gone;
//! - two structs are equal when their fields are, and one is hashable only
//!   if every field is;
//! - `+` joins two structs and refuses a field they share; it is the only
//!   operator a struct has, and it is always true.
//!
//! An instance of a `provider()` (buildfiji-mum.3.4) is the same value: it
//! prints as `struct(a = 1)`, has type `struct` and holds a reference to its
//! provider. Two are equal only if their providers are the same, and `+`
//! joins only instances of one provider.

use crate::args::{fatal, unsupported_binary};
use crate::provider::{instance_of, same_provider};
use allocative::Allocative;
use starlark::collections::StarlarkHasher;
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;
use std::hash::Hash;

/// A struct, before or after freezing. `names` is sorted and `values` runs
/// alongside it.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub struct StructGen<V> {
    #[trace(static)]
    names: Vec<String>,
    values: Vec<V>,
    /// The provider this is an instance of; none for `struct()` itself.
    provider: Vec<V>,
}

starlark_complex_value!(pub Struct);

impl<'v> Freeze for Struct<'v> {
    type Frozen = FrozenStruct;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenStruct> {
        Ok(StructGen {
            names: self.names,
            values: self
                .values
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
            provider: self
                .provider
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
        })
    }
}

impl<'v, V: ValueLike<'v>> StructGen<V> {
    fn get(&self, name: &str) -> Option<Value<'v>> {
        let at = self.names.binary_search_by(|n| n.as_str().cmp(name)).ok()?;
        Some(self.values[at].to_value())
    }

    fn is_output_groups(&self) -> bool {
        instance_of(self.provider.first().map(|p| p.to_value())) == "OutputGroupInfo"
    }
}

/// A field a rule cannot name: Rust keeps state in it, as `$executable` of
/// a `DefaultInfo`. It is not listed and does not print.
fn hidden(name: &str) -> bool {
    name.starts_with('$')
}

impl<'v, V> fmt::Display for StructGen<V>
where
    V: ValueLike<'v> + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Bazel prints what a `DefaultInfo` holds of runfiles and of files to
        // run as the Java objects they are.
        let default_info =
            instance_of(self.provider.first().map(|p| p.to_value())) == "DefaultInfo";
        write!(f, "struct(")?;
        let mut first = true;
        for (name, value) in self.names.iter().zip(&self.values) {
            if hidden(name) {
                continue;
            }
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            let java = default_info
                .then(|| match value.to_value().get_type() {
                    "runfiles" => Some("com.google.devtools.build.lib.analysis.Runfiles"),
                    "FilesToRunProvider" => Some(
                        "com.google.devtools.build.lib.analysis.FilesToRunProvider$FullFilesToRunProvider",
                    ),
                    _ => None,
                })
                .flatten();
            match java {
                Some(class) => write!(f, "{name} = <unknown object {class}>")?,
                None => write!(f, "{name} = {value}")?,
            }
        }
        write!(f, ")")
    }
}

/// The fields of `value` if it is a struct.
pub(crate) fn fields_of<'v>(value: Value<'v>) -> Option<Vec<(&'v str, Value<'v>)>> {
    fn collect<'v, V: ValueLike<'v>>(s: &'v StructGen<V>) -> Vec<(&'v str, Value<'v>)> {
        s.names
            .iter()
            .map(String::as_str)
            .zip(s.values.iter().map(|v| v.to_value()))
            .collect()
    }
    if let Some(live) = value.downcast_ref::<Struct<'v>>() {
        Some(collect(live))
    } else {
        value.downcast_ref::<FrozenStruct>().map(collect)
    }
}

/// The provider of `value` if it is a struct: `Some(None)` for what
/// `struct()` made, `Some(Some(p))` for an instance of `p`.
pub(crate) fn provider_of<'v>(value: Value<'v>) -> Option<Option<Value<'v>>> {
    if let Some(live) = value.downcast_ref::<Struct<'v>>() {
        Some(live.provider.first().copied())
    } else {
        value
            .downcast_ref::<FrozenStruct>()
            .map(|frozen| frozen.provider.first().map(|p| p.to_value()))
    }
}

/// Build the struct with these fields, in any order.
pub(crate) fn new_struct<'v>(heap: Heap<'v>, fields: Vec<(String, Value<'v>)>) -> Value<'v> {
    build(heap, None, fields)
}

/// Build an instance of `provider` with these fields, in any order.
pub(crate) fn new_instance<'v>(
    heap: Heap<'v>,
    provider: Value<'v>,
    fields: Vec<(String, Value<'v>)>,
) -> Value<'v> {
    build(heap, Some(provider), fields)
}

fn build<'v>(
    heap: Heap<'v>,
    provider: Option<Value<'v>>,
    mut fields: Vec<(String, Value<'v>)>,
) -> Value<'v> {
    fields.sort_by(|a, b| a.0.cmp(&b.0));
    let (names, values) = fields.into_iter().unzip();
    heap.alloc_complex(StructGen {
        names,
        values,
        provider: provider.into_iter().collect(),
    })
}

#[starlark_value(type = "struct")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for StructGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn get_attr(&self, attribute: &str, _heap: Heap<'v>) -> Option<Value<'v>> {
        self.get(attribute).filter(|_| !hidden(attribute))
    }

    fn has_attr(&self, attribute: &str, _heap: Heap<'v>) -> bool {
        !hidden(attribute) && self.get(attribute).is_some()
    }

    fn dir_attr(&self) -> Vec<String> {
        self.names.iter().filter(|n| !hidden(n)).cloned().collect()
    }

    /// An instance of a provider is named by it in an error, and lists
    /// what it has.
    fn no_attr_message(&self, attribute: &str) -> Option<String> {
        let provider = self.provider.first()?.to_value();
        let mut names: Vec<String> = self.names.iter().filter(|n| !hidden(n)).cloned().collect();
        names.sort();
        Some(format!(
            "'{}' value has no field or method '{attribute}'\nAvailable attributes: {}",
            crate::provider::instance_type_in_errors(provider),
            names.join(", ")
        ))
    }

    /// `"group" in output_groups` and `output_groups["group"]`: an
    /// `OutputGroupInfo` is also a mapping of its groups.
    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        match (self.is_output_groups(), other.unpack_str()) {
            (true, Some(name)) => Ok(self.get(name).is_some()),
            _ => Err(unsupported_binary("in", other.get_type(), "struct")),
        }
    }

    fn at(&self, index: Value<'v>, _heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        match (self.is_output_groups(), index.unpack_str()) {
            (true, Some(name)) => self
                .get(name)
                .ok_or_else(|| fatal(format!("key {name:?} not found in OutputGroupInfo"))),
            _ => starlark::values::ValueError::unsupported_with(self, "[]", index),
        }
    }

    fn set_attr(&self, _attribute: &str, _new_value: Value<'v>) -> starlark::Result<()> {
        Err(fatal("struct value does not support field assignment"))
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        for (name, value) in self.names.iter().zip(&self.values) {
            name.hash(hasher);
            value
                .to_value()
                .write_hash(hasher)
                .map_err(|_| fatal("unhashable type: 'struct'"))?;
        }
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        let Some(theirs) = fields_of(other) else {
            return Ok(false);
        };
        let mine = self.provider.first().map(|p| p.to_value());
        if !same_provider(mine, provider_of(other).flatten()) || theirs.len() != self.names.len() {
            return Ok(false);
        }
        for ((name, value), (their_name, their_value)) in
            self.names.iter().zip(&self.values).zip(theirs)
        {
            if name != their_name || !value.to_value().equals(their_value)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        // A select takes the sum over (`select`'s `radd`).
        if crate::select::is_select(rhs) {
            return None;
        }
        let Some(theirs) = fields_of(rhs) else {
            return Some(Err(unsupported_binary("+", "struct", rhs.get_type())));
        };
        let mine = self.provider.first().map(|p| p.to_value());
        let their_provider = provider_of(rhs).flatten();
        if !same_provider(mine, their_provider) {
            return Some(Err(fatal(format!(
                "Cannot use '+' operator on instances of different providers ({} and {})",
                instance_of(mine),
                instance_of(their_provider)
            ))));
        }
        let mut fields: Vec<(String, Value<'v>)> = self
            .names
            .iter()
            .cloned()
            .zip(self.values.iter().map(|v| v.to_value()))
            .collect();
        for (name, value) in theirs {
            if self.get(name).is_some() {
                return Some(Err(fatal(format!(
                    "cannot add struct instances with common field '{name}'"
                ))));
            }
            fields.push((name.to_owned(), value));
        }
        Some(Ok(build(heap, mine, fields)))
    }

    fn radd(&self, lhs: Value<'v>, _heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        Some(Err(unsupported_binary("+", lhs.get_type(), "struct")))
    }
}

/// `struct`: Bazel's is a `Provider` that prints as `<function struct>`.
#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct StructConstructor;

impl fmt::Display for StructConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<function struct>")
    }
}

starlark::starlark_simple_value!(StructConstructor);

#[starlark_value(type = "Provider")]
impl<'v> StarlarkValue<'v> for StructConstructor {
    fn name_for_call_stack(&self, _me: Value<'v>) -> String {
        "struct".to_owned()
    }

    /// `struct(**fields)`.
    fn invoke(
        &self,
        _me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        if args.positions(eval.heap())?.next().is_some() {
            return Err(fatal("struct() got unexpected positional argument"));
        }
        let fields = args
            .names_map()?
            .iter()
            .map(|(name, value)| (name.as_str().to_owned(), *value))
            .collect();
        Ok(new_struct(eval.heap(), fields))
    }
}

pub(crate) fn struct_globals(builder: &mut starlark::environment::GlobalsBuilder) {
    builder.set("struct", StructConstructor);
    type_global(builder);
}

/// The builtin provider that is the type of an instance of itself in Bazel,
/// and the type's name: the providers Bazel implements in Java say their own
/// name, where an instance of a provider a `.bzl` made is a `struct`.
const NATIVE_PROVIDER_TYPES: [(&str, &str); 11] = [
    ("DefaultInfo", "DefaultInfo"),
    ("OutputGroupInfo", "OutputGroupInfo"),
    ("RunEnvironmentInfo", "RunEnvironmentInfo"),
    ("InstrumentedFilesInfo", "InstrumentedFilesInfo"),
    ("AnalysisTestResultInfo", "AnalysisTestResultInfo"),
    ("AnalysisFailureInfo", "AnalysisFailureInfo"),
    ("_ToolchainInfo", "ToolchainInfo"),
    ("_FeatureFlagInfo", "FeatureFlagInfo"),
    ("_ExecutionInfo", "ExecutionInfo"),
    ("_TemplateVariableInfo", "TemplateVariableInfo"),
    ("_PlatformInfo", "PlatformInfo"),
];

#[starlark::starlark_module]
fn type_global(builder: &mut starlark::environment::GlobalsBuilder) {
    /// `type(x)`: the name of the type of `x`; for an instance of a provider
    /// Bazel implements natively, the provider's.
    fn r#type<'v>(
        #[starlark(require = pos)] a: Value<'v>,
        heap: Heap<'v>,
    ) -> starlark::Result<Value<'v>> {
        if let Some(Some(provider)) = provider_of(a) {
            for (builtin_name, type_name) in NATIVE_PROVIDER_TYPES {
                if same_provider(
                    crate::analysis::target::builtin(builtin_name),
                    Some(provider),
                ) {
                    return Ok(heap.alloc(type_name));
                }
            }
        }
        Ok(a.get_type_value().to_value())
    }
}
