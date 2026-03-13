// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Prim's minimum spanning tree algorithm.

/// A weighted undirected edge in the MST.
#[derive(Debug, Clone, Copy)]
pub struct MstEdge {
    pub u: usize,
    pub v: usize,
    pub weight: f32,
}

/// An undirected weighted graph.
pub struct PrimGraph {
    pub n: usize,
    pub adj: Vec<Vec<(usize, f32)>>,
}

impl PrimGraph {
    pub fn new(n: usize) -> Self {
        PrimGraph {
            n,
            adj: vec![Vec::new(); n],
        }
    }
}

/// Create a new Prim graph with `n` nodes.
pub fn new_prim_graph(n: usize) -> PrimGraph {
    PrimGraph::new(n)
}

/// Add an undirected weighted edge.
pub fn prim_add_edge(g: &mut PrimGraph, u: usize, v: usize, w: f32) {
    if u < g.n && v < g.n {
        g.adj[u].push((v, w));
        g.adj[v].push((u, w));
    }
}

/// Run Prim's algorithm from vertex 0. Returns the MST edges, or `None` if
/// the graph is disconnected.
pub fn prim_mst(g: &PrimGraph) -> Option<Vec<MstEdge>> {
    if g.n == 0 {
        return Some(Vec::new());
    }
    let mut in_mst = vec![false; g.n];
    let mut key = vec![f32::INFINITY; g.n];
    let mut parent = vec![usize::MAX; g.n];
    key[0] = 0.0;

    for _ in 0..g.n {
        /* find min-key vertex not yet in MST */
        let u = (0..g.n).filter(|&v| !in_mst[v]).min_by(|&a, &b| {
            key[a]
                .partial_cmp(&key[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        in_mst[u] = true;
        for &(v, w) in &g.adj[u] {
            if !in_mst[v] && w < key[v] {
                key[v] = w;
                parent[v] = u;
            }
        }
    }

    /* collect edges */
    let mut edges = Vec::with_capacity(g.n.saturating_sub(1));
    for v in 1..g.n {
        if parent[v] == usize::MAX {
            return None; /* disconnected */
        }
        edges.push(MstEdge {
            u: parent[v],
            v,
            weight: key[v],
        });
    }
    Some(edges)
}

/// Return the total weight of the MST, or `None` if not connected.
pub fn prim_mst_weight(g: &PrimGraph) -> Option<f32> {
    prim_mst(g).map(|edges| edges.iter().map(|e| e.weight).sum())
}

/// Return number of nodes.
pub fn prim_node_count(g: &PrimGraph) -> usize {
    g.n
}

/// Return number of edges (undirected count).
pub fn prim_edge_count(g: &PrimGraph) -> usize {
    g.adj.iter().map(|v| v.len()).sum::<usize>() / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle() {
        let mut g = new_prim_graph(3);
        prim_add_edge(&mut g, 0, 1, 1.0);
        prim_add_edge(&mut g, 1, 2, 2.0);
        prim_add_edge(&mut g, 0, 2, 5.0);
        let edges = prim_mst(&g).expect("should succeed");
        assert_eq!(edges.len(), 2);
        let w = prim_mst_weight(&g).expect("should succeed");
        assert!((w - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_single_node() {
        let g = new_prim_graph(1);
        let edges = prim_mst(&g).expect("should succeed");
        assert!(edges.is_empty());
    }

    #[test]
    fn test_empty_graph() {
        let g = new_prim_graph(0);
        let edges = prim_mst(&g).expect("should succeed");
        assert!(edges.is_empty());
    }

    #[test]
    fn test_mst_edge_count() {
        /* n nodes -> n-1 MST edges */
        let mut g = new_prim_graph(5);
        prim_add_edge(&mut g, 0, 1, 1.0);
        prim_add_edge(&mut g, 1, 2, 1.0);
        prim_add_edge(&mut g, 2, 3, 1.0);
        prim_add_edge(&mut g, 3, 4, 1.0);
        let edges = prim_mst(&g).expect("should succeed");
        assert_eq!(edges.len(), 4);
    }

    #[test]
    fn test_minimum_weight_chosen() {
        let mut g = new_prim_graph(3);
        prim_add_edge(&mut g, 0, 1, 10.0);
        prim_add_edge(&mut g, 0, 1, 1.0); /* cheaper parallel edge via adj */
        prim_add_edge(&mut g, 1, 2, 1.0);
        let w = prim_mst_weight(&g).expect("should succeed");
        assert!(w <= 11.0);
    }

    #[test]
    fn test_node_count() {
        let g = new_prim_graph(7);
        assert_eq!(prim_node_count(&g), 7);
    }

    #[test]
    fn test_edge_count() {
        let mut g = new_prim_graph(3);
        prim_add_edge(&mut g, 0, 1, 1.0);
        prim_add_edge(&mut g, 1, 2, 2.0);
        assert_eq!(prim_edge_count(&g), 2);
    }

    #[test]
    fn test_disconnected_returns_none() {
        let mut g = new_prim_graph(4);
        prim_add_edge(&mut g, 0, 1, 1.0);
        /* nodes 2 and 3 are isolated */
        assert!(prim_mst(&g).is_none());
    }

    #[test]
    fn test_star_topology() {
        let mut g = new_prim_graph(5);
        for i in 1..5 {
            prim_add_edge(&mut g, 0, i, i as f32);
        }
        let edges = prim_mst(&g).expect("should succeed");
        assert_eq!(edges.len(), 4);
    }
}
