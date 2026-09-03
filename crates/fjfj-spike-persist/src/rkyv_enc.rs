//! Candidate B: rkyv 0.8 zero-copy archive of the same interned model,
//! measured raw (mmap-able, validated and unchecked access) and zstd-wrapped.

use anyhow::{Context, Result};
use rkyv::rancor::Error;
use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize};

use crate::graph::{Graph, Kind, Node, Strings};

#[derive(Archive, Serialize, Deserialize)]
pub struct RNode {
    kind: u8,
    key: Vec<u32>,
    value: Vec<u32>,
    digest: Option<[u8; 32]>,
    deps: Vec<u32>,
}

#[derive(Archive, Serialize, Deserialize)]
pub struct RGraph {
    strings: Vec<String>,
    nodes: Vec<RNode>,
}

pub fn to_rgraph(g: &Graph) -> RGraph {
    RGraph {
        strings: g.strings.table.clone(),
        nodes: g
            .nodes
            .iter()
            .map(|n| RNode {
                kind: n.kind as u8,
                key: n.key.clone(),
                value: n.value.clone(),
                digest: n.digest,
                deps: n.deps.clone(),
            })
            .collect(),
    }
}

pub fn encode(g: &Graph) -> Result<AlignedVec> {
    rkyv::to_bytes::<Error>(&to_rgraph(g)).context("rkyv serialize")
}

/// Validated zero-copy access (what a daemon would do on an untrusted file).
pub fn access_checked(bytes: &[u8]) -> Result<&ArchivedRGraph> {
    rkyv::access::<ArchivedRGraph, Error>(bytes).context("rkyv access")
}

/// Full scan over the archive without materialising anything.
pub fn scan(a: &ArchivedRGraph) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for n in a.nodes.iter() {
        h = h.wrapping_mul(0x100_0000_01b3) ^ u64::from(n.kind);
        for d in n.deps.iter() {
            h = h.wrapping_mul(0x100_0000_01b3) ^ u64::from(d.to_native());
        }
    }
    h
}

/// Materialise into the owned graph, for the eager-load comparison.
pub fn materialise(a: &ArchivedRGraph) -> Result<Graph> {
    let r: RGraph = rkyv::deserialize::<RGraph, Error>(a).context("rkyv deserialize")?;
    let nodes = r
        .nodes
        .into_iter()
        .map(|n| Node {
            kind: Kind::from_u8(n.kind).expect("kind"),
            key: n.key,
            value: n.value,
            digest: n.digest,
            deps: n.deps,
        })
        .collect();
    Ok(Graph {
        strings: Strings::from_table(r.strings),
        nodes,
    })
}

pub fn zstd_wrap(bytes: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::bulk::compress(bytes, level).context("zstd")
}

pub fn zstd_unwrap(z: &[u8]) -> Result<AlignedVec> {
    let raw = zstd::bulk::decompress(z, 1 << 30).context("zstd")?;
    let mut v = AlignedVec::with_capacity(raw.len());
    v.extend_from_slice(&raw);
    Ok(v)
}
