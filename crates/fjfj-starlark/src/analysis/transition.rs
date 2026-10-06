//! Running a transition (buildfiji-136.6): the `implementation` of a
//! `transition()` is called with the settings it reads and, on an attribute's
//! edge, the attributes of the rule; it returns the settings to change.

use super::run::builtins_owner;
use crate::label::{BzlEval, RepoMappings};
use crate::rule::transition_of;
use fjfj_graph::SettingValue;
use fjfj_graph::rule::AttrValue;
use starlark::environment::{FrozenModule, Module};
use starlark::eval::Evaluator;
use starlark::values::dict::DictRef;
use starlark::values::list::ListRef;
use starlark::values::{Heap, Value};
use std::collections::BTreeMap;

/// Which transition of a rule: the one on an attribute's edge, or the rule's
/// own `cfg`.
#[derive(Debug, Clone, Copy)]
pub enum Edge<'a> {
    Incoming,
    Attr(&'a str),
}

impl Edge<'_> {
    fn attr(&self) -> Option<&str> {
        match self {
            Edge::Incoming => None,
            Edge::Attr(name) => Some(name),
        }
    }
}

/// Starts the error of a transition that Bazel words as two events: what went
/// wrong at a place in the `.bzl`, then that the target's transition failed at
/// the target. The fields follow, each after a [`TRANSITION_ERROR`]: where, what,
/// the edge it was on (empty for a rule's own transition) and the target.
pub const TRANSITION_ERROR: char = '\u{4}';

/// The error of a transition: `text` happened at `location`, which is
/// `<file>:<line>:<column>` or empty when the text says where itself.
pub fn transition_error(location: &str, text: &str) -> String {
    format!(
        "{TRANSITION_ERROR}{location}{TRANSITION_ERROR}{text}{TRANSITION_ERROR}{TRANSITION_ERROR}"
    )
}

/// A transition error that says which target it was for and on which edge.
/// Any other error is returned as it is.
pub fn transition_error_for(message: String, edge: &str, target: &str) -> String {
    match split_transition_error(&message) {
        Some(parts) => format!(
            "{TRANSITION_ERROR}{}{TRANSITION_ERROR}{}{TRANSITION_ERROR}{edge}{TRANSITION_ERROR}{target}",
            parts.location, parts.text
        ),
        None => message,
    }
}

/// The parts of [`transition_error`].
pub struct TransitionError<'a> {
    pub location: &'a str,
    pub text: &'a str,
    pub edge: &'a str,
    pub target: &'a str,
}

/// The parts of an error made by [`transition_error`], if it is one.
pub fn split_transition_error(message: &str) -> Option<TransitionError<'_>> {
    let mut parts = message
        .strip_prefix(TRANSITION_ERROR)?
        .split(TRANSITION_ERROR);
    Some(TransitionError {
        location: parts.next()?,
        text: parts.next()?,
        edge: parts.next()?,
        target: parts.next()?,
    })
}

/// One configuration a transition asks for: the key of its split (empty for
/// an ordinary transition) and the settings it sets.
pub type Outcome = (String, BTreeMap<String, SettingValue>);

/// The settings a transition reads and writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionSpec {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    /// Where `transition()` was called: `@@repo//pkg:file.bzl:line:col`.
    pub defined_at: String,
}

/// The settings each transition on `edge` of `rule` reads and writes, in the
/// order they run: one, unless the transition is `a.and_then(b)`. `None` if
/// the edge has no transition defined by a `.bzl`.
pub fn transition_specs(
    module: &FrozenModule,
    rule: &str,
    edge: Edge<'_>,
) -> Option<Vec<TransitionSpec>> {
    let (rule, _) = module.get_any_visibility(rule).ok()?;
    let value = rule.value().unpack_frozen()?.to_value();
    let transition = transition_of(value, edge.attr())?;
    crate::decl::defined_parts(transition)?
        .into_iter()
        .map(|part| {
            if let Some(settings) = crate::decl::analysis_test_settings(part) {
                return Some(TransitionSpec {
                    inputs: Vec::new(),
                    outputs: settings.into_iter().map(|(name, _)| name).collect(),
                    defined_at: String::new(),
                });
            }
            let (_, inputs, outputs) = crate::decl::defined_transition(part)?;
            Some(TransitionSpec {
                inputs,
                outputs,
                defined_at: crate::decl::transition_defined_at(part).unwrap_or_default(),
            })
        })
        .collect()
}

/// The settings the transition on `edge` of `rule` reads and writes: for a
/// composed one, those of its first part. `None` if the edge has no
/// transition defined by a `.bzl`.
pub fn transition_spec(
    module: &FrozenModule,
    rule: &str,
    edge: Edge<'_>,
) -> Option<TransitionSpec> {
    transition_specs(module, rule, edge)?.into_iter().next()
}

pub(super) fn to_starlark<'v>(heap: Heap<'v>, name: &str, value: &SettingValue) -> Value<'v> {
    // `--platforms` holds labels.
    if name == "//command_line_option:platforms"
        && let SettingValue::List(items) = value
    {
        let labels: Vec<Value<'v>> = items
            .iter()
            .filter_map(|text| {
                fjfj_graph::Label::parse(
                    text,
                    fjfj_graph::LabelContext {
                        repo: "",
                        package: "",
                    },
                )
                .ok()
            })
            .map(|l| heap.alloc(crate::label::StarlarkLabel::from(l)))
            .collect();
        return heap.alloc(starlark::values::list::AllocList(labels));
    }
    match value {
        SettingValue::Bool(b) => Value::new_bool(*b),
        SettingValue::Int(i) => heap.alloc(*i),
        SettingValue::Str(s) => heap.alloc(s.as_str()),
        SettingValue::None => Value::new_none(),
        SettingValue::Label(text) => fjfj_graph::Label::parse(
            text,
            fjfj_graph::LabelContext {
                repo: "",
                package: "",
            },
        )
        .map_or_else(
            |_| heap.alloc(text.as_str()),
            |l| heap.alloc(crate::label::StarlarkLabel::from(l)),
        ),
        SettingValue::List(items) => heap.alloc(starlark::values::list::AllocList(
            items.iter().map(String::as_str),
        )),
    }
}

fn from_starlark(name: &str, value: Value<'_>) -> Result<SettingValue, String> {
    if let Some(b) = value.unpack_bool() {
        return Ok(SettingValue::Bool(b));
    }
    if let Some(i) = value.unpack_i32() {
        return Ok(SettingValue::Int(i64::from(i)));
    }
    if let Some(s) = value.unpack_str() {
        return Ok(SettingValue::Str(s.to_owned()));
    }
    // `--platforms` takes a single label too.
    if name == "//command_line_option:platforms"
        && let Some(label) = crate::label::label_of_value(value)
    {
        return Ok(SettingValue::List(vec![fjfj_graph::expand::label_text(
            &label,
        )]));
    }
    if let Some(items) = crate::args::sequence(value) {
        let mut out = Vec::new();
        for item in items {
            match item.unpack_str().map(str::to_owned).or_else(|| {
                crate::label::label_of_value(item).map(|l| fjfj_graph::expand::label_text(&l))
            }) {
                Some(s) => out.push(s),
                None => {
                    return Err(format!(
                        "transition output '{name}' has a list with a {}, want strings",
                        item.get_type()
                    ));
                }
            }
        }
        return Ok(SettingValue::List(out));
    }
    Err(format!(
        "transition output '{name}' is a {}, want a bool, int, string or list of strings",
        value.get_type()
    ))
}

/// Call the transition on `edge` of `rule` with `settings` (the value of each
/// of its inputs) and, for an attribute's edge, the non-label `attrs` of the
/// rule. Each configuration it asks for, by the key of a split (empty for an
/// ordinary transition), with the settings it sets. `part` is which of the
/// transitions of a composed one runs, from 0.
pub fn apply_transition(
    module: &FrozenModule,
    rule: &str,
    edge: Edge<'_>,
    part: usize,
    settings: &BTreeMap<String, SettingValue>,
    attrs: &[(String, AttrValue)],
    mappings: &RepoMappings,
) -> Result<Vec<Outcome>, String> {
    let (rule_global, _) = module
        .get_any_visibility(rule)
        .map_err(|_| format!("no rule named {rule} in its .bzl"))?;
    Module::with_temp_heap(|module| -> Result<_, String> {
        module.frozen_heap().add_reference(rule_global.owner());
        module.frozen_heap().add_reference(builtins_owner());
        let rule_value = rule_global
            .value()
            .unpack_frozen()
            .expect("a global is frozen")
            .to_value();
        let transition = transition_of(rule_value, edge.attr())
            .ok_or_else(|| format!("{rule} has no transition there"))?;
        let parts = crate::decl::defined_parts(transition)
            .ok_or_else(|| "only a transition() can be applied".to_owned())?;
        // An analysis test's transition sets what it was given.
        if let Some(settings) = parts
            .get(part)
            .and_then(|p| crate::decl::analysis_test_settings(*p))
        {
            let mut set = BTreeMap::new();
            for (name, value) in settings {
                set.insert(name.clone(), from_starlark(&name, value)?);
            }
            return Ok(vec![(String::new(), set)]);
        }
        let (implementation, _inputs, outputs) = parts
            .get(part)
            .and_then(|part| crate::decl::defined_transition(*part))
            .ok_or_else(|| "only a transition() can be applied".to_owned())?;
        let heap = module.heap();
        let input = heap.alloc(starlark::values::dict::AllocDict(
            settings
                .iter()
                .map(|(k, v)| (heap.alloc(k.as_str()), to_starlark(heap, k, v))),
        ));
        let mut fields = Vec::new();
        for (name, value) in attrs {
            let value = match value {
                AttrValue::Bool(b) => Value::new_bool(*b),
                AttrValue::Int(i) => heap.alloc(*i),
                AttrValue::String(s) => heap.alloc(s.as_str()),
                AttrValue::StringList(items) => heap.alloc(starlark::values::list::AllocList(
                    items.iter().map(String::as_str),
                )),
                AttrValue::Label(l) => heap.alloc(crate::label::StarlarkLabel::from(l.clone())),
                AttrValue::LabelList(items) => heap.alloc(starlark::values::list::AllocList(
                    items
                        .iter()
                        .map(|l| heap.alloc(crate::label::StarlarkLabel::from(l.clone()))),
                )),
                AttrValue::StringDict(items) => heap.alloc(starlark::values::dict::AllocDict(
                    items
                        .iter()
                        .map(|(k, v)| (heap.alloc(k.as_str()), heap.alloc(v.as_str()))),
                )),
                AttrValue::LabelKeyedStringDict(items) => heap.alloc(
                    starlark::values::dict::AllocDict(items.iter().map(|(k, v)| {
                        (
                            heap.alloc(crate::label::StarlarkLabel::from(k.clone())),
                            heap.alloc(v.as_str()),
                        )
                    })),
                ),
                _ => continue,
            };
            fields.push((name.clone(), value));
        }
        // An attribute with no value, a label that defaults to None for one, is None.
        for name in crate::rule::declared_attr_names(rule_value) {
            if !fields.iter().any(|(n, _)| *n == name) {
                fields.push((name, Value::new_none()));
            }
        }
        let attr = crate::structs::new_struct(heap, fields);
        let running = BzlEval::running(mappings);
        let returned = {
            let mut eval = Evaluator::new(&module);
            eval.extra = Some(&running);
            let args: Vec<Value<'_>> = match edge {
                Edge::Incoming => vec![input, attr],
                Edge::Attr(_) => vec![input, attr],
            };
            eval.eval_function(implementation, &args, &[])
                .map_err(|e| transition_error("", &crate::traceback(&e)))?
        };
        // Bazel says a result it does not accept came from where the function
        // is written.
        read_outputs(returned, &outputs).map_err(|text| {
            let at = starlark::eval::definition_span(implementation).map(|span| {
                let resolved = span.resolve();
                format!(
                    "{}:{}:{}",
                    resolved.file,
                    resolved.span.begin.line + 1,
                    resolved.span.begin.column + 1
                )
            });
            transition_error(at.as_deref().unwrap_or_default(), &text)
        })
    })
}

/// A transition returns a dict of settings, a list of such dicts, or a dict
/// of them by key (a split).
fn read_outputs(returned: Value<'_>, outputs: &[String]) -> Result<Vec<Outcome>, String> {
    let one = |dict: Value<'_>, alone: bool| -> Result<BTreeMap<String, SettingValue>, String> {
        let dict = DictRef::from_value(dict).ok_or_else(|| {
            format!(
                "transition output must be a dict, got a {}",
                dict.get_type()
            )
        })?;
        let mut out = BTreeMap::new();
        for (k, v) in dict.iter() {
            let key = k
                .unpack_str()
                .ok_or_else(|| "transition output keys must be strings".to_owned())?;
            if !outputs.iter().any(|o| o == key) {
                return Err(format!(
                    "invalid result from transition function: transition function returned \
                     undeclared output '{key}'"
                ));
            }
            out.insert(key.to_owned(), from_starlark(key, v)?);
        }
        // A lone empty dict leaves the configuration as it was.
        let missing: Vec<&str> = outputs
            .iter()
            .filter(|declared| !out.contains_key(*declared))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() && !(alone && out.is_empty()) {
            return Err(format!(
                "invalid result from transition function: transition outputs [{}] were not \
                 defined by transition function",
                missing.join(",")
            ));
        }
        Ok(out)
    };
    if let Some(list) = ListRef::from_value(returned) {
        return list
            .iter()
            .enumerate()
            .map(|(i, d)| Ok((i.to_string(), one(d, false)?)))
            .collect();
    }
    let Some(dict) = DictRef::from_value(returned) else {
        return Err(format!(
            "transition function returned {}, want dict or list of dicts",
            returned.get_type()
        ));
    };
    // A dict whose values are dicts is a split.
    if !dict.is_empty() && dict.iter().all(|(_, v)| DictRef::from_value(v).is_some()) {
        let mut out = Vec::new();
        for (k, v) in dict.iter() {
            out.push((
                k.unpack_str().unwrap_or_default().to_owned(),
                one(v, false)?,
            ));
        }
        return Ok(out);
    }
    Ok(vec![(String::new(), one(returned, true)?)])
}
