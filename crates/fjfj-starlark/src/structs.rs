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

use crate::args::{fatal, unsupported_binary};
use allocative::Allocative;
use starlark::collections::StarlarkHasher;
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::starlark_module;
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
        })
    }
}

impl<'v, V: ValueLike<'v>> StructGen<V> {
    fn get(&self, name: &str) -> Option<Value<'v>> {
        let at = self.names.binary_search_by(|n| n.as_str().cmp(name)).ok()?;
        Some(self.values[at].to_value())
    }
}

impl<V> fmt::Display for StructGen<V>
where
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct(")?;
        for (i, (name, value)) in self.names.iter().zip(&self.values).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name} = {value}")?;
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

/// Build the struct with these fields, in any order.
pub(crate) fn new_struct<'v>(heap: Heap<'v>, mut fields: Vec<(String, Value<'v>)>) -> Value<'v> {
    fields.sort_by(|a, b| a.0.cmp(&b.0));
    let (names, values) = fields.into_iter().unzip();
    heap.alloc_complex(StructGen { names, values })
}

#[starlark_value(type = "struct")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for StructGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn get_attr(&self, attribute: &str, _heap: Heap<'v>) -> Option<Value<'v>> {
        self.get(attribute)
    }

    fn has_attr(&self, attribute: &str, _heap: Heap<'v>) -> bool {
        self.get(attribute).is_some()
    }

    fn dir_attr(&self) -> Vec<String> {
        self.names.clone()
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
        if theirs.len() != self.names.len() {
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
        let Some(theirs) = fields_of(rhs) else {
            return Some(Err(unsupported_binary("+", "struct", rhs.get_type())));
        };
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
        Some(Ok(new_struct(heap, fields)))
    }

    fn radd(&self, lhs: Value<'v>, _heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        Some(Err(unsupported_binary("+", lhs.get_type(), "struct")))
    }
}

#[starlark_module]
pub(crate) fn struct_globals(builder: &mut starlark::environment::GlobalsBuilder) {
    /// `struct(**fields)`.
    fn r#struct<'v>(
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
