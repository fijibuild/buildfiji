//! The native rules, analysed in Rust (buildfiji-136.10).

use crate::select::ConfigMatching;
use crate::target::{ConfiguredTarget, ConfiguredTargetKey, Env};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::expand::{Expander, Prerequisite, label_text};
use fjfj_graph::package::Package;
use fjfj_graph::rule::{AttrValue, native_rule};
use fjfj_graph::{Action, ActionKind, Artifact, Label, LabelContext, NestedSet};
use fjfj_starlark::{Field, native_provider};
use std::collections::BTreeMap;
use std::sync::Arc;

type Attrs = [(String, AttrValue)];

fn attr<'a>(attrs: &'a Attrs, name: &str) -> Option<&'a AttrValue> {
    attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v)
}

/// The value of an attribute that may be a `select()` which no condition
/// decides. Choosing among branches is buildfiji-136.5's.
fn plain<'a>(
    label: &Label,
    name: &str,
    value: &'a AttrValue,
) -> Result<std::borrow::Cow<'a, AttrValue>, Error> {
    match value {
        AttrValue::Select(list) => list.flatten().map(std::borrow::Cow::Owned).ok_or_else(|| {
            Error::msg(format!(
                "{}: attribute '{name}' uses select(), which is not supported yet (buildfiji-136.5)",
                label_text(label)
            ))
        }),
        other => Ok(std::borrow::Cow::Borrowed(other)),
    }
}

fn labels(label: &Label, attrs: &Attrs, name: &str) -> Result<Vec<Label>, Error> {
    match attr(attrs, name) {
        None => Ok(Vec::new()),
        Some(value) => match &*plain(label, name, value)? {
            AttrValue::LabelList(list) => Ok(list.clone()),
            AttrValue::Label(one) => Ok(vec![one.clone()]),
            _ => Ok(Vec::new()),
        },
    }
}

fn string(label: &Label, attrs: &Attrs, name: &str) -> Result<Option<String>, Error> {
    match attr(attrs, name) {
        None => Ok(None),
        Some(value) => match &*plain(label, name, value)? {
            AttrValue::String(s) => Ok(Some(s.clone())),
            _ => Ok(None),
        },
    }
}

fn flag(label: &Label, attrs: &Attrs, name: &str) -> Result<bool, Error> {
    match attr(attrs, name) {
        None => Ok(false),
        Some(value) => Ok(matches!(
            &*plain(label, name, value)?,
            AttrValue::Bool(true)
        )),
    }
}

/// Analyse the rule instance `key` is. `target` has the label and the
/// configuration.
pub(crate) async fn analyze(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    package: &Arc<Package>,
    rule_class: &str,
    defined_in: Option<&Label>,
    attrs: &Attrs,
    target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    if let Some(bzl) = defined_in {
        return crate::starlark_rule::analyze(ctx, key, package, rule_class, bzl, attrs, target)
            .await;
    }
    match rule_class {
        "filegroup" => {
            let builtins = Label {
                repo: crate::starlark_rule::NATIVE_REPO.into(),
                package: String::new(),
                name: "providers.bzl".into(),
            };
            crate::starlark_rule::analyze(ctx, key, package, rule_class, &builtins, attrs, target)
                .await
        }
        "alias" => alias(ctx, key, attrs, target).await,
        "label_flag" | "label_setting" => label_flag(ctx, key, attrs, target).await,
        "genrule" => genrule(ctx, key, package, attrs, target).await,
        "config_setting" => config_setting(ctx, key, attrs, target).await,
        "toolchain" => {
            let mappings = ctx.data::<Env>()?.rules.mappings();
            toolchain_rule(key, attrs, &mappings, target)
        }
        "constraint_setting" => constraint_setting(key, target),
        "constraint_value" => constraint_value(ctx, key, attrs, target).await,
        "platform" => platform(ctx, key, attrs, target).await,
        // Rules that give providers for other rules to read and no files.
        other if native_rule(other).is_some() => Ok(target),
        other => Err(Error::msg(format!(
            "{}: the native rule '{other}' is not implemented yet",
            label_text(&key.label)
        ))),
    }
}

fn dep(key: &ConfiguredTargetKey, label: Label) -> ConfiguredTargetKey {
    ConfiguredTargetKey {
        label,
        configuration: key.configuration.clone(),
    }
}

/// The targets `labels` name, in the configuration of `key`.
async fn targets(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    labels: Vec<Label>,
) -> Result<Vec<(ConfiguredTargetKey, Arc<ConfiguredTarget>)>, Error> {
    let keys: Vec<ConfiguredTargetKey> = labels.into_iter().map(|l| dep(key, l)).collect();
    let results = ctx.get_all(keys.clone()).await;
    let mut out = Vec::with_capacity(results.len());
    for (k, result) in keys.into_iter().zip(results) {
        out.push((k, result?));
    }
    Ok(out)
}

async fn alias(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let actual = labels(&key.label, attrs, "actual")?;
    forward(ctx, key, actual, target, "alias needs exactly one 'actual'").await
}

/// `label_flag` and `label_setting`: the target their value names, which is
/// the default unless a flag or a transition set another.
async fn label_flag(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let setting = fjfj_graph::expand::label_text(&key.label);
    let chosen = match key.configuration.settings.get(&setting) {
        Some(fjfj_graph::SettingValue::Str(text)) => {
            let mappings = ctx.data::<Env>()?.rules.mappings();
            let parsed = Label::parse_mapped(
                text,
                LabelContext {
                    repo: "",
                    package: "",
                },
                &mut |apparent| mappings.resolve_apparent("", apparent),
            )
            .map_err(|e| Error::msg(format!("{setting}: bad value '{text}': {e}")))?;
            vec![parsed]
        }
        _ => labels(&key.label, attrs, "build_setting_default")?,
    };
    forward(
        ctx,
        key,
        chosen,
        target,
        "a label_flag needs a build_setting_default",
    )
    .await
}

/// This target is what `actual` is.
async fn forward(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    actual: Vec<Label>,
    mut target: ConfiguredTarget,
    message: &str,
) -> Result<ConfiguredTarget, Error> {
    let [(k, actual)] = targets(ctx, key, actual)
        .await?
        .try_into()
        .map_err(|_| Error::msg(format!("{}: {message}", label_text(&key.label))))?;
    target.files = actual.files.clone();
    target.executable = actual.executable.clone();
    target.runfiles = actual.runfiles.clone();
    // The tree of its runfiles, for what runs it as a tool.
    target.extra_outputs = actual.extra_outputs.clone();
    target.providers = actual.providers.clone();
    target.config_matching = actual.config_matching.clone();
    target.deps.push(k);
    Ok(target)
}

/// The shell Bazel runs a genrule's command with, and what it sources first.
const SHELL: &str = "/bin/bash";

async fn genrule(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    package: &Arc<Package>,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let label = &key.label;
    let config = &key.configuration;
    let location = package
        .target(&label.name)
        .map(|t| t.location.as_str())
        .unwrap_or_default();
    let bin_dir = config.bin_dir();
    let env = ctx.data::<Env>()?;

    let srcs = targets(ctx, key, labels(label, attrs, "srcs")?).await?;
    let tools = {
        let keys = labels(label, attrs, "tools")?
            .into_iter()
            .map(|l| ConfiguredTargetKey {
                label: l,
                configuration: config.to_exec(),
            })
            .collect::<Vec<_>>();
        let results = ctx.get_all(keys.clone()).await;
        let mut out = Vec::with_capacity(keys.len());
        for (k, result) in keys.into_iter().zip(results) {
            out.push((k, result?));
        }
        out
    };
    let setup_label = Label {
        repo: "bazel_tools".into(),
        package: "tools/genrule".into(),
        name: "genrule-setup.sh".into(),
    };
    let setup = targets(ctx, key, vec![setup_label]).await?;

    // The command: Bazel reads `cmd_bash` on a Unix host, then `cmd`.
    let command = match string(label, attrs, "cmd_bash")? {
        Some(c) if !c.is_empty() => c,
        _ => match string(label, attrs, "cmd")? {
            Some(c) if !c.is_empty() => c,
            _ => {
                return Err(Error::msg(format!(
                    "{}: missing value for `cmd` attribute, you can also set `cmd_ps` on Windows and `cmd_bat` as a fallback",
                    label_text(label)
                )));
            }
        },
    };

    // The files it creates. `outs` is `Output`-typed: each is a label in this
    // package whose name is the path.
    let mut outs: Vec<Artifact> = Vec::new();
    let mut outputs = BTreeMap::new();
    for out in labels(label, attrs, "outs")? {
        let artifact = Artifact::derived(&bin_dir, &label.repo, &label.package, &out.name);
        outputs.insert(out.name.clone(), artifact.clone());
        outs.push(artifact);
    }
    let _ = package;

    let src_files: Vec<Artifact> = srcs.iter().flat_map(|(_, t)| t.files.to_vec()).collect();
    let prerequisites: Vec<Prerequisite> = srcs
        .iter()
        .chain(&tools)
        .map(|(k, t)| Prerequisite {
            label: k.label.clone(),
            files: t.files.to_vec(),
        })
        .collect();
    let expander = Expander {
        rule_class: "genrule",
        attribute: "cmd",
        label,
        srcs: &src_files,
        outs: &outs,
        prerequisites: &prerequisites,
        bin_dir: &bin_dir,
        target_cpu: target_cpu(&config.cpu),
        compilation_mode: config.compilation_mode.name(),
        defines: &config.defines,
        main_repo_name: &env.main_repo_name,
        context: LabelContext {
            repo: &label.repo,
            package: &label.package,
        },
    };
    let expanded = expander
        .expand(&command)
        .map_err(|e| Error::msg(e.to_string()))?;

    let setup_script = setup[0].1.files.to_vec();
    // Bazel nests them as the setup script, the sources and the tools.
    // A tool that is an executable brings its runfiles tree.
    let tool_sets: Vec<Arc<NestedSet<Artifact>>> = tools
        .iter()
        .map(|(_, t)| {
            let tree = t.executable.as_ref().and(t.extra_outputs.first());
            NestedSet::join([
                &t.files,
                &Arc::new(NestedSet::of(tree.cloned().into_iter().collect())),
            ])
        })
        .collect();
    let input_set = NestedSet::join([
        &NestedSet::join(srcs.iter().map(|(_, t)| &t.files)),
        &NestedSet::join(&tool_sets),
        &setup[0].1.files,
    ]);
    let script = setup_script
        .first()
        .map(Artifact::exec_path)
        .ok_or_else(|| Error::msg("@bazel_tools has no genrule-setup.sh"))?;
    let mut inputs = setup_script;
    inputs.extend(src_files);
    inputs.extend(tool_sets.iter().flat_map(|set| set.to_vec()));

    let mut execution_requirements = BTreeMap::new();
    if flag(label, attrs, "local")? {
        execution_requirements.insert("local".to_owned(), "1".to_owned());
    }
    target.actions.push(Action {
        owner: label.clone(),
        owner_kind: "genrule".to_owned(),
        location: location.to_owned(),
        configuration: config.mnemonic(),
        mnemonic: "Genrule".to_owned(),
        progress_message: Some(
            string(label, attrs, "message")?
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| format!("Executing genrule {}", label_text(label))),
        ),
        kind: ActionKind::Spawn {
            argv: vec![
                SHELL.to_owned(),
                "-c".to_owned(),
                format!("source {script}; {expanded}"),
            ],
            env: config.default_shell_env(),
            execution_requirements,
        },
        inputs,
        input_set: Some(input_set),
        outputs: outs.clone(),
        exec_group: None,
    });
    if flag(label, attrs, "executable")? && outs.len() == 1 {
        target.executable = Some(outs[0].clone());
    }
    target.files = Arc::new(NestedSet::of(outs));
    target.outputs = outputs;
    target.deps = srcs
        .into_iter()
        .chain(tools)
        .chain(setup)
        .map(|(k, _)| k)
        .collect();
    Ok(target)
}

/// What `$(TARGET_CPU)` is for a `--cpu`.
fn target_cpu(cpu: &str) -> &str {
    match cpu {
        "k8" => "x86_64",
        other => other,
    }
}

/// `constraint_setting`: its `ConstraintSettingInfo`.
fn constraint_setting(
    key: &ConfiguredTargetKey,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let info = native_provider(
        "platform_common.ConstraintSettingInfo",
        vec![
            ("label".to_owned(), Field::Label(key.label.clone())),
            ("default_constraint_value".to_owned(), Field::None),
        ],
    )
    .map_err(Error::msg)?;
    target.providers.push(info);
    Ok(target)
}

/// `platform`: the constraint values it has, each overriding its parents' value
/// of the same setting.
async fn platform(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let mut decl = crate::target::PlatformDecl::default();
    let parents = targets(ctx, key, labels(&key.label, attrs, "parents")?).await?;
    for (_, parent) in &parents {
        let Some(parent) = &parent.platform else {
            return Err(Error::msg(format!(
                "{}: '{}' in parents is not a platform",
                label_text(&key.label),
                label_text(&parent.label)
            )));
        };
        decl.constraints.extend(parent.constraints.clone());
    }
    let values = targets(ctx, key, labels(&key.label, attrs, "constraint_values")?).await?;
    for (k, value) in &values {
        let Some(setting) = &value.constraint_setting else {
            return Err(Error::msg(format!(
                "{}: '{}' is not a constraint_value",
                label_text(&key.label),
                label_text(&k.label)
            )));
        };
        if let Some(earlier) = decl.constraints.get(setting)
            && earlier != &k.label
            && values.iter().any(|(o, _)| o.label == *earlier)
        {
            return Err(Error::msg(format!(
                "{}: Duplicate constraint values detected: constraint_setting {} has [{}, {}] set on the same platform",
                label_text(&key.label),
                label_text(setting),
                label_text(earlier),
                label_text(&k.label)
            )));
        }
        decl.constraints.insert(setting.clone(), k.label.clone());
    }
    target.deps.extend(parents.into_iter().map(|(k, _)| k));
    target.deps.extend(values.into_iter().map(|(k, _)| k));
    target.platform = Some(decl);
    Ok(target)
}

/// `constraint_value`: its `ConstraintValueInfo`, which holds its setting's.
async fn constraint_value(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let setting = labels(&key.label, attrs, "constraint_setting")?;
    let [(k, setting)] = targets(ctx, key, setting).await?.try_into().map_err(|_| {
        Error::msg(format!(
            "{}: constraint_value needs a constraint_setting",
            label_text(&key.label)
        ))
    })?;
    let Some(setting_info) = setting.providers.first().cloned() else {
        return Err(Error::msg(format!(
            "{}: '{}' is not a constraint_setting",
            label_text(&key.label),
            label_text(&k.label)
        )));
    };
    let info = native_provider(
        "platform_common.ConstraintValueInfo",
        vec![
            ("constraint".to_owned(), Field::Provider(setting_info)),
            ("label".to_owned(), Field::Label(key.label.clone())),
        ],
    )
    .map_err(Error::msg)?;
    target.providers.push(info);
    target.constraint_setting = Some(k.label.clone());
    // A constraint_value is also a condition of a select(), as the
    // config_setting that lists only it.
    target.config_matching = Some(ConfigMatching {
        matches: crate::constraints::with_defaults_for(
            ctx,
            &key.configuration.constraints,
            std::slice::from_ref(&key.label),
        )
        .await?
        .contains(&key.label),
        conditions: [format!("constraint:{}", label_text(&key.label))].into(),
    });
    target.deps.push(k);
    Ok(target)
}

/// `config_setting`: does this configuration have the flags, defines,
/// Starlark flags and constraints it lists?
async fn config_setting(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let config = &key.configuration;
    let mut matching = ConfigMatching {
        matches: true,
        conditions: Default::default(),
    };
    let mut check = |condition: String, holds: bool| {
        matching.conditions.insert(condition);
        matching.matches &= holds;
    };
    if let Some(AttrValue::StringDict(values)) = attr(attrs, "values") {
        for (flag, value) in values {
            check(
                format!("values:{flag}={value}"),
                crate::select::flag_matches(config, flag, value),
            );
        }
    }
    if let Some(AttrValue::StringDict(defines)) = attr(attrs, "define_values") {
        for (name, value) in defines {
            check(
                format!("define:{name}={value}"),
                config.defines.get(name).is_some_and(|d| d == value),
            );
        }
    }
    for constraint in labels(&key.label, attrs, "constraint_values")? {
        let holds = crate::constraints::with_defaults_for(
            ctx,
            &config.constraints,
            std::slice::from_ref(&constraint),
        )
        .await?
        .contains(&constraint);
        check(format!("constraint:{}", label_text(&constraint)), holds);
    }
    if let Some(AttrValue::LabelKeyedStringDict(flags)) = attr(attrs, "flag_values") {
        for (flag, wanted) in flags {
            // The setting is a dependency of the `config_setting`.
            let flag_key = ConfiguredTargetKey {
                label: flag.clone(),
                configuration: config.clone(),
            };
            ctx.get(flag_key.clone()).await?;
            target.deps.push(flag_key);
            let current = if crate::transition::is_build_setting(ctx, flag).await? {
                crate::transition::setting_in(ctx, config, flag).await?
            } else {
                // A rule that gives `config_common.FeatureFlagInfo`.
                let flag_key = ConfiguredTargetKey {
                    label: flag.clone(),
                    configuration: config.clone(),
                };
                let flag_target = ctx.get(flag_key).await?;
                let value = flag_target
                    .providers
                    .iter()
                    .find_map(fjfj_starlark::feature_flag_value)
                    .ok_or_else(|| {
                        Error::msg(format!(
                            "{} is not a build setting or a feature flag",
                            label_text(flag)
                        ))
                    })?;
                fjfj_graph::SettingValue::Str(value)
            };
            check(
                format!("flag:{}={wanted}", label_text(flag)),
                crate::transition::matches_text(&current, wanted),
            );
        }
    }
    target.config_matching = Some(matching);
    Ok(target)
}

/// `toolchain`: what it offers and what it needs. The implementation is not
/// analysed until a rule resolves to it.
fn toolchain_rule(
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    mappings: &fjfj_starlark::RepoMappings,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let one = |name: &str| -> Result<Label, Error> {
        let found = match attr(attrs, name) {
            Some(AttrValue::Label(l)) => Some(l.clone()),
            // `toolchain` is a string in the schema; it names a label.
            // It is written in the package's repository, so `@name` is as that
            // repository maps it.
            Some(AttrValue::String(text)) => Label::parse_mapped(
                text,
                LabelContext {
                    repo: &key.label.repo,
                    package: &key.label.package,
                },
                &mut |apparent| mappings.resolve_apparent(&key.label.repo, apparent),
            )
            .ok(),
            _ => None,
        };
        found.ok_or_else(|| {
            Error::msg(format!(
                "{}: toolchain needs a '{name}'",
                label_text(&key.label)
            ))
        })
    };
    target.toolchain_decl = Some(crate::target::ToolchainDecl {
        toolchain_type: one("toolchain_type")?,
        toolchain: one("toolchain")?,
        exec_compatible_with: labels(&key.label, attrs, "exec_compatible_with")?,
        target_compatible_with: labels(&key.label, attrs, "target_compatible_with")?,
        target_settings: labels(&key.label, attrs, "target_settings")?,
    });
    Ok(target)
}
