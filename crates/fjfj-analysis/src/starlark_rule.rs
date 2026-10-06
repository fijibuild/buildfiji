//! Rules written in Starlark (buildfiji-136.2): run the `implementation`
//! with the targets its attributes name.

use crate::aspect::AspectKey;
use crate::target::{ConfiguredTarget, ConfiguredTargetKey, Env, RuleInfo};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::package::Package;
use fjfj_graph::rule::AttrValue;
use fjfj_graph::rule::Cfg;
use fjfj_graph::{Label, NestedSet};
use fjfj_starlark::{
    DepInfo, Edge, RuleRequest, attr_aspects, computed_defaults, labels_of_attrs, resolved_attrs,
    rule_schema, run_rule,
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn label_text(label: &Label) -> String {
    fjfj_graph::expand::label_text(label)
}

/// What a rule sees of a target it depends on.
pub fn dep_info(target: &ConfiguredTarget, generated: bool) -> DepInfo {
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

/// The repository that stands for the builtins as the file a rule is defined in.
pub(crate) const NATIVE_REPO: &str = "_builtins";

/// Whether the rule class is one of tests, which is known without analysing a
/// target of it.
pub(crate) async fn is_test(ctx: &Ctx, rule_class: &str, bzl: Option<&Label>) -> bool {
    let native = || {
        fjfj_graph::rule::native_rule(rule_class)
            .is_some_and(|class| fjfj_graph::schema::RuleSchema::native(class).test)
    };
    let Some(bzl) = bzl.filter(|b| b.repo != NATIVE_REPO) else {
        return native();
    };
    let Ok(env) = ctx.data::<Env>() else {
        return false;
    };
    let (rules, bzl) = (env.rules.clone(), bzl.clone());
    let module = tokio::task::spawn_blocking(move || rules.module(&bzl)).await;
    match module {
        Ok(Ok(module)) => rule_schema(&module, rule_class).is_some_and(|s| s.test),
        _ => false,
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
    // A native rule written in Starlark is a function of the builtins and the
    // schema Bazel gives the class.
    let native = (bzl.repo == NATIVE_REPO).then(|| {
        fjfj_graph::rule::native_rule(rule_class)
            .map(|class| Arc::new(fjfj_graph::schema::RuleSchema::native(class)))
    });
    let (module, schema) = match native {
        Some(schema) => (
            fjfj_starlark::native_builtins(),
            Some(schema.ok_or_else(|| {
                Error::msg(format!(
                    "{}: no native rule '{rule_class}'",
                    label_text(label)
                ))
            })?),
        ),
        None => {
            let (rules, bzl) = (rules.clone(), bzl.clone());
            let module = tokio::task::spawn_blocking(move || rules.module(&bzl))
                .await
                .map_err(|e| Error::msg(format!("loading a .bzl panicked: {e}")))?
                .map_err(Error::msg)?;
            (module, None)
        }
    };
    let native = schema.clone();
    let schema = match schema {
        Some(schema) => schema,
        None => rule_schema(&module, rule_class).ok_or_else(|| {
            Error::msg(format!(
                "{}: rule '{rule_class}' is not defined by {}",
                label_text(label),
                label_text(bzl)
            ))
        })?,
    };

    // A rule that transitions itself is analysed in the configuration the
    // transition makes of the one asked for.
    if schema.incoming_transition {
        let flat: Vec<(String, AttrValue)> = attrs
            .iter()
            .filter(|(_, v)| !matches!(v, AttrValue::Select(_)))
            .cloned()
            .collect();
        let mut made = crate::transition::apply(
            ctx,
            &key.configuration,
            bzl,
            rule_class,
            Edge::Incoming,
            &resolved_attrs(&schema, &flat),
        )
        .await
        .map_err(|e| Error::msg(format!("{}: {e}", label_text(label))))?;
        if made.len() != 1 {
            return Err(Error::msg(format!(
                "{}: a split transition is not supported yet (buildfiji-7w6)",
                label_text(label)
            )));
        }
        let configuration = made.remove(0);
        if configuration != key.configuration {
            let done = ctx
                .get(crate::transition::key(label.clone(), configuration))
                .await?;
            return Ok((*done).clone());
        }
    }

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

    // Defaults that are functions of the other attributes.
    if schema.attrs.iter().any(|a| a.def.computed_default) {
        let mut known = resolved_attrs(&schema, &set);
        known.push(("name".to_owned(), AttrValue::String(key.label.name.clone())));
        let (module, mappings, repo) = (module.clone(), rules.mappings(), bzl.repo.clone());
        let options = key.configuration.options.clone();
        let rule = rule_class.to_owned();
        let computed = tokio::task::spawn_blocking(move || {
            computed_defaults(&module, &rule, &known, &mappings, &repo, &options)
        })
        .await
        .map_err(|e| Error::msg(format!("computing defaults panicked: {e}")))?
        .map_err(|message| Error::msg(format!("{}: {message}", label_text(label))))?;
        for (name, value) in computed {
            if !set.iter().any(|(n, _)| *n == name) {
                set.push((name, value));
            }
        }
    }

    // The targets its attributes name, each in the configuration its edge asks.
    let edges = labels_of_attrs(&schema, &set);
    let resolved = resolved_attrs(&schema, &set);
    let mut dep_keys: Vec<ConfiguredTargetKey> = Vec::new();
    for edge in &edges {
        let configuration = match edge.cfg {
            Cfg::Target => key.configuration.clone(),
            Cfg::Exec | Cfg::Host => key.configuration.to_exec(),
            Cfg::Transition => {
                let mut made = crate::transition::apply(
                    ctx,
                    &key.configuration,
                    bzl,
                    rule_class,
                    Edge::Attr(&edge.attr),
                    &resolved,
                )
                .await
                .map_err(|e| {
                    Error::msg(format!(
                        "{}: on dependency edge {} -|{}|-> {}: {e}",
                        label_text(label),
                        label_text(label),
                        edge.attr,
                        label_text(&edge.label)
                    ))
                })?;
                if made.len() != 1 {
                    return Err(Error::msg(format!(
                        "{}: a split transition on attribute '{}' is not supported yet (buildfiji-7w6)",
                        label_text(label),
                        edge.attr
                    )));
                }
                made.remove(0)
            }
        };
        dep_keys.push(ConfiguredTargetKey {
            label: edge.label.clone(),
            configuration,
        });
    }
    let mut deps = BTreeMap::new();
    let gathered = ctx.get_all(dep_keys.clone()).await;
    let started = std::time::Instant::now();
    for (dep_key, result) in dep_keys.iter().zip(gathered) {
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
    let dep_info_time = started.elapsed();

    // The aspects its attributes ask for, on the targets they name: what they
    // provide is added to what the rule sees of those targets.
    let mut aspect_keys: Vec<AspectKey> = Vec::new();
    if native.is_none() {
        for (edge, dep_key) in edges.iter().zip(&dep_keys) {
            for aspect in attr_aspects(&module, rule_class, &edge.attr) {
                aspect_keys.push(AspectKey {
                    target: dep_key.clone(),
                    aspect,
                });
            }
        }
        for (aspect_key, result) in aspect_keys
            .iter()
            .zip(ctx.get_all(aspect_keys.clone()).await)
        {
            let result = result?;
            if let Some(dep) = deps.get_mut(&aspect_key.target.label) {
                dep.providers.extend(result.providers.iter().cloned());
            }
        }
    }

    // The toolchains it asked for, resolved.
    let exec_compatible: Vec<Label> = set
        .iter()
        .find_map(|(n, v)| match (n.as_str(), v) {
            ("exec_compatible_with", AttrValue::LabelList(list)) => Some(list.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let (toolchains, mut toolchain_keys, chosen_platform) =
        resolve_toolchains(ctx, key, &schema.toolchains, &exec_compatible).await?;
    // Each exec group runs on a platform of its own, with its own toolchains.
    let mut exec_groups = Vec::new();
    for group in &schema.exec_groups {
        let (resolved, keys, _) =
            resolve_toolchains(ctx, key, &group.toolchains, &group.exec_compatible_with).await?;
        toolchain_keys.extend(keys);
        exec_groups.push((group.name.clone(), resolved));
    }
    if chosen_platform.is_some() {
        target.execution_platform = chosen_platform;
    }

    // The outputs the class declares: `outputs = {...}` templates, and the
    // `attr.output`s the call set.
    let mut outputs: Vec<(String, String)> = Vec::new();
    for (name, template) in &schema.outputs {
        outputs.push((name.clone(), template.replace("%{name}", &label.name)));
    }
    for (name, value) in resolved_attrs(&schema, &set) {
        let ty = schema
            .attrs
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.def.ty);
        match (ty, &value) {
            (Some(fjfj_graph::rule::AttrType::Output), AttrValue::Label(out)) => {
                outputs.push((name, out.name.clone()));
            }
            // A list of outputs is one entry for each file, under the name.
            (Some(fjfj_graph::rule::AttrType::OutputList), AttrValue::LabelList(outs)) => {
                outputs.extend(outs.iter().map(|out| (name.clone(), out.name.clone())));
            }
            _ => {}
        }
    }

    let location = package
        .target(&label.name)
        .map(|t| t.location.clone())
        .unwrap_or_default();
    let build_file = location.split(':').next().unwrap_or_default().to_owned();
    let build_setting_value = match schema.build_setting {
        Some(_) => Some(crate::transition::setting_value(ctx, key).await?),
        None => None,
    };
    let rule_info = native.is_none().then(|| {
        Arc::new(RuleInfo {
            bzl: bzl.clone(),
            rule_class: rule_class.to_owned(),
            schema: schema.clone(),
            attrs: set.clone(),
            edges: edges
                .iter()
                .zip(&dep_keys)
                .map(|(e, k)| (e.attr.clone(), k.clone()))
                .collect(),
            location: location.clone(),
            build_file: build_file.clone(),
        })
    });
    let request = RuleRequest {
        module,
        rule_name: rule_class.to_owned(),
        label: label.clone(),
        location,
        build_file,
        configuration: key.configuration.clone(),
        main_repo_name: env.main_repo_name.clone(),
        attrs: set.clone(),
        deps,
        outputs,
        mappings: rules.mappings(),
        toolchains,
        exec_groups,
        build_setting_value,
        native,
    };
    let started = std::time::Instant::now();
    let result = tokio::task::spawn_blocking(move || run_rule(&request))
        .await
        .map_err(|e| Error::msg(format!("running a rule panicked: {e}")))?
        .map_err(|message| {
            // What the rule printed stays ahead of its error, which is the
            // part that is wrapped; the errors of attributes say which rule
            // they are of, and the mark that tells so stays, for whoever
            // prints one event per line.
            let (printed, message) = fjfj_starlark::split_printed(&message);
            let message = if message.starts_with(fjfj_starlark::ATTRIBUTE_ERRORS) {
                message.to_owned()
            } else {
                format!("in {rule_class} rule {}: {message}", label_text(label))
            };
            Error::msg(fjfj_starlark::with_printed(&printed, message))
        })?;
    let run_time = started.elapsed();
    let started = std::time::Instant::now();
    target.files = Arc::new(NestedSet::of(result.files));
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
    target.aspect_deps = aspect_keys;
    target.rule_info = rule_info;
    target.deps.extend(toolchain_keys);
    target.transitive_repos = crate::target::transitive_repos(ctx, &target).await?;
    let mappings = rules.mappings();
    crate::runfiles_tree::register(&mut target, &env.main_repo_name, &mappings, &|r| {
        r.to_owned()
    });
    if schema.test {
        let resolved = resolved_attrs(&schema, &set);
        crate::test_action::register(ctx, key, &resolved, &mut target).await?;
    }
    tracing::debug!(
        ?dep_info_time,
        ?run_time,
        finish_time = ?started.elapsed(),
        deps = target.deps.len(),
        %rule_class,
        label = %label_text(label),
        "analysis phases"
    );
    Ok(target)
}

/// The toolchains `types` asks for (each with whether it is mandatory), the
/// implementation resolved for each in `key`'s configuration, and the keys of
/// those implementations.
///
/// The target runs on the first execution platform that has the `exec`
/// constraints its `exec_compatible_with` asks for and has every mandatory
/// type resolve; toolchains are matched to that platform and to the target
/// platform of `key`'s configuration.
pub(crate) async fn resolve_toolchains(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    types: &[(Label, bool)],
    exec: &[Label],
) -> Result<
    (
        Vec<(Label, Option<DepInfo>)>,
        Vec<ConfiguredTargetKey>,
        Option<Label>,
    ),
    Error,
> {
    if types.is_empty() {
        if ctx.data::<Env>()?.toolchain_resolution_debug.is_some() {
            let platform = crate::toolchain::default_execution_platform(ctx, key, exec).await?;
            let outcome = crate::debug::Outcome {
                platform,
                toolchains: Vec::new(),
            };
            crate::debug::trace(ctx, key, &[], &[], &[], Some(&outcome)).await?;
        }
        return Ok((Vec::new(), Vec::new(), None));
    }
    let extra = match key
        .configuration
        .settings
        .get("//command_line_option:extra_execution_platforms")
    {
        Some(fjfj_graph::SettingValue::List(items)) => Some(items.clone()),
        _ => None,
    };
    let mut platforms: Vec<(Option<Label>, std::collections::BTreeSet<Label>)> = ctx
        .get(crate::toolchain::ExecutionPlatforms { extra })
        .await?
        .iter()
        .map(|p| (Some(p.label.clone()), p.constraints.clone()))
        .collect();
    // With none named, the target platform is where it runs.
    if platforms.is_empty() {
        platforms.push((None, key.configuration.constraints.clone()));
    }
    // The platforms the target may run on, and the types some of them serve.
    let mut candidates: Vec<(Option<Label>, std::collections::BTreeSet<Label>)> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    for (platform_label, platform) in platforms {
        let held = crate::constraints::with_defaults_for(ctx, &platform, exec).await?;
        match exec.iter().find(|c| !held.contains(*c)) {
            None => candidates.push((platform_label, platform)),
            Some(missing) => {
                if let Some(label) = &platform_label {
                    removed.push(format!(
                        "Removed execution platform {} from available execution platforms, it is missing constraint {}",
                        crate::expand_label_text(label),
                        crate::expand_label_text(missing)
                    ));
                }
            }
        }
    }
    let tried: Vec<Label> = candidates.iter().filter_map(|(l, _)| l.clone()).collect();
    let mut served: std::collections::BTreeSet<Label> = std::collections::BTreeSet::new();
    let candidates_for_trace = candidates.clone();
    for (platform_label, platform) in candidates {
        let mut toolchains: Vec<(Label, Option<DepInfo>)> = Vec::new();
        let mut unmet: Vec<Label> = Vec::new();
        let mut toolchain_keys: Vec<ConfiguredTargetKey> = Vec::new();
        for (toolchain_type, mandatory) in types {
            // The type is a dependency of the target, resolved or not.
            let type_key = ConfiguredTargetKey {
                label: toolchain_type.clone(),
                configuration: key.configuration.clone(),
            };
            ctx.get(type_key.clone()).await?;
            toolchain_keys.push(type_key);
            match crate::toolchain::resolve(ctx, key, toolchain_type, &platform).await? {
                Some(decl) => {
                    let implementation = ConfiguredTargetKey {
                        label: decl.toolchain.clone(),
                        configuration: key.configuration.clone(),
                    };
                    let done = ctx.get(implementation.clone()).await?;
                    toolchains.push((toolchain_type.clone(), Some(dep_info(&done, false))));
                    toolchain_keys.push(implementation);
                    served.insert(toolchain_type.clone());
                }
                None if *mandatory => unmet.push(toolchain_type.clone()),
                None => toolchains.push((toolchain_type.clone(), None)),
            }
        }
        if unmet.is_empty() {
            let outcome = crate::debug::Outcome {
                platform: platform_label.clone(),
                toolchains: toolchains
                    .iter()
                    .filter_map(|(t, d)| Some((t.clone(), d.as_ref()?.label.clone())))
                    .collect(),
            };
            crate::debug::trace(
                ctx,
                key,
                types,
                &candidates_for_trace,
                &removed,
                Some(&outcome),
            )
            .await?;
            return Ok((toolchains, toolchain_keys, platform_label));
        }
    }
    // The types Bazel names are those no candidate platform has a toolchain
    // for; when each has one, it is the platforms that do not agree.
    let without: Vec<Label> = types
        .iter()
        .filter(|(t, mandatory)| *mandatory && !served.contains(t))
        .map(|(t, _)| t.clone())
        .collect();
    crate::debug::trace(ctx, key, types, &candidates_for_trace, &removed, None).await?;
    if without.is_empty() {
        let all: Vec<Label> = types.iter().map(|(t, _)| t.clone()).collect();
        return Err(Error::msg(crate::toolchain::no_execution_platform(
            key, &all, &tried,
        )));
    }
    Err(Error::msg(crate::toolchain::no_match(key, &without)))
}
