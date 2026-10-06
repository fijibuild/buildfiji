//! What `existing_rule` and `existing_rules` return: a read-only `Map`
//! (buildfiji-9mb5).
//!
//! Bazel 9.2.0's are views of the rule's attributes, not dicts: `type` says
//! `Map`, `dir` lists `get`, `items`, `keys` and `values` and nothing else, a
//! view is equal only to itself, a missing key is `key "k" not found in view`,
//! and `dict(view)` and `view | {...}` make a dict of the entries.

use allocative::Allocative;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_complex_value;
use starlark::starlark_module;
use starlark::values::dict::DictRef;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;

/// A view of `dict`, which is shown as `repr`.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub struct MapViewGen<V> {
    dict: V,
    #[trace(static)]
    repr: String,
}

starlark_complex_value!(pub MapView);

impl<'v> Freeze for MapView<'v> {
    type Frozen = FrozenMapView;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenMapView> {
        Ok(MapViewGen {
            dict: self.dict.freeze(freezer)?,
            repr: self.repr,
        })
    }
}

impl<V> fmt::Display for MapViewGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.repr)
    }
}

/// A view of the entries `dict` holds, shown as `repr`.
pub(crate) fn map_view<'v>(heap: Heap<'v>, dict: Value<'v>, repr: String) -> Value<'v> {
    heap.alloc(MapViewGen { dict, repr })
}

impl<'v, V: ValueLike<'v>> MapViewGen<V> {
    fn entries(&self) -> DictRef<'v> {
        DictRef::from_value(self.dict.to_value()).expect("a view of a dict")
    }
}

#[starlark_value(type = "Map")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for MapViewGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("Map", map_view_methods);
        Some(RES.methods())
    }

    fn at(&self, index: Value<'v>, _heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        self.entries()
            .get(index)?
            .ok_or_else(|| crate::args::fatal(format!("key {} not found in view", index.to_repr())))
    }

    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(self.entries().get(other)?.is_some())
    }

    fn length(&self) -> starlark::Result<i32> {
        Ok(self.entries().len() as i32)
    }

    fn iterate_collect(&self, _heap: Heap<'v>) -> starlark::Result<Vec<Value<'v>>> {
        Ok(self.entries().keys().collect())
    }

    fn bit_or(&self, other: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        self.dict.to_value().bit_or(other, heap)
    }

    fn as_mapping(&self, _heap: Heap<'v>) -> Option<Value<'v>> {
        Some(self.dict.to_value())
    }
}

#[starlark_module]
fn map_view_methods(builder: &mut MethodsBuilder) {
    /// The value of `key`, or `default`.
    fn get<'v>(
        this: Value<'v>,
        key: Value<'v>,
        default: Option<Value<'v>>,
        heap: Heap<'v>,
    ) -> starlark::Result<Value<'v>> {
        let found = match this.as_mapping(heap) {
            Some(dict) => DictRef::from_value(dict).expect("a dict").get(key)?,
            None => None,
        };
        Ok(found.or(default).unwrap_or_else(Value::new_none))
    }

    fn keys<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Vec<Value<'v>>> {
        Ok(entries_of(this, heap).into_iter().map(|(k, _)| k).collect())
    }

    fn values<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Vec<Value<'v>>> {
        Ok(entries_of(this, heap).into_iter().map(|(_, v)| v).collect())
    }

    fn items<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Vec<Value<'v>>> {
        Ok(entries_of(this, heap)
            .into_iter()
            .map(|(k, v)| heap.alloc((k, v)))
            .collect())
    }
}

fn entries_of<'v>(view: Value<'v>, heap: Heap<'v>) -> Vec<(Value<'v>, Value<'v>)> {
    view.as_mapping(heap)
        .and_then(DictRef::from_value)
        .map(|d| d.iter().collect())
        .unwrap_or_default()
}
