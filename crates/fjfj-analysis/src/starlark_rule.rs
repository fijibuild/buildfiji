//! Rules written in Starlark (buildfiji-136.2): run the `implementation`
//! with the targets its attributes name.

use crate::target::{ConfiguredTarget, ConfiguredTargetKey, Env};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::package::Package;
use fjfj_graph::rule::AttrValue;
use fjfj_graph::{Label, NestedSet};
use fjfj_starlark::{DepInfo, RuleRequest, labels_of_attrs, resolved_attrs, rule_schema, run_rule};
use std::collections::BTreeMap;
use std::sync::Arc;

fn label_text(label: &Label) -> String {
    fjfj_graph::expand::label_text(label)
}

/// What a rule sees of a target it depends on.
fn dep_info(target: &ConfiguredTarget, generated: bool) -> DepInfo {
    DepInfo {
        label: target.label.clone(),
        rule_class: target.rule_class.clone(),
        generated,
        files: target.files.to_vec(),
        executable: target.executable.clone(),
        runfiles: target.runfiles.clone(),
        providers: target.providers.clone(),
    }
}

pub(crate) async fn analyze(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    package: &Arc<Package>,
    rule_class: &str,
    bzl: &Label,
    attrs: &[(String, AttrValue)],
    mut target: ConfiguredTarget,
) -> Result<ConfiguredTarget, Error> {
    let env = ctx.data::<Env>()?;
    let label = &key.label;
    let rules = env.rules.clone();
    let module = {
        let (rules, bzl) = (rules.clone(), bzl.clone());
        tokio::task::spawn_blocking(move || rules.module(&bzl))
            .await
            .map_err(|e| Error::msg(format!("loading a .bzl panicked: {e}")))?
            .map_err(Error::msg)?
    };
    let schema = rule_schema(&module, rule_class).ok_or_else(|| {
        Error::msg(format!(
            "{}: rule '{rule_class}' is not defined by {}",
            label_text(label),
            label_text(bzl)
        ))
    })?;

    // A `select()` that no configuration decides is flattened, or refused.
    let mut set: Vec<(String, AttrValue)> = Vec::new();
    for (name, value) in attrs {
        match value {
            AttrValue::Select(list) => match list.flatten() {
                Some(flat) => set.push((name.clone(), flat)),
                None => {
                    return Err(Error::msg(format!(
                        "{}: attribute '{name}' uses select(), which is not supported yet (buildfiji-136.5)",
                        label_text(label)
                    )));
                }
            },
            other => set.push((name.clone(), other.clone())),
        }
    }

    // The targets its attributes name, in this configuration.
    let dep_labels = labels_of_attrs(&schema, &set);
    let dep_keys: Vec<ConfiguredTargetKey> = dep_labels
        .iter()
        .map(|l| ConfiguredTargetKey {
            label: l.clone(),
            configuration: key.configuration.clone(),
        })
        .collect();
    let mut deps = BTreeMap::new();
    for (dep_key, result) in dep_keys.iter().zip(ctx.get_all(dep_keys.clone()).await) {
        let done = result?;
        let generated = package.target(&dep_key.label.name).is_some_and(|t| {
            dep_key.label.repo == label.repo
                && dep_key.label.package == label.package
                && matches!(
                    t.kind,
                    fjfj_graph::package::TargetKind::GeneratedFile { .. }
                )
        });
        deps.insert(dep_key.label.clone(), dep_info(&done, generated));
    }

    // The outputs the class declares: `outputs = {...}` templates, and the
    // `attr.output`s the call set.
    let mut outputs: Vec<(String, String)> = Vec::new();
    for (name, template) in &schema.outputs {
        outputs.push((name.clone(), template.replace("%{name}", &label.name)));
    }
    for (name, value) in resolved_attrs(&schema, &set) {
        let is_output = schema
            .attrs
            .iter()
            .any(|a| a.name == name && a.def.ty == fjfj_graph::rule::AttrType::Output);
        if let (true, AttrValue::Label(out)) = (is_output, &value) {
            outputs.push((name, out.name.clone()));
        }
    }

    let location = package
        .target(&label.name)
        .map(|t| t.location.clone())
        .unwrap_or_default();
    let build_file = location.split(':').next().unwrap_or_default().to_owned();
    let request = RuleRequest {
        module,
        rule_name: rule_class.to_owned(),
        label: label.clone(),
        location,
        build_file,
        configuration: key.configuration.clone(),
        main_repo_name: env.main_repo_name.clone(),
        attrs: set,
        deps,
        outputs,
        mappings: rules.mappings(),
    };
    let result = tokio::task::spawn_blocking(move || run_rule(&request))
        .await
        .map_err(|e| Error::msg(format!("running a rule panicked: {e}")))?
        .map_err(|message| {
            Error::msg(format!(
                "{}: in {rule_class} rule {}: {message}",
                label_text(label),
                label_text(label)
            ))
        })?;
    target.files = NestedSet::of(result.files);
    target.executable = result.executable;
    target.runfiles = result.runfiles;
    target.actions = result.actions;
    target.providers = result.providers;
    target.printed = result.printed;
    target.outputs = result
        .outputs
        .into_values()
        .map(|a| {
            let name = a
                .path
                .strip_prefix(&format!("{}/", label.package))
                .unwrap_or(&a.path)
                .to_owned();
            (name, a)
        })
        .collect();
    target.deps = dep_keys;
    Ok(target)
}
