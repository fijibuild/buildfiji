//! `select()` (buildfiji-mum.3.6): a value that depends on the
//! configuration, and what `+` and `|` do with one.
//!
//! What Bazel 9.2.0 does, read off probes:
//!
//! - `select(x, no_match_error = "")` takes a non-empty dict, whose keys are
//!   label strings or `Label`s, and copies it. The keys are not read as labels
//!   until an attribute takes the value. The result has type `select`, is
//!   always true, hashable and iterable never, and prints as
//!   `select({"a": 1})`.
//! - `+` and `|` build a list of the operands: a plain value and a `select`
//!   are each one element, and the elements of a longer list are kept, so
//!   `[1] + select(..) + [2]` prints as written and adjacent plain values are
//!   added first. Every element has a type (a plain value's, or the type of a
//!   `select`'s first value), and the operation is refused unless
//!   [`combine`]'s rules hold, in this order:
//!   - `+`: a plain dict is unsupported; two types that differ (list, tuple
//!     and range count as one) are incompatible; and a dict type is
//!     unsupported again, since dicts join with `|`;
//!   - `|`: a plain value that is not a dict is unsupported; neither type a
//!     dict is unsupported; exactly one is incompatible.
//! - Two selects are equal when their elements are and each `select` has the
//!   same `no_match_error`; Java's equality, so `1` is not `1.0`.
//!
//! The crate gives `dict | select` no hook: a plain dict on the left of `|`
//! fails in the dict's own words (buildfiji-v32).

use crate::args::{
    Param, Wording, bind, fatal, param, positional_only, sequence, unsupported_binary,
};
use crate::label::label_of_value;
use allocative::Allocative;
use starlark::environment::GlobalsBuilder;
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::starlark_module;
use starlark::values::dict::{AllocDict, DictRef};
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;

/// One element of a select: a plain value, or a `select()`'s dict.
#[derive(Debug, Trace, Coerce, Allocative)]
#[repr(C)]
struct ElementGen<V> {
    /// The dict of a `select()`, or the plain value.
    value: V,
    #[trace(static)]
    select: bool,
    #[trace(static)]
    no_match_error: String,
}

/// A select, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub struct SelectGen<V> {
    elements: Vec<ElementGen<V>>,
    #[trace(static)]
    pipe: bool,
}

starlark_complex_value!(pub Select);

impl<'v> Freeze for Select<'v> {
    type Frozen = FrozenSelect;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenSelect> {
        Ok(SelectGen {
            elements: self
                .elements
                .into_iter()
                .map(|e| {
                    Ok(ElementGen {
                        value: e.value.freeze(freezer)?,
                        select: e.select,
                        no_match_error: e.no_match_error,
                    })
                })
                .collect::<FreezeResult<Vec<ElementGen<FrozenValue>>>>()?,
            pipe: self.pipe,
        })
    }
}

impl<V: fmt::Display> fmt::Display for SelectGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, "{}", if self.pipe { " | " } else { " + " })?;
            }
            if e.select {
                write!(f, "select({})", e.value)?;
            } else {
                write!(f, "{}", e.value)?;
            }
        }
        Ok(())
    }
}

/// An element as a reader sees it.
#[derive(Clone)]
pub(crate) struct Element<'v> {
    /// A `select()` (and then `value` is its dict) rather than a plain value.
    pub(crate) select: bool,
    pub(crate) value: Value<'v>,
    pub(crate) no_match_error: String,
}

/// The elements of `value` if it is a select, and whether they are joined
/// with `|`.
pub(crate) fn view<'v>(value: Value<'v>) -> Option<(Vec<Element<'v>>, bool)> {
    fn collect<'v, V: ValueLike<'v>>(s: &SelectGen<V>) -> (Vec<Element<'v>>, bool) {
        let elements = s
            .elements
            .iter()
            .map(|e| Element {
                select: e.select,
                value: e.value.to_value(),
                no_match_error: e.no_match_error.clone(),
            })
            .collect();
        (elements, s.pipe)
    }
    if let Some(live) = value.downcast_ref::<Select<'v>>() {
        Some(collect(live))
    } else {
        value.downcast_ref::<FrozenSelect>().map(collect)
    }
}

pub(crate) fn is_select(value: Value<'_>) -> bool {
    value.downcast_ref::<FrozenSelect>().is_some() || value.get_type() == "select"
}

/// Build a select of `elements`, which are `(is a select, value, its
/// no_match_error)`.
pub(crate) fn alloc<'v>(heap: Heap<'v>, elements: Vec<Element<'v>>, pipe: bool) -> Value<'v> {
    heap.alloc_complex(SelectGen {
        elements: elements
            .into_iter()
            .map(|e| ElementGen {
                value: e.value,
                select: e.select,
                no_match_error: e.no_match_error,
            })
            .collect(),
        pipe,
    })
}

/// The type the first element of a select gives it.
fn type_of(elements: &[Element<'_>]) -> String {
    let first = &elements[0];
    if first.select {
        DictRef::from_value(first.value)
            .and_then(|d| d.iter().next().map(|(_, v)| v.get_type().to_owned()))
            .unwrap_or_default()
    } else {
        first.value.get_type().to_owned()
    }
}

/// An operand of `+` or `|`: a select, with the type it has, or a plain
/// value.
struct Operand<'v> {
    select: bool,
    ty: String,
    elements: Vec<Element<'v>>,
}

impl<'v> Operand<'v> {
    fn of(value: Value<'v>) -> Operand<'v> {
        match view(value) {
            Some((elements, _)) => Operand {
                select: true,
                ty: type_of(&elements),
                elements,
            },
            None => Operand {
                select: false,
                ty: value.get_type().to_owned(),
                elements: vec![Element {
                    select: false,
                    value,
                    no_match_error: String::new(),
                }],
            },
        }
    }

    /// How an error names it.
    fn described(&self) -> String {
        if self.select {
            format!("select of {}", self.ty)
        } else {
            self.ty.clone()
        }
    }

    /// How "unsupported binary operation" names it.
    fn kind(&self) -> &str {
        if self.select { "select" } else { &self.ty }
    }
}

/// Two types that `+` may join: the same, or both a kind of sequence.
fn compatible(a: &str, b: &str) -> bool {
    let sequence = |t: &str| matches!(t, "list" | "tuple" | "range");
    a == b || (sequence(a) && sequence(b))
}

/// `lhs op rhs` where at least one is a select, `op` being `+` or `|`.
fn combine<'v>(
    heap: Heap<'v>,
    op: &str,
    lhs: Value<'v>,
    rhs: Value<'v>,
) -> starlark::Result<Value<'v>> {
    let (a, b) = (Operand::of(lhs), Operand::of(rhs));
    let unsupported = || unsupported_binary(op, a.kind(), b.kind());
    let incompatible = || {
        fatal(format!(
            "Cannot combine incompatible types ({}, {})",
            a.described(),
            b.described()
        ))
    };
    if op == "+" {
        if (!a.select && a.ty == "dict") || (!b.select && b.ty == "dict") {
            return Err(unsupported());
        }
        if !compatible(&a.ty, &b.ty) {
            return Err(incompatible());
        }
        if a.ty == "dict" {
            return Err(unsupported());
        }
    } else {
        if (!a.select && a.ty != "dict") || (!b.select && b.ty != "dict") {
            return Err(unsupported());
        }
        match (a.ty == "dict", b.ty == "dict") {
            (false, false) => return Err(unsupported()),
            (true, true) => {}
            _ => return Err(incompatible()),
        }
    }
    let mut elements = a.elements;
    elements.extend(b.elements);
    Ok(alloc(heap, elements, op == "|"))
}

/// Java's `equals` on the values of a select: a dict ignores order and `1`
/// is not `1.0`.
fn same<'v>(a: Value<'v>, b: Value<'v>) -> starlark::Result<bool> {
    if a.get_type() != b.get_type() {
        return Ok(false);
    }
    if let (Some(x), Some(y)) = (DictRef::from_value(a), DictRef::from_value(b)) {
        if x.len() != y.len() {
            return Ok(false);
        }
        for (k, v) in x.iter() {
            match y.get(k)? {
                Some(w) if same(v, w)? => {}
                _ => return Ok(false),
            }
        }
        return Ok(true);
    }
    if let (Some(x), Some(y)) = (sequence(a), sequence(b)) {
        if x.len() != y.len() {
            return Ok(false);
        }
        for (p, q) in x.into_iter().zip(y) {
            if !same(p, q)? {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    a.equals(b)
}

#[starlark_value(type = "select")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for SelectGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        let Some((theirs, pipe)) = view(other) else {
            return Ok(false);
        };
        if theirs.len() != self.elements.len() || pipe != self.pipe {
            return Ok(false);
        }
        for (mine, theirs) in self.elements.iter().zip(&theirs) {
            if mine.select != theirs.select
                || mine.no_match_error != theirs.no_match_error
                || !same(mine.value.to_value(), theirs.value)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn add(&self, rhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        Some(combine(heap, "+", self_value(self, heap), rhs))
    }

    fn radd(&self, lhs: Value<'v>, heap: Heap<'v>) -> Option<starlark::Result<Value<'v>>> {
        Some(combine(heap, "+", lhs, self_value(self, heap)))
    }

    fn bit_or(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        combine(heap, "|", self_value(self, heap), rhs)
    }
}

/// `this` as a value again, to hand to [`combine`].
fn self_value<'v, V: ValueLike<'v>>(this: &SelectGen<V>, heap: Heap<'v>) -> Value<'v> {
    alloc(
        heap,
        this.elements
            .iter()
            .map(|e| Element {
                select: e.select,
                value: e.value.to_value(),
                no_match_error: e.no_match_error.clone(),
            })
            .collect(),
        this.pipe,
    )
}

const SELECT_PARAMS: [Param; 2] = [
    positional_only("x", true),
    param("no_match_error", true, false),
];

/// `select(x, no_match_error = "")`.
fn make_select<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    let bound = bind("select", Wording::Signature, &SELECT_PARAMS, args, eval)?;
    let x = bound[0].expect("required");
    let want = |param: &str, value: Value<'_>, want: &str| {
        fatal(format!(
            "in call to select(), parameter '{param}' got value of type '{}', want '{want}'",
            value.get_type()
        ))
    };
    let dict = DictRef::from_value(x).ok_or_else(|| want("x", x, "dict"))?;
    let no_match_error = match bound[1] {
        None => String::new(),
        Some(v) => v
            .unpack_str()
            .ok_or_else(|| want("no_match_error", v, "string"))?
            .to_owned(),
    };
    if dict.is_empty() {
        return Err(fatal(
            "select({}) with an empty dictionary can never resolve because it includes no \
             conditions to match",
        ));
    }
    for (key, _) in dict.iter() {
        if key.unpack_str().is_none() && label_of_value(key).is_none() {
            return Err(fatal(format!(
                "select: got {} for dict key, want a Label or label string",
                key.get_type()
            )));
        }
    }
    let heap = eval.heap();
    let copy = heap.alloc(AllocDict(dict.iter()));
    Ok(alloc(
        heap,
        vec![Element {
            select: true,
            value: copy,
            no_match_error,
        }],
        false,
    ))
}

#[starlark_module]
pub(crate) fn select_globals(builder: &mut GlobalsBuilder) {
    /// `select(x, no_match_error = "")`.
    fn select<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_select(args, eval)
    }
}
