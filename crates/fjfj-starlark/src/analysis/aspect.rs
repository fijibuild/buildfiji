//! Aspects (buildfiji-136.8): which ones an attribute asks for, where one
//! propagates and whether it applies to a target, and running its
//! implementation with the target it looks at.

use super::ctx::CtxState;
use super::run::{RuleResult, execute, resolved_attrs};
use super::target::{DepInfo, StoredProvider};
use crate::decl::{aspect_arg, aspect_data};
use crate::label::RepoMappings;
use crate::provider::same_provider;
use crate::structs::provider_of;
use fjfj_graph::rule::AttrValue;
use fjfj_graph::schema::RuleSchema;
use fjfj_graph::{Configuration, Label};
use starlark::environment::FrozenModule;
use starlark::values::Value;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// An aspect: the `.bzl` that defines it and the name it is bound to there.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AspectRef {
    pub bzl: Label,
    pub name: String,
}

/// The aspects the attribute `attr` of the rule `rule_name` of `module` asks for.
pub fn attr_aspects(module: &FrozenModule, rule_name: &str, attr: &str) -> Vec<AspectRef> {
    let Ok((rule, _)) = module.get_any_visibility(rule_name) else {
        return Vec::new();
    };
    let Some(rule) = rule.value().unpack_frozen() else {
        return Vec::new();
    };
    crate::rule::aspects_of(rule.to_value(), attr)
        .into_iter()
        .filter_map(|aspect| {
            let data = aspect_data(aspect)?;
            Some(AspectRef {
                bzl: data.defined_in?,
                name: data.name?,
            })
        })
        .collect()
}

/// What propagating an aspect needs to know.
#[derive(Clone, Debug)]
pub struct AspectSpec {
    /// The attributes it follows to the targets they name; `*` is all of them.
    pub attr_aspects: Vec<String>,
    /// Its own attributes and toolchains.
    pub schema: Arc<RuleSchema>,
    /// The aspects it `requires`: they run on the same target first, and
    /// their providers are visible on the target it looks at.
    pub requires: Vec<AspectRef>,
}

/// The aspect `name` of `module`.
pub fn aspect_spec(module: &FrozenModule, name: &str) -> Option<AspectSpec> {
    let (value, _) = module.get_any_visibility(name).ok()?;
    let value = value.value().unpack_frozen()?.to_value();
    let data = aspect_data(value)?;
    let attr_aspects = aspect_arg(value, "attr_aspects")
        .and_then(crate::args::sequence)
        .map(|items| {
            items
                .into_iter()
                .filter_map(|v| v.unpack_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let requires = aspect_arg(value, "requires")
        .and_then(crate::args::sequence)
        .map(|items| {
            items
                .into_iter()
                .filter_map(|v| {
                    let data = aspect_data(v)?;
                    Some(AspectRef {
                        bzl: data.defined_in?,
                        name: data.name?,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(AspectSpec {
        attr_aspects,
        schema: data.schema,
        requires,
    })
}

/// Whether the aspect `name` applies to a target that gave `providers`: it
/// has every provider of one of the `required_providers` alternatives, or the
/// aspect requires none.
pub fn aspect_applies(module: &FrozenModule, name: &str, providers: &[StoredProvider]) -> bool {
    let Ok((value, _)) = module.get_any_visibility(name) else {
        return false;
    };
    let Some(value) = value.value().unpack_frozen().map(|v| v.to_value()) else {
        return false;
    };
    let Some(required) = aspect_arg(value, "required_providers").and_then(crate::args::sequence)
    else {
        return true;
    };
    if required.is_empty() {
        return true;
    }
    let has = |wanted: Value<'_>| {
        providers.iter().any(|p| {
            provider_of(p.value.value()).is_some_and(|kind| same_provider(kind, Some(wanted)))
        })
    };
    required.into_iter().any(|alternative| {
        match crate::args::sequence(alternative) {
            // A list is providers that must all be there.
            Some(all) if alternative.get_type() != "Provider" => all.into_iter().all(has),
            _ => has(alternative),
        }
    })
}

/// What the engine hands an aspect to run.
pub struct AspectRequest {
    /// The `.bzl` that defines the aspect, evaluated.
    pub module: FrozenModule,
    pub aspect: String,
    pub aspect_schema: Arc<RuleSchema>,
    /// The aspects applied so far to this target, this one last.
    pub aspect_ids: Vec<String>,
    /// The target it looks at, with what the aspects before this one added.
    pub target: DepInfo,
    pub rule_kind: String,
    pub rule_schema: Arc<RuleSchema>,
    /// The attributes the target's BUILD file set, `select()`s decided.
    pub rule_attrs: Vec<(String, AttrValue)>,
    /// The targets the rule's attributes name, aspects applied where they propagate.
    pub rule_deps: BTreeMap<Label, DepInfo>,
    /// The targets the aspect's own attributes name.
    pub deps: BTreeMap<Label, DepInfo>,
    pub location: String,
    pub build_file: String,
    pub configuration: Configuration,
    pub main_repo_name: String,
    pub mappings: Arc<RepoMappings>,
    pub toolchains: Vec<(Label, Option<DepInfo>)>,
}

/// Run the aspect's `implementation(target, ctx)`.
pub fn run_aspect(req: &AspectRequest) -> Result<RuleResult, String> {
    let (aspect, _) = req
        .module
        .get_any_visibility(&req.aspect)
        .map_err(|_| format!("no aspect named {} in its .bzl", req.aspect))?;
    let label = req.target.label.clone();
    let make = |schema: &Arc<RuleSchema>,
                set: &[(String, AttrValue)],
                deps: &BTreeMap<Label, DepInfo>,
                rule: Option<Arc<CtxState>>| {
        Arc::new(CtxState {
            label: label.clone(),
            rule_kind: req.rule_kind.clone(),
            location: req.location.clone(),
            build_file: req.build_file.clone(),
            configuration: req.configuration.clone(),
            main_repo_name: req.main_repo_name.clone(),
            mappings: req.mappings.clone(),
            schema: schema.clone(),
            build_setting_value: None,
            attrs: resolved_attrs(schema, set),
            deps: deps
                .iter()
                .map(|(l, d)| (l.clone(), Arc::new(d.clone())))
                .collect(),
            outputs: Vec::new(),
            toolchains: req
                .toolchains
                .iter()
                .map(|(l, d)| (l.clone(), d.clone().map(Arc::new)))
                .collect(),
            rule,
            aspect_ids: req.aspect_ids.clone(),
            actions: Mutex::new(Vec::new()),
            nested: Mutex::default(),
            declared: Mutex::new(Default::default()),
        })
    };
    let rule = make(&req.rule_schema, &req.rule_attrs, &req.rule_deps, None);
    let state = make(&req.aspect_schema, &[], &req.deps, Some(rule));
    let all_deps: Vec<&DepInfo> = req
        .rule_deps
        .values()
        .chain(req.deps.values())
        .chain(std::iter::once(&req.target))
        .collect();
    execute(
        state,
        &aspect,
        &|aspect_value| crate::decl::aspect_arg(aspect_value, "implementation"),
        all_deps.into_iter(),
        &req.aspect,
        Some(&req.target),
    )
}
