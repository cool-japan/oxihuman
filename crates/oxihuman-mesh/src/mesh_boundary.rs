#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{HashMap, HashSet};

/// A single boundary loop as an ordered list of vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryLoop {
    pub vertices: Vec<u32>,
}

/// Information about all boundary loops in a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryInfo {
    pub loops: Vec<BoundaryLoop>,
    pub boundary_vertex_set: HashSet<u32>,
}

/// Find all boundary loops in a triangle mesh.
///
/// A boundary edge is one that appears in exactly one triangle.
#[allow(dead_code)]
pub fn find_boundary_loops(indices: &[u32], vertex_count: usize) -> BoundaryInfo {
    let _ = vertex_count;
    // Count edge usage. An edge (a,b) with a<b is boundary if used once.
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3];
        let i1 = indices[t * 3 + 1];
        let i2 = indices[t * 3 + 2];
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Collect boundary edges (used exactly once)
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut boundary_vertex_set = HashSet::new();
    for (&(a, b), &cnt) in &edge_count {
        if cnt == 1 {
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
            boundary_vertex_set.insert(a);
            boundary_vertex_set.insert(b);
        }
    }

    // Walk loops
    let mut visited: HashSet<u32> = HashSet::new();
    let mut loops = Vec::new();
    for &start in &boundary_vertex_set {
        if visited.contains(&start) {
            continue;
        }
        let mut loop_verts = Vec::new();
        let mut current = start;
        loop {
            visited.insert(current);
            loop_verts.push(current);
            let neighbors = match adj.get(&current) {
                Some(n) => n,
                None => break,
            };
            let next = neighbors.iter().find(|&&n| !visited.contains(&n));
            match next {
                Some(&n) => current = n,
                None => break,
            }
        }
        if !loop_verts.is_empty() {
            loops.push(BoundaryLoop { vertices: loop_verts });
        }
    }

    BoundaryInfo { loops, boundary_vertex_set }
}

/// Return the number of boundary loops.
#[allow(dead_code)]
pub fn boundary_loop_count(info: &BoundaryInfo) -> usize {
    info.loops.len()
}

/// Return the total number of boundary edges across all loops.
#[allow(dead_code)]
pub fn boundary_edge_count(info: &BoundaryInfo) -> usize {
    info.loops.iter().map(|l| {
        if l.vertices.len() <= 1 { 0 } else { l.vertices.len() }
    }).sum()
}

/// Check if a vertex is on a boundary.
#[allow(dead_code)]
pub fn is_boundary_vertex(info: &BoundaryInfo, vertex: u32) -> bool {
    info.boundary_vertex_set.contains(&vertex)
}

/// Compute the total length of a boundary loop given vertex positions.
/// `positions` is a flat array [x,y,z, x,y,z, ...].
#[allow(dead_code)]
pub fn boundary_length(loop_data: &BoundaryLoop, positions: &[f32]) -> f32 {
    if loop_data.vertices.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    let n = loop_data.vertices.len();
    for i in 0..n {
        let a = loop_data.vertices[i] as usize;
        let b = loop_data.vertices[(i + 1) % n] as usize;
        let dx = positions[b * 3] - positions[a * 3];
        let dy = positions[b * 3 + 1] - positions[a * 3 + 1];
        let dz = positions[b * 3 + 2] - positions[a * 3 + 2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

/// Compute the centroid of a boundary loop.
#[allow(dead_code)]
pub fn boundary_centroid(loop_data: &BoundaryLoop, positions: &[f32]) -> [f32; 3] {
    if loop_data.vertices.is_empty() {
        return [0.0; 3];
    }
    let mut cx = 0.0_f32;
    let mut cy = 0.0_f32;
    let mut cz = 0.0_f32;
    for &v in &loop_data.vertices {
        let i = v as usize;
        cx += positions[i * 3];
        cy += positions[i * 3 + 1];
        cz += positions[i * 3 + 2];
    }
    let n = loop_data.vertices.len() as f32;
    [cx / n, cy / n, cz / n]
}

/// Close a boundary loop by adding a fan of triangles from the centroid.
/// Returns new positions and indices to append.
#[allow(dead_code)]
pub fn close_boundary(
    loop_data: &BoundaryLoop,
    positions: &[f32],
) -> (Vec<f32>, Vec<u32>) {
    if loop_data.vertices.len() < 3 {
        return (Vec::new(), Vec::new());
    }
    let center = boundary_centroid(loop_data, positions);
    let center_idx = (positions.len() / 3) as u32;
    let new_pos = center.to_vec();
    let n = loop_data.vertices.len();
    let mut new_idx = Vec::new();
    for i in 0..n {
        new_idx.push(loop_data.vertices[i]);
        new_idx.push(loop_data.vertices[(i + 1) % n]);
        new_idx.push(center_idx);
    }
    (new_pos, new_idx)
}

/// Return all boundary vertices from a loop.
#[allow(dead_code)]
pub fn boundary_vertices(loop_data: &BoundaryLoop) -> &[u32] {
    &loop_data.vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    // A single triangle has all edges as boundary.
    fn single_tri() -> (Vec<f32>, Vec<u32>) {
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let idx = vec![0, 1, 2];
        (pos, idx)
    }

    // Two triangles sharing an edge (0-1). Boundary = 4 edges.
    fn two_tris() -> (Vec<f32>, Vec<u32>) {
        let pos = vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0, 0.5, -1.0, 0.0,
        ];
        let idx = vec![0, 1, 2, 1, 0, 3];
        (pos, idx)
    }

    #[test]
    fn test_find_boundary_single_tri() {
        let (pos, idx) = single_tri();
        let info = find_boundary_loops(&idx, pos.len() / 3);
        assert_eq!(boundary_loop_count(&info), 1);
    }

    #[test]
    fn test_boundary_edge_count_single() {
        let (pos, idx) = single_tri();
        let info = find_boundary_loops(&idx, pos.len() / 3);
        assert_eq!(boundary_edge_count(&info), 3);
    }

    #[test]
    fn test_is_boundary_vertex() {
        let (pos, idx) = single_tri();
        let info = find_boundary_loops(&idx, pos.len() / 3);
        assert!(is_boundary_vertex(&info, 0));
        assert!(is_boundary_vertex(&info, 1));
        assert!(is_boundary_vertex(&info, 2));
        assert!(!is_boundary_vertex(&info, 99));
    }

    #[test]
    fn test_boundary_length() {
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
        let bl = BoundaryLoop { vertices: vec![0, 1, 2] };
        let len = boundary_length(&bl, &pos);
        // edges: 1.0, 1.0, sqrt(2)
        let expected = 1.0 + 1.0 + std::f32::consts::SQRT_2;
        assert!((len - expected).abs() < 1e-5);
    }

    #[test]
    fn test_boundary_centroid() {
        let pos = vec![0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 3.0, 0.0];
        let bl = BoundaryLoop { vertices: vec![0, 1, 2] };
        let c = boundary_centroid(&bl, &pos);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
        assert!((c[2]).abs() < 1e-5);
    }

    #[test]
    fn test_close_boundary() {
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let bl = BoundaryLoop { vertices: vec![0, 1, 2] };
        let (new_pos, new_idx) = close_boundary(&bl, &pos);
        assert_eq!(new_pos.len(), 3); // one new vertex
        assert_eq!(new_idx.len(), 9); // 3 triangles * 3
    }

    #[test]
    fn test_boundary_vertices() {
        let bl = BoundaryLoop { vertices: vec![5, 6, 7] };
        assert_eq!(boundary_vertices(&bl), &[5, 6, 7]);
    }

    #[test]
    fn test_two_tris_boundary() {
        let (_pos, idx) = two_tris();
        let info = find_boundary_loops(&idx, 4);
        // shared edge 0-1 is used twice so not boundary; 4 boundary edges remain
        assert_eq!(boundary_edge_count(&info), 4);
    }

    #[test]
    fn test_empty_mesh() {
        let info = find_boundary_loops(&[], 0);
        assert_eq!(boundary_loop_count(&info), 0);
        assert_eq!(boundary_edge_count(&info), 0);
    }

    #[test]
    fn test_close_boundary_small() {
        let bl = BoundaryLoop { vertices: vec![0, 1] };
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let (new_pos, new_idx) = close_boundary(&bl, &pos);
        assert!(new_pos.is_empty());
        assert!(new_idx.is_empty());
    }
}
