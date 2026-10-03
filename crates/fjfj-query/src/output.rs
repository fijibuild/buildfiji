//! The output formats of `query`.

use crate::eval::{Evaluator, Set};
use crate::graph::{Edge, NodeKind};
use fjfj_graph::Label;
use fjfj_graph::rule::AttrValue;
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
    Xml,
    /// `QueryResult` in the wire format.
    Proto,
    /// Each target as a length-prefixed `Target`.
    StreamedProto,
    /// Each target as one line of JSON.
    StreamedJsonProto,
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
            "xml" => Format::Xml,
            "proto" => Format::Proto,
            "streamed_proto" => Format::StreamedProto,
            "streamed_jsonproto" => Format::StreamedJsonProto,
            _ => return None,
        })
    }

    /// Whether the output is bytes rather than text.
    pub fn is_binary(self) -> bool {
        matches!(self, Format::Proto | Format::StreamedProto)
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

/// What `bazel query --output=<format>` prints for `set`, as bytes: the text
/// of `render` for a text format, a `QueryResult` or a stream of `Target`s
/// for the protocol buffer ones.
pub fn render_bytes(
    ev: &Evaluator<'_>,
    set: &Set,
    format: Format,
    wanted: Order,
    terminator: char,
    proto: &crate::target_proto::ProtoOptions,
) -> Result<Vec<u8>, String> {
    let targets = |ev: &Evaluator<'_>| -> Result<Vec<crate::proto::Msg>, String> {
        order(ev, set, wanted)?
            .iter()
            .map(|l| crate::target_proto::target(ev, l, proto))
            .collect()
    };
    Ok(match format {
        Format::Proto => crate::proto::Msg::new()
            .many(1, "target", targets(ev)?)
            .binary(),
        Format::StreamedProto => targets(ev)?.iter().flat_map(|t| t.delimited()).collect(),
        Format::StreamedJsonProto => {
            let mut out = String::new();
            for target in targets(ev)? {
                out.push_str(&target.json_compact());
                out.push('\n');
            }
            out.into_bytes()
        }
        _ => render(ev, set, format, wanted, terminator)?.into_bytes(),
    })
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
                line(graph.output_name(&label));
            }
        }
        Format::LabelKind => {
            for label in order(ev, set, wanted)? {
                line(format!(
                    "{} {}",
                    ev.node(&label)?.kind.description(),
                    graph.output_name(&label)
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
                    graph.output_name(&label)
                ));
            }
        }
        Format::MinRank | Format::MaxRank => {
            let sub = subgraph(ev, set)?;
            let rank = ranks(&sub, format == Format::MaxRank);
            let mut rows: Vec<(usize, &Label)> = rank.iter().map(|(l, r)| (*r, l)).collect();
            rows.sort();
            for (r, label) in rows {
                line(format!("{r} {}", graph.output_name(label)));
            }
        }
        Format::Proto | Format::StreamedProto | Format::StreamedJsonProto => {
            let proto = crate::target_proto::ProtoOptions::default();
            return render_bytes(ev, set, format, wanted, terminator, &proto).and_then(|bytes| {
                String::from_utf8(bytes).map_err(|_| "the output is not text".to_owned())
            });
        }
        Format::Xml => {
            line("<?xml version=\"1.1\" encoding=\"UTF-8\" standalone=\"no\"?>".to_owned());
            line("<query version=\"2\">".to_owned());
            for label in order(ev, set, wanted)? {
                for text in xml_element(ev, &label)? {
                    line(text);
                }
            }
            line("</query>".to_owned());
        }
        Format::Graph => {
            let sub = subgraph(ev, set)?;
            line("digraph mygraph {".to_owned());
            line("  node [shape=box];".to_owned());
            for label in dependency_order(&sub) {
                line(format!("  \"{}\"", graph.output_name(&label)));
                let mut edges: Vec<&Edge> = sub[&label].iter().collect();
                if graph.sorts_edges() {
                    edges.sort_by_key(|e| graph.output_name(&e.to));
                }
                for edge in edges {
                    line(format!(
                        "  \"{}\" -> \"{}\"",
                        graph.output_name(&label),
                        graph.output_name(&edge.to)
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

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            c => out.push(c),
        }
    }
    out
}

/// The lines of one target in `--output=xml`.
fn xml_element(ev: &Evaluator<'_>, label: &Label) -> Result<Vec<String>, String> {
    let graph = ev.graph();
    let node = ev.node(label)?;
    let name = escape(&graph.output_name(label));
    let location = escape(&node.location);
    let mut out = Vec::new();
    match &node.kind {
        NodeKind::SourceFile => {
            let errors = if node.build_file {
                " package_contains_errors=\"false\""
            } else {
                ""
            };
            out.push(format!(
                "    <source-file location=\"{location}\" name=\"{name}\"{errors}>"
            ));
            let mut loads: Vec<&Label> = node.loads.iter().collect();
            loads.sort();
            for l in loads {
                out.push(format!(
                    "        <load name=\"{}\"/>",
                    escape(&graph.output_name(l))
                ));
            }
            for v in &node.visibility {
                out.push(format!(
                    "        <visibility-label name=\"{}\"/>",
                    escape(v)
                ));
            }
            out.push("    </source-file>".to_owned());
        }
        NodeKind::GeneratedFile { rule } => {
            out.push(format!(
                "    <generated-file generating-rule=\"{}\" location=\"{location}\" name=\"{name}\"/>",
                escape(&graph.output_name(rule))
            ));
        }
        NodeKind::PackageGroup => {
            out.push(format!(
                "    <package-group location=\"{location}\" name=\"{name}\">"
            ));
            let (includes, packages) = node.group.clone().unwrap_or_default();
            if includes.is_empty() {
                out.push("        <list name=\"includes\"/>".to_owned());
            } else {
                out.push("        <list name=\"includes\">".to_owned());
                for i in &includes {
                    out.push(format!(
                        "            <label value=\"{}\"/>",
                        escape(&graph.output_name(i))
                    ));
                }
                out.push("        </list>".to_owned());
            }
            if packages.is_empty() {
                out.push("        <list name=\"packages\"/>".to_owned());
            } else {
                out.push("        <list name=\"packages\">".to_owned());
                for p in &packages {
                    out.push(format!("            <string value=\"{}\"/>", escape(p)));
                }
                out.push("        </list>".to_owned());
            }
            out.push("    </package-group>".to_owned());
        }
        NodeKind::Rule { class, .. } => {
            out.push(format!(
                "    <rule class=\"{}\" location=\"{location}\" name=\"{name}\">",
                escape(class)
            ));
            for attr in &node.attrs {
                if attr.explicit || attr.name == "name" {
                    xml_attr(graph, attr, &mut out);
                }
            }
            let inputs: BTreeSet<Label> = ev
                .edges(&node)
                .into_iter()
                .filter(|e| !e.visibility)
                .map(|e| e.to)
                .collect();
            for to in &inputs {
                out.push(format!(
                    "        <rule-input name=\"{}\"/>",
                    escape(&graph.output_name(to))
                ));
            }
            for o in &node.outputs {
                out.push(format!(
                    "        <rule-output name=\"{}\"/>",
                    escape(&graph.output_name(o))
                ));
            }
            out.push("    </rule>".to_owned());
        }
    }
    Ok(out)
}

/// Every value a `select()` could give, joined, which is what `--output=xml`
/// shows of it.
pub(crate) fn flatten(value: &AttrValue) -> AttrValue {
    let AttrValue::Select(list) = value else {
        return value.clone();
    };
    let mut joined: Option<AttrValue> = None;
    for selector in &list.elements {
        for (_, branch) in &selector.branches {
            let Some(branch) = branch else { continue };
            let branch = flatten(branch);
            joined = Some(match joined {
                None => branch,
                Some(so_far) => AttrValue::concat(&so_far, &branch).unwrap_or(so_far),
            });
        }
    }
    joined.unwrap_or(AttrValue::StringList(Vec::new()))
}

fn xml_attr(graph: &dyn crate::graph::Graph, attr: &crate::graph::NodeAttr, out: &mut Vec<String>) {
    let name = escape(&attr.name);
    let label = |l: &Label| escape(&graph.output_name(l));
    let is_output = matches!(
        attr.ty,
        fjfj_graph::rule::AttrType::Output | fjfj_graph::rule::AttrType::OutputList
    );
    let item = |l: &Label| {
        let tag = if is_output { "output" } else { "label" };
        format!("<{tag} value=\"{}\"/>", label(l))
    };
    let pad = "        ";
    match flatten(&attr.value) {
        AttrValue::Bool(b) => out.push(format!("{pad}<boolean name=\"{name}\" value=\"{b}\"/>")),
        AttrValue::Int(i) => out.push(format!("{pad}<int name=\"{name}\" value=\"{i}\"/>")),
        AttrValue::String(s) => out.push(format!(
            "{pad}<string name=\"{name}\" value=\"{}\"/>",
            escape(&s)
        )),
        AttrValue::Label(l) => {
            let tag = if is_output { "output" } else { "label" };
            out.push(format!(
                "{pad}<{tag} name=\"{name}\" value=\"{}\"/>",
                label(&l)
            ));
        }
        AttrValue::StringList(items) => {
            list(
                out,
                &name,
                items.iter().map(|s| {
                    if attr.ty == fjfj_graph::rule::AttrType::LabelList {
                        format!("<label value=\"{}\"/>", escape(s))
                    } else {
                        format!("<string value=\"{}\"/>", escape(s))
                    }
                }),
            );
        }
        AttrValue::IntList(items) => {
            list(
                out,
                &name,
                items.iter().map(|i| format!("<int value=\"{i}\"/>")),
            );
        }
        AttrValue::LabelList(items) => list(out, &name, items.iter().map(item)),
        AttrValue::StringDict(items) => dict(
            out,
            &name,
            items.iter().map(|(k, v)| {
                (
                    format!("<string value=\"{}\"/>", escape(k)),
                    format!("<string value=\"{}\"/>", escape(v)),
                )
            }),
        ),
        AttrValue::LabelKeyedStringDict(items) => dict(
            out,
            &name,
            items.iter().map(|(k, v)| {
                (
                    format!("<label value=\"{}\"/>", label(k)),
                    format!("<string value=\"{}\"/>", escape(v)),
                )
            }),
        ),
        AttrValue::StringKeyedLabelDict(items) => dict(
            out,
            &name,
            items.iter().map(|(k, v)| {
                (
                    format!("<string value=\"{}\"/>", escape(k)),
                    format!("<label value=\"{}\"/>", label(v)),
                )
            }),
        ),
        AttrValue::StringListDict(items) => dict(
            out,
            &name,
            items.iter().map(|(k, v)| {
                (
                    format!("<string value=\"{}\"/>", escape(k)),
                    format!(
                        "<list>{}</list>",
                        v.iter()
                            .map(|s| format!("<string value=\"{}\"/>", escape(s)))
                            .collect::<String>()
                    ),
                )
            }),
        ),
        AttrValue::LabelListDict(items) => dict(
            out,
            &name,
            items.iter().map(|(k, v)| {
                (
                    format!("<string value=\"{}\"/>", escape(k)),
                    format!(
                        "<list>{}</list>",
                        v.iter()
                            .map(|l| format!("<label value=\"{}\"/>", label(l)))
                            .collect::<String>()
                    ),
                )
            }),
        ),
        AttrValue::Select(_) => {}
    }
}

fn list(out: &mut Vec<String>, name: &str, items: impl Iterator<Item = String>) {
    let items: Vec<String> = items.collect();
    if items.is_empty() {
        out.push(format!("        <list name=\"{name}\"/>"));
        return;
    }
    out.push(format!("        <list name=\"{name}\">"));
    for i in items {
        out.push(format!("            {i}"));
    }
    out.push("        </list>".to_owned());
}

fn dict(out: &mut Vec<String>, name: &str, pairs: impl Iterator<Item = (String, String)>) {
    let pairs: Vec<(String, String)> = pairs.collect();
    if pairs.is_empty() {
        out.push(format!("        <dict name=\"{name}\"/>"));
        return;
    }
    out.push(format!("        <dict name=\"{name}\">"));
    for (k, v) in pairs {
        out.push("            <pair>".to_owned());
        out.push(format!("                {k}"));
        out.push(format!("                {v}"));
        out.push("            </pair>".to_owned());
    }
    out.push("        </dict>".to_owned());
}
