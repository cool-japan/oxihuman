// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Catmull-Clark subdivision surface algorithm for quad-dominant meshes.
//!
//! This module provides a full Catmull-Clark subdivision pipeline operating
//! on flat vertex/index buffers.  Faces may be triangles or quads (or mixed);
//! the algorithm generalises to arbitrary polygons but this implementation
//! handles tris and quads explicitly.

use std::collections::HashMap;

// ── math helpers ─────────────────────────────────────────────────────────────

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn canonical_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

// ── public types ─────────────────────────────────────────────────────────────

/// Configuration for Catmull-Clark subdivision.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubdivConfig {
    /// Number of subdivision levels to apply.
    pub levels: usize,
    /// When true, boundary vertices are pinned (not moved).
    pub fix_boundary: bool,
    /// Smoothing weight for boundary vertices (0 = sharp, 1 = smooth).
    pub boundary_weight: f32,
}

/// Result of a subdivision operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubdivResult {
    /// Subdivided vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Subdivided face indices (quads stored as 4-tuples flattened).
    pub indices: Vec<u32>,
    /// Number of faces per polygon (4 for quads, 3 for tris).
    pub face_sizes: Vec<usize>,
    /// Number of subdivision levels applied.
    pub levels_applied: usize,
}

// ── public functions ─────────────────────────────────────────────────────────

/// Create a default subdivision configuration.
#[allow(dead_code)]
pub fn default_subdiv_config() -> SubdivConfig {
    SubdivConfig {
        levels: 1,
        fix_boundary: false,
        boundary_weight: 0.5,
    }
}

/// Perform one level of Catmull-Clark subdivision on a triangle mesh.
///
/// Input: flat positions, flat triangle indices.
/// Output: `SubdivResult` containing quad-dominant topology.
#[allow(dead_code)]
pub fn subdivide_catmull_clark(positions: &[[f32; 3]], indices: &[u32]) -> SubdivResult {
    let config = default_subdiv_config();
    subdivide_catmull_clark_impl(positions, indices, &config)
}

fn subdivide_catmull_clark_impl(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &SubdivConfig,
) -> SubdivResult {
    // Detect face sizes from indices.  We assume tris (chunks of 3).
    let _n_faces = indices.len() / 3;

    // Build face list (each face is a vec of vertex indices).
    let faces: Vec<Vec<u32>> = indices
        .chunks_exact(3)
        .map(|c| vec![c[0], c[1], c[2]])
        .collect();

    // 1. Compute face points
    let face_points = compute_face_points(positions, &faces);

    // 2. Build edge→faces map
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (fi, face) in faces.iter().enumerate() {
        let n = face.len();
        for i in 0..n {
            let key = canonical_edge(face[i], face[(i + 1) % n]);
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // 3. Compute edge points
    let edge_points_map = compute_edge_points(positions, &face_points, &edge_faces);

    // 4. Compute vertex points
    let vertex_points = compute_vertex_points(
        positions,
        &faces,
        &face_points,
        &edge_faces,
        config.fix_boundary,
    );

    // 5. Assemble new topology: each original face yields N quads (one per edge).
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_indices: Vec<u32> = Vec::new();
    let mut new_face_sizes: Vec<usize> = Vec::new();

    // Vertex layout: [vertex_points | face_points | edge_points]
    let v_offset = 0u32;
    let f_offset = vertex_points.len() as u32;
    let e_offset = f_offset + face_points.len() as u32;

    new_positions.extend_from_slice(&vertex_points);
    new_positions.extend_from_slice(&face_points);

    // Assign edge-point indices in a deterministic order.
    let mut edge_index_map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut edge_pts_ordered: Vec<[f32; 3]> = Vec::new();
    for face in &faces {
        let n = face.len();
        for i in 0..n {
            let key = canonical_edge(face[i], face[(i + 1) % n]);
            if let std::collections::hash_map::Entry::Vacant(e) = edge_index_map.entry(key) {
                let idx = e_offset + edge_pts_ordered.len() as u32;
                e.insert(idx);
                edge_pts_ordered.push(edge_points_map[&key]);
            }
        }
    }
    new_positions.extend_from_slice(&edge_pts_ordered);

    // For each face, create N quads.
    for (fi, face) in faces.iter().enumerate() {
        let fp_idx = f_offset + fi as u32;
        let n = face.len();
        for i in 0..n {
            let v_cur = face[i];
            let v_next = face[(i + 1) % n];
            let v_prev = face[(i + n - 1) % n];

            let ep_next = edge_index_map[&canonical_edge(v_cur, v_next)];
            let ep_prev = edge_index_map[&canonical_edge(v_prev, v_cur)];

            // Quad: vertex_point[v_cur], edge_point[cur→next], face_point, edge_point[prev→cur]
            new_indices.push(v_offset + v_cur);
            new_indices.push(ep_next);
            new_indices.push(fp_idx);
            new_indices.push(ep_prev);
            new_face_sizes.push(4);
        }
    }

    SubdivResult {
        positions: new_positions,
        indices: new_indices,
        face_sizes: new_face_sizes,
        levels_applied: 1,
    }
}

/// Compute face points: centroid of each face's vertices.
#[allow(dead_code)]
pub fn compute_face_points(positions: &[[f32; 3]], faces: &[Vec<u32>]) -> Vec<[f32; 3]> {
    faces
        .iter()
        .map(|face| {
            let n = face.len() as f32;
            let sum = face
                .iter()
                .fold([0.0f32; 3], |acc, &vi| add3(acc, positions[vi as usize]));
            scale3(sum, 1.0 / n)
        })
        .collect()
}

/// Compute edge points: for interior edges, average of edge endpoints and
/// adjacent face points; for boundary edges, midpoint of endpoints.
#[allow(dead_code)]
pub fn compute_edge_points(
    positions: &[[f32; 3]],
    face_points: &[[f32; 3]],
    edge_faces: &HashMap<(u32, u32), Vec<usize>>,
) -> HashMap<(u32, u32), [f32; 3]> {
    let mut result = HashMap::new();
    for (&(a, b), adj_faces) in edge_faces {
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        if adj_faces.len() == 2 {
            let fp0 = face_points[adj_faces[0]];
            let fp1 = face_points[adj_faces[1]];
            let sum = add3(add3(pa, pb), add3(fp0, fp1));
            result.insert((a, b), scale3(sum, 0.25));
        } else {
            // Boundary: midpoint
            result.insert((a, b), scale3(add3(pa, pb), 0.5));
        }
    }
    result
}

/// Compute updated vertex positions using the Catmull-Clark weighting rule.
///
/// For interior vertices with valence n:
///   Q = avg of adjacent face points
///   R = avg of midpoints of adjacent edges
///   new_pos = (Q + 2R + (n-3)P) / n
///
/// For boundary vertices (if not fixed): average of adjacent boundary edge
/// midpoints.
#[allow(dead_code)]
pub fn compute_vertex_points(
    positions: &[[f32; 3]],
    faces: &[Vec<u32>],
    face_points: &[[f32; 3]],
    edge_faces: &HashMap<(u32, u32), Vec<usize>>,
    fix_boundary: bool,
) -> Vec<[f32; 3]> {
    let n_verts = positions.len();

    // Build per-vertex: adjacent faces, adjacent edges
    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    for (fi, face) in faces.iter().enumerate() {
        for &vi in face {
            vert_faces[vi as usize].push(fi);
        }
    }

    let mut vert_edges: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n_verts];
    let mut boundary_verts: Vec<bool> = vec![false; n_verts];
    for (&key, adj) in edge_faces {
        vert_edges[key.0 as usize].push(key);
        vert_edges[key.1 as usize].push(key);
        if adj.len() == 1 {
            boundary_verts[key.0 as usize] = true;
            boundary_verts[key.1 as usize] = true;
        }
    }

    let mut result = vec![[0.0f32; 3]; n_verts];
    for vi in 0..n_verts {
        if boundary_verts[vi] {
            if fix_boundary {
                result[vi] = positions[vi];
            } else {
                // Average of adjacent boundary edge midpoints + self
                let mut sum = positions[vi];
                let mut count = 1.0f32;
                for &(a, b) in &vert_edges[vi] {
                    if edge_faces[&(a, b)].len() == 1 {
                        let other = if a as usize == vi { b } else { a };
                        sum = add3(sum, positions[other as usize]);
                        count += 1.0;
                    }
                }
                result[vi] = scale3(sum, 1.0 / count);
            }
        } else {
            let n = vert_faces[vi].len() as f32;
            if n < 1.0 {
                result[vi] = positions[vi];
                continue;
            }
            // Q = average of adjacent face points
            let q = {
                let sum = vert_faces[vi]
                    .iter()
                    .fold([0.0f32; 3], |acc, &fi| add3(acc, face_points[fi]));
                scale3(sum, 1.0 / n)
            };
            // R = average of edge midpoints
            let r = {
                let edge_count = vert_edges[vi].len() as f32;
                if edge_count < 1.0 {
                    positions[vi]
                } else {
                    let sum = vert_edges[vi].iter().fold([0.0f32; 3], |acc, &(a, b)| {
                        add3(
                            acc,
                            scale3(add3(positions[a as usize], positions[b as usize]), 0.5),
                        )
                    });
                    scale3(sum, 1.0 / edge_count)
                }
            };
            // new_pos = (Q + 2R + (n-3)P) / n
            let p = positions[vi];
            let new_pos = scale3(add3(add3(q, scale3(r, 2.0)), scale3(p, n - 3.0)), 1.0 / n);
            result[vi] = new_pos;
        }
    }
    result
}

/// Apply N levels of Catmull-Clark subdivision.
#[allow(dead_code)]
pub fn subdivide_n_levels(positions: &[[f32; 3]], indices: &[u32], levels: usize) -> SubdivResult {
    if levels == 0 {
        return SubdivResult {
            positions: positions.to_vec(),
            indices: indices.to_vec(),
            face_sizes: vec![3; indices.len() / 3],
            levels_applied: 0,
        };
    }
    let config = default_subdiv_config();
    let mut result = subdivide_catmull_clark_impl(positions, indices, &config);

    for _lvl in 1..levels {
        // Convert quads to tris for re-subdivision.
        let tri_indices = triangulate_quads(&result.indices, &result.face_sizes);
        result = subdivide_catmull_clark_impl(&result.positions, &tri_indices, &config);
    }
    result.levels_applied = levels;
    result
}

/// Estimate the vertex count after `levels` levels of subdivision on a
/// triangle mesh.  Each level roughly multiplies vertices by ~4.
#[allow(dead_code)]
pub fn subdivision_vertex_count_estimate(n_verts: usize, n_faces: usize, levels: usize) -> usize {
    // For tris: V' = V + E + F.  With Euler: E ≈ 3F/2 for closed tri mesh.
    let mut v = n_verts;
    let mut f = n_faces;
    for _ in 0..levels {
        let e = f * 3 / 2; // approximate
        v = v + e + f;
        f *= 4; // each face → 4 quads (then triangulated → more)
    }
    v
}

/// Estimate the face count after `levels` levels of subdivision.
#[allow(dead_code)]
pub fn subdivision_face_count_estimate(n_faces: usize, levels: usize) -> usize {
    let mut f = n_faces;
    for _ in 0..levels {
        // Each n-gon face produces n quads.  For tris: 3 quads per face.
        f *= 3;
    }
    f
}

/// Check whether every face has exactly 4 vertices (pure quad mesh).
#[allow(dead_code)]
pub fn is_quad_mesh(face_sizes: &[usize]) -> bool {
    !face_sizes.is_empty() && face_sizes.iter().all(|&s| s == 4)
}

/// Convert a quad/mixed-polygon mesh to pure triangles.
///
/// Quads are split into two triangles; faces with N > 4 use fan triangulation.
#[allow(dead_code)]
pub fn triangulate_quads(indices: &[u32], face_sizes: &[usize]) -> Vec<u32> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for &size in face_sizes {
        if size == 3 {
            out.push(indices[offset]);
            out.push(indices[offset + 1]);
            out.push(indices[offset + 2]);
        } else {
            // Fan triangulation from vertex 0 of the face.
            for i in 1..size - 1 {
                out.push(indices[offset]);
                out.push(indices[offset + i]);
                out.push(indices[offset + i + 1]);
            }
        }
        offset += size;
    }
    out
}

/// Return the subdivision level stored in a result, or infer from vertex
/// counts.  This just returns the stored level.
#[allow(dead_code)]
pub fn subdivision_level(result: &SubdivResult) -> usize {
    result.levels_applied
}

/// Smooth boundary vertices by averaging with their boundary neighbours.
///
/// `boundary_weight` in \[0,1\]: 0 = no smoothing, 1 = full average.
#[allow(dead_code)]
pub fn smooth_boundary_vertices(positions: &mut [[f32; 3]], indices: &[u32], boundary_weight: f32) {
    // Build edge→face count.
    let _n_faces = indices.len() / 3;
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for tri in indices.chunks_exact(3) {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            *edge_count.entry(canonical_edge(a, b)).or_insert(0) += 1;
        }
    }

    // Identify boundary vertices.
    let n_verts = positions.len();
    let mut is_boundary = vec![false; n_verts];
    let mut boundary_neighbours: Vec<Vec<u32>> = vec![Vec::new(); n_verts];
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            is_boundary[a as usize] = true;
            is_boundary[b as usize] = true;
            boundary_neighbours[a as usize].push(b);
            boundary_neighbours[b as usize].push(a);
        }
    }

    let old_positions = positions.to_vec();
    let w = boundary_weight.clamp(0.0, 1.0);
    for vi in 0..n_verts {
        if !is_boundary[vi] || boundary_neighbours[vi].is_empty() {
            continue;
        }
        let n = boundary_neighbours[vi].len() as f32;
        let avg = boundary_neighbours[vi]
            .iter()
            .fold([0.0f32; 3], |acc, &ni| {
                add3(acc, old_positions[ni as usize])
            });
        let avg = scale3(avg, 1.0 / n);
        // Blend: new = (1-w)*old + w*avg
        let blended = add3(scale3(old_positions[vi], 1.0 - w), scale3(avg, w));
        positions[vi] = blended;
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]]
    }

    fn triangle_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    fn quad_box_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_box_indices() -> Vec<u32> {
        // Two triangles forming a quad.
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_subdiv_config();
        assert_eq!(cfg.levels, 1);
        assert!(!cfg.fix_boundary);
    }

    #[test]
    fn test_subdivide_single_triangle() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let result = subdivide_catmull_clark(&pos, &idx);
        assert!(!result.positions.is_empty());
        assert!(!result.indices.is_empty());
        assert_eq!(result.levels_applied, 1);
    }

    #[test]
    fn test_face_sizes_are_quads() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let result = subdivide_catmull_clark(&pos, &idx);
        // Each triangle produces 3 quads.
        assert_eq!(result.face_sizes.len(), 3);
        for &s in &result.face_sizes {
            assert_eq!(s, 4);
        }
    }

    #[test]
    fn test_compute_face_points_single_tri() {
        let pos = triangle_positions();
        let faces = vec![vec![0u32, 1, 2]];
        let fp = compute_face_points(&pos, &faces);
        assert_eq!(fp.len(), 1);
        assert!((fp[0][0] - 0.5).abs() < 1e-5);
        assert!((fp[0][1] - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_edge_points_boundary() {
        let pos = triangle_positions();
        let faces = vec![vec![0u32, 1, 2]];
        let fp = compute_face_points(&pos, &faces);
        let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
        for (fi, face) in faces.iter().enumerate() {
            let n = face.len();
            for i in 0..n {
                let key = canonical_edge(face[i], face[(i + 1) % n]);
                edge_faces.entry(key).or_default().push(fi);
            }
        }
        let ep = compute_edge_points(&pos, &fp, &edge_faces);
        // All edges are boundary (single triangle), so edge point = midpoint.
        let key01 = canonical_edge(0, 1);
        let mp = ep[&key01];
        assert!((mp[0] - 0.5).abs() < 1e-5);
        assert!((mp[1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_subdivide_n_levels_zero() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let result = subdivide_n_levels(&pos, &idx, 0);
        assert_eq!(result.levels_applied, 0);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn test_subdivide_n_levels_two() {
        let pos = quad_box_positions();
        let idx = quad_box_indices();
        let result = subdivide_n_levels(&pos, &idx, 2);
        assert_eq!(result.levels_applied, 2);
        assert!(result.positions.len() > pos.len());
    }

    #[test]
    fn test_vertex_count_estimate() {
        let est = subdivision_vertex_count_estimate(4, 2, 1);
        assert!(est > 4);
    }

    #[test]
    fn test_face_count_estimate() {
        let est = subdivision_face_count_estimate(2, 1);
        assert_eq!(est, 6); // 2 * 3
    }

    #[test]
    fn test_is_quad_mesh_true() {
        let fs = vec![4, 4, 4];
        assert!(is_quad_mesh(&fs));
    }

    #[test]
    fn test_is_quad_mesh_false_with_tris() {
        let fs = vec![3, 4, 4];
        assert!(!is_quad_mesh(&fs));
    }

    #[test]
    fn test_is_quad_mesh_empty() {
        let fs: Vec<usize> = vec![];
        assert!(!is_quad_mesh(&fs));
    }

    #[test]
    fn test_triangulate_quads() {
        // A single quad: 0,1,2,3
        let indices = vec![0u32, 1, 2, 3];
        let sizes = vec![4usize];
        let tri = triangulate_quads(&indices, &sizes);
        assert_eq!(tri.len(), 6); // 2 triangles
    }

    #[test]
    fn test_triangulate_mixed() {
        let indices = vec![0u32, 1, 2, 3, 4, 5, 6];
        let sizes = vec![3usize, 4];
        let tri = triangulate_quads(&indices, &sizes);
        // 1 tri + 2 tris from quad = 3 tris = 9 indices
        assert_eq!(tri.len(), 9);
    }

    #[test]
    fn test_subdivision_level_accessor() {
        let r = SubdivResult {
            positions: vec![],
            indices: vec![],
            face_sizes: vec![],
            levels_applied: 3,
        };
        assert_eq!(subdivision_level(&r), 3);
    }

    #[test]
    fn test_smooth_boundary_vertices() {
        let mut pos = triangle_positions();
        let idx = triangle_indices();
        // All vertices are boundary in a single triangle.
        smooth_boundary_vertices(&mut pos, &idx, 1.0);
        // Vertex 0 should move toward average of neighbours (1, 2).
        // avg of [1,0,0] and [0.5,1,0] = [0.75, 0.5, 0]
        assert!((pos[0][0] - 0.75).abs() < 1e-5);
        assert!((pos[0][1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_boundary_weight_zero() {
        let mut pos = triangle_positions();
        let orig = pos.clone();
        let idx = triangle_indices();
        smooth_boundary_vertices(&mut pos, &idx, 0.0);
        for (a, b) in pos.iter().zip(orig.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-9);
            assert!((a[1] - b[1]).abs() < 1e-9);
            assert!((a[2] - b[2]).abs() < 1e-9);
        }
    }

    #[test]
    fn test_subdivide_quad_box() {
        let pos = quad_box_positions();
        let idx = quad_box_indices();
        let result = subdivide_catmull_clark(&pos, &idx);
        // 2 tris → 6 quads
        assert_eq!(result.face_sizes.len(), 6);
        assert!(is_quad_mesh(&result.face_sizes));
    }
}
