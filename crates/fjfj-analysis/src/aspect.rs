//! Aspects (buildfiji-136.8): an aspect applied to a configured target is a
//! key of its own, whose value is the providers, files and actions the
//! aspect's implementation made from looking at the target.
//!
//! An aspect follows the attributes of a rule that its `attr_aspects` name to
//! the targets they name, and applies there too, so the aspect on a target
//! reads the aspect on each of those. A target an aspect does not apply to (its
//! `required_providers` are not met) has an empty result and the aspect goes no
//! further through it.

use crate::starlark_rule::{dep_info, resolve_toolchains};
use crate::target::{ConfiguredTarget, ConfiguredTargetKey, Env};
use fjfj_engine::{Ctx, Error, Key};
use fjfj_graph::rule::Cfg;
use fjfj_graph::{Label, NestedSet};
use fjfj_starlark::{
    AspectRef, AspectRequest, DepInfo, aspect_applies, aspect_spec, labels_of_attrs, run_aspect,
};
use std::collections::BTreeMap;

/// An aspect applied to a target.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AspectKey {
    pub target: ConfiguredTargetKey,
    pub aspect: AspectRef,
}

fn label_text(label: &Label) -> String {
    fjfj_graph::expand::label_text(label)
}

impl Key for AspectKey {
    type Value = ConfiguredTarget;

    async fn compute(&self, ctx: &Ctx) -> Result<ConfiguredTarget, Error> {
        let env = ctx.data::<Env>()?;
        let base = ctx.get(self.target.clone()).await?;
        let mut out = ConfiguredTarget::new(&self.target);
        out.rule_class = Some(self.aspect.name.clone());
        out.aspect = Some(self.aspect.clone());
        let Some(info) = base.rule_info.clone() else {
            return Ok(out);
        };
        let rules = env.rules.clone();
        let bzl = self.aspect.bzl.clone();
        let module = {
            let (rules, bzl) = (rules.clone(), bzl.clone());
            tokio::task::spawn_blocking(move || rules.module(&bzl))
                .await
                .map_err(|e| Error::msg(format!("loading a .bzl panicked: {e}")))?
                .map_err(Error::msg)?
        };
        let name = &self.aspect.name;
        if !aspect_applies(&module, name, &base.providers) {
            return Ok(out);
        }
        let spec = aspect_spec(&module, name).ok_or_else(|| {
            Error::msg(format!("{} is not an aspect of {}", name, label_text(&bzl)))
        })?;
        // The aspects it requires run on this target first; what they made is
        // part of the target it looks at.
        let mut seen = dep_info(&base, false);
        let required_keys: Vec<AspectKey> = spec
            .requires
            .iter()
            .map(|aspect| AspectKey {
                target: self.target.clone(),
                aspect: aspect.clone(),
            })
            .collect();
        for (key, result) in required_keys
            .iter()
            .zip(ctx.get_all(required_keys.clone()).await)
        {
            seen.providers.extend(result?.providers.iter().cloned());
            out.aspect_deps.push(key.clone());
        }
        let follows = |attr: &str| spec.attr_aspects.iter().any(|a| a == "*" || a == attr);

        // The targets the rule's attributes name, each with this aspect's
        // result where the aspect follows the attribute.
        let dep_keys: Vec<ConfiguredTargetKey> =
            info.edges.iter().map(|(_, k)| k.clone()).collect();
        let dones = ctx.get_all(dep_keys).await;
        let aspect_keys: Vec<Option<AspectKey>> = info
            .edges
            .iter()
            .map(|(attr, key)| {
                follows(attr).then(|| AspectKey {
                    target: key.clone(),
                    aspect: self.aspect.clone(),
                })
            })
            .collect();
        let results = ctx
            .get_all(aspect_keys.iter().flatten().cloned().collect::<Vec<_>>())
            .await;
        let mut results = results.into_iter();
        let mut rule_deps: BTreeMap<Label, DepInfo> = BTreeMap::new();
        for (((_, key), done), aspect_key) in info.edges.iter().zip(dones).zip(aspect_keys) {
            let mut dep = dep_info(&*done?, false);
            let followed = match aspect_key {
                Some(k) => {
                    let result = results.next().expect("one result per key")?;
                    dep.providers.extend(result.providers.iter().cloned());
                    out.aspect_deps.push(k);
                    true
                }
                None => false,
            };
            if followed || !rule_deps.contains_key(&key.label) {
                rule_deps.insert(key.label.clone(), dep);
            }
        }

        // The aspect's own attributes name targets too.
        let mut own_keys: Vec<ConfiguredTargetKey> = Vec::new();
        for edge in labels_of_attrs(&spec.schema, &[]) {
            let configuration = match edge.cfg {
                Cfg::Target => self.target.configuration.clone(),
                Cfg::Exec | Cfg::Host => self.target.configuration.to_exec(),
                Cfg::Transition => {
                    return Err(Error::msg(format!(
                        "{}: a transition on attribute '{}' of an aspect is not supported yet (buildfiji-136.8)",
                        label_text(&bzl),
                        edge.attr
                    )));
                }
            };
            own_keys.push(ConfiguredTargetKey {
                label: edge.label,
                configuration,
            });
        }
        let mut deps: BTreeMap<Label, DepInfo> = BTreeMap::new();
        for (key, done) in own_keys.iter().zip(ctx.get_all(own_keys.clone()).await) {
            deps.insert(key.label.clone(), dep_info(&*done?, false));
        }
        let (toolchains, toolchain_keys, _) =
            resolve_toolchains(ctx, &self.target, &spec.schema.toolchains, &[]).await?;

        let request = AspectRequest {
            module,
            aspect: name.clone(),
            aspect_schema: spec.schema.clone(),
            aspect_ids: vec![format!("{}%{}", label_text(&bzl), name)],
            target: seen,
            rule_kind: info.rule_class.clone(),
            rule_schema: info.schema.clone(),
            rule_attrs: info.attrs.clone(),
            rule_deps,
            deps,
            location: info.location.clone(),
            build_file: info.build_file.clone(),
            configuration: self.target.configuration.clone(),
            main_repo_name: env.main_repo_name.clone(),
            mappings: rules.mappings(),
            toolchains,
        };
        let label = self.target.label.clone();
        let result = tokio::task::spawn_blocking(move || run_aspect(&request))
            .await
            .map_err(|e| Error::msg(format!("running an aspect panicked: {e}")))?
            .map_err(|message| {
                Error::msg(format!(
                    "{}: in aspect {}%{name}: {message}",
                    label_text(&label),
                    label_text(&bzl)
                ))
            })?;
        out.files = NestedSet::of(result.files);
        out.runfiles = result.runfiles;
        out.actions = result.actions;
        out.providers = result.providers;
        out.printed = result.printed;
        out.deps = own_keys;
        out.deps.extend(toolchain_keys);
        Ok(out)
    }
}
