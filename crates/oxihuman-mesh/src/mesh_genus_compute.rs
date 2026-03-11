// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Genus and Euler characteristic computation for triangle meshes.

use std::collections::HashSet;

/// Result of genus/Euler computation.
#[derive(Clone, Debug, Default)]
pub struct GenusResult {
    pub vertex_count: usize,
    pub edge_count: usize,
    pub face_count: usize,
    pub euler_characteristic: i64,
    /// Genus for an orientable surface: g = (2 - chi) / 2 (valid when chi ≤ 2 and even)
    pub genus: i64,
    pub boundary_loop_count: i64,
}

/// Count unique edges in a triangle mesh.
pub fn count_unique_edges(indices: &[u32]) -> usize {
    let mut edges: HashSet<(u32, u32)> = HashSet::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = (a.min(b), a.max(b));
            edges.insert(key);
        }
    }
    edges.len()
}

/// Count boundary loops (edges appearing exactly once as undirected).
pub fn count_boundary_loops_genus(indices: &[u32]) -> i64 {
    let mut edge_count_map: std::collections::HashMap<(u32, u32), u32> =
        std::collections::HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = (a.min(b), a.max(b));
            *edge_count_map.entry(key).or_insert(0) += 1;
        }
    }
    let boundary_edges: Vec<_> = edge_count_map.iter().filter(|(_, &c)| c == 1).collect();

    // Trace loops from boundary edges
    if boundary_edges.is_empty() {
        return 0;
    }
    let mut next: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for &(&(a, b), _) in &boundary_edges {
        // Determine directed boundary edge
        next.entry(a).or_insert(b);
    }
    let mut visited: HashSet<u32> = HashSet::new();
    let mut loop_count = 0i64;
    for &start in next.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut cur = start;
        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            if let Some(&nxt) = next.get(&cur) {
                cur = nxt;
            } else {
                break;
            }
        }
        loop_count += 1;
    }
    loop_count
}

/// Compute Euler characteristic and genus for a triangle mesh.
///
/// χ = V - E + F
/// For a closed orientable surface: g = (2 - χ) / 2
/// For a surface with b boundary loops: χ = 2 - 2g - b → g = (2 - χ - b) / 2
pub fn compute_genus(positions: &[[f32; 3]], indices: &[u32]) -> GenusResult {
    let v = positions.len();
    let e = count_unique_edges(indices);
    let f = indices.len() / 3;
    let chi = v as i64 - e as i64 + f as i64;
    let b = count_boundary_loops_genus(indices);
    // g = (2 - chi - b) / 2, clamped to >= 0
    let genus = ((2 - chi - b) / 2).max(0);
    GenusResult {
        vertex_count: v,
        edge_count: e,
        face_count: f,
        euler_characteristic: chi,
        genus,
        boundary_loop_count: b,
    }
}

/// Return Euler characteristic.
pub fn euler_char(r: &GenusResult) -> i64 {
    r.euler_characteristic
}

/// Return genus.
pub fn genus_value(r: &GenusResult) -> i64 {
    r.genus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetrahedron() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        /* 4 faces (closed) */
        let idx = vec![0, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3];
        (pos, idx)
    }

    fn open_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn tetrahedron_euler_chi_2() {
        let (pos, idx) = tetrahedron();
        let r = compute_genus(&pos, &idx);
        assert_eq!(
            euler_char(&r),
            2,
            "V={} E={} F={}",
            r.vertex_count,
            r.edge_count,
            r.face_count
        );
    }

    #[test]
    fn tetrahedron_genus_0() {
        let (pos, idx) = tetrahedron();
        let r = compute_genus(&pos, &idx);
        assert_eq!(genus_value(&r), 0);
    }

    #[test]
    fn open_mesh_has_boundary_loops() {
        let (pos, idx) = open_quad();
        let r = compute_genus(&pos, &idx);
        assert!(r.boundary_loop_count >= 1);
    }

    #[test]
    fn count_unique_edges_quad() {
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let e = count_unique_edges(&idx);
        assert_eq!(e, 5); // 4 outer + 1 diagonal
    }

    #[test]
    fn genus_result_nonneg() {
        let (pos, idx) = open_quad();
        let r = compute_genus(&pos, &idx);
        assert!(r.genus >= 0);
    }

    #[test]
    fn empty_mesh_zeros() {
        let r = compute_genus(&[], &[]);
        assert_eq!(r.vertex_count, 0);
        assert_eq!(r.face_count, 0);
    }

    #[test]
    fn vertex_edge_face_counts_match() {
        let (pos, idx) = tetrahedron();
        let r = compute_genus(&pos, &idx);
        assert_eq!(r.vertex_count, pos.len());
        assert_eq!(r.face_count, idx.len() / 3);
    }

    #[test]
    fn count_boundary_loops_closed_is_zero() {
        let (_, idx) = tetrahedron();
        let b = count_boundary_loops_genus(&idx);
        assert_eq!(b, 0);
    }
}
