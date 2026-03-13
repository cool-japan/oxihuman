// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Weighted adjacency list graph with Dijkstra shortest-path support.

use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// An edge in the graph.
#[derive(Debug, Clone)]
pub struct WeightedEdge {
    pub to: usize,
    pub weight: f64,
}

/// A weighted directed graph using adjacency lists.
#[derive(Debug, Default, Clone)]
pub struct WeightedGraph {
    adj: Vec<Vec<WeightedEdge>>,
}

impl WeightedGraph {
    /// Create a new graph with `n` vertices.
    pub fn new(n: usize) -> Self {
        WeightedGraph { adj: vec![vec![]; n] }
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.adj.len()
    }

    /// Add a directed edge from `u` to `v` with `weight`.
    pub fn add_edge(&mut self, u: usize, v: usize, weight: f64) {
        self.adj[u].push(WeightedEdge { to: v, weight });
    }

    /// Add an undirected edge.
    pub fn add_undirected_edge(&mut self, u: usize, v: usize, weight: f64) {
        self.add_edge(u, v, weight);
        self.add_edge(v, u, weight);
    }

    /// Out-degree of vertex `u`.
    pub fn degree(&self, u: usize) -> usize {
        self.adj[u].len()
    }

    /// Neighbours of `u`.
    pub fn neighbours(&self, u: usize) -> &[WeightedEdge] {
        &self.adj[u]
    }

    /// Dijkstra shortest paths from `src`.  Returns distance vector.
    pub fn dijkstra(&self, src: usize) -> Vec<f64> {
        let n = self.adj.len();
        let mut dist = vec![f64::INFINITY; n];
        dist[src] = 0.0;
        /* min-heap of (distance * 1e9 as u64, vertex) */
        let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
        heap.push(Reverse((0, src)));
        while let Some(Reverse((d_raw, u))) = heap.pop() {
            let d = d_raw as f64 / 1e9;
            if d > dist[u] { continue; }
            for e in &self.adj[u] {
                let nd = dist[u] + e.weight;
                if nd < dist[e.to] {
                    dist[e.to] = nd;
                    heap.push(Reverse(((nd * 1e9) as u64, e.to)));
                }
            }
        }
        dist
    }

    /// Total number of edges.
    pub fn edge_count(&self) -> usize {
        self.adj.iter().map(|a| a.len()).sum()
    }

    /// Detect if the graph has any self-loops.
    pub fn has_self_loop(&self) -> bool {
        for (u, edges) in self.adj.iter().enumerate() {
            if edges.iter().any(|e| e.to == u) {
                return true;
            }
        }
        false
    }
}

/// Create a new weighted graph with `n` vertices.
pub fn new_weighted_graph(n: usize) -> WeightedGraph {
    WeightedGraph::new(n)
}

/// Add a directed edge.
pub fn wg_add_edge(g: &mut WeightedGraph, u: usize, v: usize, w: f64) {
    g.add_edge(u, v, w);
}

/// Add an undirected edge.
pub fn wg_add_undirected(g: &mut WeightedGraph, u: usize, v: usize, w: f64) {
    g.add_undirected_edge(u, v, w);
}

/// Run Dijkstra from `src`.
pub fn wg_dijkstra(g: &WeightedGraph, src: usize) -> Vec<f64> {
    g.dijkstra(src)
}

/// Vertex count.
pub fn wg_vertex_count(g: &WeightedGraph) -> usize {
    g.vertex_count()
}

/// Edge count.
pub fn wg_edge_count(g: &WeightedGraph) -> usize {
    g.edge_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_count() {
        let g = new_weighted_graph(5);
        assert_eq!(wg_vertex_count(&g), 5 /* five vertices */);
    }

    #[test]
    fn test_add_edge() {
        let mut g = new_weighted_graph(3);
        wg_add_edge(&mut g, 0, 1, 2.0);
        assert_eq!(wg_edge_count(&g), 1 /* one edge */);
    }

    #[test]
    fn test_undirected_edge() {
        let mut g = new_weighted_graph(3);
        wg_add_undirected(&mut g, 0, 1, 5.0);
        assert_eq!(wg_edge_count(&g), 2 /* two directed edges */);
    }

    #[test]
    fn test_dijkstra_simple() {
        let mut g = new_weighted_graph(3);
        wg_add_edge(&mut g, 0, 1, 1.0);
        wg_add_edge(&mut g, 1, 2, 2.0);
        let d = wg_dijkstra(&g, 0);
        assert!((d[2] - 3.0).abs() < 0.01 /* dist 0->2 = 3 */);
    }

    #[test]
    fn test_dijkstra_unreachable() {
        let g = new_weighted_graph(3);
        let d = wg_dijkstra(&g, 0);
        assert!(d[1].is_infinite() /* unreachable */);
    }

    #[test]
    fn test_dijkstra_shorter_path() {
        let mut g = new_weighted_graph(4);
        wg_add_edge(&mut g, 0, 1, 10.0);
        wg_add_edge(&mut g, 0, 2, 1.0);
        wg_add_edge(&mut g, 2, 1, 1.0);
        let d = wg_dijkstra(&g, 0);
        assert!((d[1] - 2.0).abs() < 0.01 /* via 2 is shorter */);
    }

    #[test]
    fn test_degree() {
        let mut g = new_weighted_graph(4);
        wg_add_edge(&mut g, 0, 1, 1.0);
        wg_add_edge(&mut g, 0, 2, 2.0);
        assert_eq!(g.degree(0), 2 /* two out-edges */);
    }

    #[test]
    fn test_no_self_loop() {
        let mut g = new_weighted_graph(3);
        wg_add_edge(&mut g, 0, 1, 1.0);
        assert!(!g.has_self_loop() /* no self loops */);
    }

    #[test]
    fn test_self_loop_detected() {
        let mut g = new_weighted_graph(2);
        wg_add_edge(&mut g, 0, 0, 1.0);
        assert!(g.has_self_loop() /* self loop present */);
    }

    #[test]
    fn test_dijkstra_source_zero() {
        let g = new_weighted_graph(3);
        let d = wg_dijkstra(&g, 0);
        assert_eq!(d[0], 0.0 /* source dist is 0 */);
    }
}
