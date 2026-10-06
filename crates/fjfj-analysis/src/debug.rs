//! `--toolchain_resolution_debug` (buildfiji-136.7): what Bazel 9.2.0 says of
//! how it resolved the toolchains of a target whose label, or one of whose
//! toolchain types, the flag's regular expression finds.

use crate::expand_label_text;
use crate::target::{ConfiguredTargetKey, Env};
use crate::toolchain::{RegisteredToolchains, follow_aliases};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::Label;
use std::collections::BTreeSet;
use std::sync::Arc;

/// A test of a label's text.
pub type LabelFilter = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Which labels to explain, and where the explanation goes.
#[derive(Clone)]
pub struct ResolutionDebug {
    /// Whether the flag asks to hear of a label.
    pub matches: LabelFilter,
    /// Takes one message: `INFO: ToolchainResolution: ...` with its
    /// continuation lines, no newline at the end.
    pub emit: Arc<dyn Fn(&str) + Send + Sync>,
}

/// A platform a target may run on, with its constraint values.
pub(crate) type Candidate = (Option<Label>, BTreeSet<Label>);

/// What was resolved: the platform chosen, and the implementation of each
/// type that got one.
pub(crate) struct Outcome {
    pub platform: Option<Label>,
    pub toolchains: Vec<(Label, Label)>,
}

const PREFIX: &str = "ToolchainResolution:";

fn target_platform(key: &ConfiguredTargetKey) -> String {
    match key
        .configuration
        .settings
        .get("//command_line_option:platforms")
    {
        Some(fjfj_graph::SettingValue::List(items)) if !items.is_empty() => items[0].clone(),
        _ => "@@platforms//host:host".to_owned(),
    }
}

/// A message of `lines`, each with how far it is indented.
fn message(lines: &[(usize, String)]) -> String {
    let mut out = String::new();
    for (i, (indent, text)) in lines.iter().enumerate() {
        if i == 0 {
            out.push_str(&format!("INFO: {PREFIX} {text}"));
        } else {
            out.push_str(&format!("\n      {PREFIX} {}{text}", "  ".repeat(*indent)));
        }
    }
    out
}

fn names(labels: &[&Label]) -> String {
    labels
        .iter()
        .map(|l| l.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Say how `types` resolved for the target of `key`, if the flag asks.
pub(crate) async fn trace(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    types: &[(Label, bool)],
    candidates: &[Candidate],
    removed: &[String],
    outcome: Option<&Outcome>,
) -> Result<(), Error> {
    let env = ctx.data::<Env>()?;
    let Some(debug) = env.toolchain_resolution_debug.clone() else {
        return Ok(());
    };
    let target_asked = (debug.matches)(&expand_label_text(&key.label));
    // The optional types come first, then the mandatory ones, each in order.
    let asked: Vec<&Label> = types
        .iter()
        .filter(|(_, mandatory)| !mandatory)
        .chain(types.iter().filter(|(_, mandatory)| *mandatory))
        .map(|(t, _)| t)
        .filter(|t| target_asked || (debug.matches)(&expand_label_text(t)))
        .collect();
    if !target_asked && asked.is_empty() {
        return Ok(());
    }
    let platform = target_platform(key);
    // The platforms the target's own constraints rule out are said of twice
    // before the types and once after.
    let said_removed = || {
        for line in removed {
            (debug.emit)(&message(&[(0, line.clone())]));
        }
    };
    if !asked.is_empty() {
        said_removed();
        said_removed();
    }
    let any_types = !asked.is_empty();
    for toolchain_type in asked {
        let lines = explain(ctx, key, toolchain_type, candidates, &platform).await?;
        (debug.emit)(&message(&lines));
    }
    if any_types {
        said_removed();
    }
    if let Some(outcome) = outcome {
        let chosen = outcome
            .platform
            .as_ref()
            .map(expand_label_text)
            .unwrap_or_else(|| platform.clone());
        let resolved: Vec<String> = outcome
            .toolchains
            .iter()
            .map(|(t, i)| {
                format!(
                    "type {} -> toolchain {}",
                    expand_label_text(t),
                    expand_label_text(i)
                )
            })
            .collect();
        (debug.emit)(&message(&[(
            0,
            format!(
                "Target platform {platform}: Selected execution platform {chosen}, {}",
                resolved.join(", ")
            ),
        )]));
    }
    Ok(())
}

/// The resolution of one type, as lines.
async fn explain(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    toolchain_type: &Label,
    candidates: &[Candidate],
    platform: &str,
) -> Result<Vec<(usize, String)>, Error> {
    let type_text = expand_label_text(toolchain_type);
    let mut lines = vec![(
        0,
        format!("Performing resolution of {type_text} for target platform {platform}"),
    )];
    let config = &key.configuration;
    let extra = match config
        .settings
        .get("//command_line_option:extra_toolchains")
    {
        Some(fjfj_graph::SettingValue::List(items)) => Some(items.clone()),
        _ => None,
    };
    let registered = ctx.get(RegisteredToolchains { extra }).await?;
    let wanted = follow_aliases(ctx, toolchain_type).await?;
    let mut found = Vec::new();
    for label in registered.iter() {
        let done = ctx
            .get(ConfiguredTargetKey {
                label: label.clone(),
                configuration: config.clone(),
            })
            .await?;
        let Some(decl) = &done.toolchain_decl else {
            continue;
        };
        if decl.toolchain_type == wanted
            || follow_aliases(ctx, &decl.toolchain_type).await? == wanted
        {
            found.push((label.clone(), decl.clone()));
        }
    }
    // The settings are checked first, for all, then the constraints.
    let mut remaining = Vec::new();
    for (label, decl) in found {
        let mut unmatched: Vec<Label> = Vec::new();
        for setting in &decl.target_settings {
            let target = ctx
                .get(ConfiguredTargetKey {
                    label: setting.clone(),
                    configuration: config.clone(),
                })
                .await?;
            if !target.config_matching.as_ref().is_some_and(|m| m.matches) {
                unmatched.push(setting.clone());
            }
        }
        if unmatched.is_empty() {
            remaining.push((label, decl));
        } else {
            lines.push((
                1,
                format!(
                    "Rejected toolchain {}; mismatching target_settings: {}",
                    expand_label_text(&label),
                    names(&unmatched.iter().collect::<Vec<_>>())
                ),
            ));
        }
    }
    let mut assigned: Vec<Option<Label>> = vec![None; candidates.len()];
    for (label, decl) in remaining {
        let held = crate::constraints::with_defaults_for(
            ctx,
            &config.constraints,
            &decl.target_compatible_with,
        )
        .await?;
        let missing: Vec<&Label> = decl
            .target_compatible_with
            .iter()
            .filter(|c| !held.contains(*c))
            .collect();
        let resolves_to = expand_label_text(&decl.toolchain);
        if !missing.is_empty() {
            lines.push((
                1,
                format!(
                    "Rejected toolchain {} (resolves to {resolves_to}) ; mismatching values: {}",
                    expand_label_text(&label),
                    names(&missing)
                ),
            ));
            continue;
        }
        lines.push((
            1,
            format!(
                "Toolchain {} (resolves to {resolves_to}) is compatible with target platform, searching for execution platforms:",
                expand_label_text(&label)
            ),
        ));
        for (i, (platform_label, constraints)) in candidates.iter().enumerate() {
            if assigned[i].is_some() {
                continue;
            }
            let Some(platform_label) = platform_label else {
                continue;
            };
            let held =
                crate::constraints::with_defaults_for(ctx, constraints, &decl.exec_compatible_with)
                    .await?;
            let missing: Vec<&Label> = decl
                .exec_compatible_with
                .iter()
                .filter(|c| !held.contains(*c))
                .collect();
            let platform_text = expand_label_text(platform_label);
            if missing.is_empty() {
                lines.push((2, format!("Compatible execution platform {platform_text}")));
                assigned[i] = Some(decl.toolchain.clone());
            } else {
                lines.push((
                    2,
                    format!(
                        "Incompatible execution platform {platform_text}; mismatching values: {}",
                        names(&missing)
                    ),
                ));
            }
        }
        if assigned.iter().all(Option::is_some) {
            lines.push((
                1,
                format!(
                    "All execution platforms have been assigned a {type_text} toolchain, stopping"
                ),
            ));
            break;
        }
    }
    if assigned.iter().all(Option::is_none) {
        lines.push((
            0,
            format!("No {type_text} toolchain found for target platform {platform}."),
        ));
        return Ok(lines);
    }
    lines.push((
        0,
        format!("Recap of selected {type_text} toolchains for target platform {platform}:"),
    ));
    for ((platform_label, _), implementation) in candidates.iter().zip(&assigned) {
        if let (Some(platform_label), Some(implementation)) = (platform_label, implementation) {
            lines.push((
                1,
                format!(
                    "Selected {} to run on execution platform {}",
                    expand_label_text(implementation),
                    expand_label_text(platform_label)
                ),
            ));
        }
    }
    Ok(lines)
}
