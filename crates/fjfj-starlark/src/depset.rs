//! `depset` (buildfiji-mum.14).
//!
//! A depset is a DAG of small sets that is flattened only on demand, so
//! building one is O(direct + transitive.len()) however big the sets below
//! it, and a set reached through many paths exists once. That sharing is the
//! point: it is what lets a build of a hundred thousand targets pass link
//! inputs up a dependency tree without copying them at every level.
//!
//! The flattening rules are Bazel's `NestedSet`, and were read off Bazel
//! 9.2.0 rather than its documentation, which is loose about them:
//!
//! - every set lays out its children when it is built, by *its own* order:
//!   `default` and `postorder` put transitive sets before direct items,
//!   `preorder` puts direct items first, and `topological` is the reverse of
//!   the `preorder` layout;
//! - `to_list` is then one uniform depth-first walk over those layouts,
//!   keeping the first occurrence of each element and never re-entering a set
//!   it has been in, and the result is reversed when the *root* is
//!   topological. Nothing else distinguishes the orders: `default` is
//!   `postorder`, and a `topological` set inside a `default` one shows its
//!   own layout, reversed items and all;
//! - the direct items are de-duplicated when the set is built, the elements
//!   of all the sets are compared with `==`, and elements must be hashable
//!   (a stand-in for Bazel's "immutable") and all of one type;
//! - a set with no direct items and one non-empty transitive set of the same
//!   order *is* that set, so `depset(transitive = [a]) == a`, while two
//!   depsets are otherwise equal only if they are the same set or both empty
//!   with one order.

use crate::args::{Wording, bind, fatal, param, want_sequence};
use allocative::Allocative;
use starlark::collections::SmallSet;
use starlark::environment::{GlobalsBuilder, Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::starlark_module;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueIdentity, ValueLike,
};
use starlark_derive::starlark_value;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;

/// A depset's `order`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Allocative)]
pub enum Order {
    Default,
    Postorder,
    Preorder,
    Topological,
}

impl Order {
    /// The four names Bazel 9 accepts; `stable`, `compile`, `link` and
    /// `naive_link` were removed.
    pub fn parse(name: &str) -> Option<Order> {
        Some(match name {
            "default" => Order::Default,
            "postorder" => Order::Postorder,
            "preorder" => Order::Preorder,
            "topological" => Order::Topological,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Order::Default => "default",
            Order::Postorder => "postorder",
            Order::Preorder => "preorder",
            Order::Topological => "topological",
        }
    }

    /// `default` goes with any order, any other only with itself.
    pub fn compatible(self, other: Order) -> bool {
        self == other || self == Order::Default || other == Order::Default
    }
}

/// A depset, before or after freezing.
///
/// `items` and `sets` are each in layout order (see the module docs); which
/// comes first in the combined layout is `sets_first`.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub struct DepsetGen<V> {
    #[trace(static)]
    order: Order,
    /// The type of every element, `None` for an empty set.
    #[trace(static)]
    item_type: Option<&'static str>,
    #[trace(static)]
    sets_first: bool,
    items: Vec<V>,
    /// Non-empty depsets, as values.
    sets: Vec<V>,
}

starlark_complex_value!(pub Depset);

/// Free stack below which freezing a nested set moves onto a new segment, and
/// the size of that segment.
const RED_ZONE: usize = 256 * 1024;
const NEW_STACK: usize = 8 * 1024 * 1024;

impl<'v> Freeze for Depset<'v> {
    type Frozen = FrozenDepset;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenDepset> {
        let freeze_all = |values: Vec<Value<'v>>| -> FreezeResult<Vec<FrozenValue>> {
            values
                .into_iter()
                // Each nested set is frozen by a call into this function, so a
                // chain of depsets is as deep in stack as it is in sets: grow
                // the stack rather than cap the chain.
                .map(|v| stacker::maybe_grow(RED_ZONE, NEW_STACK, || v.freeze(freezer)))
                .collect()
        };
        Ok(DepsetGen {
            order: self.order,
            item_type: self.item_type,
            sets_first: self.sets_first,
            items: freeze_all(self.items)?,
            sets: freeze_all(self.sets)?,
        })
    }
}

/// One child of a set: a direct element or a nested depset.
enum Kid<'v> {
    Item(Value<'v>),
    Set(Value<'v>),
}

/// What the walk needs of a depset, whichever heap it lives in.
trait Layout<'v> {
    fn len(&self) -> usize;
    /// The `i`th child in layout order.
    fn kid(&self, i: usize) -> Kid<'v>;
    fn order(&self) -> Order;
    fn item_type(&self) -> Option<&'static str>;
}

impl<'v, V: ValueLike<'v>> Layout<'v> for DepsetGen<V> {
    fn len(&self) -> usize {
        self.items.len() + self.sets.len()
    }

    fn kid(&self, i: usize) -> Kid<'v> {
        let (first, second, first_is_set) = if self.sets_first {
            (&self.sets, &self.items, true)
        } else {
            (&self.items, &self.sets, false)
        };
        let (list, index, is_set) = if i < first.len() {
            (first, i, first_is_set)
        } else {
            (second, i - first.len(), !first_is_set)
        };
        let value = list[index].to_value();
        if is_set {
            Kid::Set(value)
        } else {
            Kid::Item(value)
        }
    }

    fn order(&self) -> Order {
        self.order
    }

    fn item_type(&self) -> Option<&'static str> {
        self.item_type
    }
}

/// The layout of `value` if it is a depset.
fn layout_of<'v>(value: Value<'v>) -> Option<&'v dyn Layout<'v>> {
    match value.downcast_ref::<Depset<'v>>() {
        Some(live) => Some(live),
        None => value
            .downcast_ref::<FrozenDepset>()
            .map(|frozen| frozen as &dyn Layout<'v>),
    }
}

/// Every element of `top`, in `to_list` order. `root` is `top` as a value,
/// when the caller has it, so that a set is never walked into twice.
fn flatten<'v>(top: &dyn Layout<'v>, root: Option<Value<'v>>) -> starlark::Result<Vec<Value<'v>>> {
    let mut out = Vec::new();
    let mut seen: SmallSet<Value<'v>> = SmallSet::new();
    let mut entered: HashSet<ValueIdentity<'v>> = HashSet::new();
    entered.extend(root.map(Value::identity));
    // An explicit stack: depsets nest as deep as a dependency chain, and the
    // walk must not.
    let mut stack: Vec<(&dyn Layout<'v>, usize)> = vec![(top, 0)];
    while let Some((set, next)) = stack.last_mut() {
        if *next >= set.len() {
            stack.pop();
            continue;
        }
        let kid = set.kid(*next);
        *next += 1;
        match kid {
            Kid::Item(value) => {
                let hashed = value.get_hashed()?;
                if seen.insert_hashed(hashed) {
                    out.push(value);
                }
            }
            Kid::Set(value) => {
                if entered.insert(value.identity())
                    && let Some(child) = layout_of(value)
                {
                    stack.push((child, 0));
                }
            }
        }
    }
    if top.order() == Order::Topological {
        out.reverse();
    }
    Ok(out)
}

impl<'v, V: ValueLike<'v>> DepsetGen<V> {
    fn is_empty(&self) -> bool {
        self.items.is_empty() && self.sets.is_empty()
    }
}

impl<'v, V: ValueLike<'v>> fmt::Display for DepsetGen<V> {
    /// `depset([1, 2])`, plus `, order = "postorder"` when it is not the
    /// default: what Bazel's `repr` gives.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "depset([")?;
        for (i, item) in flatten(self, None).unwrap_or_default().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item.to_repr())?;
        }
        write!(f, "]")?;
        if self.order != Order::Default {
            write!(f, ", order = \"{}\"", self.order.name())?;
        }
        write!(f, ")")
    }
}

fn unsupported(op: &str, lhs: &str, rhs: &str) -> starlark::Error {
    fatal(format!("unsupported binary operation: {lhs} {op} {rhs}"))
}

#[starlark_value(type = "depset")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for DepsetGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("depset", depset_methods);
        Some(RES.methods())
    }

    fn to_bool(&self) -> bool {
        !self.is_empty()
    }

    /// Stable across freezing, and equal for sets that are `==`.
    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        self.order.hash(hasher);
        (self.items.len() + self.sets.len()).hash(hasher);
        Ok(())
    }

    /// Only asked when the two are not the same object: equal then means
    /// both empty, with the same order.
    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(layout_of(other)
            .is_some_and(|o| self.is_empty() && o.len() == 0 && o.order() == self.order))
    }

    fn compare(&self, other: Value<'v>) -> starlark::Result<std::cmp::Ordering> {
        Err(fatal(format!(
            "unsupported comparison: depset <=> {}",
            other.get_type()
        )))
    }

    fn add(&self, rhs: Value<'v>, _heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        // A select takes the sum over (`select`'s `radd`).
        if crate::select::is_select(rhs) {
            return None;
        }
        Some(Err(unsupported("+", "depset", rhs.get_type())))
    }

    fn radd(&self, lhs: Value<'v>, _heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        Some(Err(unsupported("+", lhs.get_type(), "depset")))
    }

    fn bit_or(&self, other: Value<'v>, _heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Err(unsupported("|", "depset", other.get_type()))
    }

    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        Err(unsupported("in", other.get_type(), "depset"))
    }

    fn length(&self) -> starlark::Result<i32> {
        Err(fatal(
            "in call to len(), parameter 'x' got value of type 'depset', want 'iterable or string'",
        ))
    }
}

#[starlark_module]
fn depset_methods(builder: &mut MethodsBuilder) {
    /// The elements, in this depset's order, each once.
    fn to_list<'v>(this: Value<'v>) -> starlark::Result<Vec<Value<'v>>> {
        flatten(
            layout_of(this).expect("a depset method's receiver is a depset"),
            Some(this),
        )
    }
}

/// Build the depset `depset(direct, order, transitive = ...)` builds.
///
/// Errors are Bazel's. A depset that is exactly one of `transitive` is
/// returned as that value.
pub fn new_depset<'v>(
    heap: Heap<'v>,
    direct: &[Value<'v>],
    order: Order,
    transitive: &[Value<'v>],
) -> starlark::Result<Value<'v>> {
    // Every child must be an order this one goes with, empty or not.
    let mut children = Vec::with_capacity(transitive.len());
    for &value in transitive {
        let child = layout_of(value).expect("checked to be a depset");
        if !order.compatible(child.order()) {
            return Err(fatal(format!(
                "Order '{}' is incompatible with order '{}'",
                order.name(),
                child.order().name()
            )));
        }
        children.push((value, child));
    }

    let mut item_type: Option<&'static str> = None;
    let mut items: Vec<Value<'v>> = Vec::with_capacity(direct.len());
    let mut seen: SmallSet<Value<'v>> = SmallSet::new();
    for &value in direct {
        // Hashable stands in for Bazel's "not mutable".
        let hashed = value
            .get_hashed()
            .map_err(|_| fatal("depset elements must not be mutable values"))?;
        let ty = value.get_type();
        match item_type {
            None => item_type = Some(ty),
            Some(expected) if expected != ty => {
                return Err(fatal(format!(
                    "cannot add an item of type '{ty}' to a depset of '{expected}'"
                )));
            }
            Some(_) => {}
        }
        if seen.insert_hashed(hashed) {
            items.push(value);
        }
    }

    let mut sets: Vec<Value<'v>> = Vec::with_capacity(children.len());
    for (value, child) in children {
        let Some(ty) = child.item_type() else {
            continue; // empty
        };
        match item_type {
            None => item_type = Some(ty),
            Some(expected) if expected != ty => {
                return Err(fatal(format!(
                    "cannot add an item of type '{ty}' to a depset of '{expected}'"
                )));
            }
            Some(_) => {}
        }
        sets.push(value);
    }

    if items.is_empty()
        && let [only] = sets.as_slice()
        && layout_of(*only).is_some_and(|o| o.order() == order)
    {
        return Ok(*only);
    }

    let sets_first = order != Order::Preorder;
    if order == Order::Topological {
        // The reverse of the preorder layout.
        items.reverse();
        sets.reverse();
    }
    Ok(heap.alloc_complex(DepsetGen {
        order,
        item_type,
        sets_first,
        items,
        sets,
    }))
}

#[starlark_module]
pub(crate) fn depset_globals(builder: &mut GlobalsBuilder) {
    /// `depset(direct = None, order = "default", *, transitive = None)`.
    fn depset<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "depset",
            Wording::Signature,
            &[
                param("direct", true, false),
                param("order", true, false),
                param("transitive", false, false),
            ],
            args,
            eval,
        )?;
        let direct = match bound[0] {
            Some(v) => want_sequence("depset", "direct", v, true)?.unwrap_or_default(),
            None => Vec::new(),
        };
        let order = match bound[1] {
            Some(v) => {
                let name = v.unpack_str().ok_or_else(|| {
                    fatal(format!(
                        "in call to depset(), parameter 'order' got value of type '{}', want \
                         'string'",
                        v.get_type()
                    ))
                })?;
                Order::parse(name).ok_or_else(|| fatal(format!("Invalid order: {name}")))?
            }
            None => Order::Default,
        };
        let transitive = match bound[2] {
            Some(v) => want_sequence("depset", "transitive", v, true)?.unwrap_or_default(),
            None => Vec::new(),
        };
        for (i, value) in transitive.iter().enumerate() {
            if layout_of(*value).is_none() {
                return Err(fatal(format!(
                    "at index {i} of transitive, got element of type {}, want depset",
                    value.get_type()
                )));
            }
        }
        new_depset(eval.heap(), &direct, order, &transitive)
    }
}

/// The depset `value` as the nested set it is, each depset once however
/// many sets hold it, or `None` if it is not a depset or an element is not
/// one `element` knows. Order is not kept: the nested set lists a set's own
/// elements and the sets below it apart.
pub fn nested_of<'v, T: Clone + Ord>(
    value: Value<'v>,
    element: &dyn Fn(Value<'v>) -> Option<T>,
) -> Option<fjfj_graph::NestedSet<T>> {
    use fjfj_graph::NestedSet;
    use std::collections::HashMap;
    use std::sync::Arc;
    struct Frame<'v, T> {
        value: Value<'v>,
        layout: &'v dyn Layout<'v>,
        next: usize,
        direct: Vec<T>,
        transitive: Vec<Arc<NestedSet<T>>>,
    }
    let top = layout_of(value)?;
    let mut built: HashMap<ValueIdentity<'v>, Arc<NestedSet<T>>> = HashMap::new();
    let mut stack = vec![Frame {
        value,
        layout: top,
        next: 0,
        direct: Vec::new(),
        transitive: Vec::new(),
    }];
    loop {
        let frame = stack.last_mut()?;
        if frame.next < frame.layout.len() {
            let kid = frame.layout.kid(frame.next);
            frame.next += 1;
            match kid {
                Kid::Item(item) => {
                    let item = element(item)?;
                    if !frame.direct.contains(&item) {
                        frame.direct.push(item);
                    }
                }
                Kid::Set(set) => {
                    if let Some(done) = built.get(&set.identity()) {
                        frame.transitive.push(done.clone());
                    } else if let Some(layout) = layout_of(set) {
                        stack.push(Frame {
                            value: set,
                            layout,
                            next: 0,
                            direct: Vec::new(),
                            transitive: Vec::new(),
                        });
                    }
                }
            }
            continue;
        }
        let frame = stack.pop()?;
        let set = Arc::new(NestedSet::new(frame.direct, frame.transitive));
        built.insert(frame.value.identity(), set.clone());
        match stack.last_mut() {
            Some(parent) => parent.transitive.push(set),
            None => return Some(Arc::unwrap_or_clone(set)),
        }
    }
}

/// Whether `value` is a depset.
pub fn is_depset(value: Value<'_>) -> bool {
    layout_of(value).is_some()
}

/// The elements of a depset value in `to_list` order, or `None` if `value`
/// is not one.
pub fn depset_to_list<'v>(value: Value<'v>) -> Option<starlark::Result<Vec<Value<'v>>>> {
    layout_of(value).map(|d| flatten(d, Some(value)))
}
