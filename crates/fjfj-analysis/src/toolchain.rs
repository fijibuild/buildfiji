//! Toolchain resolution (buildfiji-136.7), as Bazel 9.2.0 does it:
//!
//! - toolchains are the `toolchain` targets that `register_toolchains` names,
//!   in the order modules register them (`--extra_toolchains` first);
//! - for each toolchain type a rule asks for, the first registered toolchain
//!   of that type whose constraints the platforms meet and whose
//!   `target_settings` match is the one;
//! - a type with none is an error unless the rule said it is optional.

use crate::expand_label_text;
use crate::target::{ConfiguredTargetKey, Env, PackageKey, ToolchainDecl};
use fjfj_engine::{Ctx, Error, Key};
use fjfj_graph::Label;
use fjfj_graph::package::{Package, PackageBuilder};
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_loading::{PackageLookup, PackageSource};
use fjfj_starlark::RepoMappings;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

/// Every registered `toolchain` target, in the order they are tried.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RegisteredToolchains {
    /// The `--extra_toolchains` a configuration sets, if it does.
    pub extra: Option<Vec<String>>,
}

impl Key for RegisteredToolchains {
    type Value = Vec<Label>;

    async fn compute(&self, ctx: &Ctx) -> Result<Vec<Label>, Error> {
        let env = ctx.data::<Env>()?;
        let extra = self
            .extra
            .clone()
            .unwrap_or_else(|| env.extra_toolchains.clone());
        let patterns: Vec<(String, String)> = extra
            .into_iter()
            .map(|pattern| (String::new(), pattern))
            .chain(env.registered_toolchains.iter().cloned())
            .collect();
        expand_all(ctx, patterns, "toolchain", "register_toolchains").await
    }
}

/// A platform a target can run its actions on, with the constraint values it has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExecPlatform {
    pub label: Label,
    pub constraints: BTreeSet<Label>,
}

/// The execution platforms, in the order they are tried: `--extra_execution_platforms`,
/// the registered ones, then the host.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExecutionPlatforms {
    /// The `--extra_execution_platforms` a configuration sets, if it does.
    pub extra: Option<Vec<String>>,
}

impl Key for ExecutionPlatforms {
    type Value = Vec<ExecPlatform>;

    async fn compute(&self, ctx: &Ctx) -> Result<Vec<ExecPlatform>, Error> {
        let env = ctx.data::<Env>()?;
        let extra = self
            .extra
            .clone()
            .unwrap_or_else(|| env.extra_execution_platforms.clone());
        let patterns: Vec<(String, String)> = extra
            .into_iter()
            .map(|pattern| (String::new(), pattern))
            .chain(env.registered_execution_platforms.iter().cloned())
            .collect();
        let labels = expand_all(ctx, patterns, "platform", "register_execution_platforms").await?;
        let mut platforms = Vec::new();
        for label in labels {
            let constraints = crate::platform_constraints_in(ctx, &label).await?;
            platforms.push(ExecPlatform { label, constraints });
        }
        if let Some(host) = &env.host_constraints {
            platforms.push(ExecPlatform {
                label: Label {
                    repo: "platforms".into(),
                    package: "host".into(),
                    name: "host".into(),
                },
                constraints: host.clone(),
            });
        }
        Ok(platforms)
    }
}

/// The first execution platform that has the `exec` constraints, which is
/// where a target runs its actions when it needs no toolchain to say
/// otherwise.
pub(crate) async fn default_execution_platform(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    exec: &[Label],
) -> Result<Option<Label>, Error> {
    let extra = match key
        .configuration
        .settings
        .get("//command_line_option:extra_execution_platforms")
    {
        Some(fjfj_graph::SettingValue::List(items)) => Some(items.clone()),
        _ => None,
    };
    let platforms = ctx.get(ExecutionPlatforms { extra }).await?;
    for p in platforms.iter() {
        let held = crate::constraints::with_defaults_for(ctx, &p.constraints, exec).await?;
        if exec.iter().all(|c| held.contains(c)) {
            return Ok(Some(p.label.clone()));
        }
    }
    Ok(None)
}

/// The configuration the tools of the target `key` are built in, when it runs
/// on `platform` (the first execution platform that has the `exec`
/// constraints, if none is chosen): that platform is the target platform of the
/// tools.
pub(crate) async fn exec_configuration(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    platform: Option<&Label>,
    exec: &[Label],
) -> Result<fjfj_graph::Configuration, Error> {
    let extra = match key
        .configuration
        .settings
        .get("//command_line_option:extra_execution_platforms")
    {
        Some(fjfj_graph::SettingValue::List(items)) => Some(items.clone()),
        _ => None,
    };
    let platforms = ctx.get(ExecutionPlatforms { extra }).await?;
    let mut chosen = None;
    for p in platforms.iter() {
        let wanted = match platform {
            Some(label) => p.label == *label,
            None => {
                let held = crate::constraints::with_defaults_for(ctx, &p.constraints, exec).await?;
                exec.iter().all(|c| held.contains(c))
            }
        };
        if wanted {
            chosen = Some(p);
            break;
        }
    }
    Ok(match chosen {
        Some(p) => key.configuration.to_exec_on(&p.label, &p.constraints),
        None => key.configuration.to_exec(),
    })
}

/// The `rule_class` targets `patterns` name, in order.
async fn expand_all(
    ctx: &Ctx,
    patterns: Vec<(String, String)>,
    rule_class: &'static str,
    function: &'static str,
) -> Result<Vec<Label>, Error> {
    let env = ctx.data::<Env>()?;
    let mappings = env.rules.mappings();
    // Expansion is synchronous, so it runs against the packages fetched so
    // far and notes the ones it lacked; those are then fetched from the
    // engine, in parallel and shared with analysis, and it runs again.
    let mut known: HashMap<(String, String), Result<Arc<Package>, String>> = HashMap::new();
    loop {
        let source = Prefetched {
            inner: env.source.clone(),
            known: std::mem::take(&mut known),
            missing: Mutex::new(BTreeSet::new()),
        };
        let patterns = patterns.clone();
        let mappings = mappings.clone();
        let (source, expanded) = tokio::task::spawn_blocking(move || {
            let expanded = expand(&patterns, &mappings, &source, rule_class, function);
            (source, expanded)
        })
        .await
        .map_err(|e| Error::msg(format!("expanding {function} panicked: {e}")))?;
        let Prefetched {
            known: fetched,
            missing,
            ..
        } = source;
        let missing: Vec<(String, String)> = missing.into_inner().unwrap().into_iter().collect();
        if missing.is_empty() {
            return expanded;
        }
        known = fetched;
        let keys = missing
            .iter()
            .map(|(repo, package)| PackageKey {
                repo: repo.clone(),
                package: package.clone(),
            })
            .collect();
        for (at, loaded) in missing.into_iter().zip(ctx.get_all(keys).await) {
            known.insert(at, loaded.map(|p| (*p).clone()).map_err(|e| e.to_string()));
        }
    }
}

/// Packages the engine has already loaded; one it has not is recorded as
/// missing and stands in as an empty package, so a pass finds all it needs.
struct Prefetched {
    inner: Arc<dyn PackageSource>,
    known: HashMap<(String, String), Result<Arc<Package>, String>>,
    missing: Mutex<BTreeSet<(String, String)>>,
}

impl PackageSource for Prefetched {
    fn lookup(&self, repo: &str) -> Result<Arc<PackageLookup>, String> {
        self.inner.lookup(repo)
    }

    fn package(&self, repo: &str, package: &str) -> Result<Arc<Package>, String> {
        let at = (repo.to_owned(), package.to_owned());
        if let Some(found) = self.known.get(&at) {
            return found.clone();
        }
        self.missing.lock().unwrap().insert(at);
        Ok(Arc::new(
            PackageBuilder::new(repo, package, &|_: &str| false).build(),
        ))
    }
}

/// The `rule_class` targets `patterns` name, in order.
fn expand(
    patterns: &[(String, String)],
    mappings: &RepoMappings,
    source: &dyn PackageSource,
    wanted_class: &str,
    function: &str,
) -> Result<Vec<Label>, Error> {
    let mut labels: Vec<Label> = Vec::new();
    for (module_repo, text) in patterns {
        let context = PatternContext {
            repo: module_repo,
            offset: "",
        };
        let parsed = TargetPattern::parse(text, context, &mut |apparent| {
            mappings
                .find_apparent(module_repo, apparent)
                .unwrap_or_else(|| apparent.to_owned())
        })
        .map_err(|e| Error::msg(format!("{function}('{text}'): {e}")))?;
        let found = fjfj_loading::resolve_with(&[parsed], source, true);
        if let Some(failure) = found.failures.first() {
            return Err(Error::msg(format!(
                "{function}('{text}'): {}",
                failure.message
            )));
        }
        for label in found.targets {
            // Only the one rule class: a wildcard names everything, and a
            // rule that itself needs a toolchain must not be analysed to find
            // out which there are.
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
                            } if rule_class == wanted_class
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
}

/// The target a label stands for: an alias of one is that one (a toolchain
/// type, a build setting).
pub(crate) async fn follow_aliases(ctx: &Ctx, label: &Label) -> Result<Label, Error> {
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

/// The toolchain target that serves `toolchain_type` for a target in `key`'s
/// configuration that runs its actions on a platform with the `exec` constraints,
/// and what it says.
pub(crate) async fn resolve(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    toolchain_type: &Label,
    exec: &BTreeSet<Label>,
) -> Result<Option<Arc<ToolchainDecl>>, Error> {
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
            && follow_aliases(ctx, &decl.toolchain_type).await? != wanted
        {
            continue;
        }
        let has = |needed: &[Label], platform: &BTreeSet<Label>| {
            needed.iter().all(|c| platform.contains(c))
        };
        let exec_held =
            crate::constraints::with_defaults_for(ctx, exec, &decl.exec_compatible_with).await?;
        let exec_met = has(&decl.exec_compatible_with, &exec_held);
        let target_held = crate::constraints::with_defaults_for(
            ctx,
            &config.constraints,
            &decl.target_compatible_with,
        )
        .await?;
        if !exec_met || !has(&decl.target_compatible_with, &target_held) {
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
        .join("|");
    format!(
        "While resolving toolchains for target {} ({}): No matching toolchains found for types:{list}\nTo debug, rerun with --toolchain_resolution_debug='{debug}'\nFor more information on platforms or toolchains see https://bazel.build/concepts/platforms-intro.",
        expand_label_text(&key.label),
        configuration_checksum(&key.configuration)
    )
}

/// Bazel's words for types that each have a toolchain for the target
/// platform, though no execution platform has them all.
pub(crate) fn no_execution_platform(
    key: &ConfiguredTargetKey,
    types: &[Label],
    platforms: &[Label],
) -> String {
    let words = |labels: &[Label]| {
        labels
            .iter()
            .map(expand_label_text)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let target = match key
        .configuration
        .settings
        .get("//command_line_option:platforms")
    {
        Some(fjfj_graph::SettingValue::List(items)) if !items.is_empty() => items[0].clone(),
        _ => "@@platforms//host:host".to_owned(),
    };
    format!(
        "While resolving toolchains for target {} ({}): Unable to find an execution platform for toolchains [{}] and target platform {target} from available execution platforms [{}]",
        expand_label_text(&key.label),
        configuration_checksum(&key.configuration),
        words(types),
        words(platforms)
    )
}

/// A short stand-in for the configuration checksum Bazel prints.
pub(crate) fn configuration_checksum(configuration: &fjfj_graph::Configuration) -> String {
    use sha2::Digest as _;
    let hash = sha2::Sha256::digest(format!("{configuration:?}").as_bytes());
    hex::encode(&hash[..4])[..7].to_owned()
}
