//! Build a [`Graph`] from `bazel aquery --output=jsonproto` and
//! `bazel cquery --output=jsonproto` dumps.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::graph::{Graph, Kind, Node, NodeId, sort_dedup};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AQuery {
    #[serde(default)]
    artifacts: Vec<Artifact>,
    #[serde(default)]
    actions: Vec<Action>,
    #[serde(default)]
    targets: Vec<Target>,
    #[serde(default)]
    dep_set_of_files: Vec<DepSet>,
    #[serde(default)]
    configuration: Vec<Configuration>,
    #[serde(default)]
    path_fragments: Vec<PathFragment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Artifact {
    id: u32,
    path_fragment_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Action {
    target_id: u32,
    #[serde(default)]
    mnemonic: String,
    #[serde(default)]
    configuration_id: u32,
    #[serde(default)]
    arguments: Vec<String>,
    #[serde(default)]
    environment_variables: Vec<KeyValue>,
    #[serde(default)]
    input_dep_set_ids: Vec<u32>,
    #[serde(default)]
    output_ids: Vec<u32>,
    #[serde(default)]
    execution_info: Vec<KeyValue>,
    #[serde(default)]
    primary_output_id: u32,
}

#[derive(Deserialize)]
struct KeyValue {
    key: String,
    #[serde(default)]
    value: String,
}

#[derive(Deserialize)]
struct Target {
    id: u32,
    label: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepSet {
    id: u32,
    #[serde(default)]
    direct_artifact_ids: Vec<u32>,
    #[serde(default)]
    transitive_dep_set_ids: Vec<u32>,
}

#[derive(Deserialize)]
struct Configuration {
    id: u32,
    #[serde(default)]
    checksum: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathFragment {
    id: u32,
    label: String,
    #[serde(default)]
    parent_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CQuery {
    #[serde(default)]
    results: Vec<CResult>,
    #[serde(default)]
    configurations: Vec<CConfiguration>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CResult {
    target: CTarget,
    #[serde(default)]
    configuration_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CTarget {
    #[serde(default)]
    rule: Option<Rule>,
    #[serde(default)]
    source_file: Option<Named>,
    #[serde(default)]
    generated_file: Option<Named>,
    #[serde(default)]
    package_group: Option<Named>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Rule {
    name: String,
    #[serde(default)]
    rule_class: String,
    #[serde(default)]
    attribute: Vec<Attribute>,
    #[serde(default)]
    rule_input: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Attribute {
    name: String,
    #[serde(default)]
    string_value: Option<String>,
    #[serde(default)]
    string_list_value: Vec<String>,
    #[serde(default)]
    explicitly_specified: bool,
}

#[derive(Deserialize)]
struct CConfiguration {
    id: u32,
    #[serde(default)]
    checksum: String,
}

pub struct LoadStats {
    pub configured_targets: usize,
    pub actions: usize,
    pub depsets: usize,
    pub files: usize,
}

pub fn load(aquery: &Path, cquery: &Path) -> Result<(Graph, LoadStats)> {
    let a: AQuery = serde_json::from_slice(&std::fs::read(aquery)?).context("parse aquery")?;
    let c: CQuery = serde_json::from_slice(&std::fs::read(cquery)?).context("parse cquery")?;
    let mut g = Graph::default();

    // Configured targets.
    let cfg_checksum: HashMap<u32, &str> = c
        .configurations
        .iter()
        .map(|x| (x.id, x.checksum.as_str()))
        .collect();
    let mut ct_by_label_cfg: HashMap<(String, u32), NodeId> = HashMap::new();
    let mut ct_by_label: HashMap<String, NodeId> = HashMap::new();
    let mut pending_inputs: Vec<(NodeId, u32, Vec<String>)> = Vec::new();
    for r in &c.results {
        let (label, mut value, inputs) = match (&r.target.rule, other_name(&r.target)) {
            (Some(rule), _) => {
                let mut v = vec![g.strings.intern(&rule.rule_class)];
                for at in rule.attribute.iter().filter(|at| at.explicitly_specified) {
                    v.push(g.strings.intern(&at.name));
                    if let Some(s) = &at.string_value {
                        v.push(g.strings.intern(s));
                    }
                    for s in &at.string_list_value {
                        v.push(g.strings.intern(s));
                    }
                }
                (rule.name.clone(), v, rule.rule_input.clone())
            }
            (None, Some(name)) => (name.to_owned(), Vec::new(), Vec::new()),
            (None, None) => continue,
        };
        value.shrink_to_fit();
        let checksum = cfg_checksum.get(&r.configuration_id).copied().unwrap_or("");
        let key = vec![g.strings.intern(&label), g.strings.intern(checksum)];
        let id = g.push(Node {
            kind: Kind::ConfiguredTarget,
            key,
            value,
            digest: None,
            deps: Vec::new(),
        });
        ct_by_label_cfg.insert((label.clone(), r.configuration_id), id);
        ct_by_label.entry(label).or_insert(id);
        if !inputs.is_empty() {
            pending_inputs.push((id, r.configuration_id, inputs));
        }
    }
    for (id, cfg, inputs) in pending_inputs {
        let deps = inputs
            .iter()
            .filter_map(|l| {
                ct_by_label_cfg
                    .get(&(l.clone(), cfg))
                    .or_else(|| ct_by_label.get(l))
                    .copied()
            })
            .collect();
        g.nodes[id as usize].deps = sort_dedup(deps);
    }
    let configured_targets = g.nodes.len();

    // Files.
    let frag: HashMap<u32, &PathFragment> = a.path_fragments.iter().map(|f| (f.id, f)).collect();
    let mut path_cache: HashMap<u32, String> = HashMap::new();
    let mut file_node: HashMap<u32, NodeId> = HashMap::new();
    for art in &a.artifacts {
        let path = resolve_path(art.path_fragment_id, &frag, &mut path_cache);
        let digest: [u8; 32] = Sha256::digest(path.as_bytes()).into();
        let key = vec![g.strings.intern(&path)];
        let id = g.push(Node {
            kind: Kind::File,
            key,
            value: Vec::new(),
            digest: Some(digest),
            deps: Vec::new(),
        });
        file_node.insert(art.id, id);
    }
    let files = g.nodes.len() - configured_targets;

    // Depsets: reserve ids first so transitive references resolve.
    let mut depset_node: HashMap<u32, NodeId> = HashMap::new();
    for ds in &a.dep_set_of_files {
        let id = g.push(Node {
            kind: Kind::Depset,
            key: Vec::new(),
            value: Vec::new(),
            digest: None,
            deps: Vec::new(),
        });
        depset_node.insert(ds.id, id);
    }
    for ds in &a.dep_set_of_files {
        let deps = ds
            .direct_artifact_ids
            .iter()
            .filter_map(|x| file_node.get(x).copied())
            .chain(
                ds.transitive_dep_set_ids
                    .iter()
                    .filter_map(|x| depset_node.get(x).copied()),
            )
            .collect();
        g.nodes[depset_node[&ds.id] as usize].deps = sort_dedup(deps);
    }
    let depsets = a.dep_set_of_files.len();

    // Actions.
    let target_label: HashMap<u32, &str> =
        a.targets.iter().map(|t| (t.id, t.label.as_str())).collect();
    let acfg: HashMap<u32, &str> = a
        .configuration
        .iter()
        .map(|x| (x.id, x.checksum.as_str()))
        .collect();
    for act in &a.actions {
        let owner = target_label.get(&act.target_id).copied().unwrap_or("");
        let primary = act
            .output_ids
            .iter()
            .find(|&&o| o == act.primary_output_id)
            .and_then(|o| file_node.get(o))
            .map(|&n| g.nodes[n as usize].key[0]);
        let mut key = vec![
            g.strings.intern(owner),
            g.strings
                .intern(acfg.get(&act.configuration_id).copied().unwrap_or("")),
            g.strings.intern(&act.mnemonic),
        ];
        key.extend(primary);
        let mut value =
            Vec::with_capacity(act.arguments.len() + 2 * act.environment_variables.len());
        for s in &act.arguments {
            value.push(g.strings.intern(s));
        }
        for kv in act.environment_variables.iter().chain(&act.execution_info) {
            value.push(g.strings.intern(&kv.key));
            value.push(g.strings.intern(&kv.value));
        }
        let owner_ct = ct_by_label.get(owner).copied();
        let deps = act
            .input_dep_set_ids
            .iter()
            .filter_map(|x| depset_node.get(x).copied())
            .chain(
                act.output_ids
                    .iter()
                    .filter_map(|x| file_node.get(x).copied()),
            )
            .chain(owner_ct)
            .collect();
        g.push(Node {
            kind: Kind::Action,
            key,
            value,
            digest: None,
            deps: sort_dedup(deps),
        });
    }
    let actions = a.actions.len();

    Ok((
        g,
        LoadStats {
            configured_targets,
            actions,
            depsets,
            files,
        },
    ))
}

fn other_name(t: &CTarget) -> Option<&str> {
    t.source_file
        .as_ref()
        .or(t.generated_file.as_ref())
        .or(t.package_group.as_ref())
        .map(|n| n.name.as_str())
}

fn resolve_path(
    id: u32,
    frag: &HashMap<u32, &PathFragment>,
    cache: &mut HashMap<u32, String>,
) -> String {
    if let Some(p) = cache.get(&id) {
        return p.clone();
    }
    let Some(f) = frag.get(&id) else {
        return String::new();
    };
    let path = if f.parent_id == 0 {
        f.label.clone()
    } else {
        let mut p = resolve_path(f.parent_id, frag, cache);
        p.push('/');
        p.push_str(&f.label);
        p
    };
    cache.insert(id, path.clone());
    path
}
