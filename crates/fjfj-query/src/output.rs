//! The output formats of `query`.

use crate::eval::{Evaluator, Set};
use crate::graph::{Edge, NodeKind};
use fjfj_graph::Label;
use fjfj_graph::rule::AttrValue;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

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
    /// Each rule as the BUILD text that makes it.
    Build,
    /// The name of each package a target is in.
    Package,
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
            "build" => Format::Build,
            "package" => Format::Package,
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

/// The `--graph:` flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphOptions {
    /// `--[no]graph:factored` (default true): nodes with the same
    /// predecessors and successors are one node, their labels joined.
    pub factored: bool,
    /// `--graph:node_limit` (default 1000): a merged node shows labels up to
    /// this many characters and says how many more there are; -1 for all.
    pub node_limit: i64,
}

impl Default for GraphOptions {
    fn default() -> GraphOptions {
        GraphOptions {
            factored: true,
            node_limit: 1000,
        }
    }
}

impl GraphOptions {
    /// Read the flag `name` (without its dashes) with its `value` if it is a
    /// `--graph:` one. `Ok(false)` if it is not.
    pub fn flag(&mut self, name: &str, value: Option<&str>) -> Result<bool, String> {
        match name {
            "graph:factored" => self.factored = value != Some("false") && value != Some("0"),
            "nograph:factored" => self.factored = false,
            "graph:node_limit" => {
                let text = value.ok_or("--graph:node_limit needs a value")?;
                self.node_limit = text.parse().map_err(|_| {
                    format!(
                        "While parsing option --graph:node_limit={text}: '{text}' is not an int"
                    )
                })?;
            }
            _ => return Ok(false),
        }
        Ok(true)
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

/// The nodes of a graph after merging the equivalent ones.
struct Factored {
    /// The representative of each group, in dependency order.
    order: Vec<Label>,
    /// Each node's representative.
    of: BTreeMap<Label, Label>,
    /// The members of each group, by representative.
    groups: BTreeMap<Label, Vec<Label>>,
}

impl Factored {
    fn representative<'a>(&'a self, label: &'a Label) -> &'a Label {
        self.of.get(label).unwrap_or(label)
    }

    fn members(&self, label: &Label) -> &[Label] {
        self.groups.get(label).map_or(&[], Vec::as_slice)
    }
}

/// Merge the nodes of `sub` that have the same predecessors and successors,
/// if `factored`; the nodes in `order` otherwise.
fn factor(sub: &BTreeMap<Label, Vec<Edge>>, order: &[Label], factored: bool) -> Factored {
    let mut preds: BTreeMap<&Label, BTreeSet<&Label>> =
        sub.keys().map(|k| (k, BTreeSet::new())).collect();
    for (from, edges) in sub {
        for edge in edges {
            if let Some(p) = preds.get_mut(&edge.to) {
                p.insert(from);
            }
        }
    }
    let mut by_shape: BTreeMap<(BTreeSet<&Label>, BTreeSet<&Label>), Label> = BTreeMap::new();
    let mut out = Factored {
        order: Vec::new(),
        of: BTreeMap::new(),
        groups: BTreeMap::new(),
    };
    for label in order {
        let shape = (
            preds[label].clone(),
            sub[label].iter().map(|e| &e.to).collect::<BTreeSet<_>>(),
        );
        let rep = if factored {
            by_shape
                .entry(shape)
                .or_insert_with(|| label.clone())
                .clone()
        } else {
            label.clone()
        };
        if rep == *label {
            out.order.push(label.clone());
        }
        out.of.insert(label.clone(), rep.clone());
        out.groups.entry(rep).or_default().push(label.clone());
    }
    out
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
        _ => ev.listed(set),
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
    graph: &GraphOptions,
) -> Result<Vec<u8>, String> {
    let targets = |ev: &Evaluator<'_>| -> Result<Vec<crate::proto::Msg>, String> {
        let mut classes = std::collections::BTreeSet::new();
        order(ev, set, wanted)?
            .iter()
            .map(|l| crate::target_proto::target(ev, l, proto, &mut classes))
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
        _ => render_with(ev, set, format, wanted, terminator, graph)?.into_bytes(),
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
    render_with(
        ev,
        set,
        format,
        wanted,
        terminator,
        &GraphOptions::default(),
    )
}

/// [`render`] with the `--graph:` flags.
pub fn render_with(
    ev: &Evaluator<'_>,
    set: &Set,
    format: Format,
    wanted: Order,
    terminator: char,
    graph_options: &GraphOptions,
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
                    node.shown_location(graph),
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
        Format::Package => {
            // Each package once, by name: the main repository's without the
            // `//`, the others' as `@repo//pkg`.
            let mut packages: BTreeSet<String> = BTreeSet::new();
            for label in set {
                let shown = graph.display(label);
                let package = shown.rsplit_once(':').map_or(shown.as_str(), |(p, _)| p);
                packages.insert(package.strip_prefix("//").unwrap_or(package).to_owned());
            }
            for package in packages {
                line(package);
            }
        }
        Format::Build => {
            // Once for each name: `cquery` has a target in several configurations
            // and shows one of them.
            let mut done: BTreeSet<String> = BTreeSet::new();
            for label in order(ev, set, wanted)? {
                // A generated file is shown as the rule that makes it, once.
                let rule = match &ev.node(&label)?.kind {
                    NodeKind::Rule { .. } => label.clone(),
                    NodeKind::GeneratedFile { rule } => rule.clone(),
                    _ => continue,
                };
                if done.insert(graph.display(&rule)) {
                    out.push_str(&build_text(ev, &rule)?);
                }
            }
        }
        Format::Proto | Format::StreamedProto | Format::StreamedJsonProto => {
            let proto = crate::target_proto::ProtoOptions::default();
            let graph_options = GraphOptions::default();
            return render_bytes(ev, set, format, wanted, terminator, &proto, &graph_options)
                .and_then(|bytes| {
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
            let order = dependency_order(&sub);
            let nodes = factor(&sub, &order, graph_options.factored);
            let name_of = |label: &Label| -> String {
                let members = nodes.members(label);
                if members.len() == 1 {
                    return graph.output_name(&members[0]);
                }
                let mut names: Vec<String> = members.iter().map(|m| graph.output_name(m)).collect();
                // Bazel lists them from the last in order; when they do not all
                // fit it shows those that do, starting from the first.
                names.sort();
                let limit = graph_options.node_limit;
                let all = names
                    .iter()
                    .rev()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join("\\n");
                if limit < 0 || all.chars().count() <= limit as usize {
                    return all;
                }
                let mut shown: Vec<&String> = Vec::new();
                let mut length = 0usize;
                for name in &names {
                    if !shown.is_empty() && length + 1 + name.len() > limit as usize {
                        break;
                    }
                    length += name.len() + usize::from(!shown.is_empty());
                    shown.push(name);
                }
                let more = names.len() - shown.len();
                let mut text = shown
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<_>>()
                    .join("\\n");
                if more > 0 {
                    let _ = write!(text, "\\n...and {more} more items");
                }
                text
            };
            for label in &nodes.order {
                line(format!("  \"{}\"", name_of(label)));
                let mut edges: Vec<&Edge> = sub[label].iter().collect();
                if graph.sorts_edges() {
                    edges.sort_by_key(|e| graph.output_name(&e.to));
                }
                let mut seen: BTreeSet<&Label> = BTreeSet::new();
                for edge in edges {
                    let to = nodes.representative(&edge.to);
                    if !seen.insert(to) {
                        continue;
                    }
                    line(format!("  \"{}\" -> \"{}\"", name_of(label), name_of(to)));
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
    let location = escape(node.shown_location(graph));
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
        NodeKind::EnvironmentGroup => {
            out.push(format!(
                "    <environment-group location=\"{location}\" name=\"{name}\">"
            ));
            let (environments, defaults) = node.environment_group.clone().unwrap_or_default();
            for (list, labels) in [("environments", environments), ("defaults", defaults)] {
                if labels.is_empty() {
                    out.push(format!("        <list name=\"{list}\"/>"));
                    continue;
                }
                out.push(format!("        <list name=\"{list}\">"));
                for l in &labels {
                    out.push(format!(
                        "            <label value=\"{}\"/>",
                        escape(&graph.output_name(l))
                    ));
                }
                out.push("        </list>".to_owned());
            }
            out.push("    </environment-group>".to_owned());
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
            let edges: Vec<_> = ev
                .edges(&node)
                .into_iter()
                .filter(|e| !e.visibility)
                .collect();
            // The rule's own inputs in label order, then those of its
            // aspects in theirs; a label in both is listed twice, as Bazel
            // does.
            let mut inputs: Vec<Label> = edges
                .iter()
                .filter(|e| !e.aspect)
                .map(|e| e.to.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let mut seen = BTreeSet::new();
            for e in edges.iter().filter(|e| e.aspect) {
                if seen.insert(e.to.clone()) {
                    inputs.push(e.to.clone());
                }
            }
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

/// A string as Starlark's `repr` writes it.
fn repr(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// An attribute value as BUILD text.
fn build_value(graph: &dyn crate::graph::Graph, value: &AttrValue) -> String {
    let label = |l: &Label| repr(&graph.display(l));
    let list = |items: Vec<String>| format!("[{}]", items.join(", "));
    let dict = |items: Vec<String>| format!("{{{}}}", items.join(", "));
    match value {
        AttrValue::Bool(true) => "True".to_owned(),
        AttrValue::Bool(false) => "False".to_owned(),
        AttrValue::Int(i) => i.to_string(),
        AttrValue::String(s) => repr(s),
        AttrValue::Label(l) => label(l),
        AttrValue::StringList(items) => list(items.iter().map(|s| repr(s)).collect()),
        AttrValue::IntList(items) => list(items.iter().map(i32::to_string).collect()),
        AttrValue::LabelList(items) => list(items.iter().map(label).collect()),
        AttrValue::StringDict(items) => dict(
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", repr(k), repr(v)))
                .collect(),
        ),
        AttrValue::StringListDict(items) => dict(
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", repr(k), list(v.iter().map(|s| repr(s)).collect())))
                .collect(),
        ),
        AttrValue::LabelKeyedStringDict(items) => dict(
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", label(k), repr(v)))
                .collect(),
        ),
        AttrValue::StringKeyedLabelDict(items) => dict(
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", repr(k), label(v)))
                .collect(),
        ),
        AttrValue::LabelListDict(items) => dict(
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", repr(k), list(v.iter().map(label).collect())))
                .collect(),
        ),
        AttrValue::Select(selectors) => {
            let join = if selectors.pipe { " | " } else { " + " };
            selectors
                .elements
                .iter()
                .map(|selector| {
                    let branches: Vec<String> = selector
                        .branches
                        .iter()
                        .map(|(condition, branch)| {
                            let shown = match branch {
                                Some(v) => build_value(graph, v),
                                None => "None".to_owned(),
                            };
                            format!("{}: {shown}", label(condition))
                        })
                        .collect();
                    if selector.unconditional {
                        branches
                            .first()
                            .and_then(|b| b.split_once(": ").map(|(_, v)| v.to_owned()))
                            .unwrap_or_default()
                    } else {
                        format!("select({{{}}})", branches.join(", "))
                    }
                })
                .collect::<Vec<_>>()
                .join(join)
        }
    }
}

/// The calls of a stack as the lines of a comment, the functions lined up.
fn stack_lines(frames: &[crate::graph::Frame]) -> String {
    let width = frames
        .iter()
        .map(|f| f.location.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for frame in frames {
        let pad = " ".repeat(width - frame.location.chars().count());
        let _ = writeln!(out, "#   {}{pad} in {}", frame.location, frame.function);
    }
    out
}

/// The text `--output=build` gives a rule: where it is, the rule as written
/// with the attributes that were set, and where it was made and defined.
fn build_text(ev: &Evaluator<'_>, label: &Label) -> Result<String, String> {
    let graph = ev.graph();
    let node = ev.node(label)?;
    let NodeKind::Rule { class, .. } = &node.kind else {
        return Ok(String::new());
    };
    let mut out = String::new();
    let _ = writeln!(out, "# {}", node.location);
    let _ = writeln!(out, "{class}(");
    for attr in &node.attrs {
        if attr.unset || !(attr.explicit || attr.name == "name") {
            continue;
        }
        let value = if attr.name == "name" {
            repr(&label.name)
        } else {
            build_value(graph, &attr.value)
        };
        let _ = writeln!(out, "  {} = {value},", attr.name);
    }
    out.push_str(")\n");
    let _ = writeln!(
        out,
        "# Rule {} instantiated at (most recent call last):",
        label.name
    );
    out.push_str(&stack_lines(&node.stack));
    if !node.definition_stack.is_empty() {
        let _ = writeln!(out, "# Rule {class} defined at (most recent call last):");
        out.push_str(&stack_lines(&node.definition_stack));
    }
    out.push('\n');
    Ok(out)
}
