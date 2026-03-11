// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
pub struct ManifoldReport {
    pub is_manifold: bool,
    pub non_manifold_edges: Vec<[u32; 2]>,
    pub non_manifold_vertices: Vec<u32>,
    pub boundary_edges: Vec<[u32; 2]>,
    pub isolated_vertices: Vec<u32>,
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[allow(dead_code)]
pub fn check_manifold(positions: &[[f32; 3]], indices: &[u32]) -> ManifoldReport {
    let n_verts = positions.len();
    let tri_count = indices.len() / 3;
    let mut edge_face_count: HashMap<(u32, u32), u32> = HashMap::new();
    let mut vertex_faces: HashMap<u32, u32> = HashMap::new();

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let ia = indices[t * 3];
        let ib = indices[t * 3 + 1];
        let ic = indices[t * 3 + 2];
        for &(a, b) in &[(ia, ib), (ib, ic), (ic, ia)] {
            *edge_face_count.entry(edge_key(a, b)).or_insert(0) += 1;
        }
        for &v in &[ia, ib, ic] {
            *vertex_faces.entry(v).or_insert(0) += 1;
        }
    }

    let mut non_manifold_edges: Vec<[u32; 2]> = Vec::new();
    let mut boundary_edges: Vec<[u32; 2]> = Vec::new();
    for (&(a, b), &count) in &edge_face_count {
        if count == 1 {
            boundary_edges.push([a, b]);
        } else if count > 2 {
            non_manifold_edges.push([a, b]);
        }
    }

    // Non-manifold vertices: referenced by > 2 boundary edges (simplified check)
    let mut boundary_vertex_degree: HashMap<u32, u32> = HashMap::new();
    for e in &boundary_edges {
        *boundary_vertex_degree.entry(e[0]).or_insert(0) += 1;
        *boundary_vertex_degree.entry(e[1]).or_insert(0) += 1;
    }
    let mut non_manifold_vertices: Vec<u32> = boundary_vertex_degree
        .iter()
        .filter(|(_, &d)| d > 2)
        .map(|(&v, _)| v)
        .collect();
    non_manifold_vertices.sort_unstable();

    // Isolated vertices: appear in positions but never in faces
    let mut referenced = vec![false; n_verts];
    for &idx in indices {
        let i = idx as usize;
        if i < n_verts {
            referenced[i] = true;
        }
    }
    let isolated_vertices: Vec<u32> = (0..n_verts)
        .filter(|&i| !referenced[i])
        .map(|i| i as u32)
        .collect();

    let is_manifold = non_manifold_edges.is_empty()
        && non_manifold_vertices.is_empty()
        && isolated_vertices.is_empty();

    ManifoldReport {
        is_manifold,
        non_manifold_edges,
        non_manifold_vertices,
        boundary_edges,
        isolated_vertices,
    }
}

#[allow(dead_code)]
pub fn non_manifold_edge_count(report: &ManifoldReport) -> usize {
    report.non_manifold_edges.len()
}

#[allow(dead_code)]
pub fn boundary_edge_count_mc(report: &ManifoldReport) -> usize {
    report.boundary_edges.len()
}

#[allow(dead_code)]
pub fn is_closed_manifold(report: &ManifoldReport) -> bool {
    report.is_manifold && report.boundary_edges.is_empty()
}

#[allow(dead_code)]
pub fn manifold_report_to_json(report: &ManifoldReport) -> String {
    format!(
        "{{\"is_manifold\":{},\"non_manifold_edges\":{},\"boundary_edges\":{},\"isolated_vertices\":{}}}",
        report.is_manifold,
        report.non_manifold_edges.len(),
        report.boundary_edges.len(),
        report.isolated_vertices.len()
    )
}

#[allow(dead_code)]
pub fn count_boundary_loops(report: &ManifoldReport) -> usize {
    if report.boundary_edges.is_empty() {
        return 0;
    }
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in &report.boundary_edges {
        adj.entry(e[0]).or_default().push(e[1]);
        adj.entry(e[1]).or_default().push(e[0]);
    }
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut loops = 0;
    for &v in adj.keys() {
        if !visited.contains(&v) {
            loops += 1;
            let mut stack = vec![v];
            while let Some(cur) = stack.pop() {
                if visited.insert(cur) {
                    if let Some(neighbors) = adj.get(&cur) {
                        for &n in neighbors {
                            if !visited.contains(&n) {
                                stack.push(n);
                            }
                        }
                    }
                }
            }
        }
    }
    loops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    fn closed_tet() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let idx = vec![0, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_open_mesh_has_boundary() {
        let (pos, idx) = quad_mesh();
        let r = check_manifold(&pos, &idx);
        assert!(!r.boundary_edges.is_empty());
    }

    #[test]
    fn test_closed_tet_no_boundary() {
        let (pos, idx) = closed_tet();
        let r = check_manifold(&pos, &idx);
        assert_eq!(r.boundary_edges.len(), 0);
    }

    #[test]
    fn test_closed_tet_is_manifold() {
        let (pos, idx) = closed_tet();
        let r = check_manifold(&pos, &idx);
        assert!(r.is_manifold);
    }

    #[test]
    fn test_empty_mesh() {
        let r = check_manifold(&[], &[]);
        assert!(r.is_manifold);
        assert_eq!(r.non_manifold_edges.len(), 0);
    }

    #[test]
    fn test_isolated_vertex() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [99.0, 99.0, 99.0],
        ];
        let idx = vec![0, 1, 2];
        let r = check_manifold(&pos, &idx);
        assert_eq!(r.isolated_vertices, vec![3]);
    }

    #[test]
    fn test_non_manifold_edge_count() {
        let (pos, idx) = quad_mesh();
        let r = check_manifold(&pos, &idx);
        assert_eq!(non_manifold_edge_count(&r), 0);
    }

    #[test]
    fn test_boundary_edge_count_fn() {
        let (pos, idx) = quad_mesh();
        let r = check_manifold(&pos, &idx);
        assert_eq!(boundary_edge_count_mc(&r), r.boundary_edges.len());
    }

    #[test]
    fn test_is_closed_manifold_false_for_open() {
        let (pos, idx) = quad_mesh();
        let r = check_manifold(&pos, &idx);
        assert!(!is_closed_manifold(&r));
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = closed_tet();
        let r = check_manifold(&pos, &idx);
        let j = manifold_report_to_json(&r);
        assert!(j.contains("is_manifold"));
    }

    #[test]
    fn test_count_boundary_loops_open() {
        let (pos, idx) = quad_mesh();
        let r = check_manifold(&pos, &idx);
        let loops = count_boundary_loops(&r);
        assert_eq!(loops, 1);
    }
}
