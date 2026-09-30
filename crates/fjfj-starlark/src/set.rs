//! `set` (buildfiji-mum.3.1), as Bazel 9.2.0 has it.
//!
//! The `starlark` crate has a set of its own, and it is not Bazel's: it
//! lacks half the methods (`isdisjoint`, every `*_update`), takes the
//! variadic ones one argument at a time, pops from the end, and words its
//! errors differently. So this is a set of its own, insertion-ordered:
//!
//! - `set(elements)` takes a list, tuple, dict, set or range, and refuses a
//!   string or a depset; the `set` methods that take "others" take the same;
//! - `pop()` removes the *first* element, `remove` of a missing element is an
//!   error and `discard` is not;
//! - a set is not hashable, and mutating one that is frozen, or that a `for`
//!   loop is walking, is an error even when the call would change nothing;
//! - `|`, `&`, `-` and `^` need a set on both sides and make a new one.
//!
//! What the `starlark` crate's interpreter fixes and this cannot: `s |= t`
//! and its `&=`, `-=` and `^=` siblings rebind `s` rather than mutating it
//! (buildfiji-tg2).

use crate::args::{Wording, bind, fatal, positional_only, unsupported_binary};
use allocative::Allocative;
use starlark::collections::{SmallSet, StarlarkHasher};
use starlark::environment::{GlobalsBuilder, Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_values;
use starlark::starlark_module;
use starlark::values::none::NoneType;
use starlark::values::{
    Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::cell::{Cell, RefCell};
use std::fmt;

/// The members of a set, live or frozen.
pub(crate) trait Members<'v>: fmt::Debug + Allocative {
    /// Look at the members.
    fn look<R>(&self, f: impl FnOnce(&SmallSet<Value<'v>>) -> R) -> R;
    /// A `for` loop starts or stops walking this set.
    fn walking(&self, _delta: i32) {}
}

/// A set that can still change.
#[derive(Debug, Trace, ProvidesStaticType, Allocative)]
pub struct Live<'v> {
    content: RefCell<SmallSet<Value<'v>>>,
    /// How many `for` loops are walking it.
    #[trace(static)]
    #[allocative(skip)]
    walkers: Cell<u32>,
}

/// A set after freezing.
#[derive(Debug, Trace, ProvidesStaticType, Allocative)]
pub struct Frozen {
    content: SmallSet<FrozenValue>,
}

impl<'v> Members<'v> for Live<'v> {
    fn look<R>(&self, f: impl FnOnce(&SmallSet<Value<'v>>) -> R) -> R {
        f(&self.content.borrow())
    }

    fn walking(&self, delta: i32) {
        self.walkers
            .set(self.walkers.get().saturating_add_signed(delta));
    }
}

impl<'v> Members<'v> for Frozen {
    fn look<R>(&self, f: impl FnOnce(&SmallSet<Value<'v>>) -> R) -> R {
        f(starlark::coerce::coerce(&self.content))
    }
}

#[derive(Debug, Trace, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative)]
pub struct SetOf<T>(T);

pub type Set<'v> = SetOf<Live<'v>>;
pub type FrozenSet = SetOf<Frozen>;

starlark_complex_values!(Set);

impl<'v> Freeze for Set<'v> {
    type Frozen = FrozenSet;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenSet> {
        Ok(SetOf(Frozen {
            content: self.0.content.into_inner().freeze(freezer)?,
        }))
    }
}

impl<'v, T: Members<'v>> fmt::Display for SetOf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.look(|c| show(c, f))
    }
}

fn show(members: &SmallSet<Value<'_>>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if members.is_empty() {
        return write!(f, "set()");
    }
    write!(f, "set([")?;
    for (i, v) in members.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", v.to_repr())?;
    }
    write!(f, "])")
}

/// The members of `value` if it is a set.
fn members_of<'v>(value: Value<'v>) -> Option<Vec<Value<'v>>> {
    if let Some(live) = value.downcast_ref::<Set<'v>>() {
        Some(live.0.look(|c| c.iter().copied().collect()))
    } else {
        value
            .downcast_ref::<FrozenSet>()
            .map(|f| Members::<'v>::look(&f.0, |c| c.iter().copied().collect()))
    }
}

fn contains<'v>(members: &SmallSet<Value<'v>>, value: Value<'v>) -> starlark::Result<bool> {
    Ok(members.contains_hashed(value.get_hashed()?.as_ref()))
}

/// A new set from `values`, in order, each once.
fn collect<'v>(
    heap: Heap<'v>,
    values: impl IntoIterator<Item = Value<'v>>,
) -> starlark::Result<Value<'v>> {
    let mut content = SmallSet::new();
    for v in values {
        content.insert_hashed(v.get_hashed()?);
    }
    Ok(alloc(heap, content))
}

fn alloc<'v>(heap: Heap<'v>, content: SmallSet<Value<'v>>) -> Value<'v> {
    heap.alloc(SetOf(Live {
        content: RefCell::new(content),
        walkers: Cell::new(0),
    }))
}

/// The elements of a list, tuple, dict (its keys), set or range: what a set
/// is built from and combined with.
fn elements<'v>(value: Value<'v>, heap: Heap<'v>) -> Option<Vec<Value<'v>>> {
    if matches!(
        value.get_type(),
        "list" | "tuple" | "dict" | "set" | "range"
    ) {
        Some(value.iterate(heap).ok()?.collect())
    } else {
        None
    }
}

/// [`elements`] of a method's argument, with Bazel's complaint.
fn others<'v>(method: &str, value: Value<'v>, heap: Heap<'v>) -> starlark::Result<Vec<Value<'v>>> {
    elements(value, heap).ok_or_else(|| {
        fatal(format!(
            "for {method} argument got value of type '{}', want a collection of hashable elements",
            value.get_type()
        ))
    })
}

/// The live set behind a method's receiver, if it may change now.
fn mutable<'v>(this: Value<'v>) -> starlark::Result<&'v Live<'v>> {
    let Some(set) = this.downcast_ref::<Set<'v>>() else {
        return Err(fatal("trying to mutate a frozen set value"));
    };
    if set.0.walkers.get() > 0 {
        return Err(fatal(
            "set value is temporarily immutable due to active for-loop iteration",
        ));
    }
    Ok(&set.0)
}

fn snapshot<'v>(this: Value<'v>) -> Vec<Value<'v>> {
    members_of(this).expect("a set method's receiver is a set")
}

/// Bazel's message for the `*others` methods given a keyword.
fn no_keywords<'v>(method: &str, args: &Arguments<'v, '_>) -> starlark::Result<()> {
    match args.names_map()?.keys().next() {
        Some(name) => Err(fatal(format!(
            "{method}() got unexpected keyword argument '{}'",
            name.as_str()
        ))),
        None => Ok(()),
    }
}

/// The positional arguments of a variadic method, each a collection.
fn variadic<'v>(
    method: &str,
    args: &Arguments<'v, '_>,
    heap: Heap<'v>,
) -> starlark::Result<Vec<Vec<Value<'v>>>> {
    no_keywords(method, args)?;
    args.positions(heap)?
        .map(|v| others(method, v, heap))
        .collect()
}

/// The one positional argument of `method(other)`, as a collection.
fn one_other<'v>(
    method: &'static str,
    param: &'static str,
    args: &Arguments<'v, '_>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<Value<'v>>> {
    let bound = bind(
        method,
        Wording::Signature,
        &[positional_only(param, true)],
        args,
        eval,
    )?;
    others(method, bound[0].expect("required"), eval.heap())
}

/// `keep` of the members, in order, that are in every one of `others`.
fn intersect<'v>(
    mine: &[Value<'v>],
    others: &[Vec<Value<'v>>],
    heap: Heap<'v>,
) -> starlark::Result<SmallSet<Value<'v>>> {
    let mut sets = Vec::new();
    for o in others {
        let mut s = SmallSet::new();
        for v in o {
            s.insert_hashed(v.get_hashed()?);
        }
        sets.push(s);
    }
    let _ = heap;
    let mut out = SmallSet::new();
    'members: for v in mine {
        for s in &sets {
            if !contains(s, *v)? {
                continue 'members;
            }
        }
        out.insert_hashed(v.get_hashed()?);
    }
    Ok(out)
}

fn subtract<'v>(
    mine: &[Value<'v>],
    others: &[Vec<Value<'v>>],
) -> starlark::Result<SmallSet<Value<'v>>> {
    let mut gone = SmallSet::new();
    for o in others {
        for v in o {
            gone.insert_hashed(v.get_hashed()?);
        }
    }
    let mut out = SmallSet::new();
    for v in mine {
        if !contains(&gone, *v)? {
            out.insert_hashed(v.get_hashed()?);
        }
    }
    Ok(out)
}

fn symmetric<'v>(
    mine: &[Value<'v>],
    theirs: &[Value<'v>],
) -> starlark::Result<SmallSet<Value<'v>>> {
    let mut a = SmallSet::new();
    for v in mine {
        a.insert_hashed(v.get_hashed()?);
    }
    let mut b = SmallSet::new();
    for v in theirs {
        b.insert_hashed(v.get_hashed()?);
    }
    let mut out = SmallSet::new();
    for v in a.iter() {
        if !contains(&b, *v)? {
            out.insert_hashed(v.get_hashed()?);
        }
    }
    for v in b.iter() {
        if !contains(&a, *v)? {
            out.insert_hashed(v.get_hashed()?);
        }
    }
    Ok(out)
}

fn replace<'v>(live: &Live<'v>, content: SmallSet<Value<'v>>) {
    *live.content.borrow_mut() = content;
}

#[starlark_value(type = "set")]
impl<'v, T: Members<'v> + 'v> StarlarkValue<'v> for SetOf<T>
where
    Self: ProvidesStaticType<'v>,
{
    type Canonical = FrozenSet;

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("set", set_methods);
        Some(RES.methods())
    }

    fn to_bool(&self) -> bool {
        self.0.look(|c| !c.is_empty())
    }

    fn length(&self) -> starlark::Result<i32> {
        Ok(self.0.look(|c| c.len()) as i32)
    }

    /// An unhashable value is simply not there.
    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        let Ok(hashed) = other.get_hashed() else {
            return Ok(false);
        };
        Ok(self.0.look(|c| c.contains_hashed(hashed.as_ref())))
    }

    fn write_hash(&self, _hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        Err(fatal("unhashable type: 'set'"))
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        let Some(theirs) = members_of(other) else {
            return Ok(false);
        };
        let mine: Vec<Value<'v>> = self.0.look(|c| c.iter().copied().collect());
        if mine.len() != theirs.len() {
            return Ok(false);
        }
        let mut lookup = SmallSet::new();
        for v in &theirs {
            lookup.insert_hashed(v.get_hashed()?);
        }
        for v in mine {
            if !contains(&lookup, v)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    unsafe fn iterate(&self, me: Value<'v>, _heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        self.0.walking(1);
        Ok(me)
    }

    unsafe fn iter_size_hint(&self, index: usize) -> (usize, Option<usize>) {
        let rest = self.0.look(|c| c.len()).saturating_sub(index);
        (rest, Some(rest))
    }

    unsafe fn iter_next(&self, index: usize, _heap: Heap<'v>) -> Option<Value<'v>> {
        self.0.look(|c| c.get_index(index).copied())
    }

    unsafe fn iter_stop(&self) {
        self.0.walking(-1);
    }

    fn bit_or(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let Some(theirs) = members_of(rhs) else {
            return Err(unsupported_binary("|", "set", rhs.get_type()));
        };
        collect(
            heap,
            self.0
                .look(|c| c.iter().copied().collect::<Vec<_>>())
                .into_iter()
                .chain(theirs),
        )
    }

    fn bit_and(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let Some(theirs) = members_of(rhs) else {
            return Err(unsupported_binary("&", "set", rhs.get_type()));
        };
        let mine: Vec<Value<'v>> = self.0.look(|c| c.iter().copied().collect());
        Ok(alloc(heap, intersect(&mine, &[theirs], heap)?))
    }

    fn sub(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let Some(theirs) = members_of(rhs) else {
            return Err(unsupported_binary("-", "set", rhs.get_type()));
        };
        let mine: Vec<Value<'v>> = self.0.look(|c| c.iter().copied().collect());
        Ok(alloc(heap, subtract(&mine, &[theirs])?))
    }

    fn bit_xor(&self, rhs: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let Some(theirs) = members_of(rhs) else {
            return Err(unsupported_binary("^", "set", rhs.get_type()));
        };
        let mine: Vec<Value<'v>> = self.0.look(|c| c.iter().copied().collect());
        Ok(alloc(heap, symmetric(&mine, &theirs)?))
    }
}

#[starlark_module]
fn set_methods(builder: &mut MethodsBuilder) {
    fn add<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let bound = bind(
            "add",
            Wording::Signature,
            &[positional_only("element", true)],
            args,
            eval,
        )?;
        let hashed = bound[0].expect("required").get_hashed()?;
        live.content.borrow_mut().insert_hashed(hashed);
        Ok(NoneType)
    }

    fn clear<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        bind("clear", Wording::Signature, &[], args, eval)?;
        live.content.borrow_mut().clear();
        Ok(NoneType)
    }

    fn pop<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let live = mutable(this)?;
        bind("pop", Wording::Signature, &[], args, eval)?;
        live.content
            .borrow_mut()
            .shift_remove_index(0)
            .ok_or_else(|| fatal("set is empty"))
    }

    fn remove<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let bound = bind(
            "remove",
            Wording::Signature,
            &[positional_only("element", true)],
            args,
            eval,
        )?;
        let element = bound[0].expect("required");
        let hashed = element.get_hashed()?;
        if live
            .content
            .borrow_mut()
            .shift_remove_hashed(hashed.as_ref())
        {
            Ok(NoneType)
        } else {
            Err(fatal(format!(
                "element {} not found in set",
                element.to_repr()
            )))
        }
    }

    fn discard<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let bound = bind(
            "discard",
            Wording::Signature,
            &[positional_only("element", true)],
            args,
            eval,
        )?;
        let hashed = bound[0].expect("required").get_hashed()?;
        live.content
            .borrow_mut()
            .shift_remove_hashed(hashed.as_ref());
        Ok(NoneType)
    }

    fn update<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let mut content: SmallSet<Value<'v>> = live.look(|c| c.clone());
        for group in variadic("update", args, eval.heap())? {
            for v in group {
                content.insert_hashed(v.get_hashed()?);
            }
        }
        replace(live, content);
        Ok(NoneType)
    }

    fn union<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let groups = variadic("union", args, eval.heap())?;
        collect(
            eval.heap(),
            snapshot(this)
                .into_iter()
                .chain(groups.into_iter().flatten()),
        )
    }

    fn intersection<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let groups = variadic("intersection", args, eval.heap())?;
        Ok(alloc(
            eval.heap(),
            intersect(&snapshot(this), &groups, eval.heap())?,
        ))
    }

    fn difference<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let groups = variadic("difference", args, eval.heap())?;
        Ok(alloc(eval.heap(), subtract(&snapshot(this), &groups)?))
    }

    fn symmetric_difference<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let other = one_other("symmetric_difference", "other", args, eval)?;
        Ok(alloc(eval.heap(), symmetric(&snapshot(this), &other)?))
    }

    fn intersection_update<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let groups = variadic("intersection_update", args, eval.heap())?;
        let content = intersect(&snapshot(this), &groups, eval.heap())?;
        replace(live, content);
        Ok(NoneType)
    }

    fn difference_update<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let groups = variadic("difference_update", args, eval.heap())?;
        let content = subtract(&snapshot(this), &groups)?;
        replace(live, content);
        Ok(NoneType)
    }

    fn symmetric_difference_update<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let live = mutable(this)?;
        let other = one_other("symmetric_difference_update", "other", args, eval)?;
        let content = symmetric(&snapshot(this), &other)?;
        replace(live, content);
        Ok(NoneType)
    }

    fn isdisjoint<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<bool> {
        let other = one_other("isdisjoint", "other", args, eval)?;
        let common = intersect(&snapshot(this), &[other], eval.heap())?;
        Ok(common.is_empty())
    }

    fn issubset<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<bool> {
        let other = one_other("issubset", "other", args, eval)?;
        let mine = snapshot(this);
        Ok(subtract(&mine, &[other])?.is_empty())
    }

    fn issuperset<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<bool> {
        let other = one_other("issuperset", "other", args, eval)?;
        Ok(subtract(&other, &[snapshot(this)])?.is_empty())
    }
}

#[starlark_module]
pub(crate) fn set_globals(builder: &mut GlobalsBuilder) {
    /// `set(elements = [])`.
    fn set<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "set",
            Wording::Signature,
            &[positional_only("elements", false)],
            args,
            eval,
        )?;
        let items = match bound[0] {
            None => Vec::new(),
            Some(v) => elements(v, eval.heap()).ok_or_else(|| {
                fatal(format!(
                    "in call to set(), parameter 'elements' got value of type '{}', want \
                     'iterable'",
                    v.get_type()
                ))
            })?,
        };
        collect(eval.heap(), items)
    }
}
