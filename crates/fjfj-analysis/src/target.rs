//! The keys of analysis and what they compute.

use crate::native;
use fjfj_engine::{Ctx, Engine, Error, Key};
use fjfj_graph::package::{Package, TargetKind};
use fjfj_graph::{Action, Artifact, Configuration, Label, NestedSet};
use fjfj_loading::PackageSource;
use fjfj_starlark::{RuleSource, StoredProvider};
use std::collections::BTreeMap;
use std::sync::Arc;

/// What analysis reads that is not a key.
pub struct Env {
    /// Where packages come from.
    pub source: Arc<dyn PackageSource>,
    /// Where the `.bzl` files of rules come from.
    pub rules: Arc<dyn RuleSource>,
    /// What the main repository is called in a runfiles tree: `_main`.
    pub main_repo_name: String,
    /// `register_toolchains` patterns, each with the canonical repo of the module that
    /// wrote it, in the order they are considered.
    pub registered_toolchains: Vec<(String, String)>,
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
    /// `DefaultInfo.default_runfiles`.
    pub runfiles: fjfj_graph::Runfiles,
    /// The files the rule declares it creates, by target name.
    pub outputs: BTreeMap<String, Artifact>,
    /// Files built with the target that its results do not list: the
    /// runfiles tree of an executable.
    pub extra_outputs: Vec<Artifact>,
    /// For a test: how to run it.
    pub test: Option<fjfj_graph::TestInfo>,
    /// The providers it gave besides `DefaultInfo`.
    pub providers: Vec<StoredProvider>,
    /// What its rule printed.
    pub printed: Vec<String>,
    /// For a `toolchain`: what it offers and what it needs.
    pub toolchain_decl: Option<ToolchainDecl>,
    /// For a `constraint_value`: the `constraint_setting` it is a value of.
    pub constraint_setting: Option<Label>,
    /// For a `platform`: the constraints it has, a value for each setting.
    pub platform: Option<PlatformDecl>,
    /// For a `config_setting`: whether it matches this configuration.
    pub config_matching: Option<crate::select::ConfigMatching>,
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
            runfiles: fjfj_graph::Runfiles::default(),
            outputs: BTreeMap::new(),
            extra_outputs: Vec::new(),
            test: None,
            providers: Vec::new(),
            printed: Vec::new(),
            toolchain_decl: None,
            constraint_setting: None,
            platform: None,
            config_matching: None,
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
                    let attrs = crate::select::resolve(ctx, self, attrs).await?;
                    native::analyze(
                        ctx,
                        self,
                        &package,
                        rule_class,
                        defined_in.as_ref(),
                        &attrs,
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

/// What a `toolchain` target says: the type it is of, the target that
/// implements it, and the constraints under which it applies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolchainDecl {
    pub toolchain_type: Label,
    pub toolchain: Label,
    pub exec_compatible_with: Vec<Label>,
    pub target_compatible_with: Vec<Label>,
    pub target_settings: Vec<Label>,
}

/// What a `platform` says: a `constraint_value` for each `constraint_setting`
/// it constrains, its parents' included.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlatformDecl {
    pub constraints: BTreeMap<Label, Label>,
}

impl PlatformDecl {
    /// The values, which is what a configuration holds of a platform.
    pub fn values(&self) -> std::collections::BTreeSet<Label> {
        self.constraints.values().cloned().collect()
    }
}
