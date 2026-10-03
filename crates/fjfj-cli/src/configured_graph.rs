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
            let at = match (target.rule_class.is_none(), by_label.get(&target.label)) {
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
            .filter(|t| t.rule_class.is_some())
            .map(|t| t.configuration.checksum())
            .collect();
        ConfiguredGraph {
            loading,
            targets,
            index,
            by_label,
            top_level: top_level.clone(),
            checksums,
        }
    }

    /// The configured target a label of this graph stands for.
    pub(crate) fn target(&self, label: &Label) -> Option<&Arc<ConfiguredTarget>> {
        let (_, at) = label.repo.split_once(SEPARATOR)?;
        self.targets.get(at.parse::<usize>().ok()?)
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
            });
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
        let configured = target.rule_class.is_some();
        match name {
            // A file has no configuration, so it is in any that is asked for.
            "target" => !configured || target.configuration == self.top_level,
            "null" => !configured,
            checksum => configured && target.configuration.checksum().starts_with(checksum),
        }
    }

    fn sorts_edges(&self) -> bool {
        true
    }

    /// `//a:b (a7a71fd)`: the target and the first digits of its
    /// configuration, `(null)` for a file, which has none.
    fn output_name(&self, label: &Label) -> String {
        let configuration = match self.target(label) {
            Some(t) if t.rule_class.is_some() => t.configuration.checksum()[..7].to_owned(),
            _ => "null".to_owned(),
        };
        format!("{} ({configuration})", self.display(label))
    }
}
