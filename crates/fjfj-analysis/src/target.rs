//! The keys of analysis and what they compute.

use crate::native;
use fjfj_engine::{Ctx, Engine, Error, Key};
use fjfj_graph::package::{Package, TargetKind};
use fjfj_graph::rule::AttrValue;
use fjfj_graph::{Action, Artifact, Configuration, Label, NestedSet};
use fjfj_loading::PackageSource;
use fjfj_starlark::{RuleSource, StoredProvider};
use std::collections::{BTreeMap, BTreeSet};
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
    /// `--extra_toolchains` patterns (main repository), tried before those;
    /// a configuration that sets the option replaces them.
    pub extra_toolchains: Vec<String>,
    /// `register_execution_platforms` patterns, likewise.
    pub registered_execution_platforms: Vec<(String, String)>,
    /// `--extra_execution_platforms` patterns, tried before those.
    pub extra_execution_platforms: Vec<String>,
    /// The constraint values of the host platform, an execution platform tried
    /// after the others; `None` for none, in which case a target with no
    /// execution platform registered runs on its target platform.
    pub host_constraints: Option<std::collections::BTreeSet<Label>>,
    /// Work out which execution platform each rule's target runs on, for
    /// `aquery` to show; builds do not need it for a rule without toolchains.
    pub record_execution_platforms: bool,
    /// `--toolchain_resolution_debug`: say how the toolchains of the targets
    /// and types it finds were resolved.
    pub toolchain_resolution_debug: Option<crate::ResolutionDebug>,
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
    pub files: Arc<NestedSet<Artifact>>,
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
    /// For a rule: the attributes it was declared with, `select()`s decided
    /// for this configuration.
    pub attrs: Vec<(String, AttrValue)>,
    /// For what an aspect made of a target: the aspect. Such a value has the
    /// label and configuration of the target it was applied to.
    pub aspect: Option<fjfj_starlark::AspectRef>,
    /// The platform its actions run on, when [`Env::record_execution_platforms`]
    /// or its toolchains decided it.
    pub execution_platform: Option<Label>,
    /// The configuration its tools (`cfg = "exec"` attributes) are built in,
    /// when it has any: that of the platform it runs on.
    pub exec_configuration: Option<fjfj_graph::Configuration>,
    /// The platform each exec group the rule declared runs its actions on.
    pub exec_group_platforms: Vec<(String, Option<Label>)>,
    /// The toolchain implementations it resolved, each with the platform they
    /// were resolved to run on, which is where their own actions run.
    pub toolchain_platforms: Vec<(Label, Label)>,
    /// A rule that asked for no toolchain, whose execution platform was
    /// worked out only because `--toolchain_resolution_debug` was on.
    pub debug_no_toolchains: bool,
    /// For an `alias` or `label_flag`: the label of the target it is, which
    /// is the label a rule that depends on it sees.
    pub actual: Option<Label>,
    /// The actions this target registered.
    pub actions: Vec<Action>,
    /// The targets it read, for finding every action a build needs.
    pub deps: Vec<ConfiguredTargetKey>,
    /// The aspects it read: those its attributes ask for on the targets they
    /// name, and for an aspect, the ones it propagated to.
    pub aspect_deps: Vec<crate::aspect::AspectKey>,
    /// For a target of a rule written in Starlark: what an aspect needs to
    /// look at it as its rule's attributes.
    pub rule_info: Option<Arc<RuleInfo>>,
    /// The repositories of the packages of everything it depends on, and
    /// its own: what Bazel calls the transitive packages, which decide the
    /// source repositories of a runfiles tree's repo mapping.
    pub transitive_repos: Arc<BTreeSet<String>>,
    /// Why the target cannot be built for its configuration's platform, if it
    /// cannot: it, or a target it reads, asks for constraints the platform
    /// lacks.
    pub incompatible: Option<Arc<Incompatible>>,
}

/// Why a target is incompatible with the platform it is configured for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Incompatible {
    /// From the target that reads an incompatible one, to the one that is
    /// incompatible itself, which is last.
    pub chain: Vec<ConfiguredTargetKey>,
    /// The `target_compatible_with` constraint values of the last that the
    /// platform does not have.
    pub unsatisfied: Vec<Label>,
    /// The rule of the incompatible target is a test rule.
    pub is_test: bool,
}

/// What an aspect sees of the rule that made a target.
#[derive(Clone, Debug)]
pub struct RuleInfo {
    pub bzl: Label,
    pub rule_class: String,
    pub schema: Arc<fjfj_graph::schema::RuleSchema>,
    /// The attributes the BUILD file set, `select()`s decided.
    pub attrs: Vec<(String, fjfj_graph::rule::AttrValue)>,
    /// Each target an attribute named, by the attribute, in the configuration
    /// the edge asked for.
    pub edges: Vec<(String, ConfiguredTargetKey)>,
    pub location: String,
    pub build_file: String,
}

impl PartialEq for RuleInfo {
    fn eq(&self, other: &RuleInfo) -> bool {
        self.bzl == other.bzl
            && self.rule_class == other.rule_class
            && Arc::ptr_eq(&self.schema, &other.schema)
            && self.attrs == other.attrs
            && self.edges == other.edges
    }
}

impl Eq for RuleInfo {}

impl ConfiguredTarget {
    /// Whether the target has a configuration, as a rule and a generated file
    /// do and a source file does not.
    pub fn has_configuration(&self) -> bool {
        self.rule_class.is_some() || self.files.to_vec().first().is_some_and(|f| !f.is_source())
    }

    pub fn new(key: &ConfiguredTargetKey) -> ConfiguredTarget {
        ConfiguredTarget {
            label: key.label.clone(),
            configuration: key.configuration.clone(),
            rule_class: None,
            files: Arc::new(NestedSet::empty()),
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
            execution_platform: None,
            exec_configuration: None,
            exec_group_platforms: Vec::new(),
            toolchain_platforms: Vec::new(),
            debug_no_toolchains: false,
            actual: None,
            attrs: Vec::new(),
            aspect: None,
            actions: Vec::new(),
            deps: Vec::new(),
            aspect_deps: Vec::new(),
            rule_info: None,
            transitive_repos: Arc::new(BTreeSet::from([key.label.repo.clone()])),
            incompatible: None,
        }
    }
}

impl Key for ConfiguredTargetKey {
    type Value = ConfiguredTarget;

    async fn compute(&self, ctx: &Ctx) -> Result<ConfiguredTarget, Error> {
        let mut target = self.analyse(ctx).await?;
        target.transitive_repos = transitive_repos(ctx, &target).await?;
        // A target that reads an incompatible one is incompatible too, which
        // is known once what it reads is analysed.
        if target.incompatible.is_none() && target.aspect.is_none() {
            for dep in ctx.get_all(target.deps.clone()).await {
                if let Some(inner) = &dep?.incompatible {
                    target.incompatible = Some(Arc::new(Incompatible {
                        chain: std::iter::once(self.clone())
                            .chain(inner.chain.iter().cloned())
                            .collect(),
                        unsatisfied: inner.unsatisfied.clone(),
                        is_test: target.test.is_some(),
                    }));
                    break;
                }
            }
        }
        Ok(target)
    }
}

/// The repositories of `target`'s package and of every package it reads.
pub(crate) async fn transitive_repos(
    ctx: &Ctx,
    target: &ConfiguredTarget,
) -> Result<Arc<BTreeSet<String>>, Error> {
    let mut repos: BTreeSet<String> = (*target.transitive_repos).clone();
    for dep in ctx.get_all(target.deps.clone()).await {
        repos.extend(dep?.transitive_repos.iter().cloned());
    }
    if repos.len() == target.transitive_repos.len() {
        return Ok(target.transitive_repos.clone());
    }
    Ok(Arc::new(repos))
}

impl ConfiguredTargetKey {
    async fn analyse(&self, ctx: &Ctx) -> Result<ConfiguredTarget, Error> {
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
                    let attrs_declared = attrs;
                    let attrs = crate::select::resolve(ctx, self, attrs).await?;
                    target.attrs = attrs.clone();
                    // A platform without the constraints the target asks for
                    // cannot build it, and what it reads is not analysed: only
                    // the constraint values that decided it are.
                    if let Some((values, unsatisfied)) =
                        unsatisfied_constraints(ctx, &self.configuration, rule_class, &attrs)
                            .await?
                    {
                        // The `select()` conditions were read to get here.
                        for label in values
                            .into_iter()
                            .chain(declared_conditions(attrs_declared))
                        {
                            let key = ConfiguredTargetKey {
                                label,
                                configuration: self.configuration.clone(),
                            };
                            if !target.deps.contains(&key) {
                                target.deps.push(key);
                            }
                        }
                        target.incompatible = Some(Arc::new(Incompatible {
                            chain: vec![self.clone()],
                            unsatisfied,
                            is_test: crate::starlark_rule::is_test(
                                ctx,
                                rule_class,
                                defined_in.as_ref(),
                            )
                            .await,
                        }));
                        return Ok(target);
                    }
                    // A platform or a constraint is what execution platforms are
                    // made of: it has none of its own, and asking for it while
                    // the platforms are being resolved would be a cycle.
                    if ctx.data::<Env>()?.record_execution_platforms
                        && !matches!(
                            rule_class.as_str(),
                            "platform" | "constraint_setting" | "constraint_value" | "toolchain"
                        )
                    {
                        let exec: Vec<Label> = attrs
                            .iter()
                            .find_map(|(n, v)| match (n.as_str(), v) {
                                ("exec_compatible_with", AttrValue::LabelList(list)) => {
                                    Some(list.clone())
                                }
                                _ => None,
                            })
                            .unwrap_or_default();
                        target.execution_platform =
                            crate::toolchain::default_execution_platform(ctx, self, &exec).await?;
                    }
                    // The `config_setting`s its `select()`s read are targets it
                    // depends on, which `cquery` shows.
                    let conditions: Vec<ConfiguredTargetKey> = declared_conditions(attrs_declared)
                        .into_iter()
                        .map(|label| ConfiguredTargetKey {
                            label,
                            configuration: self.configuration.clone(),
                        })
                        .collect();
                    let mut analysed = native::analyze(
                        ctx,
                        self,
                        &package,
                        rule_class,
                        defined_in.as_ref(),
                        &attrs,
                        target,
                    )
                    .await?;
                    for condition in conditions {
                        if !analysed.deps.contains(&condition) {
                            analysed.deps.push(condition);
                        }
                    }
                    Ok(analysed)
                }
                TargetKind::SourceFile => Ok(source_file(self, target)),
                TargetKind::PackageGroup(_) => {
                    // What a rule that takes an allowlist reads; `contains` is
                    // not answered yet.
                    let info =
                        fjfj_starlark::native_provider("PackageSpecificationInfo", Vec::new())
                            .map_err(Error::msg)?;
                    target.providers.push(info);
                    Ok(target)
                }
                TargetKind::EnvironmentGroup { .. } => Err(Error::msg(format!(
                    "{} is an environment group, which no rule can depend on",
                    label
                ))),
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
                    target.files = Arc::new(NestedSet::of(vec![artifact]));
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

/// The `target_compatible_with` values of a rule and those among them the
/// platform of `configuration` lacks, when there are any. A configuration
/// that does not know its platform (no `@platforms`) asks nothing, and a
/// `toolchain` says what it is for by the attribute, not what it needs.
async fn unsatisfied_constraints(
    ctx: &Ctx,
    configuration: &Configuration,
    rule_class: &str,
    attrs: &[(String, AttrValue)],
) -> Result<Option<(Vec<Label>, Vec<Label>)>, Error> {
    if rule_class == "toolchain" || configuration.constraints.is_empty() {
        return Ok(None);
    }
    let Some(values) = attrs.iter().find_map(|(name, value)| match value {
        AttrValue::LabelList(list) if name == "target_compatible_with" => Some(list.clone()),
        _ => None,
    }) else {
        return Ok(None);
    };
    let held =
        crate::constraints::with_defaults_for(ctx, &configuration.constraints, &values).await?;
    let missing: Vec<Label> = values
        .iter()
        .filter(|v| !held.contains(*v))
        .cloned()
        .collect();
    Ok((!missing.is_empty()).then_some((values, missing)))
}

/// The conditions of every `select()` among `attrs`, once each, in the order
/// they are written.
fn declared_conditions(attrs: &[(String, AttrValue)]) -> Vec<Label> {
    let default = fjfj_graph::rule::default_condition();
    let mut out: Vec<Label> = Vec::new();
    for (_, value) in attrs {
        let AttrValue::Select(list) = value else {
            continue;
        };
        for selector in &list.elements {
            for (condition, _) in &selector.branches {
                if *condition != default && !out.contains(condition) {
                    out.push(condition.clone());
                }
            }
        }
    }
    out
}

/// The target of a source file.
fn source_file(key: &ConfiguredTargetKey, mut target: ConfiguredTarget) -> ConfiguredTarget {
    target.files = Arc::new(NestedSet::of(vec![Artifact::source(
        &key.label.repo,
        &key.label.package,
        &key.label.name,
    )]));
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
