// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Kruskal's MST algorithm with union-find.

/// A weighted edge.
#[derive(Debug, Clone, Copy)]
pub struct KruskalEdge {
    pub u: usize,
    pub v: usize,
    pub weight: f32,
}

/// Union-Find (disjoint set) structure.
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]); /* path compression */
        }
        self.parent[x]
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
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

/// Create a new Union-Find for `n` elements.
pub fn new_union_find(n: usize) -> UnionFind {
    UnionFind::new(n)
}

/// Run Kruskal's MST. Returns MST edges sorted by weight (ascending).
pub fn kruskal_mst(n: usize, edges: &[KruskalEdge]) -> Vec<KruskalEdge> {
    let mut sorted: Vec<KruskalEdge> = edges.to_vec();
    sorted.sort_by(|a, b| {
        a.weight
            .partial_cmp(&b.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut uf = UnionFind::new(n);
    let mut mst = Vec::with_capacity(n.saturating_sub(1));

    for e in &sorted {
        if e.u < n && e.v < n && uf.union(e.u, e.v) {
            mst.push(*e);
            if mst.len() == n.saturating_sub(1) {
                break;
            }
        }
    }
    mst
}

/// Return the total MST weight for the given edges and node count.
pub fn kruskal_mst_weight(n: usize, edges: &[KruskalEdge]) -> f32 {
    kruskal_mst(n, edges).iter().map(|e| e.weight).sum()
}

/// Return `true` if the MST spans all `n` nodes.
pub fn kruskal_is_spanning(n: usize, edges: &[KruskalEdge]) -> bool {
    if n == 0 {
        return true;
    }
    kruskal_mst(n, edges).len() == n - 1
}

/// Build a list of `KruskalEdge` from tuples.
pub fn kruskal_edges_from(data: &[(usize, usize, f32)]) -> Vec<KruskalEdge> {
    data.iter()
        .map(|&(u, v, w)| KruskalEdge { u, v, weight: w })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_triangle() {
        let edges = kruskal_edges_from(&[(0, 1, 1.0), (1, 2, 2.0), (0, 2, 5.0)]);
        let mst = kruskal_mst(3, &edges);
        assert_eq!(mst.len(), 2);
        let w: f32 = mst.iter().map(|e| e.weight).sum();
        assert!((w - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_mst_weight_function() {
        let edges = kruskal_edges_from(&[(0, 1, 4.0), (1, 2, 2.0), (0, 2, 3.0)]);
        let w = kruskal_mst_weight(3, &edges);
        assert!((w - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_spanning_true() {
        let edges = kruskal_edges_from(&[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]);
        assert!(kruskal_is_spanning(4, &edges));
    }

    #[test]
    fn test_not_spanning() {
        let edges = kruskal_edges_from(&[(0, 1, 1.0)]);
        assert!(!kruskal_is_spanning(4, &edges));
    }

    #[test]
    fn test_empty_edges() {
        let mst = kruskal_mst(1, &[]);
        assert!(mst.is_empty());
    }

    #[test]
    fn test_union_find_basic() {
        let mut uf = new_union_find(5);
        assert!(uf.union(0, 1));
        assert!(!uf.union(0, 1)); /* already connected */
        assert_eq!(uf.find(0), uf.find(1));
    }

    #[test]
    fn test_union_find_separate() {
        let mut uf = new_union_find(4);
        uf.union(0, 1);
        uf.union(2, 3);
        assert_ne!(uf.find(0), uf.find(2));
    }

    #[test]
    fn test_kruskal_selects_minimum_edges() {
        /* two parallel edges: pick cheaper */
        let edges = kruskal_edges_from(&[(0, 1, 10.0), (0, 1, 1.0), (1, 2, 2.0)]);
        let mst = kruskal_mst(3, &edges);
        assert_eq!(mst.len(), 2);
        assert!(mst.iter().all(|e| e.weight <= 2.0));
    }

    #[test]
    fn test_kruskal_five_nodes() {
        let edges = kruskal_edges_from(&[
            (0, 1, 2.0),
            (0, 3, 6.0),
            (1, 2, 3.0),
            (1, 3, 8.0),
            (1, 4, 5.0),
            (2, 4, 7.0),
            (3, 4, 9.0),
        ]);
        let mst = kruskal_mst(5, &edges);
        assert_eq!(mst.len(), 4);
        let w: f32 = mst.iter().map(|e| e.weight).sum();
        assert!((w - 16.0).abs() < 1e-6);
    }
}
