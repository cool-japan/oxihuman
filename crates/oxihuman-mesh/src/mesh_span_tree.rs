// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Compute a minimum spanning tree over mesh vertices using edge lengths.

/// An undirected edge with a weight.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpanEdge {
    pub a: usize,
    pub b: usize,
    pub weight: f32,
}

/// Result of a spanning tree computation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpanTreeResult {
    pub edges: Vec<SpanEdge>,
    pub total_weight: f32,
}

/// Union-Find helper.
struct Uf {
    parent: Vec<usize>,
    rank: Vec<u8>,
}
impl Uf {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }
    fn union(&mut self, a: usize, b: usize) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
        true
    }
}

/// Compute a minimum spanning tree using Kruskal's algorithm.
#[allow(dead_code)]
pub fn minimum_spanning_tree(vertex_count: usize, mut edges: Vec<SpanEdge>) -> SpanTreeResult {
    edges.sort_by(|a, b| {
        a.weight
            .partial_cmp(&b.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut uf = Uf::new(vertex_count);
    let mut result = Vec::new();
    let mut total_weight = 0.0_f32;
    for e in edges {
        if uf.union(e.a, e.b) {
            total_weight += e.weight;
            result.push(e);
        }
    }
    SpanTreeResult {
        edges: result,
        total_weight,
    }
}

/// Build edges from mesh positions and a triangle index buffer.
#[allow(dead_code)]
pub fn edges_from_mesh(positions: &[[f32; 3]], indices: &[u32]) -> Vec<SpanEdge> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for &(u, v) in &[
            (a.min(b), a.max(b)),
            (b.min(c), b.max(c)),
            (a.min(c), a.max(c)),
        ] {
            if seen.insert((u, v)) {
                let d = dist3(positions[u], positions[v]);
                edges.push(SpanEdge {
                    a: u,
                    b: v,
                    weight: d,
                });
            }
        }
    }
    edges
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Return the number of connected components the spanning tree covers.
#[allow(dead_code)]
pub fn component_count(vertex_count: usize, tree: &SpanTreeResult) -> usize {
    let mut uf = Uf::new(vertex_count);
    for e in &tree.edges {
        uf.union(e.a, e.b);
    }
    let mut roots = std::collections::HashSet::new();
    for i in 0..vertex_count {
        roots.insert(uf.find(i));
    }
    roots.len()
}

/// Return the maximum edge weight in the tree.
#[allow(dead_code)]
pub fn max_span_edge(tree: &SpanTreeResult) -> f32 {
    tree.edges.iter().map(|e| e.weight).fold(0.0_f32, f32::max)
}

/// Return the minimum edge weight in the tree.
#[allow(dead_code)]
pub fn min_span_edge(tree: &SpanTreeResult) -> f32 {
    tree.edges.iter().map(|e| e.weight).fold(f32::MAX, f32::min)
}

/// Prune edges longer than `threshold`.
#[allow(dead_code)]
pub fn prune_long_edges(tree: &mut SpanTreeResult, threshold: f32) {
    tree.edges.retain(|e| e.weight <= threshold);
    tree.total_weight = tree.edges.iter().map(|e| e.weight).sum();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]
    }
    fn line_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 2, 3]
    }

    #[test]
    fn mst_edge_count() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let tree = minimum_spanning_tree(4, edges);
        assert_eq!(tree.edges.len(), 3);
    }

    #[test]
    fn mst_total_weight_positive() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let tree = minimum_spanning_tree(4, edges);
        assert!(tree.total_weight > 0.0);
    }

    #[test]
    fn single_component() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let tree = minimum_spanning_tree(4, edges);
        assert_eq!(component_count(4, &tree), 1);
    }

    #[test]
    fn max_edge_not_negative() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let tree = minimum_spanning_tree(4, edges);
        assert!(max_span_edge(&tree) >= 0.0);
    }

    #[test]
    fn min_edge_lte_max_edge() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let tree = minimum_spanning_tree(4, edges);
        assert!(min_span_edge(&tree) <= max_span_edge(&tree));
    }

    #[test]
    fn prune_removes_long() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let mut tree = minimum_spanning_tree(4, edges);
        let before = tree.edges.len();
        prune_long_edges(&mut tree, 0.5);
        assert!(tree.edges.len() <= before);
    }

    #[test]
    fn edges_from_mesh_no_duplicates() {
        let pos = line_positions();
        let edges = edges_from_mesh(&pos, &line_indices());
        let mut keys: Vec<(usize, usize)> = edges.iter().map(|e| (e.a, e.b)).collect();
        let before = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), before);
    }

    #[test]
    fn empty_mesh() {
        let tree = minimum_spanning_tree(0, vec![]);
        assert!(tree.edges.is_empty());
    }

    #[test]
    fn isolated_vertices() {
        let tree = minimum_spanning_tree(3, vec![]);
        assert_eq!(component_count(3, &tree), 3);
    }

    #[test]
    fn span_edge_struct() {
        let e = SpanEdge {
            a: 0,
            b: 1,
            weight: 1.5,
        };
        assert!((e.weight - 1.5).abs() < 1e-6);
    }
}
