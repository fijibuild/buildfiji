//! Synthesise a larger graph by replicating a loaded one (buildfiji-23d.19):
//! `replicate(g, n)` makes `n` copies of every node, offsetting dependency
//! ids per copy so each copy's edges stay internal and valid. The string
//! table is shared across copies rather than duplicated — real replication
//! (e.g. many near-identical `go_library` targets) would reuse rule class
//! and attribute name strings just as heavily, and duplicating them would
//! only inflate the string section, not the node/edge sections this spike
//! is measuring at scale.
use crate::graph::{Graph, NodeId, Strings};

pub fn replicate(g: &Graph, times: usize) -> Graph {
    let n = NodeId::try_from(g.nodes.len()).expect("node id overflow");
    let mut nodes = Vec::with_capacity(g.nodes.len() * times.max(1));
    for copy in 0..NodeId::try_from(times).expect("replica count overflow") {
        let base = copy * n;
        for node in &g.nodes {
            let mut node = node.clone();
            for d in &mut node.deps {
                *d += base;
            }
            nodes.push(node);
        }
    }
    Graph {
        strings: Strings::from_table(g.strings.table.clone()),
        nodes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Kind, Node};

    #[test]
    fn replicate_preserves_internal_edges_and_scales_node_count() {
        let mut g = Graph::default();
        let s = g.strings.intern("a");
        let a = g.push(Node {
            kind: Kind::File,
            key: vec![s],
            value: vec![],
            digest: None,
            deps: vec![],
        });
        g.push(Node {
            kind: Kind::Action,
            key: vec![s],
            value: vec![],
            digest: None,
            deps: vec![a],
        });

        let r = replicate(&g, 3);
        assert_eq!(r.nodes.len(), 6);
        // Each copy's action depends on that copy's file (index 2*copy),
        // not copy 0's.
        assert_eq!(r.nodes[1].deps, vec![0]);
        assert_eq!(r.nodes[3].deps, vec![2]);
        assert_eq!(r.nodes[5].deps, vec![4]);
    }
}
