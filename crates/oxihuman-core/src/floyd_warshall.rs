// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Floyd-Warshall all-pairs shortest paths.

/// All-pairs shortest path matrix (n x n).
pub struct FwResult {
    pub n: usize,
    /// `dist[i][j]` = shortest path from i to j; `f32::INFINITY` = unreachable.
    pub dist: Vec<Vec<f32>>,
    /// `next[i][j]` = next node on shortest path i->j; `usize::MAX` = none.
    pub next: Vec<Vec<usize>>,
}

impl FwResult {
    /// Return shortest distance i->j.
    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.dist[i][j]
    }

    /// Reconstruct path from i to j; returns `None` if unreachable.
    pub fn path(&self, mut i: usize, j: usize) -> Option<Vec<usize>> {
        if !self.dist[i][j].is_finite() {
            return None;
        }
        let mut path = vec![i];
        while i != j {
            i = self.next[i][j];
            if i == usize::MAX {
                return None;
            }
            path.push(i);
        }
        Some(path)
    }
}

/// Create a FwResult initialized for no edges.
pub fn new_fw_result(n: usize) -> FwResult {
    let dist = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| if i == j { 0.0 } else { f32::INFINITY })
                .collect()
        })
        .collect();
    let next = vec![vec![usize::MAX; n]; n];
    FwResult { n, dist, next }
}

/// Add a directed weighted edge `u->v` to the matrix before running Floyd-Warshall.
pub fn fw_add_edge(fw: &mut FwResult, u: usize, v: usize, w: f32) {
    if u < fw.n && v < fw.n && w < fw.dist[u][v] {
        fw.dist[u][v] = w;
        fw.next[u][v] = v;
    }
}

/// Run Floyd-Warshall in-place on the result.
pub fn floyd_warshall(fw: &mut FwResult) {
    let n = fw.n;
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if fw.dist[i][k].is_finite() && fw.dist[k][j].is_finite() {
                    let new_d = fw.dist[i][k] + fw.dist[k][j];
                    if new_d < fw.dist[i][j] {
                        fw.dist[i][j] = new_d;
                        fw.next[i][j] = fw.next[i][k];
                    }
                }
            }
        }
    }
}

/// Build and run Floyd-Warshall from an edge list. Returns the completed FwResult.
pub fn fw_solve(n: usize, edges: &[(usize, usize, f32)]) -> FwResult {
    let mut fw = new_fw_result(n);
    for &(u, v, w) in edges {
        fw_add_edge(&mut fw, u, v, w);
    }
    floyd_warshall(&mut fw);
    fw
}

/// Return `true` if any diagonal `dist[i][i]` < 0 (negative cycle).
pub fn fw_has_negative_cycle(fw: &FwResult) -> bool {
    (0..fw.n).any(|i| fw.dist[i][i] < 0.0)
}

/// Return the shortest distance i->j, or `None` if unreachable.
pub fn fw_distance(fw: &FwResult, i: usize, j: usize) -> Option<f32> {
    if i < fw.n && j < fw.n && fw.dist[i][j].is_finite() {
        Some(fw.dist[i][j])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_triangle() {
        let fw = fw_solve(3, &[(0, 1, 1.0), (1, 2, 2.0), (0, 2, 10.0)]);
        assert!((fw.get(0, 2) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_unreachable() {
        let fw = fw_solve(3, &[(0, 1, 1.0)]);
        assert!(fw_distance(&fw, 0, 2).is_none());
    }

    #[test]
    fn test_self_distance_zero() {
        let fw = fw_solve(3, &[(0, 1, 5.0)]);
        assert_eq!(fw.get(0, 0), 0.0);
    }

    #[test]
    fn test_path_reconstruction() {
        let fw = fw_solve(3, &[(0, 1, 1.0), (1, 2, 1.0)]);
        let path = fw.path(0, 2).unwrap();
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn test_no_negative_cycle() {
        let fw = fw_solve(3, &[(0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0)]);
        assert!(!fw_has_negative_cycle(&fw));
    }

    #[test]
    fn test_fw_distance_finite() {
        let fw = fw_solve(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]);
        assert_eq!(fw_distance(&fw, 0, 3), Some(3.0));
    }

    #[test]
    fn test_symmetric_edges() {
        let fw = fw_solve(3, &[(0, 1, 2.0), (1, 0, 2.0), (1, 2, 3.0), (2, 1, 3.0)]);
        assert!((fw.get(0, 2) - 5.0).abs() < 1e-6);
        assert!((fw.get(2, 0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_fw_result_diagonal() {
        let fw = new_fw_result(5);
        for i in 0..5 {
            assert_eq!(fw.dist[i][i], 0.0);
        }
    }

    #[test]
    fn test_large_graph_all_pairs() {
        /* star topology: 0 connects to 1..4 */
        let edges: Vec<(usize, usize, f32)> = (1..5).map(|i| (0, i, 1.0)).collect();
        let fw = fw_solve(5, &edges);
        /* 1->2 goes through 0: cost 2 (directed, so only from 0 outward) */
        assert!(fw_distance(&fw, 0, 4).is_some());
    }
}
