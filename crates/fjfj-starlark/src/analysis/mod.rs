//! A Starlark rule's analysis: running its `implementation` with a `ctx`
//! (buildfiji-136.2, 136.3, 136.4).
//!
//! The engine side (`fjfj-analysis`) finds the rule, its dependencies and
//! their [`DepInfo`], and calls [`run_rule`] on a blocking thread. The
//! implementation runs on a heap of its own; what it returns leaves as plain
//! data (the files, the actions) and as frozen provider instances, which a
//! dependent's `ctx` sees again as `Target`s.

mod actions;
mod args_object;
mod ctx;
pub(crate) use ctx::{CtxState, alloc_ctx, attr_named};

use starlark::values::{Heap, Value, ValueLike};

/// The `ctx` whose `actions` this is (what `actions2ctx_cheat` of rules_cc is).
pub(crate) fn ctx_of_actions<'v>(actions: Value<'v>, heap: Heap<'v>) -> Option<Value<'v>> {
    let actions = actions.downcast_ref::<actions::ActionsValue>()?;
    Some(alloc_ctx(heap, actions.state.clone()))
}

/// An `Args` that belongs to no `ctx.actions` (there is nothing in it that does).
pub(crate) fn new_args<'v>(heap: Heap<'v>) -> Value<'v> {
    heap.alloc(args_object::ArgsValue::new())
}

/// The class of the rule a `ctx` is of.
pub(crate) fn rule_kind_of<'v>(ctx: Value<'v>) -> Option<String> {
    Some(ctx.downcast_ref::<ctx::CtxValue>()?.state.rule_kind.clone())
}
mod file;
mod fragments;
mod native_providers;
pub(crate) mod run;
mod runfiles;
mod target;

use crate::label::RepoMappings;
use fjfj_graph::Label;
use starlark::environment::FrozenModule;
use std::sync::Arc;

/// Where the `.bzl` files of rules come from.
pub trait RuleSource: Send + Sync {
    /// The evaluated `.bzl` file `bzl` (canonical).
    fn module(&self, bzl: &Label) -> Result<FrozenModule, String>;

    /// What each repository calls the others, as of now.
    fn mappings(&self) -> Arc<RepoMappings>;
}

pub use native_providers::{Field, native_provider};
pub use run::{
    DepEdge, RuleRequest, RuleResult, labels_of_attrs, resolved_attrs, rule_schema, run_rule,
};
pub use target::{DepInfo, StoredProvider};
pub use transition::{Edge, TransitionSpec, apply_transition, transition_spec};

mod computed;
mod transition;

pub use computed::computed_defaults;

#[cfg(test)]
mod tests;
