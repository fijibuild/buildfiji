//! The native rules, analysed in Rust (buildfiji-136.10).

use crate::expand::{Expander, Prerequisite, label_text};
use crate::target::{ConfiguredTarget, ConfiguredTargetKey, Env};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::package::Package;
use fjfj_graph::rule::{AttrValue, native_rule};
use fjfj_graph::{Action, ActionKind, Artifact, Label, LabelContext, NestedSet};
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
        return Err(Error::msg(format!(
            "{}: rule '{rule_class}' is defined in {}, and rules written in Starlark are not analysed yet (buildfiji-136.2)",
            label_text(&key.label),
            label_text(bzl)
        )));
    }
    match rule_class {
        "filegroup" => filegroup(ctx, key, attrs, target).await,
        "alias" => alias(ctx, key, attrs, target).await,
        "genrule" => genrule(ctx, key, package, attrs, target).await,
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

async fn filegroup(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let srcs = targets(ctx, key, labels(&key.label, attrs, "srcs")?).await?;
    target.files = NestedSet::new(
        Vec::new(),
        srcs.iter()
            .map(|(_, t)| Arc::new(t.files.clone()))
            .collect(),
    );
    target.deps = srcs.into_iter().map(|(k, _)| k).collect();
    Ok(target)
}

async fn alias(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let actual = labels(&key.label, attrs, "actual")?;
    let [(k, actual)] = targets(ctx, key, actual).await?.try_into().map_err(|_| {
        Error::msg(format!(
            "{}: alias needs exactly one 'actual'",
            label_text(&key.label)
        ))
    })?;
    target.files = actual.files.clone();
    target.executable = actual.executable.clone();
    target.deps.push(k);
    Ok(target)
}

/// The shell Bazel runs a genrule's command with, and what it sources first.
const SHELL: &str = "/bin/bash";
const SHELL_PATH: &str = "/bin:/usr/bin:/usr/local/bin";

async fn genrule(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    package: &Arc<Package>,
    attrs: &Attrs,
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let label = &key.label;
    let config = &key.configuration;
    let bin_dir = config.bin_dir();
    let env = ctx.data::<Env>()?;

    let srcs = targets(ctx, key, labels(label, attrs, "srcs")?).await?;
    let tools = targets(ctx, key, labels(label, attrs, "tools")?).await?;
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
    let tool_files: Vec<Artifact> = tools.iter().flat_map(|(_, t)| t.files.to_vec()).collect();
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
    let script = setup_script
        .first()
        .map(Artifact::exec_path)
        .ok_or_else(|| Error::msg("@bazel_tools has no genrule-setup.sh"))?;
    let mut inputs = setup_script;
    inputs.extend(src_files);
    inputs.extend(tool_files);

    let mut execution_requirements = BTreeMap::new();
    if flag(label, attrs, "local")? {
        execution_requirements.insert("local".to_owned(), "1".to_owned());
    }
    target.actions.push(Action {
        owner: label.clone(),
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
            env: BTreeMap::from([("PATH".to_owned(), SHELL_PATH.to_owned())]),
            execution_requirements,
        },
        inputs,
        outputs: outs.clone(),
    });
    if flag(label, attrs, "executable")? && outs.len() == 1 {
        target.executable = Some(outs[0].clone());
    }
    target.files = NestedSet::of(outs);
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
