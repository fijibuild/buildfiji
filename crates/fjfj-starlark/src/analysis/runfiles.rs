//! `runfiles`, what `ctx.runfiles()` builds and `DefaultInfo` carries
//! (buildfiji-136.9).

use super::file::{alloc_file, artifact_of};
use crate::args::fatal;
use crate::depset::{Order, depset_to_list, is_depset, new_depset};
use allocative::Allocative;
use fjfj_graph::{Artifact, Label, Runfiles};
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;

#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct RunfilesValue {
    #[allocative(skip)]
    pub(crate) runfiles: Runfiles,
    #[allocative(skip)]
    pub(crate) owner: Label,
}

starlark_simple_value!(RunfilesValue);

impl fmt::Display for RunfilesValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("runfiles")
    }
}

fn runfiles<'v>(this: Value<'v>) -> &'v RunfilesValue {
    this.downcast_ref::<RunfilesValue>().expect("runfiles")
}

#[starlark_value(type = "runfiles")]
impl<'v> StarlarkValue<'v> for RunfilesValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("runfiles", runfiles_members);
        Some(RES.methods())
    }
}

#[starlark_module]
fn runfiles_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn files<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let me = runfiles(this);
        let items: Vec<Value<'v>> = me
            .runfiles
            .files
            .iter()
            .map(|a| alloc_file(heap, a.clone(), me.owner.clone()))
            .collect();
        new_depset(heap, &items, Order::Default, &[])
    }

    /// `runfiles.merge(other)`.
    fn merge<'v>(this: Value<'v>, other: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let a = runfiles(this);
        let b = other.downcast_ref::<RunfilesValue>().ok_or_else(|| {
            fatal(format!(
                "in call to merge(), parameter 'other' got value of type '{}', want 'runfiles'",
                other.get_type()
            ))
        })?;
        Ok(heap.alloc(RunfilesValue {
            runfiles: a.runfiles.merge(&b.runfiles),
            owner: a.owner.clone(),
        }))
    }

    /// `runfiles.merge_all(others)`.
    fn merge_all<'v>(
        this: Value<'v>,
        others: Value<'v>,
        heap: Heap<'v>,
    ) -> starlark::Result<Value<'v>> {
        let a = runfiles(this);
        let mut merged = a.runfiles.clone();
        let items = if is_depset(others) {
            depset_to_list(others).expect("a depset")?
        } else {
            crate::args::sequence(others).unwrap_or_default()
        };
        for item in items {
            let b = item.downcast_ref::<RunfilesValue>().ok_or_else(|| {
                fatal(format!(
                    "expected value of type 'runfiles' for element of others, but got {}",
                    item.get_type()
                ))
            })?;
            merged = merged.merge(&b.runfiles);
        }
        Ok(heap.alloc(RunfilesValue {
            runfiles: merged,
            owner: a.owner.clone(),
        }))
    }
}

/// The runfiles a value is, if it is one.
pub(crate) fn runfiles_of(value: Value<'_>) -> Option<Runfiles> {
    value
        .downcast_ref::<RunfilesValue>()
        .map(|r| r.runfiles.clone())
}

/// A `runfiles` value.
pub(crate) fn alloc_runfiles<'v>(heap: Heap<'v>, runfiles: Runfiles, owner: Label) -> Value<'v> {
    heap.alloc(RunfilesValue { runfiles, owner })
}

/// The files of a list or depset of `File`s.
pub(crate) fn files_in(value: Value<'_>, what: &str) -> starlark::Result<Vec<Artifact>> {
    let items = if is_depset(value) {
        depset_to_list(value).expect("a depset")?
    } else {
        crate::args::sequence(value).unwrap_or_default()
    };
    items
        .into_iter()
        .map(|item| {
            artifact_of(item).ok_or_else(|| {
                fatal(format!(
                    "expected value of type 'File' for {what}, but got {}",
                    crate::args::describe(item)
                ))
            })
        })
        .collect()
}
