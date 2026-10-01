//! The output formats of `query`.

use crate::eval::{Evaluator, Set};
use crate::graph::Edge;
use fjfj_graph::Label;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// `--output`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Label,
    LabelKind,
    Location,
    MinRank,
    MaxRank,
    Graph,
}

impl Format {
    pub fn parse(name: &str) -> Option<Format> {
        Some(match name {
            "label" => Format::Label,
            "label_kind" => Format::LabelKind,
            "location" => Format::Location,
            "minrank" => Format::MinRank,
            "maxrank" => Format::MaxRank,
            "graph" => Format::Graph,
            _ => return None,
        })
    }
}

/// `--order_output`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Order {
    /// Label order, which is what Bazel 9.2.0 prints by default.
    #[default]
    Auto,
    /// Dependency order: a target before what it depends on.
    Full,
    /// No order promised; fjfj gives label order.
    No,
}

impl Order {
    pub fn parse(name: &str) -> Option<Order> {
        Some(match name {
            "auto" => Order::Auto,
            "full" => Order::Full,
            "no" => Order::No,
            _ => return None,
        })
    }
}

/// The edges of the subgraph `set`, in the order Bazel visits them.
fn subgraph(ev: &Evaluator<'_>, set: &Set) -> Result<BTreeMap<Label, Vec<Edge>>, String> {
    let mut out = BTreeMap::new();
    for label in set {
        let node = ev.node(label)?;
        let edges = ev
            .edges(&node)
            .into_iter()
            .filter(|e| set.contains(&e.to))
            .collect();
        out.insert(label.clone(), edges);
    }
    Ok(out)
}

/// The targets of `set` before whatever they depend on: the reverse of the
/// postorder of a depth-first walk that starts from each target in label
/// order and follows edges in the order Bazel lists them. Probed on 9.2.0.
fn dependency_order(graph: &BTreeMap<Label, Vec<Edge>>) -> Vec<Label> {
    let mut visited: BTreeSet<&Label> = BTreeSet::new();
    let mut post: Vec<&Label> = Vec::with_capacity(graph.len());
    for start in graph.keys() {
        if !visited.insert(start) {
            continue;
        }
        let mut stack: Vec<(&Label, usize)> = vec![(start, 0)];
        while let Some((label, next)) = stack.pop() {
            let edges = &graph[label];
            if next < edges.len() {
                stack.push((label, next + 1));
                let (key, _) = graph.get_key_value(&edges[next].to).expect("in the set");
                if visited.insert(key) {
                    stack.push((key, 0));
                }
            } else {
                post.push(label);
            }
        }
    }
    post.into_iter().rev().cloned().collect()
}

fn order(ev: &Evaluator<'_>, set: &Set, order: Order) -> Result<Vec<Label>, String> {
    Ok(match order {
        Order::Full => dependency_order(&subgraph(ev, set)?),
        _ => set.iter().cloned().collect(),
    })
}

/// The ranks of the targets: how far each is from the targets nothing in the
/// set depends on, by the shortest way or the longest.
fn ranks(graph: &BTreeMap<Label, Vec<Edge>>, longest: bool) -> BTreeMap<Label, usize> {
    let mut incoming: BTreeMap<&Label, usize> = graph.keys().map(|k| (k, 0)).collect();
    for edges in graph.values() {
        for e in edges {
            *incoming.get_mut(&e.to).expect("in the set") += 1;
        }
    }
    let mut rank: BTreeMap<Label, usize> = BTreeMap::new();
    if longest {
        // Longest path in a DAG; a node in a cycle takes the rank it is
        // first reached at.
        let mut queue: VecDeque<&Label> = incoming
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(l, _)| *l)
            .collect();
        for l in &queue {
            rank.insert((*l).clone(), 0);
        }
        let mut remaining = incoming.clone();
        while let Some(label) = queue.pop_front() {
            let here = rank[label];
            for e in &graph[label] {
                let r = rank.entry(e.to.clone()).or_insert(0);
                *r = (*r).max(here + 1);
                let left = remaining.get_mut(&e.to).expect("in the set");
                *left -= 1;
                if *left == 0 {
                    queue.push_back(graph.get_key_value(&e.to).expect("in the set").0);
                }
            }
        }
    } else {
        let mut queue: VecDeque<&Label> = incoming
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(l, _)| *l)
            .collect();
        for l in &queue {
            rank.insert((*l).clone(), 0);
        }
        while let Some(label) = queue.pop_front() {
            let here = rank[label];
            for e in &graph[label] {
                if !rank.contains_key(&e.to) {
                    rank.insert(e.to.clone(), here + 1);
                    queue.push_back(graph.get_key_value(&e.to).expect("in the set").0);
                }
            }
        }
    }
    // Targets only in cycles.
    for label in graph.keys() {
        rank.entry(label.clone()).or_insert(0);
    }
    rank
}

/// The text `bazel query --output=<format>` prints for `set`.
pub fn render(
    ev: &Evaluator<'_>,
    set: &Set,
    format: Format,
    wanted: Order,
    terminator: char,
) -> Result<String, String> {
    let graph = ev.graph();
    let mut out = String::new();
    let mut line = |text: String| {
        out.push_str(&text);
        out.push(terminator);
    };
    match format {
        Format::Label => {
            for label in order(ev, set, wanted)? {
                line(graph.display(&label));
            }
        }
        Format::LabelKind => {
            for label in order(ev, set, wanted)? {
                line(format!(
                    "{} {}",
                    ev.node(&label)?.kind.description(),
                    graph.display(&label)
                ));
            }
        }
        Format::Location => {
            for label in order(ev, set, wanted)? {
                let node = ev.node(&label)?;
                line(format!(
                    "{}: {} {}",
                    node.location,
                    node.kind.description(),
                    graph.display(&label)
                ));
            }
        }
        Format::MinRank | Format::MaxRank => {
            let sub = subgraph(ev, set)?;
            let rank = ranks(&sub, format == Format::MaxRank);
            let mut rows: Vec<(usize, &Label)> = rank.iter().map(|(l, r)| (*r, l)).collect();
            rows.sort();
            for (r, label) in rows {
                line(format!("{r} {}", graph.display(label)));
            }
        }
        Format::Graph => {
            let sub = subgraph(ev, set)?;
            line("digraph mygraph {".to_owned());
            line("  node [shape=box];".to_owned());
            for label in dependency_order(&sub) {
                line(format!("  \"{}\"", graph.display(&label)));
                for edge in &sub[&label] {
                    line(format!(
                        "  \"{}\" -> \"{}\"",
                        graph.display(&label),
                        graph.display(&edge.to)
                    ));
                    if let Some(condition) = &edge.condition {
                        line(format!("  [label=\"{condition}\"];"));
                    }
                }
            }
            line("}".to_owned());
        }
    }
    Ok(out)
}
