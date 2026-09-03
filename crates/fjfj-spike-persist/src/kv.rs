//! Candidates C and D: one row per node in a pure-Rust KV store (redb
//! B-tree, fjall LSM), plus a strings table. Same node byte format for both.

use std::path::Path;

use anyhow::{Context, Result};
use redb::{ReadableDatabase, ReadableTable, TableDefinition};

use crate::graph::{Graph, Strings};
use crate::nodebytes;

const NODES: TableDefinition<u32, &[u8]> = TableDefinition::new("nodes");
const STRINGS: TableDefinition<u32, &str> = TableDefinition::new("strings");

pub fn redb_write(g: &Graph, path: &Path) -> Result<()> {
    let db = redb::Database::create(path).context("redb create")?;
    let txn = db.begin_write()?;
    {
        let mut nodes = txn.open_table(NODES)?;
        let mut buf = Vec::new();
        for (i, n) in g.nodes.iter().enumerate() {
            buf.clear();
            nodebytes::encode(n, &mut buf);
            nodes.insert(i as u32, buf.as_slice())?;
        }
        let mut strings = txn.open_table(STRINGS)?;
        for (i, s) in g.strings.table.iter().enumerate() {
            strings.insert(i as u32, s.as_str())?;
        }
    }
    txn.commit()?;
    Ok(())
}

pub fn redb_read(path: &Path) -> Result<Graph> {
    let db = redb::Database::open(path).context("redb open")?;
    let txn = db.begin_read()?;
    let strings = txn.open_table(STRINGS)?;
    let mut table = Vec::new();
    for row in strings.iter()? {
        let (_, v) = row?;
        table.push(v.value().to_owned());
    }
    let nodes_t = txn.open_table(NODES)?;
    let mut nodes = Vec::new();
    for row in nodes_t.iter()? {
        let (_, v) = row?;
        nodes.push(nodebytes::decode(v.value())?);
    }
    Ok(Graph {
        strings: Strings::from_table(table),
        nodes,
    })
}

pub fn fjall_write(g: &Graph, path: &Path) -> Result<()> {
    let db = fjall::Database::builder(path)
        .open()
        .context("fjall open")?;
    let nodes = db.keyspace("nodes", fjall::KeyspaceCreateOptions::default)?;
    let strings = db.keyspace("strings", fjall::KeyspaceCreateOptions::default)?;
    let mut buf = Vec::new();
    for (i, n) in g.nodes.iter().enumerate() {
        buf.clear();
        nodebytes::encode(n, &mut buf);
        nodes.insert((i as u32).to_be_bytes(), buf.as_slice())?;
    }
    for (i, s) in g.strings.table.iter().enumerate() {
        strings.insert((i as u32).to_be_bytes(), s.as_bytes())?;
    }
    db.persist(fjall::PersistMode::SyncAll)?;
    Ok(())
}

pub fn fjall_read(path: &Path) -> Result<Graph> {
    let db = fjall::Database::builder(path)
        .open()
        .context("fjall open")?;
    let strings = db.keyspace("strings", fjall::KeyspaceCreateOptions::default)?;
    let mut table = Vec::new();
    for item in strings.iter() {
        let v = item.value()?;
        table.push(String::from_utf8(v.to_vec())?);
    }
    let nodes_k = db.keyspace("nodes", fjall::KeyspaceCreateOptions::default)?;
    let mut nodes = Vec::new();
    for item in nodes_k.iter() {
        let v = item.value()?;
        nodes.push(nodebytes::decode(&v)?);
    }
    Ok(Graph {
        strings: Strings::from_table(table),
        nodes,
    })
}
