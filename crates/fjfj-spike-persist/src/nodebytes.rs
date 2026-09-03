//! One node as a self-contained byte string: the value format shared by the
//! KV-store candidates (redb, fjall), so they differ only in the store.

use anyhow::{Result, anyhow};

use crate::graph::{Kind, Node};
use crate::varint::{get_bytes, get_deltas, get_u32s, put_bytes, put_deltas, put_u32s, put_varint};

pub fn encode(n: &Node, out: &mut Vec<u8>) {
    out.push(n.kind as u8);
    put_u32s(out, &n.key);
    put_u32s(out, &n.value);
    match &n.digest {
        Some(d) => put_bytes(out, d),
        None => put_varint(out, 0),
    }
    put_deltas(out, &n.deps);
}

pub fn decode(buf: &[u8]) -> Result<Node> {
    let mut pos = 0usize;
    let bad = || anyhow!("truncated node");
    let kind = Kind::from_u8(*buf.first().ok_or_else(bad)?).ok_or_else(|| anyhow!("bad kind"))?;
    pos += 1;
    let key = get_u32s(buf, &mut pos).ok_or_else(bad)?;
    let value = get_u32s(buf, &mut pos).ok_or_else(bad)?;
    let d = get_bytes(buf, &mut pos).ok_or_else(bad)?;
    let digest = match d.len() {
        0 => None,
        32 => Some(d.try_into().expect("len checked")),
        _ => return Err(anyhow!("bad digest length")),
    };
    let deps = get_deltas(buf, &mut pos).ok_or_else(bad)?;
    Ok(Node {
        kind,
        key,
        value,
        digest,
        deps,
    })
}
