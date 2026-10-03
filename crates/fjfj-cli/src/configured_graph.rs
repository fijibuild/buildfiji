//! The graph `cquery` and `aquery` run over (buildfiji-tle.2): configured
//! targets, one node for each target in each configuration it was analysed
//! in, with the edges analysis followed, so a dependency across a transition
//! is the target in the configuration the transition chose.
//!
//! The query evaluator keys its sets by [`Label`]. A configured target is
//! given a label of its own, the label of the target with the number of the
//! configured target after a `\u{1}` in its repository name; [`plain`] takes
//! it off again. Nothing outside this module sees the number.

use crate::query_graph::QueryGraph;
use fjfj_analysis::{ConfiguredTarget, ConfiguredTargetKey};
use fjfj_graph::{Configuration, Label};
use fjfj_query::{Edge, Graph, Node};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

const SEPARATOR: char = '\u{1}';

pub(crate) struct ConfiguredGraph<'a> {
    loading: &'a QueryGraph,
    targets: Vec<Arc<ConfiguredTarget>>,
    index: HashMap<ConfiguredTargetKey, usize>,
    by_label: BTreeMap<Label, Vec<usize>>,
    /// What each aspect made of a target, by the target.
    aspects: HashMap<ConfiguredTargetKey, Vec<Arc<ConfiguredTarget>>>,
    /// The configuration of the top-level targets: `config(x, target)`.
    top_level: Configuration,
    /// The checksum of each configuration the targets were analysed in.
    checksums: BTreeSet<String>,
}

/// The label a configured target answers to in a query.
fn synthetic(label: &Label, at: usize) -> Label {
    Label {
        repo: format!("{}{SEPARATOR}{at:08}", label.repo),
        package: label.package.clone(),
        name: label.name.clone(),
    }
}

/// The label of the target, without its configuration.
pub(crate) fn plain(label: &Label) -> Label {
    Label {
        repo: label
            .repo
            .split_once(SEPARATOR)
            .map_or(label.repo.as_str(), |(repo, _)| repo)
            .to_owned(),
        package: label.package.clone(),
        name: label.name.clone(),
    }
}

impl<'a> ConfiguredGraph<'a> {
    /// A source file has no configuration, so one node stands for it
    /// whichever configurations asked for it.
    pub(crate) fn new(
        loading: &'a QueryGraph,
        analysed: &[Arc<ConfiguredTarget>],
        top_level: &Configuration,
    ) -> ConfiguredGraph<'a> {
        // In label order, so a label of this graph sorts as its target does.
        let mut analysed: Vec<&Arc<ConfiguredTarget>> = analysed.iter().collect();
        analysed.sort_by_cached_key(|t| (t.label.clone(), t.configuration.checksum()));
        // What an aspect made is not a target of its own.
        let mut aspects: HashMap<ConfiguredTargetKey, Vec<Arc<ConfiguredTarget>>> = HashMap::new();
        analysed.retain(|t| match &t.aspect {
            Some(_) => {
                aspects
                    .entry(ConfiguredTargetKey {
                        label: t.label.clone(),
                        configuration: t.configuration.clone(),
                    })
                    .or_default()
                    .push((*t).clone());
                false
            }
            None => true,
        });
        for made in aspects.values_mut() {
            made.sort_by(|a, b| a.aspect.cmp(&b.aspect));
            made.dedup_by(|a, b| a.aspect == b.aspect);
        }
        let mut targets: Vec<Arc<ConfiguredTarget>> = Vec::new();
        let mut index = HashMap::new();
        let mut by_label: BTreeMap<Label, Vec<usize>> = BTreeMap::new();
        for target in analysed {
            let key = ConfiguredTargetKey {
                label: target.label.clone(),
                configuration: target.configuration.clone(),
            };
            if index.contains_key(&key) {
                continue;
            }
            let at = match (!target.has_configuration(), by_label.get(&target.label)) {
                (true, Some(existing)) => existing[0],
                _ => {
                    targets.push(target.clone());
                    let at = targets.len() - 1;
                    by_label.entry(target.label.clone()).or_default().push(at);
                    at
                }
            };
            index.insert(key, at);
        }
        let checksums = targets
            .iter()
            .filter(|t| t.has_configuration())
            .map(|t| t.configuration.checksum())
            .collect();
        ConfiguredGraph {
            loading,
            targets,
            index,
            by_label,
            aspects,
            top_level: top_level.clone(),
            checksums,
        }
    }

    /// The configured target a label of this graph stands for.
    pub(crate) fn target(&self, label: &Label) -> Option<&Arc<ConfiguredTarget>> {
        let (_, at) = label.repo.split_once(SEPARATOR)?;
        self.targets.get(at.parse::<usize>().ok()?)
    }

    /// A label as the user reads it, without a configuration.
    pub(crate) fn loading_display(&self, label: &Label) -> String {
        self.loading.display(label)
    }

    /// What aspects applied to `target` made, one value for each aspect.
    pub(crate) fn aspects_of(&self, target: &ConfiguredTarget) -> &[Arc<ConfiguredTarget>] {
        self.aspects
            .get(&ConfiguredTargetKey {
                label: target.label.clone(),
                configuration: target.configuration.clone(),
            })
            .map_or(&[], Vec::as_slice)
    }

    /// Every configured target of `label`.
    fn lift(&self, label: &Label) -> Vec<Label> {
        self.by_label
            .get(label)
            .into_iter()
            .flatten()
            .map(|&at| synthetic(label, at))
            .collect()
    }

    fn of_key(&self, key: &ConfiguredTargetKey) -> Option<Label> {
        let at = *self.index.get(key)?;
        Some(synthetic(&key.label, at))
    }
}

/// A dependency of a configured target as `cquery --transitions` shows it.
pub(crate) struct TransitionEdge {
    pub attr: String,
    /// The label the attribute names.
    pub label: Label,
    /// `(null transition)`, `NoTransition`, `(exec + ...)`, `(Starlark
    /// transition:...)`.
    pub description: String,
    /// The configuration the edge leads to, when it is another one.
    pub configuration: Option<Configuration>,
    /// What the transition changed, for `--transitions=full`: each option, its
    /// old value and its new.
    pub changes: Vec<(String, String, String)>,
}

/// What Bazel appends to the transition of an edge into another configuration.
const TRIMMING: &str = "(TestTrimmingTransition + ConfigFeatureFlagTaggedTrimmingTransition)";

impl ConfiguredGraph<'_> {
    /// The label attributes of `label`'s target and what each leads to: the
    /// attributes in the order of the class, those Bazel adds itself last.
    pub(crate) fn transition_edges(&self, label: &Label) -> Vec<TransitionEdge> {
        let Some(target) = self.target(label) else {
            return Vec::new();
        };
        let Ok(node) = self.loading.node(&plain(label)) else {
            return Vec::new();
        };
        let mut attrs: Vec<&fjfj_query::NodeAttr> =
            node.attrs.iter().filter(|a| !a.unset).collect();
        attrs.sort_by_key(|a| a.name.starts_with('$') || a.name.starts_with('_'));
        let mut out = Vec::new();
        for attr in attrs {
            let value = target
                .attrs
                .iter()
                .find(|(n, _)| *n == attr.name)
                .map_or(&attr.value, |(_, v)| v);
            let mut named: Vec<&Label> = Vec::new();
            value.labels(&mut named);
            for dep in named {
                let loaded = node
                    .edges
                    .iter()
                    .find(|e| e.attr == attr.name && e.to == *dep);
                let tool = loaded.is_some_and(|e| e.tool);
                let transition = loaded.is_some_and(|e| e.transition);
                // The configured target the edge goes to.
                let wanted = if tool {
                    target.configuration.to_exec()
                } else {
                    target.configuration.clone()
                };
                let candidates: Vec<&ConfiguredTargetKey> =
                    target.deps.iter().filter(|k| k.label == *dep).collect();
                let key = candidates
                    .iter()
                    .find(|k| {
                        if transition {
                            k.configuration != target.configuration
                        } else {
                            k.configuration == wanted
                        }
                    })
                    .or(candidates.first());
                let Some(reached) = key
                    .and_then(|k| self.index.get(*k))
                    .map(|&i| &self.targets[i])
                else {
                    continue;
                };
                let (description, configuration) = if !reached.has_configuration() {
                    ("(null transition)".to_owned(), None)
                } else if reached.configuration == target.configuration {
                    ("NoTransition".to_owned(), None)
                } else if tool || reached.configuration.exec {
                    (
                        format!("(exec + {TRIMMING})"),
                        Some(reached.configuration.clone()),
                    )
                } else if transition {
                    let location = target
                        .rule_info
                        .as_ref()
                        .and_then(|info| {
                            self.loading.transition_location(
                                &info.bzl,
                                &info.rule_class,
                                &attr.name,
                            )
                        })
                        .map(|l| format!(":{l}"))
                        .unwrap_or_default();
                    (
                        format!("(Starlark transition{location} + {TRIMMING})"),
                        Some(reached.configuration.clone()),
                    )
                } else {
                    (
                        "NoTransition".to_owned(),
                        Some(reached.configuration.clone()),
                    )
                };
                let changes = configuration
                    .as_ref()
                    .map(|new| option_changes(&target.configuration, new))
                    .unwrap_or_default();
                out.push(TransitionEdge {
                    attr: attr.name.clone(),
                    label: dep.clone(),
                    description,
                    configuration,
                    changes,
                });
            }
        }
        // Edges Bazel adds that the schema does not list: the script every
        // genrule sources, and the allowlist of the rules that transition.
        if matches!(&node.kind, fjfj_query::NodeKind::Rule { class, .. } if class == "genrule")
            && target.rule_info.is_none()
        {
            out.push(TransitionEdge {
                attr: "$genrule_setup".to_owned(),
                label: Label {
                    repo: "bazel_tools".to_owned(),
                    package: "tools/genrule".to_owned(),
                    name: "genrule-setup.sh".to_owned(),
                },
                description: "(null transition)".to_owned(),
                configuration: None,
                changes: Vec::new(),
            });
        }
        let transitions = target.rule_info.as_ref().is_some_and(|info| {
            info.schema.incoming_transition
                || info
                    .schema
                    .attrs
                    .iter()
                    .any(|a| a.def.cfg == fjfj_graph::rule::Cfg::Transition)
        });
        if transitions {
            out.push(TransitionEdge {
                attr: "$allowlist_function_transition".to_owned(),
                label: Label {
                    repo: "bazel_tools".to_owned(),
                    package: "tools/allowlists/function_transition_allowlist".to_owned(),
                    name: "function_transition_allowlist".to_owned(),
                },
                description: "(null transition)".to_owned(),
                configuration: None,
                changes: Vec::new(),
            });
        }
        out
    }
}

/// The options that differ between two configurations, each as its name, its
/// value in the first and in the second.
fn option_changes(from: &Configuration, to: &Configuration) -> Vec<(String, String, String)> {
    let (old, new) = (from.build_options(), to.build_options());
    let mut keys: Vec<&String> = old.keys().chain(new.keys()).collect();
    keys.sort();
    keys.dedup();
    let show = |v: Option<&fjfj_graph::SettingValue>| v.map_or("null".to_owned(), |v| v.java());
    // Only the native options: Bazel lists no user-defined build setting.
    keys.into_iter()
        .filter(|k| old.get(*k) != new.get(*k))
        .filter_map(|k| {
            let name = k.strip_prefix(fjfj_graph::config::COMMAND_LINE_OPTION)?;
            Some((name.to_owned(), show(old.get(k)), show(new.get(k))))
        })
        .collect()
}

impl Graph for ConfiguredGraph<'_> {
    fn pattern(&self, text: &str) -> Result<Vec<Label>, String> {
        Ok(self
            .loading
            .pattern(text)?
            .iter()
            .flat_map(|l| self.lift(l))
            .collect())
    }

    fn node(&self, label: &Label) -> Result<Arc<Node>, String> {
        let target = self
            .target(label)
            .ok_or_else(|| format!("no configured target {}", self.display(label)))?;
        let mut node = (*self.loading.node(&plain(label))?).clone();
        node.label = label.clone();
        // A generated file's rule is the target in its configuration.
        if let fjfj_query::NodeKind::GeneratedFile { rule } = &mut node.kind
            && let Some(made) = target
                .deps
                .iter()
                .find(|k| k.label == *rule)
                .and_then(|k| self.of_key(k))
        {
            *rule = made;
        }
        // The edges analysis followed, in the configuration each went to.
        let mut edges: Vec<Edge> = Vec::new();
        for key in &target.deps {
            let Some(to) = self.of_key(key) else { continue };
            if edges.iter().any(|e| e.to == to) {
                continue;
            }
            let loaded = node.edges.iter().find(|e| e.to == key.label);
            edges.push(Edge {
                to,
                implicit: loaded.is_some_and(|e| e.implicit),
                tool: loaded.is_some_and(|e| e.tool),
                condition: loaded.and_then(|e| e.condition.clone()),
                visibility: false,
                attr: loaded.map(|e| e.attr.clone()).unwrap_or_default(),
                transition: loaded.is_some_and(|e| e.transition),
            });
        }
        // The values the target has in this configuration: a `select()` is
        // the branch that was taken.
        for attr in &mut node.attrs {
            if let Some((_, decided)) = target.attrs.iter().find(|(n, _)| *n == attr.name)
                && attr.value != *decided
            {
                attr.value = decided.clone();
                attr.text = self.loading.attr_text(decided);
            }
        }
        for attr in &mut node.attrs {
            // A rule that did not say who may see it has no value here, where
            // `query` shows the package's default.
            if attr.name == "visibility" && !attr.explicit {
                attr.value = fjfj_graph::rule::AttrValue::StringList(Vec::new());
                attr.text = "[]".to_owned();
            }
        }
        for attr in &mut node.attrs {
            attr.labels = attr
                .labels
                .iter()
                .filter_map(|l| {
                    target
                        .deps
                        .iter()
                        .find(|k| k.label == *l)
                        .and_then(|k| self.of_key(k))
                })
                .collect();
        }
        node.outputs = node.outputs.iter().flat_map(|l| self.lift(l)).collect();
        node.edges = edges;
        Ok(Arc::new(node))
    }

    fn declared_node(&self, label: &Label) -> Result<Arc<Node>, String> {
        self.loading.node(&plain(label))
    }

    fn siblings(&self, label: &Label) -> Result<Vec<Label>, String> {
        Ok(self
            .loading
            .siblings(&plain(label))?
            .iter()
            .flat_map(|l| self.lift(l))
            .collect())
    }

    fn build_files(&self, label: &Label) -> Result<Vec<Label>, String> {
        Ok(self
            .loading
            .build_files(&plain(label))?
            .iter()
            .flat_map(|l| self.lift(l))
            .collect())
    }

    fn load_files(&self, label: &Label) -> Result<Vec<Label>, String> {
        Ok(self
            .loading
            .load_files(&plain(label))?
            .iter()
            .flat_map(|l| self.lift(l))
            .collect())
    }

    fn visible(&self, from: &Label, to: &Label) -> Result<bool, String> {
        self.loading.visible(&plain(from), &plain(to))
    }

    fn display(&self, label: &Label) -> String {
        self.loading.display(&plain(label))
    }

    fn is_configuration(&self, name: &str) -> bool {
        name == "target"
            || name == "null"
            || (!name.is_empty()
                && self
                    .checksums
                    .iter()
                    .filter(|c| c.starts_with(name))
                    .count()
                    == 1)
    }

    fn in_configuration(&self, label: &Label, name: &str) -> bool {
        let Some(target) = self.target(label) else {
            return false;
        };
        let configured = target.has_configuration();
        match name {
            // A file has no configuration, so it is in any that is asked for.
            "target" => !configured || target.configuration == self.top_level,
            "null" => !configured,
            checksum => configured && target.configuration.checksum().starts_with(checksum),
        }
    }

    fn relative_locations(&self) -> bool {
        self.loading.relative_locations()
    }

    fn sorts_edges(&self) -> bool {
        true
    }

    /// `//a:b (a7a71fd)`: the target and the first digits of its
    /// configuration, `(null)` for a file, which has none.
    fn output_name(&self, label: &Label) -> String {
        let configuration = match self.target(label) {
            Some(t) if t.has_configuration() => t.configuration.checksum()[..7].to_owned(),
            _ => "null".to_owned(),
        };
        format!("{} ({configuration})", self.display(label))
    }
}
