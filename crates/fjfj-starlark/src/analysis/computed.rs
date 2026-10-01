//! Attributes whose default is a function of the others
//! (`attr.label(default = f)`, buildfiji-136.2): Bazel calls `f` at loading
//! with the values of the attributes its parameters name.

use super::run::builtins_owner;
use crate::label::{BzlEval, RepoMappings, StarlarkLabel};
use crate::rule::computed_default_of;
use fjfj_graph::rule::AttrValue;
use fjfj_graph::{Label, LabelContext};
use starlark::docs::{DocItem, DocMember};
use starlark::environment::{FrozenModule, Module};
use starlark::eval::Evaluator;
use starlark::values::list::AllocList;
use starlark::values::{Heap, Value, ValueLike};

fn to_starlark<'v>(heap: Heap<'v>, value: &AttrValue) -> Value<'v> {
    match value {
        AttrValue::Bool(b) => Value::new_bool(*b),
        AttrValue::Int(i) => heap.alloc(*i),
        AttrValue::String(s) => heap.alloc(s.as_str()),
        AttrValue::StringList(items) => heap.alloc(AllocList(items.iter().map(String::as_str))),
        AttrValue::IntList(items) => heap.alloc(AllocList(items.iter().copied())),
        AttrValue::Label(l) => heap.alloc(StarlarkLabel::from(l.clone())),
        AttrValue::LabelList(items) => heap.alloc(AllocList(
            items
                .iter()
                .map(|l| heap.alloc(StarlarkLabel::from(l.clone()))),
        )),
        _ => Value::new_none(),
    }
}

/// What a `configuration_field(fragment, name)` is with no option set to say
/// otherwise (probed with `bazel query`). The fields not listed are unset by
/// default (buildfiji-bo8 has the rest to probe).
pub fn late_bound_default(fragment: &str, name: &str) -> Option<Label> {
    let (package, target) = match (fragment, name) {
        ("apple", "xcode_config_label") => ("tools/objc", "host_xcodes"),
        _ => return None,
    };
    Some(Label {
        repo: "bazel_tools".to_owned(),
        package: package.to_owned(),
        name: target.to_owned(),
    })
}

/// The default of each attribute of `rule_name` whose default is a function:
/// `values` are the attributes that are set or have a plain default, which the
/// functions read.
pub fn computed_defaults(
    module: &FrozenModule,
    rule_name: &str,
    values: &[(String, AttrValue)],
    mappings: &RepoMappings,
    repo: &str,
) -> Result<Vec<(String, AttrValue)>, String> {
    let (rule, _) = module
        .get_any_visibility(rule_name)
        .map_err(|_| format!("no rule named {rule_name} in its .bzl"))?;
    let schema =
        crate::rule::schema_of(rule.value()).ok_or_else(|| format!("{rule_name} is not a rule"))?;
    let wanted: Vec<&str> = schema
        .attrs
        .iter()
        .filter(|a| a.def.computed_default)
        .map(|a| a.name.as_str())
        .collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }
    Module::with_temp_heap(|scratch| -> Result<Vec<(String, AttrValue)>, String> {
        scratch.frozen_heap().add_reference(rule.owner());
        scratch.frozen_heap().add_reference(builtins_owner());
        let rule_value = rule
            .value()
            .unpack_frozen()
            .expect("a global is frozen")
            .to_value();
        let heap = scratch.heap();
        let running = BzlEval::running(mappings);
        let mut out = Vec::new();
        for name in wanted {
            let Some(function) = computed_default_of(rule_value, name) else {
                continue;
            };
            // `configuration_field(...)` depends on the build's options, not on
            // other attributes: its label is the option's default.
            if let Some(late) = function.downcast_ref::<crate::decl::LateBoundDefault>() {
                if let Some(label) = late_bound_default(&late.fragment, &late.name) {
                    out.push((name.to_owned(), AttrValue::Label(label)));
                }
                continue;
            }
            let params: Vec<String> = match function.documentation() {
                DocItem::Member(DocMember::Function(f)) => {
                    f.params.regular_params().map(|p| p.name.clone()).collect()
                }
                _ => Vec::new(),
            };
            let named: Vec<(&str, Value<'_>)> = params
                .iter()
                .map(|p| {
                    // A later entry of a name replaces an earlier one.
                    let value = values
                        .iter()
                        .rev()
                        .find(|(n, _)| n == p)
                        .map(|(_, v)| to_starlark(heap, v))
                        .unwrap_or_else(Value::new_none);
                    (p.as_str(), value)
                })
                .collect();
            let mut eval = Evaluator::new(&scratch);
            eval.extra = Some(&running);
            let result = eval
                .eval_function(function, &[], &named)
                .map_err(|e| format!("computing the default of attribute '{name}': {e}"))?;
            if result.is_none() {
                continue;
            }
            let label = |v: Value<'_>| -> Option<Label> {
                if let Some(l) = crate::label::label_of_value(v) {
                    return Some(l);
                }
                Label::parse(v.unpack_str()?, LabelContext { repo, package: "" }).ok()
            };
            if let Some(l) = label(result) {
                out.push((name.to_owned(), AttrValue::Label(l)));
            } else if let Some(items) = crate::args::sequence(result) {
                let labels: Vec<Label> = items.into_iter().filter_map(label).collect();
                out.push((name.to_owned(), AttrValue::LabelList(labels)));
            } else if let Some(s) = result.unpack_str() {
                out.push((name.to_owned(), AttrValue::String(s.to_owned())));
            }
        }
        Ok(out)
    })
}
