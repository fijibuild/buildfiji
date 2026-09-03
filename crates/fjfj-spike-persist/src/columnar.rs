//! Candidate A: hand-rolled columnar snapshot. Sections (strings, edge
//! lists, node columns) are each one zstd frame; edge lists are
//! delta+varint and deduplicated by content; everything is decoded eagerly.

use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};

use crate::graph::{Graph, Kind, Node, Strings};
use crate::varint::{
    get_bytes, get_deltas, get_u32s, get_varint, put_bytes, put_deltas, put_u32s, put_varint,
};

const MAGIC: &[u8; 8] = b"FJFJSNP1";

pub struct Encoded {
    pub bytes: Vec<u8>,
    pub raw_len: usize,
    pub section_raw: [usize; 3],
    pub section_zstd: [usize; 3],
    pub unique_edge_lists: usize,
}

pub fn encode(g: &Graph, level: i32) -> Result<Encoded> {
    // Section 0: strings.
    let mut strings = Vec::new();
    put_varint(&mut strings, g.strings.table.len() as u64);
    for s in &g.strings.table {
        put_bytes(&mut strings, s.as_bytes());
    }

    // Section 1: deduplicated edge lists.
    let mut edge_index: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut edge_lists: Vec<Vec<u8>> = Vec::new();
    let mut node_edge_ref: Vec<u32> = Vec::with_capacity(g.nodes.len());
    for n in &g.nodes {
        let mut b = Vec::new();
        put_deltas(&mut b, &n.deps);
        let id = *edge_index.entry(b).or_insert_with_key(|k| {
            edge_lists.push(k.clone());
            (edge_lists.len() - 1) as u32
        });
        node_edge_ref.push(id);
    }
    let mut edges = Vec::new();
    put_varint(&mut edges, edge_lists.len() as u64);
    for e in &edge_lists {
        put_bytes(&mut edges, e);
    }

    // Section 2: node columns.
    let mut nodes = Vec::new();
    put_varint(&mut nodes, g.nodes.len() as u64);
    for n in &g.nodes {
        nodes.push(n.kind as u8);
    }
    for n in &g.nodes {
        put_u32s(&mut nodes, &n.key);
    }
    for n in &g.nodes {
        put_u32s(&mut nodes, &n.value);
    }
    for n in &g.nodes {
        match &n.digest {
            Some(d) => {
                nodes.push(1);
                nodes.extend_from_slice(d);
            }
            None => nodes.push(0),
        }
    }
    for &r in &node_edge_ref {
        put_varint(&mut nodes, u64::from(r));
    }

    let sections = [strings, edges, nodes];
    let section_raw = [sections[0].len(), sections[1].len(), sections[2].len()];
    let mut bytes = MAGIC.to_vec();
    let mut section_zstd = [0usize; 3];
    for (i, s) in sections.iter().enumerate() {
        let z = zstd::bulk::compress(s, level).context("zstd")?;
        section_zstd[i] = z.len();
        put_bytes(&mut bytes, &z);
    }
    Ok(Encoded {
        raw_len: section_raw.iter().sum(),
        bytes,
        section_raw,
        section_zstd,
        unique_edge_lists: edge_lists.len(),
    })
}

pub fn decode(bytes: &[u8]) -> Result<Graph> {
    if bytes.get(..8) != Some(MAGIC.as_slice()) {
        return Err(anyhow!("bad magic"));
    }
    let mut pos = 8usize;
    let bad = || anyhow!("truncated snapshot");
    let mut sections = Vec::with_capacity(3);
    for _ in 0..3 {
        let z = get_bytes(bytes, &mut pos).ok_or_else(bad)?;
        sections.push(zstd::bulk::decompress(z, 1 << 30).context("zstd")?);
    }

    let s = &sections[0];
    let mut p = 0usize;
    let n = get_varint(s, &mut p).ok_or_else(bad)? as usize;
    let mut table = Vec::with_capacity(n);
    for _ in 0..n {
        let b = get_bytes(s, &mut p).ok_or_else(bad)?;
        table.push(std::str::from_utf8(b)?.to_owned());
    }

    let e = &sections[1];
    p = 0;
    let n = get_varint(e, &mut p).ok_or_else(bad)? as usize;
    let mut edge_lists: Vec<Vec<u32>> = Vec::with_capacity(n);
    for _ in 0..n {
        let b = get_bytes(e, &mut p).ok_or_else(bad)?;
        let mut q = 0usize;
        edge_lists.push(get_deltas(b, &mut q).ok_or_else(bad)?);
    }

    let c = &sections[2];
    p = 0;
    let n = get_varint(c, &mut p).ok_or_else(bad)? as usize;
    let kinds = c.get(p..p + n).ok_or_else(bad)?.to_vec();
    p += n;
    let mut keys = Vec::with_capacity(n);
    for _ in 0..n {
        keys.push(get_u32s(c, &mut p).ok_or_else(bad)?);
    }
    let mut values = Vec::with_capacity(n);
    for _ in 0..n {
        values.push(get_u32s(c, &mut p).ok_or_else(bad)?);
    }
    let mut digests = Vec::with_capacity(n);
    for _ in 0..n {
        let flag = *c.get(p).ok_or_else(bad)?;
        p += 1;
        digests.push(if flag == 1 {
            let d: [u8; 32] = c.get(p..p + 32).ok_or_else(bad)?.try_into().expect("len");
            p += 32;
            Some(d)
        } else {
            None
        });
    }
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let r = get_varint(c, &mut p).ok_or_else(bad)? as usize;
        nodes.push(Node {
            kind: Kind::from_u8(kinds[i]).ok_or_else(|| anyhow!("bad kind"))?,
            key: std::mem::take(&mut keys[i]),
            value: std::mem::take(&mut values[i]),
            digest: digests[i],
            deps: edge_lists.get(r).ok_or_else(bad)?.clone(),
        });
    }
    Ok(Graph {
        strings: Strings::from_table(table),
        nodes,
    })
}
