// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;
use std::collections::HashMap;

// ── helpers ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn canonical_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

// ── Catmull-Clark single iteration ───────────────────────────────────────────

fn catmull_clark_once(mesh: &MeshBuffers, fix_boundary: bool) -> MeshBuffers {
    let positions = &mesh.positions;
    let indices = &mesh.indices;
    let n_orig = positions.len();
    let n_faces = indices.len() / 3;

    // --- Step 1: Face points (centroids) ---
    let mut face_points: Vec<[f32; 3]> = Vec::with_capacity(n_faces);
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let fp = scale3(
            add3(add3(positions[a], positions[b]), positions[c]),
            1.0 / 3.0,
        );
        face_points.push(fp);
    }

    // --- Step 2: Build edge → face list ---
    // edge key → list of face indices
    let mut edge_to_faces: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (fi, tri) in indices.chunks_exact(3).enumerate() {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        for &(a, b) in &[(v0, v1), (v1, v2), (v2, v0)] {
            edge_to_faces
                .entry(canonical_edge(a, b))
                .or_default()
                .push(fi as u32);
        }
    }

    // --- Step 3: Edge points ---
    // edge key → index in new vertex buffer
    let mut edge_to_idx: HashMap<(u32, u32), u32> = HashMap::new();
    let mut edge_positions: Vec<[f32; 3]> = Vec::new();

    // Collect edges in deterministic order (iterate triangles)
    for tri in indices.chunks_exact(3) {
        let (v0, v1, v2) = (tri[0], tri[1], tri[2]);
        for &(a, b) in &[(v0, v1), (v1, v2), (v2, v0)] {
            let key = canonical_edge(a, b);
            if edge_to_idx.contains_key(&key) {
                continue;
            }
            let pa = positions[key.0 as usize];
            let pb = positions[key.1 as usize];
            let face_list = &edge_to_faces[&key];
            let ep = if face_list.len() == 2 {
                let fp0 = face_points[face_list[0] as usize];
                let fp1 = face_points[face_list[1] as usize];
                scale3(add3(add3(add3(pa, pb), fp0), fp1), 0.25)
            } else {
                // boundary: simple midpoint
                scale3(add3(pa, pb), 0.5)
            };
            let idx = n_orig as u32 + n_faces as u32 + edge_positions.len() as u32;
            edge_positions.push(ep);
            edge_to_idx.insert(key, idx);
        }
    }

    // --- Step 4: Updated original vertex positions ---
    // For each vertex, gather adjacent face points and edge midpoints
    let mut vertex_face_pts: Vec<Vec<[f32; 3]>> = vec![Vec::new(); n_orig];
    let mut vertex_edge_mids: Vec<Vec<[f32; 3]>> = vec![Vec::new(); n_orig];
    let mut vertex_is_boundary: Vec<bool> = vec![false; n_orig];

    for (fi, tri) in indices.chunks_exact(3).enumerate() {
        for &v in tri {
            vertex_face_pts[v as usize].push(face_points[fi]);
        }
    }

    for (&(a, b), face_list) in &edge_to_faces {
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        let mid = scale3(add3(pa, pb), 0.5);
        vertex_edge_mids[a as usize].push(mid);
        vertex_edge_mids[b as usize].push(mid);
        if face_list.len() == 1 {
            vertex_is_boundary[a as usize] = true;
            vertex_is_boundary[b as usize] = true;
        }
    }

    let mut updated_positions: Vec<[f32; 3]> = Vec::with_capacity(n_orig);
    for i in 0..n_orig {
        let v = positions[i];
        if vertex_is_boundary[i] {
            if fix_boundary {
                updated_positions.push(v);
            } else {
                // Simple boundary rule: keep original
                updated_positions.push(v);
            }
        } else {
            let n = vertex_face_pts[i].len();
            if n == 0 {
                updated_positions.push(v);
                continue;
            }
            let n_f = n as f32;
            // Q = average of adjacent face points
            let mut q_sum = [0.0f32; 3];
            for &fp in &vertex_face_pts[i] {
                q_sum = add3(q_sum, fp);
            }
            let q = scale3(q_sum, 1.0 / n_f);
            // R = average of edge midpoints
            let mut r_sum = [0.0f32; 3];
            for &em in &vertex_edge_mids[i] {
                r_sum = add3(r_sum, em);
            }
            let r = scale3(r_sum, 1.0 / n_f);
            // new_v = (Q + 2R + (n-3)*V) / n
            let new_v = scale3(
                add3(add3(q, scale3(r, 2.0)), scale3(v, (n_f - 3.0).max(0.0))),
                1.0 / n_f,
            );
            updated_positions.push(new_v);
        }
    }

    // --- Assemble full vertex buffer ---
    // Layout: [updated original verts | face points | edge points]
    let mut all_positions: Vec<[f32; 3]> = updated_positions;
    // face points start at n_orig
    all_positions.extend_from_slice(&face_points);
    // edge points start at n_orig + n_faces
    all_positions.extend(edge_positions);

    let total_verts = all_positions.len();
    let all_uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; total_verts];

    // --- Step 5: New connectivity ---
    // Each triangle (a, b, c) → face point fp_abc, edge points ep_ab, ep_bc, ep_ca
    // Subdivide into 3 quads:
    //   quad1: a, ep_ab, fp_abc, ep_ca
    //   quad2: b, ep_bc, fp_abc, ep_ab
    //   quad3: c, ep_ca, fp_abc, ep_bc
    // Each quad → 2 triangles (gives 6 triangles per original triangle)
    let mut new_indices: Vec<u32> = Vec::with_capacity(n_faces * 6 * 3);

    for (fi, tri) in indices.chunks_exact(3).enumerate() {
        let (va, vb, vc) = (tri[0], tri[1], tri[2]);
        let fp = (n_orig + fi) as u32;
        let ep_ab = edge_to_idx[&canonical_edge(va, vb)];
        let ep_bc = edge_to_idx[&canonical_edge(vb, vc)];
        let ep_ca = edge_to_idx[&canonical_edge(vc, va)];

        // quad1: va, ep_ab, fp, ep_ca  → 2 tris
        new_indices.extend_from_slice(&[va, ep_ab, fp]);
        new_indices.extend_from_slice(&[va, fp, ep_ca]);

        // quad2: vb, ep_bc, fp, ep_ab  → 2 tris
        new_indices.extend_from_slice(&[vb, ep_bc, fp]);
        new_indices.extend_from_slice(&[vb, fp, ep_ab]);

        // quad3: vc, ep_ca, fp, ep_bc  → 2 tris
        new_indices.extend_from_slice(&[vc, ep_ca, fp]);
        new_indices.extend_from_slice(&[vc, fp, ep_bc]);
    }

    let mut result = MeshBuffers {
        positions: all_positions,
        normals: vec![[0.0, 1.0, 0.0]; total_verts],
        uvs: all_uvs,
        tangents: vec![],
        colors: None,
        indices: new_indices,
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut result);
    result
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for Catmull-Clark subdivision.
#[derive(Debug, Clone)]
pub struct CatmullClarkConfig {
    pub iterations: usize,
    /// Whether to fix boundary vertices (don't move them).
    pub fix_boundary: bool,
    /// Whether to recompute normals after subdivision.
    pub recompute_normals: bool,
}

impl Default for CatmullClarkConfig {
    fn default() -> Self {
        CatmullClarkConfig {
            iterations: 1,
            fix_boundary: false,
            recompute_normals: true,
        }
    }
}

impl CatmullClarkConfig {
    pub fn with_iterations(mut self, n: usize) -> Self {
        self.iterations = n;
        self
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Apply one iteration of Catmull-Clark subdivision to a triangulated mesh.
/// Input triangles are treated as degenerate quads.
/// Output is a triangulated mesh with approximately 6x the face count (3 quads × 2 tris each).
#[allow(dead_code)]
pub fn catmull_clark_subdivide(mesh: &MeshBuffers) -> MeshBuffers {
    catmull_clark_once(mesh, false)
}

/// Apply N iterations of Catmull-Clark subdivision.
#[allow(dead_code)]
pub fn catmull_clark_subdivide_n(mesh: &MeshBuffers, iterations: usize) -> MeshBuffers {
    let mut current = mesh.clone();
    for _ in 0..iterations {
        current = catmull_clark_once(&current, false);
    }
    current
}

/// Apply Catmull-Clark with configuration.
#[allow(dead_code)]
pub fn catmull_clark_with_config(mesh: &MeshBuffers, config: &CatmullClarkConfig) -> MeshBuffers {
    let mut current = mesh.clone();
    for _ in 0..config.iterations {
        current = catmull_clark_once(&current, config.fix_boundary);
    }
    if config.recompute_normals {
        compute_normals(&mut current);
    }
    current
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangles() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2, 1, 3, 2],
            has_suit: false,
        }
    }

    fn tetrahedron() -> MeshBuffers {
        // Regular tetrahedron centered roughly at origin
        let positions = vec![
            [1.0f32, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ];
        let indices = vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2];
        MeshBuffers {
            normals: vec![[0.0, 1.0, 0.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            tangents: vec![],
            colors: None,
            has_suit: false,
            positions,
            indices,
        }
    }

    #[test]
    fn catmull_clark_increases_face_count() {
        let mesh = two_triangles();
        let result = catmull_clark_subdivide(&mesh);
        assert!(
            result.face_count() > mesh.face_count(),
            "face count should increase: {} -> {}",
            mesh.face_count(),
            result.face_count()
        );
    }

    #[test]
    fn catmull_clark_positions_are_finite() {
        let mesh = two_triangles();
        let result = catmull_clark_subdivide(&mesh);
        for pos in &result.positions {
            assert!(pos[0].is_finite(), "position x is not finite: {:?}", pos);
            assert!(pos[1].is_finite(), "position y is not finite: {:?}", pos);
            assert!(pos[2].is_finite(), "position z is not finite: {:?}", pos);
        }
    }

    #[test]
    fn catmull_clark_vertex_count_increases() {
        let mesh = two_triangles();
        let result = catmull_clark_subdivide(&mesh);
        assert!(
            result.vertex_count() > mesh.vertex_count(),
            "vertex count should increase: {} -> {}",
            mesh.vertex_count(),
            result.vertex_count()
        );
    }

    #[test]
    fn catmull_clark_normals_valid_after_subdivision() {
        let mesh = two_triangles();
        let result = catmull_clark_subdivide(&mesh);
        assert_eq!(
            result.normals.len(),
            result.positions.len(),
            "normals count must match vertex count"
        );
        for n in &result.normals {
            let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
            assert!(
                (len_sq - 1.0).abs() < 0.01,
                "normal is not unit length: {:?}",
                n
            );
        }
    }

    #[test]
    fn catmull_clark_n_iterations_grows_mesh() {
        let mesh = two_triangles();
        let result1 = catmull_clark_subdivide_n(&mesh, 1);
        let result2 = catmull_clark_subdivide_n(&mesh, 2);
        assert!(
            result2.face_count() > result1.face_count(),
            "2 iterations should produce more faces than 1: {} vs {}",
            result2.face_count(),
            result1.face_count()
        );
    }

    #[test]
    fn catmull_clark_zero_iterations_unchanged() {
        let mesh = two_triangles();
        let result = catmull_clark_subdivide_n(&mesh, 0);
        assert_eq!(
            result.vertex_count(),
            mesh.vertex_count(),
            "zero iterations should not change vertex count"
        );
        assert_eq!(
            result.face_count(),
            mesh.face_count(),
            "zero iterations should not change face count"
        );
    }

    #[test]
    fn catmull_clark_config_default() {
        let config = CatmullClarkConfig::default();
        assert_eq!(config.iterations, 1);
        assert!(!config.fix_boundary);
        assert!(config.recompute_normals);
    }

    #[test]
    fn catmull_clark_with_config_one_iter() {
        let mesh = two_triangles();
        let config = CatmullClarkConfig::default().with_iterations(1);
        let result = catmull_clark_with_config(&mesh, &config);
        assert!(result.face_count() > mesh.face_count());
        assert_eq!(result.normals.len(), result.positions.len());
    }

    #[test]
    fn catmull_clark_tetrahedron_gets_rounder() {
        // After subdivision, vertices should generally move closer to the centroid
        let mesh = tetrahedron();
        // centroid of tetrahedron
        let centroid = [0.0f32, 0.0, 0.0];
        let avg_dist_before: f32 = mesh
            .positions
            .iter()
            .map(|p| {
                let dx = p[0] - centroid[0];
                let dy = p[1] - centroid[1];
                let dz = p[2] - centroid[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .sum::<f32>()
            / mesh.positions.len() as f32;

        let result = catmull_clark_subdivide(&mesh);
        let avg_dist_after: f32 = result
            .positions
            .iter()
            .map(|p| {
                let dx = p[0] - centroid[0];
                let dy = p[1] - centroid[1];
                let dz = p[2] - centroid[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .sum::<f32>()
            / result.positions.len() as f32;

        // After subdivision new face/edge points are introduced closer to center
        // so average distance should decrease
        assert!(
            avg_dist_after < avg_dist_before,
            "avg distance from centroid should decrease after subdivision: before={}, after={}",
            avg_dist_before,
            avg_dist_after
        );
    }

    #[test]
    fn catmull_clark_no_nan_positions() {
        let mesh = tetrahedron();
        let result = catmull_clark_subdivide_n(&mesh, 2);
        for pos in &result.positions {
            assert!(!pos[0].is_nan(), "NaN in position x");
            assert!(!pos[1].is_nan(), "NaN in position y");
            assert!(!pos[2].is_nan(), "NaN in position z");
        }
    }

    #[test]
    fn catmull_clark_indices_valid() {
        let mesh = two_triangles();
        let result = catmull_clark_subdivide(&mesh);
        let n = result.vertex_count() as u32;
        for &idx in &result.indices {
            assert!(idx < n, "index {} out of bounds (vertex count {})", idx, n);
        }
    }
}
