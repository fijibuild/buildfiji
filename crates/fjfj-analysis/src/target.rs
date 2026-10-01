//! The keys of analysis and what they compute.

use crate::native;
use fjfj_engine::{Ctx, Engine, Error, Key};
use fjfj_graph::package::{Package, TargetKind};
use fjfj_graph::{Action, Artifact, Configuration, Label, NestedSet};
use fjfj_loading::PackageSource;
use std::collections::BTreeMap;
use std::sync::Arc;

/// What analysis reads that is not a key.
pub struct Env {
    /// Where packages come from.
    pub source: Arc<dyn PackageSource>,
    /// What the main repository is called in a runfiles tree: `_main`.
    pub main_repo_name: String,
}

/// An engine that analyses against `env`.
pub fn engine(env: Env) -> Engine {
    let engine = Engine::new();
    engine.set_data(env);
    engine
}

/// A package, its BUILD file evaluated.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackageKey {
    pub repo: String,
    pub package: String,
}

impl Key for PackageKey {
    type Value = Arc<Package>;

    async fn compute(&self, ctx: &Ctx) -> Result<Arc<Package>, Error> {
        let env = ctx.data::<Env>()?;
        let source = env.source.clone();
        let (repo, package) = (self.repo.clone(), self.package.clone());
        tokio::task::spawn_blocking(move || source.package(&repo, &package))
            .await
            .map_err(|e| Error::msg(format!("loading a package panicked: {e}")))?
            .map_err(Error::msg)
    }
}

/// A target in a configuration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConfiguredTargetKey {
    pub label: Label,
    pub configuration: Configuration,
}

/// What a target gives the targets that depend on it, and the actions that
/// make it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredTarget {
    pub label: Label,
    pub configuration: Configuration,
    /// The native rule or `.bzl` rule class, `None` for a file.
    pub rule_class: Option<String>,
    /// `DefaultInfo.files`: what building the target builds.
    pub files: NestedSet<Artifact>,
    /// `DefaultInfo.executable`.
    pub executable: Option<Artifact>,
    /// The files the rule declares it creates, by target name.
    pub outputs: BTreeMap<String, Artifact>,
    /// The actions this target registered.
    pub actions: Vec<Action>,
    /// The targets it read, for finding every action a build needs.
    pub deps: Vec<ConfiguredTargetKey>,
}

impl ConfiguredTarget {
    pub(crate) fn new(key: &ConfiguredTargetKey) -> ConfiguredTarget {
        ConfiguredTarget {
            label: key.label.clone(),
            configuration: key.configuration.clone(),
            rule_class: None,
            files: NestedSet::empty(),
            executable: None,
            outputs: BTreeMap::new(),
            actions: Vec::new(),
            deps: Vec::new(),
        }
    }
}

impl Key for ConfiguredTargetKey {
    type Value = ConfiguredTarget;

    async fn compute(&self, ctx: &Ctx) -> Result<ConfiguredTarget, Error> {
        let label = &self.label;
        let package = ctx
            .get(PackageKey {
                repo: label.repo.clone(),
                package: label.package.clone(),
            })
            .await?;
        let mut target = ConfiguredTarget::new(self);
        match package.target(&label.name) {
            Some(declared) => match &declared.kind {
                TargetKind::Rule {
                    rule_class,
                    defined_in,
                    attrs,
                } => {
                    target.rule_class = Some(rule_class.clone());
                    native::analyze(
                        ctx,
                        self,
                        &package,
                        rule_class,
                        defined_in.as_ref(),
                        attrs,
                        target,
                    )
                    .await
                }
                TargetKind::SourceFile => Ok(source_file(self, target)),
                TargetKind::PackageGroup(_) => Ok(target),
                TargetKind::GeneratedFile { rule } => {
                    let producer = ConfiguredTargetKey {
                        label: Label {
                            name: rule.clone(),
                            ..label.clone()
                        },
                        configuration: self.configuration.clone(),
                    };
                    let made = ctx.get(producer.clone()).await?;
                    let artifact = made.outputs.get(&label.name).cloned().ok_or_else(|| {
                        Error::msg(format!("rule '{rule}' does not create '{}'", label.name))
                    })?;
                    target.files = NestedSet::of(vec![artifact]);
                    target.deps.push(producer);
                    Ok(target)
                }
            },
            None => {
                // A file only a rule names, or the BUILD file itself.
                let source = ctx.data::<Env>()?.source.clone();
                let lookup = source.lookup(&label.repo).map_err(Error::msg)?;
                fjfj_loading::declared_target(&package, &lookup, label).map_err(Error::msg)?;
                Ok(source_file(self, target))
            }
        }
    }
}

/// The target of a source file.
fn source_file(key: &ConfiguredTargetKey, mut target: ConfiguredTarget) -> ConfiguredTarget {
    target.files = NestedSet::of(vec![Artifact::source(
        &key.label.repo,
        &key.label.package,
        &key.label.name,
    )]);
    target
}
