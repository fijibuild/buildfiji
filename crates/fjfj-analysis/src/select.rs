//! `select()` (buildfiji-136.5), as Bazel 9.2.0 resolves it:
//!
//! - a branch matches if its `config_setting` matches the configuration;
//! - of several matching branches the most specialized wins: the one whose
//!   setting holds every condition of the others and more. If none does the
//!   select is ambiguous;
//! - no match falls to `//conditions:default`, and without one is an error
//!   (`doesn't match this configuration. Would a default condition help?`,
//!   or `: <no_match_error>`);
//! - the values of a sum of selects are joined, a `None` branch of a label
//!   attribute leaves the attribute unset.

use crate::expand_label_text;
use crate::target::ConfiguredTargetKey;
use fjfj_engine::{Ctx, Error};
use fjfj_graph::rule::{AttrValue, default_condition};
use fjfj_graph::{Configuration, Label};
use std::collections::BTreeSet;

/// What a `config_setting` says of a configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigMatching {
    pub matches: bool,
    /// Everything it requires, for telling which of two settings is the more
    /// specialized.
    pub conditions: BTreeSet<String>,
}

/// Whether `configuration` has the flag `flag` set to `value` (a `values`
/// entry of a `config_setting`).
pub(crate) fn flag_matches(configuration: &Configuration, flag: &str, value: &str) -> bool {
    match flag {
        "compilation_mode" | "c" => configuration.compilation_mode.name() == value,
        "cpu" | "host_cpu" => configuration.cpu == value,
        "define" => match value.split_once('=') {
            Some((k, v)) => configuration.defines.get(k).is_some_and(|d| d == v),
            None => false,
        },
        // A flag nothing set has the default Bazel gives it.
        other => match configuration.options.get(other) {
            Some(set) => set == value,
            None => fjfj_bazel_compat::bazel_flags::FLAGS
                .iter()
                .find(|f| f.name == other)
                .and_then(|f| f.default_value)
                .is_some_and(|d| d == value),
        },
    }
}

/// `attrs` with each `select()` decided.
pub(crate) async fn resolve(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &[(String, AttrValue)],
) -> Result<Vec<(String, AttrValue)>, Error> {
    let mut out = Vec::with_capacity(attrs.len());
    for (name, value) in attrs {
        let AttrValue::Select(list) = value else {
            out.push((name.clone(), value.clone()));
            continue;
        };
        let mut joined: Option<AttrValue> = None;
        let mut unset = true;
        for selector in &list.elements {
            let chosen = choose(ctx, key, name, selector).await?;
            if let Some(value) = chosen {
                unset = false;
                joined = Some(match joined {
                    None => value,
                    // `+` joins lists and `|` dicts; `concat` joins both.
                    Some(so_far) => AttrValue::concat(&so_far, &value).ok_or_else(|| {
                        Error::msg(format!(
                            "{}: attribute '{name}': the branches of a select() are of different types",
                            expand_label_text(&key.label)
                        ))
                    })?,
                });
            }
        }
        if !unset && let Some(value) = joined {
            out.push((name.clone(), value));
        }
    }
    Ok(out)
}

async fn choose(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attr: &str,
    selector: &fjfj_graph::rule::Selector,
) -> Result<Option<AttrValue>, Error> {
    let me = expand_label_text(&key.label);
    let default = default_condition();
    let mut default_value: Option<&Option<AttrValue>> = None;
    let mut conditions: Vec<(&Label, &Option<AttrValue>)> = Vec::new();
    for (condition, value) in &selector.branches {
        if *condition == default {
            default_value = Some(value);
        } else {
            conditions.push((condition, value));
        }
    }
    let keys: Vec<ConfiguredTargetKey> = conditions
        .iter()
        .map(|(l, _)| ConfiguredTargetKey {
            label: (*l).clone(),
            configuration: key.configuration.clone(),
        })
        .collect();
    let results = ctx.get_all(keys).await;
    let mut matched: Vec<(&Label, ConfigMatching, &Option<AttrValue>)> = Vec::new();
    for ((label, value), result) in conditions.iter().zip(results) {
        let target = result?;
        let Some(matching) = target.config_matching.clone() else {
            return Err(Error::msg(format!(
                "{} is not a valid select() condition for {me}.\nNote that only config_setting targets can be used as conditions",
                expand_label_text(label)
            )));
        };
        if matching.matches {
            matched.push((label, matching, value));
        }
    }
    match matched.as_slice() {
        [] => match default_value {
            Some(value) => Ok(value.clone()),
            None => {
                let why = if selector.no_match_error.is_empty() {
                    ". Would a default condition help?".to_owned()
                } else {
                    format!(": {}", selector.no_match_error)
                };
                Err(Error::msg(format!(
                    "configurable attribute \"{attr}\" in {me} doesn't match this configuration{why}"
                )))
            }
        },
        [(_, _, value)] => Ok((*value).clone()),
        many => {
            // The branch whose conditions contain every other's.
            let dominant = many.iter().find(|(_, c, _)| {
                many.iter()
                    .filter(|(_, other, _)| other != c)
                    .all(|(_, other, _)| other.conditions.is_subset(&c.conditions))
                    && many.iter().filter(|(_, other, _)| other == c).count() == 1
            });
            match dominant {
                Some((_, _, value)) => Ok((*value).clone()),
                None => Err(Error::msg(format!(
                    "Illegal ambiguous match on configurable attribute \"{attr}\" in {me}:\n{}\nMultiple matches are not allowed unless one is unambiguously more specialized.",
                    many.iter()
                        .map(|(l, _, _)| expand_label_text(l))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))),
            }
        }
    }
}
