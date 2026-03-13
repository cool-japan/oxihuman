// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Non-manifold edge/vertex detection.

use std::collections::HashMap;

#[allow(dead_code)]
pub struct NonManifoldReport {
    pub non_manifold_edges: Vec<[u32; 2]>,
    pub non_manifold_vertices: Vec<u32>,
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a <= b { (a, b) } else { (b, a) }
}

#[allow(dead_code)]
pub fn nm_detect(_positions: &[[f32; 3]], indices: &[[u32; 3]]) -> NonManifoldReport {
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in indices {
        for i in 0..3 {
            let key = edge_key(tri[i], tri[(i + 1) % 3]);
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let non_manifold_edges: Vec<[u32; 2]> = edge_count
        .iter()
        .filter(|(_, &c)| c > 2)
        .map(|(&(a, b), _)| [a, b])
        .collect();

    let mut nm_edge_verts: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &[a, b] in &non_manifold_edges {
        nm_edge_verts.insert(a);
        nm_edge_verts.insert(b);
    }
    let mut non_manifold_vertices: Vec<u32> = nm_edge_verts.into_iter().collect();
    non_manifold_vertices.sort_unstable();

    NonManifoldReport { non_manifold_edges, non_manifold_vertices }
}

#[allow(dead_code)]
pub fn nm_is_manifold(report: &NonManifoldReport) -> bool {
    report.non_manifold_edges.is_empty() && report.non_manifold_vertices.is_empty()
}

#[allow(dead_code)]
pub fn nm_edge_count(report: &NonManifoldReport) -> usize {
    report.non_manifold_edges.len()
}

#[allow(dead_code)]
pub fn nm_vertex_count(report: &NonManifoldReport) -> usize {
    report.non_manifold_vertices.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifold_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2], [0, 2, 3]];
        (positions, indices)
    }

    #[test]
    fn test_detect_simple_manifold() {
        let (pos, idx) = manifold_mesh();
        let report = nm_detect(&pos, &idx);
        assert_eq!(nm_edge_count(&report), 0);
    }

    #[test]
    fn test_nm_is_manifold() {
        let (pos, idx) = manifold_mesh();
        let report = nm_detect(&pos, &idx);
        assert!(nm_is_manifold(&report));
    }

    #[test]
    fn test_nm_is_manifold_false() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, -1.0, 0.0],
            [1.5, 0.5, 0.0],
        ];
        let idx = vec![[0u32, 1, 2], [0, 1, 3], [0, 1, 4]];
        let report = nm_detect(&pos, &idx);
        assert!(!nm_is_manifold(&report));
    }

    #[test]
    fn test_edge_count_nonmanifold() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, -1.0, 0.0],
            [1.5, 0.5, 0.0],
        ];
        let idx = vec![[0u32, 1, 2], [0, 1, 3], [0, 1, 4]];
        let report = nm_detect(&pos, &idx);
        assert!(nm_edge_count(&report) > 0);
    }

    #[test]
    fn test_vertex_count_manifold() {
        let (pos, idx) = manifold_mesh();
        let report = nm_detect(&pos, &idx);
        assert_eq!(nm_vertex_count(&report), 0);
    }

    #[test]
    fn test_empty_mesh() {
        let report = nm_detect(&[], &[]);
        assert!(nm_is_manifold(&report));
    }

    #[test]
    fn test_single_tri_manifold() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let idx = vec![[0u32, 1, 2]];
        let report = nm_detect(&pos, &idx);
        assert!(nm_is_manifold(&report));
    }

    #[test]
    fn test_vertex_count_nonmanifold() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, -1.0, 0.0],
            [1.5, 0.5, 0.0],
        ];
        let idx = vec![[0u32, 1, 2], [0, 1, 3], [0, 1, 4]];
        let report = nm_detect(&pos, &idx);
        assert!(nm_vertex_count(&report) > 0);
    }
}
