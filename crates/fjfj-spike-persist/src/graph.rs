//! Synthetic engine graph: an interned, immutable stand-in for the node
//! shapes `fjfj-engine` will persist (configured targets, actions, depsets,
//! files). Every string is a `u32` id into one table; every dependency is a
//! `u32` node id.

use std::collections::HashMap;

pub type StrId = u32;
pub type NodeId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    ConfiguredTarget = 0,
    Action = 1,
    Depset = 2,
    File = 3,
}

impl Kind {
    pub fn from_u8(v: u8) -> Option<Kind> {
        Some(match v {
            0 => Kind::ConfiguredTarget,
            1 => Kind::Action,
            2 => Kind::Depset,
            3 => Kind::File,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: Kind,
    /// Identity: interned strings (label + config, path, ...).
    pub key: Vec<StrId>,
    /// Value payload: interned strings (rule class, attributes, argv, env).
    pub value: Vec<StrId>,
    /// Content digest for file nodes.
    pub digest: Option<[u8; 32]>,
    /// Sorted, deduplicated dependency node ids.
    pub deps: Vec<NodeId>,
}

#[derive(Default, Debug)]
pub struct Strings {
    pub table: Vec<String>,
    index: HashMap<String, StrId>,
}

impl Strings {
    pub fn intern(&mut self, s: &str) -> StrId {
        if let Some(&id) = self.index.get(s) {
            return id;
        }
        let id = StrId::try_from(self.table.len()).expect("string table overflow");
        self.table.push(s.to_owned());
        self.index.insert(s.to_owned(), id);
        id
    }

    pub fn from_table(table: Vec<String>) -> Strings {
        let index = table
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i as StrId))
            .collect();
        Strings { table, index }
    }
}

#[derive(Default, Debug)]
pub struct Graph {
    pub strings: Strings,
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId::try_from(self.nodes.len()).expect("node id overflow");
        self.nodes.push(node);
        id
    }

    pub fn edge_count(&self) -> usize {
        self.nodes.iter().map(|n| n.deps.len()).sum()
    }

    pub fn string_bytes(&self) -> usize {
        self.strings.table.iter().map(String::len).sum()
    }

    /// Structural equality ignoring the string index map.
    pub fn same_as(&self, other: &Graph) -> bool {
        self.strings.table == other.strings.table && self.nodes == other.nodes
    }

    /// A cheap whole-graph checksum used to make full scans observable.
    pub fn scan_checksum(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for n in &self.nodes {
            h = h.wrapping_mul(0x100_0000_01b3) ^ (n.kind as u64);
            for &d in &n.deps {
                h = h.wrapping_mul(0x100_0000_01b3) ^ u64::from(d);
            }
        }
        h
    }
}

pub fn sort_dedup(mut v: Vec<NodeId>) -> Vec<NodeId> {
    v.sort_unstable();
    v.dedup();
    v
}
