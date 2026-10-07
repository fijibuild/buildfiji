//! Configuration transitions (buildfiji-136.6): a `transition()` reads the
//! settings it names from the configuration of the target and says what to
//! change; the dependency (or the target itself, for `rule(cfg = ...)`) is
//! then analysed in the configuration that results.
//!
//! Settings are command-line options (`//command_line_option:copt`) and
//! user-defined build settings (`//pkg:flag`). A setting at its default is
//! absent from [`Configuration::settings`].

use crate::target::{ConfiguredTargetKey, Env, PackageKey};
use fjfj_engine::{Ctx, Error, Key};
use fjfj_graph::config::COMMAND_LINE_OPTION;
use fjfj_graph::package::TargetKind;
use fjfj_graph::rule::AttrValue;
use fjfj_graph::schema::SettingKind;
use fjfj_graph::{CompilationMode, Configuration, Label, LabelContext, SettingValue};
use fjfj_starlark::{Edge, FrozenModule, apply_transition, rule_schema, transition_specs};
use std::collections::BTreeMap;

/// Command-line options a transition may read and write, with the value they
/// have when nothing sets them.
fn native_default(name: &str) -> Option<SettingValue> {
    Some(match name {
        "extra_toolchains" | "copt" | "cxxopt" | "conlyopt" | "linkopt" | "host_copt"
        | "host_cxxopt" | "host_conlyopt" | "host_linkopt" => SettingValue::List(Vec::new()),
        "stamp" | "legacy_external_runfiles" | "incompatible_default_to_explicit_init_py" => {
            SettingValue::Bool(false)
        }
        "build_runfile_links" | "enable_runfiles" => SettingValue::Bool(true),
        _ => return None,
    })
}

/// The label a setting named `text` is, in the repo of the `.bzl` that wrote it.
fn setting_label(text: &str, repo: &str) -> Result<Label, Error> {
    Label::parse(text, LabelContext { repo, package: "" })
        .map_err(|e| Error::msg(format!("build setting '{text}': {e}")))
}

/// What a build setting holds when nothing sets it, and its type.
async fn user_default(ctx: &Ctx, label: &Label) -> Result<(SettingKind, SettingValue), Error> {
    default_of(ctx, label).await?.ok_or_else(|| {
        Error::msg(format!(
            "{} is not a build setting",
            fjfj_graph::expand::label_text(label)
        ))
    })
}

/// [`user_default`], or `None` when the target is not a build setting.
async fn default_of(
    ctx: &Ctx,
    label: &Label,
) -> Result<Option<(SettingKind, SettingValue)>, Error> {
    let package = ctx
        .get(PackageKey {
            repo: label.repo.clone(),
            package: label.package.clone(),
        })
        .await?;
    let Some(target) = package.target(&label.name) else {
        return Ok(None);
    };
    let TargetKind::Rule {
        rule_class,
        defined_in,
        attrs,
    } = &target.kind
    else {
        return Ok(None);
    };
    let default = attrs
        .iter()
        .find(|(n, _)| n == "build_setting_default")
        .map(|(_, v)| v);
    // A native `label_flag` or `string_flag` holds text.
    let kind = match defined_in {
        None => SettingKind::String,
        Some(bzl) => {
            let env = ctx.data::<Env>()?;
            let module = module_of(&env, bzl).await?;
            match rule_schema(&module, rule_class).and_then(|s| s.build_setting) {
                Some(setting) => setting.kind,
                None => return Ok(None),
            }
        }
    };
    let value = match default {
        Some(AttrValue::Bool(b)) => SettingValue::Bool(*b),
        Some(AttrValue::Int(i)) => SettingValue::Int(i64::from(*i)),
        Some(AttrValue::String(s)) => SettingValue::Str(s.clone()),
        Some(AttrValue::StringList(items)) => SettingValue::List(items.clone()),
        Some(AttrValue::Label(l)) => SettingValue::Str(fjfj_graph::expand::label_text(l)),
        _ => return Ok(None),
    };
    Ok(Some((kind, value)))
}

/// Whether `label` (after aliases) is a build setting.
pub(crate) async fn is_build_setting(ctx: &Ctx, label: &Label) -> Result<bool, Error> {
    let label = crate::toolchain::follow_aliases(ctx, label).await?;
    Ok(default_of(ctx, &label).await?.is_some())
}

/// Whether a label is a build setting, as an engine value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuildSettingKey(pub Label);

impl Key for BuildSettingKey {
    type Value = bool;

    async fn compute(&self, ctx: &Ctx) -> Result<bool, Error> {
        is_build_setting(ctx, &self.0).await
    }
}

async fn module_of(env: &Env, bzl: &Label) -> Result<FrozenModule, Error> {
    let (rules, bzl) = (env.rules.clone(), bzl.clone());
    tokio::task::spawn_blocking(move || rules.module(&bzl))
        .await
        .map_err(|e| Error::msg(format!("loading a .bzl panicked: {e}")))?
        .map_err(Error::msg)
}

/// A value from the command line, which is text, as the setting's type holds it.
fn typed(kind: SettingKind, value: &SettingValue, name: &str) -> Result<SettingValue, Error> {
    let SettingValue::Str(text) = value else {
        return Ok(value.clone());
    };
    let bad = |want: &str| {
        Error::msg(format!(
            "While parsing option --{name}={text}: '{text}' is not a {want}"
        ))
    };
    Ok(match kind {
        SettingKind::String => value.clone(),
        SettingKind::Bool => match text.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "t" | "y" => SettingValue::Bool(true),
            "false" | "0" | "no" | "f" | "n" => SettingValue::Bool(false),
            _ => return Err(bad("boolean")),
        },
        SettingKind::Int => SettingValue::Int(text.parse().map_err(|_| bad("integer"))?),
        SettingKind::StringList | SettingKind::StringSet => {
            SettingValue::List(text.split(',').map(str::to_owned).collect())
        }
    })
}

/// What the build setting `label` holds in `configuration`: what was set on
/// the command line or by a transition, else its default.
pub(crate) async fn setting_in(
    ctx: &Ctx,
    configuration: &Configuration,
    label: &Label,
) -> Result<SettingValue, Error> {
    let label = &crate::toolchain::follow_aliases(ctx, label).await?;
    let name = fjfj_graph::expand::label_text(label);
    let (kind, default) = user_default(ctx, label).await?;
    match configuration.settings.get(&name) {
        Some(set) => typed(kind, set, &name),
        None => Ok(default),
    }
}

/// Whether a setting's value is what a `flag_values` entry of a
/// `config_setting` says.
pub(crate) fn matches_text(value: &SettingValue, wanted: &str) -> bool {
    match value {
        SettingValue::Bool(b) => wanted.eq_ignore_ascii_case(&b.to_string()),
        SettingValue::Int(i) => wanted.parse() == Ok(*i),
        SettingValue::Str(s) | SettingValue::Label(s) => s.eq_ignore_ascii_case(wanted),
        SettingValue::None => false,
        SettingValue::List(items) => wanted.split(',').eq(items.iter().map(String::as_str)),
    }
}

/// What `configuration` has `name` set to.
async fn read(
    ctx: &Ctx,
    configuration: &Configuration,
    name: &str,
    repo: &str,
) -> Result<SettingValue, Error> {
    if let Some(option) = name.strip_prefix(COMMAND_LINE_OPTION) {
        return match option {
            "compilation_mode" => Ok(SettingValue::Str(
                configuration.compilation_mode.name().to_owned(),
            )),
            "cpu" => Ok(SettingValue::Str(configuration.cpu.clone())),
            "platforms" => Ok(configuration
                .settings
                .get(name)
                .cloned()
                .unwrap_or_else(|| {
                    SettingValue::List(vec!["@@bazel_tools//tools:host_platform".to_owned()])
                })),
            "define" => Err(Error::msg(
                "Starlark transition on --define not supported - try using build settings \
                 (https://bazel.build/rules/config#user-defined-build-settings).",
            )),
            other => configuration
                .settings
                .get(name)
                .cloned()
                .or_else(|| native_default(other))
                .ok_or_else(|| {
                    Error::msg(format!(
                        "transitions on --{other} are not supported yet (buildfiji-136.6)"
                    ))
                }),
        };
    }
    setting_in(ctx, configuration, &setting_label(name, repo)?).await
}

/// `configuration` with `name` set to `value`.
async fn write(
    ctx: &Ctx,
    configuration: &mut Configuration,
    name: &str,
    value: SettingValue,
    repo: &str,
) -> Result<(), Error> {
    let before = read(ctx, configuration, name, repo).await?;
    if before == value {
        return Ok(());
    }
    // A setting is among those a transition changed until it is set back to
    // what it was before the first of them did.
    let original = configuration
        .affected
        .get(name)
        .cloned()
        .unwrap_or_else(|| before.clone());
    let note = |configuration: &mut Configuration, value: &SettingValue| {
        if *value == original {
            configuration.affected.remove(name);
        } else {
            configuration
                .affected
                .insert(name.to_owned(), original.clone());
        }
    };
    if let Some(option) = name.strip_prefix(COMMAND_LINE_OPTION) {
        match (option, &value) {
            ("compilation_mode", SettingValue::Str(mode)) => {
                configuration.compilation_mode = CompilationMode::parse(mode).ok_or_else(|| {
                    Error::msg(format!(
                        "Invalid value for --compilation_mode: '{mode}' (should be one of fastbuild, dbg, opt)"
                    ))
                })?;
                return Ok(());
            }
            ("cpu", SettingValue::Str(cpu)) => {
                configuration.cpu = cpu.clone();
                return Ok(());
            }
            _ => {}
        }
        if option == "platforms" {
            let SettingValue::List(labels) = &value else {
                return Err(Error::msg("--platforms takes a list of labels"));
            };
            let [text] = &labels[..] else {
                return Err(Error::msg(
                    "Multiple platforms are not supported: --platforms takes exactly one",
                ));
            };
            let label = setting_label(text, "")?;
            let constraints = crate::platform_constraints_in(ctx, &label).await?;
            configuration.constraints = constraints;
            configuration.platform = Some(label);
            configuration
                .settings
                .insert(name.to_owned(), value.clone());
            note(configuration, &value);
            return Ok(());
        }
        if native_default(option).is_none() {
            return Err(Error::msg(format!(
                "transitions on --{option} are not supported yet (buildfiji-136.6)"
            )));
        }
        if native_default(option).as_ref() == Some(&value) {
            configuration.settings.remove(name);
        } else {
            configuration
                .settings
                .insert(name.to_owned(), value.clone());
        }
        note(configuration, &value);
        return Ok(());
    }
    let default = user_default(ctx, &setting_label(name, repo)?).await?.1;
    if value == default {
        configuration.settings.remove(name);
    } else {
        configuration
            .settings
            .insert(name.to_owned(), value.clone());
    }
    note(configuration, &value);
    Ok(())
}

/// The configurations the transition on `edge` of the rule `rule_class`
/// (defined in `bzl`) makes of `from`. The same one if the edge has no
/// transition; several for a split.
pub(crate) async fn apply(
    ctx: &Ctx,
    from: &Configuration,
    bzl: &Label,
    rule_class: &str,
    edge: Edge<'_>,
    attrs: &[(String, AttrValue)],
) -> Result<Vec<(String, Configuration)>, Error> {
    let env = ctx.data::<Env>()?;
    let module = module_of(&env, bzl).await?;
    let Some(specs) = transition_specs(&module, rule_class, edge) else {
        return Ok(vec![(String::new(), from.clone())]);
    };
    let mappings = env.rules.mappings();
    let attrs = attrs.to_vec();
    let owned_edge = match edge {
        Edge::Incoming => None,
        Edge::Attr(name) => Some(name.to_owned()),
    };
    // A composed transition runs each part on every configuration the one
    // before made; a split part gives each of its keys.
    let mut out = vec![(String::new(), from.clone())];
    for (part, spec) in specs.iter().enumerate() {
        let mut next = Vec::new();
        for (key, config) in out {
            let mut settings = BTreeMap::new();
            for input in &spec.inputs {
                settings.insert(input.clone(), read(ctx, &config, input, &bzl.repo).await?);
            }
            let outcomes = {
                let (module, rule, attrs, mappings) = (
                    module.clone(),
                    rule_class.to_owned(),
                    attrs.clone(),
                    mappings.clone(),
                );
                let owned_edge = owned_edge.clone();
                tokio::task::spawn_blocking(move || {
                    let edge = match &owned_edge {
                        None => Edge::Incoming,
                        Some(name) => Edge::Attr(name),
                    };
                    apply_transition(&module, &rule, edge, part, &settings, &attrs, &mappings)
                })
                .await
                .map_err(|e| Error::msg(format!("running a transition panicked: {e}")))?
                .map_err(Error::msg)?
            };
            for (split, changes) in outcomes {
                let mut made = config.clone();
                for (name, value) in changes {
                    write(ctx, &mut made, &name, value, &bzl.repo).await?;
                }
                // Two splits make keys of both, as `a,x`.
                let joined = match (key.is_empty(), split.is_empty()) {
                    (_, true) => key.clone(),
                    (true, false) => split,
                    (false, false) => format!("{key},{split}"),
                };
                next.push((joined, made));
            }
        }
        out = next;
    }
    Ok(out)
}

/// The key of `label` after the transition of an edge.
pub(crate) fn key(label: Label, configuration: Configuration) -> ConfiguredTargetKey {
    ConfiguredTargetKey {
        label,
        configuration,
    }
}

/// The value of the build setting `key` is, in its configuration. A build
/// setting that is not a flag is always its default.
pub(crate) async fn setting_value(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
) -> Result<SettingValue, Error> {
    setting_in(ctx, &key.configuration, &key.label).await
}
