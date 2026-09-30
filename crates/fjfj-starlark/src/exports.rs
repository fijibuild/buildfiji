//! Naming what a `.bzl` file defines (buildfiji-mum.3.4, buildfiji-mum.3.5):
//! a `provider()` or a `rule()` is named by the first top-level name the
//! file binds it to, and the name is what Bazel's messages call it.
//!
//! Bazel names a value at the assignment that binds it, and by value: `P =
//! make()` names what `make()` returned, `P, R = provider(init = ...)` names
//! `P`, and one that is only in a list, a dict or a struct is never named.
//! The `starlark` crate has no hook on an assignment and will not show a
//! native function a private (`_P`) name, so a value looks for its name among
//! the public names of the module being evaluated when it needs one, and
//! [`export_all`] names the rest when the module is done, reading them from
//! the frozen module by the names the parsed file assigns. A value bound only
//! to a `_private` name is therefore anonymous until its module ends, and
//! [`is_exported`] gives it the benefit of the doubt in a module that binds
//! any.
//!
//! Naming can fail: a rule that is a test must be named `*_test` and any
//! other rule must not be.

use crate::args::fatal;
use crate::label::{assigns_private_names, evaluating_bzl};
use crate::provider;
use crate::rule;
use starlark::environment::FrozenModule;
use starlark::eval::Evaluator;
use starlark::values::Value;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// The top-level name the call being made is assigned to right away
/// (`NAME = rule(...)` on one line at the top of a `.bzl`), which is what
/// Bazel names a value by.
pub(crate) fn assigned_name(eval: &Evaluator<'_, '_, '_>) -> Option<String> {
    if !evaluating_bzl(eval) {
        return None;
    }
    // The module's frame and the call's: anything deeper is in a function.
    if eval.call_stack_count() != 2 {
        return None;
    }
    let at = eval.call_stack_top_location()?;
    let begin = at.span.begin();
    let line = at.file.find_line(begin);
    let start = at.file.line_span(line).begin();
    let text = at.file.source_line(line);
    let prefix = text.get(..(begin.get() - start.get()) as usize)?;
    let name = prefix.trim_end().strip_suffix('=')?.trim_end();
    let identifier = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    identifier.then(|| name.to_owned())
}

/// Give the value `cell` belongs to the name it is assigned to on the line
/// being run, if there is one, and the error if its kind may not have it.
pub(crate) fn name_at_assignment(
    eval: &Evaluator<'_, '_, '_>,
    kind: Kind,
    cell: &OnceLock<String>,
) -> starlark::Result<()> {
    if let Some(name) = assigned_name(eval) {
        kind.accepts(&name).map_err(fatal)?;
        let _ = cell.set(name);
    }
    Ok(())
}

/// A number no other provider or rule has, which is what stays the same
/// about one when it is frozen.
pub(crate) fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// What a value that is named is.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Provider,
    Rule { test: bool },
    Aspect,
    Subrule,
    Macro,
    RepositoryRule,
}

impl Kind {
    /// Whether a value of this kind may have this name.
    pub(crate) fn accepts(self, name: &str) -> Result<(), String> {
        match self {
            Kind::Rule { test } if test != name.ends_with("_test") => Err(format!(
                "Invalid rule class name '{name}', test rule class names must end with '_test' \
                 and other rule classes must not"
            )),
            _ => Ok(()),
        }
    }
}

/// A value a file can name, live or frozen.
pub(crate) struct Named<'v> {
    /// What it is across freezing: two are the same if these are.
    pub(crate) id: u64,
    pub(crate) name: &'v OnceLock<String>,
    pub(crate) kind: Kind,
}

/// `value`, if it is something a file names.
pub(crate) fn named<'v>(value: Value<'v>) -> Option<Named<'v>> {
    provider::named(value)
        .or_else(|| rule::named(value))
        .or_else(|| crate::decl::named(value))
        .or_else(|| crate::macros::named(value))
        .or_else(|| crate::ext::named(value))
}

/// The name of `value` if it is bound to one, looking for it in the module
/// `eval` is running if it is not named yet. The error is a name it may not
/// have.
pub(crate) fn resolve_name<'v>(
    value: Value<'v>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Option<String>> {
    let Some(me) = named(value) else {
        return Ok(None);
    };
    if let Some(name) = me.name.get() {
        return Ok(Some(name.clone()));
    }
    if !evaluating_bzl(eval) {
        return Ok(None);
    }
    let module = eval.module();
    for name in module.names() {
        let bound = module.get(name.as_str()).and_then(named);
        if bound.is_some_and(|bound| bound.id == me.id) {
            me.kind.accepts(name.as_str()).map_err(fatal)?;
            let _ = me.name.set(name.as_str().to_owned());
            return Ok(Some(name.as_str().to_owned()));
        }
    }
    Ok(None)
}

/// Whether `value`, a provider or rule, is bound to a top-level name of a
/// `.bzl`, as `providers` and `provides` require.
pub(crate) fn is_exported<'v>(value: Value<'v>, eval: &Evaluator<'v, '_, '_>) -> bool {
    matches!(resolve_name(value, eval), Ok(Some(_)))
        || (evaluating_bzl(eval) && assigns_private_names(eval))
}

/// Name what `module` holds in `assigned`, which are its top-level
/// assignments in order, when it is finished: the first name of a value is
/// its name.
pub(crate) fn export_all(module: &FrozenModule, assigned: &[String]) -> starlark::Result<()> {
    for name in assigned {
        let Ok((value, _)) = module.get_any_visibility(name) else {
            continue;
        };
        if let Some(bound) = named(value.value())
            && bound.name.get().is_none()
        {
            bound.kind.accepts(name).map_err(fatal)?;
            let _ = bound.name.set(name.clone());
        }
    }
    Ok(())
}
