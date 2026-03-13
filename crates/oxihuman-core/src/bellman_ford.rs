// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bellman-Ford single-source shortest paths on a weighted directed graph.

/// A weighted directed edge.
#[derive(Debug, Clone, Copy)]
pub struct BfEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

/// A weighted directed graph for Bellman-Ford.
pub struct BfGraph {
    pub n: usize,
    pub edges: Vec<BfEdge>,
}

impl BfGraph {
    pub fn new(n: usize) -> Self {
        BfGraph {
            n,
            edges: Vec::new(),
        }
    }
}

/// Create a new Bellman-Ford graph with `n` vertices.
pub fn new_bf_graph(n: usize) -> BfGraph {
    BfGraph::new(n)
}

/// Add a directed weighted edge.
pub fn bf_add_edge(g: &mut BfGraph, from: usize, to: usize, weight: f32) {
    g.edges.push(BfEdge { from, to, weight });
}

/// Result of Bellman-Ford.
pub struct BfResult {
    /// Shortest distances from source; `f32::INFINITY` if unreachable.
    pub dist: Vec<f32>,
    /// `true` if a negative-weight cycle is reachable from `src`.
    pub has_negative_cycle: bool,
}

/// Run Bellman-Ford from `src`. Returns `BfResult`.
pub fn bellman_ford(g: &BfGraph, src: usize) -> BfResult {
    let n = g.n;
    let mut dist = vec![f32::INFINITY; n];
    if src < n {
        dist[src] = 0.0;
    }

    /* relax edges n-1 times */
    for _ in 0..n.saturating_sub(1) {
        for e in &g.edges {
            if dist[e.from].is_finite() && dist[e.from] + e.weight < dist[e.to] {
                dist[e.to] = dist[e.from] + e.weight;
            }
        }
    }

    /* check for negative cycles */
    let mut has_negative_cycle = false;
    for e in &g.edges {
        if dist[e.from].is_finite() && dist[e.from] + e.weight < dist[e.to] {
            has_negative_cycle = true;
            break;
        }
    }

    BfResult {
        dist,
        has_negative_cycle,
    }
}

/// Return the shortest distance from `src` to `dst`, or `None` if unreachable.
pub fn bf_distance(g: &BfGraph, src: usize, dst: usize) -> Option<f32> {
    if dst >= g.n {
        return None;
    }
    let r = bellman_ford(g, src);
    if r.dist[dst].is_finite() {
        Some(r.dist[dst])
    } else {
        None
    }
}

/// Return `true` if the graph has a negative cycle reachable from `src`.
pub fn bf_has_negative_cycle(g: &BfGraph, src: usize) -> bool {
    bellman_ford(g, src).has_negative_cycle
}

/// Return number of edges in the graph.
pub fn bf_edge_count(g: &BfGraph) -> usize {
    g.edges.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_positive_weights() {
        /* 0 -1-> 1 -2-> 2 */
        let mut g = new_bf_graph(3);
        bf_add_edge(&mut g, 0, 1, 1.0);
        bf_add_edge(&mut g, 1, 2, 2.0);
        let d = bf_distance(&g, 0, 2).expect("should succeed");
        assert!((d - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_negative_edge_no_cycle() {
        /* 0 -5-> 1; 0 -2-> 2; 2 -(-3)-> 1 */
        let mut g = new_bf_graph(3);
        bf_add_edge(&mut g, 0, 1, 5.0);
        bf_add_edge(&mut g, 0, 2, 2.0);
        bf_add_edge(&mut g, 2, 1, -3.0);
        let d = bf_distance(&g, 0, 1).expect("should succeed");
        assert!((d - (-1.0)).abs() < 1e-6); /* 2 + (-3) = -1 */
    }

    #[test]
    fn test_negative_cycle_detected() {
        let mut g = new_bf_graph(3);
        bf_add_edge(&mut g, 0, 1, 1.0);
        bf_add_edge(&mut g, 1, 2, -5.0);
        bf_add_edge(&mut g, 2, 0, 1.0); /* cycle weight -3 */
        assert!(bf_has_negative_cycle(&g, 0));
    }

    #[test]
    fn test_unreachable_node() {
        let mut g = new_bf_graph(4);
        bf_add_edge(&mut g, 0, 1, 1.0);
        /* node 2, 3 disconnected */
        assert!(bf_distance(&g, 0, 3).is_none());
    }

    #[test]
    fn test_src_to_itself() {
        let g = new_bf_graph(3);
        assert_eq!(bf_distance(&g, 1, 1), Some(0.0));
    }

    #[test]
    fn test_edge_count() {
        let mut g = new_bf_graph(3);
        bf_add_edge(&mut g, 0, 1, 1.0);
        bf_add_edge(&mut g, 1, 2, 2.0);
        assert_eq!(bf_edge_count(&g), 2);
    }

    #[test]
    fn test_no_negative_cycle() {
        let mut g = new_bf_graph(3);
        bf_add_edge(&mut g, 0, 1, 3.0);
        bf_add_edge(&mut g, 1, 2, 3.0);
        assert!(!bf_has_negative_cycle(&g, 0));
    }

    #[test]
    fn test_parallel_edges_take_minimum() {
        let mut g = new_bf_graph(2);
        bf_add_edge(&mut g, 0, 1, 10.0);
        bf_add_edge(&mut g, 0, 1, 3.0);
        let d = bf_distance(&g, 0, 1).expect("should succeed");
        assert!((d - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_distances() {
        let mut g = new_bf_graph(4);
        bf_add_edge(&mut g, 0, 1, 1.0);
        bf_add_edge(&mut g, 0, 2, 4.0);
        bf_add_edge(&mut g, 1, 2, 2.0);
        bf_add_edge(&mut g, 1, 3, 6.0);
        bf_add_edge(&mut g, 2, 3, 1.0);
        let r = bellman_ford(&g, 0);
        assert!((r.dist[3] - 4.0).abs() < 1e-6); /* 0->1->2->3 = 4 */
    }
}
