//! Toolchain resolution (buildfiji-136.7), as Bazel 9.2.0 does it:
//!
//! - toolchains are the `toolchain` targets that `register_toolchains` names,
//!   in the order modules register them (`--extra_toolchains` first);
//! - for each toolchain type a rule asks for, the first registered toolchain
//!   of that type whose constraints the platforms meet and whose
//!   `target_settings` match is the one;
//! - a type with none is an error unless the rule said it is optional.

use crate::expand_label_text;
use crate::target::{ConfiguredTargetKey, Env, ToolchainDecl};
use fjfj_engine::{Ctx, Error, Key};
use fjfj_graph::Label;
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use std::sync::Arc;

/// Every registered `toolchain` target, in the order they are tried.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RegisteredToolchains;

impl Key for RegisteredToolchains {
    type Value = Vec<Label>;

    async fn compute(&self, ctx: &Ctx) -> Result<Vec<Label>, Error> {
        let env = ctx.data::<Env>()?;
        let patterns = env.registered_toolchains.clone();
        let source = env.source.clone();
        let mappings = env.rules.mappings();
        tokio::task::spawn_blocking(move || {
            let mut labels: Vec<Label> = Vec::new();
            for (module_repo, text) in &patterns {
                let context = PatternContext {
                    repo: module_repo,
                    offset: "",
                };
                let parsed = TargetPattern::parse(text, context, &mut |apparent| {
                    mappings
                        .find_apparent(module_repo, apparent)
                        .unwrap_or_else(|| apparent.to_owned())
                })
                .map_err(|e| Error::msg(format!("register_toolchains('{text}'): {e}")))?;
                let found = fjfj_loading::resolve_with(&[parsed], &*source, true);
                if let Some(failure) = found.failures.first() {
                    return Err(Error::msg(format!(
                        "register_toolchains('{text}'): {}",
                        failure.message
                    )));
                }
                for label in found.targets {
                    // Only the `toolchain` rules: a wildcard names everything,
                    // and a rule that itself needs a toolchain must not be
                    // analysed to find out which there are.
                    let is_toolchain = source
                        .package(&label.repo, &label.package)
                        .ok()
                        .and_then(|p| {
                            p.target(&label.name).map(|t| {
                                matches!(
                                    &t.kind,
                                    fjfj_graph::package::TargetKind::Rule {
                                        rule_class,
                                        defined_in: None,
                                        ..
                                    } if rule_class == "toolchain"
                                )
                            })
                        })
                        .unwrap_or(false);
                    if is_toolchain && !labels.contains(&label) {
                        labels.push(label);
                    }
                }
            }
            Ok(labels)
        })
        .await
        .map_err(|e| Error::msg(format!("expanding toolchains panicked: {e}")))?
    }
}

/// The `toolchain_type` a label stands for: an alias of one is that one.
async fn canonical_type(ctx: &Ctx, label: &Label) -> Result<Label, Error> {
    let mut current = label.clone();
    for _ in 0..16 {
        let package = ctx
            .get(crate::target::PackageKey {
                repo: current.repo.clone(),
                package: current.package.clone(),
            })
            .await?;
        let next = package.target(&current.name).and_then(|t| match &t.kind {
            fjfj_graph::package::TargetKind::Rule {
                rule_class,
                defined_in: None,
                attrs,
            } if rule_class == "alias" => attrs.iter().find_map(|(n, v)| match (n.as_str(), v) {
                ("actual", fjfj_graph::rule::AttrValue::Label(l)) => Some(l.clone()),
                _ => None,
            }),
            _ => None,
        });
        match next {
            Some(actual) => current = actual,
            None => break,
        }
    }
    Ok(current)
}

/// The toolchain target that serves `toolchain_type` in `key`'s configuration,
/// and what it says.
pub(crate) async fn resolve(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    toolchain_type: &Label,
) -> Result<Option<Arc<ToolchainDecl>>, Error> {
    let registered = ctx.get(RegisteredToolchains).await?;
    let config = &key.configuration;
    let wanted = canonical_type(ctx, toolchain_type).await?;
    for label in registered.iter() {
        let candidate = ctx
            .get(ConfiguredTargetKey {
                label: label.clone(),
                configuration: config.clone(),
            })
            .await?;
        let Some(decl) = &candidate.toolchain_decl else {
            continue;
        };
        if decl.toolchain_type != wanted
            && canonical_type(ctx, &decl.toolchain_type).await? != wanted
        {
            continue;
        }
        let has = |needed: &[Label]| needed.iter().all(|c| config.constraints.contains(c));
        // Execution and target platform are the host's for now.
        if !has(&decl.exec_compatible_with) || !has(&decl.target_compatible_with) {
            if std::env::var_os("FJFJ_TOOLCHAIN_DEBUG").is_some() {
                eprintln!(
                    "toolchain {} of {} rejected: constraints {:?} / {:?} not all in {:?}",
                    expand_label_text(label),
                    expand_label_text(toolchain_type),
                    decl.exec_compatible_with,
                    decl.target_compatible_with,
                    config.constraints
                );
            }
            continue;
        }
        let mut settings_match = true;
        for setting in &decl.target_settings {
            let target = ctx
                .get(ConfiguredTargetKey {
                    label: setting.clone(),
                    configuration: config.clone(),
                })
                .await?;
            settings_match &= target.config_matching.as_ref().is_some_and(|m| m.matches);
            if std::env::var_os("FJFJ_TOOLCHAIN_DEBUG").is_some() {
                eprintln!(
                    "toolchain {}: setting {} -> {:?}",
                    expand_label_text(label),
                    expand_label_text(setting),
                    target.config_matching
                );
            }
        }
        if settings_match {
            return Ok(Some(Arc::new(decl.clone())));
        }
    }
    Ok(None)
}

/// Bazel's words for a mandatory type that resolved to nothing.
pub(crate) fn no_match(key: &ConfiguredTargetKey, types: &[Label]) -> String {
    let list: String = types
        .iter()
        .map(|t| format!("\n  {}", expand_label_text(t)))
        .collect();
    let debug: String = types
        .iter()
        .map(expand_label_text)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "While resolving toolchains for target {} ({}): No matching toolchains found for types:{list}\nTo debug, rerun with --toolchain_resolution_debug='{debug}'\nFor more information on platforms or toolchains see https://bazel.build/concepts/platforms-intro.",
        expand_label_text(&key.label),
        configuration_checksum(&key.configuration)
    )
}

/// A short stand-in for the configuration checksum Bazel prints.
pub(crate) fn configuration_checksum(configuration: &fjfj_graph::Configuration) -> String {
    use sha2::Digest as _;
    let hash = sha2::Sha256::digest(format!("{configuration:?}").as_bytes());
    hex::encode(&hash[..4])[..7].to_owned()
}
