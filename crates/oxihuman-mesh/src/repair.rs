// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::mesh::MeshBuffers;

/// Report of mesh issues found.
#[derive(Debug, Clone, Default)]
pub struct MeshRepairReport {
    pub degenerate_faces_removed: usize,
    pub duplicate_faces_removed: usize,
    pub out_of_range_indices_fixed: usize,
    pub zero_length_edges_found: usize,
}

/// Remove degenerate triangles (zero area, or two/three identical vertex indices).
/// Returns the number removed.
#[allow(dead_code)]
pub fn remove_degenerate_faces(mesh: &mut MeshBuffers) -> usize {
    let positions = &mesh.positions;
    let original_count = mesh.indices.len() / 3;

    let mut kept: Vec<u32> = Vec::with_capacity(mesh.indices.len());

    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);

        // Degenerate: repeated index
        if a == b || b == c || a == c {
            continue;
        }

        // Degenerate: zero area (cross product length < 1e-10)
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];

        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];

        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];

        let cross_len_sq =
            (cross[0] as f64).powi(2) + (cross[1] as f64).powi(2) + (cross[2] as f64).powi(2);

        if cross_len_sq < 1e-20 {
            continue;
        }

        kept.push(tri[0]);
        kept.push(tri[1]);
        kept.push(tri[2]);
    }

    let removed = original_count - kept.len() / 3;
    mesh.indices = kept;
    removed
}

/// Remove duplicate faces (triangles referencing the same set of vertices, regardless of order).
/// Returns the number removed.
#[allow(dead_code)]
pub fn remove_duplicate_faces(mesh: &mut MeshBuffers) -> usize {
    let original_count = mesh.indices.len() / 3;
    let mut seen: HashSet<(u32, u32, u32)> = HashSet::new();
    let mut kept: Vec<u32> = Vec::with_capacity(mesh.indices.len());

    for tri in mesh.indices.chunks_exact(3) {
        let mut sorted = [tri[0], tri[1], tri[2]];
        sorted.sort_unstable();
        let key = (sorted[0], sorted[1], sorted[2]);

        if seen.insert(key) {
            kept.push(tri[0]);
            kept.push(tri[1]);
            kept.push(tri[2]);
        }
    }

    let removed = original_count - kept.len() / 3;
    mesh.indices = kept;
    removed
}

/// Clamp any out-of-range indices to the last valid vertex index.
/// Returns the number of indices fixed.
#[allow(dead_code)]
pub fn fix_out_of_range_indices(mesh: &mut MeshBuffers) -> usize {
    if mesh.positions.is_empty() {
        return 0;
    }
    let max_valid = (mesh.positions.len() - 1) as u32;
    let mut fixed = 0usize;

    for idx in mesh.indices.iter_mut() {
        if *idx > max_valid {
            *idx = max_valid;
            fixed += 1;
        }
    }
    fixed
}

/// Flip the winding order of all faces (reverses normal direction).
#[allow(dead_code)]
pub fn flip_winding(mesh: &mut MeshBuffers) {
    for tri in mesh.indices.chunks_exact_mut(3) {
        tri.swap(1, 2);
    }
}

/// Flip the winding order of specific faces by their face index (into indices[]).
#[allow(dead_code)]
pub fn flip_face_winding(mesh: &mut MeshBuffers, face_indices: &[usize]) {
    let total_faces = mesh.indices.len() / 3;
    for &fi in face_indices {
        if fi >= total_faces {
            continue;
        }
        let base = fi * 3;
        mesh.indices.swap(base + 1, base + 2);
    }
}

/// Count edges that connect vertices with near-zero distance (< epsilon).
#[allow(dead_code)]
pub fn count_zero_length_edges(mesh: &MeshBuffers, epsilon: f32) -> usize {
    let eps_sq = epsilon * epsilon;
    let mut count = 0usize;

    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);

        if a >= mesh.positions.len() || b >= mesh.positions.len() || c >= mesh.positions.len() {
            continue;
        }

        let pa = mesh.positions[a];
        let pb = mesh.positions[b];
        let pc = mesh.positions[c];

        let dist_sq = |p: [f32; 3], q: [f32; 3]| -> f32 {
            (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)
        };

        if dist_sq(pa, pb) < eps_sq {
            count += 1;
        }
        if dist_sq(pb, pc) < eps_sq {
            count += 1;
        }
        if dist_sq(pa, pc) < eps_sq {
            count += 1;
        }
    }

    count
}

/// Run all repair operations and return a combined report.
#[allow(dead_code)]
pub fn repair_mesh(mesh: &mut MeshBuffers) -> MeshRepairReport {
    let out_of_range_indices_fixed = fix_out_of_range_indices(mesh);
    let degenerate_faces_removed = remove_degenerate_faces(mesh);
    let duplicate_faces_removed = remove_duplicate_faces(mesh);
    let zero_length_edges_found = count_zero_length_edges(mesh, 1e-6);

    MeshRepairReport {
        degenerate_faces_removed,
        duplicate_faces_removed,
        out_of_range_indices_fixed,
        zero_length_edges_found,
    }
}

/// Check if all indices are within the valid vertex range.
#[allow(dead_code)]
pub fn has_valid_indices(mesh: &MeshBuffers) -> bool {
    if mesh.positions.is_empty() {
        return mesh.indices.is_empty();
    }
    let max_valid = (mesh.positions.len() - 1) as u32;
    mesh.indices.iter().all(|&i| i <= max_valid)
}

/// Check if mesh has any degenerate triangles.
#[allow(dead_code)]
pub fn has_degenerate_faces(mesh: &MeshBuffers) -> bool {
    let positions = &mesh.positions;

    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);

        if a == b || b == c || a == c {
            return true;
        }

        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }

        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];

        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];

        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];

        let cross_len_sq =
            (cross[0] as f64).powi(2) + (cross[1] as f64).powi(2) + (cross[2] as f64).powi(2);

        if cross_len_sq < 1e-20 {
            return true;
        }
    }

    false
}

/// Ensure index count is a multiple of 3 (trim if not).
#[allow(dead_code)]
pub fn ensure_complete_triangles(mesh: &mut MeshBuffers) {
    let remainder = mesh.indices.len() % 3;
    if remainder != 0 {
        let new_len = mesh.indices.len() - remainder;
        mesh.indices.truncate(new_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0f32, 0.0, 1.0]; n],
            uvs: vec![[0.0f32, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    fn triangle_mesh() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn remove_degenerate_same_index_triangle() {
        // Triangle with two identical indices: degenerate
        let mut mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 0, 2, 0, 1, 2], // first tri is degenerate (a==b)
        );
        let removed = remove_degenerate_faces(&mut mesh);
        assert_eq!(removed, 1);
        assert_eq!(mesh.indices, vec![0, 1, 2]);
    }

    #[test]
    fn remove_degenerate_zero_area_triangle() {
        // Collinear points => zero area
        let mut mesh = make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0], // collinear with first two
                [0.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 0, 1, 3], // first tri is zero area
        );
        let removed = remove_degenerate_faces(&mut mesh);
        assert_eq!(removed, 1);
        assert_eq!(mesh.indices, vec![0, 1, 3]);
    }

    #[test]
    fn remove_duplicate_faces_deduplicates() {
        let mut mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            // Same triangle twice, second with different winding
            vec![0, 1, 2, 2, 1, 0],
        );
        let removed = remove_duplicate_faces(&mut mesh);
        assert_eq!(removed, 1);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn remove_duplicate_faces_keeps_uniques() {
        let mut mesh = make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 1, 3, 2],
        );
        let removed = remove_duplicate_faces(&mut mesh);
        assert_eq!(removed, 0);
        assert_eq!(mesh.indices.len(), 6);
    }

    #[test]
    fn fix_out_of_range_clamps() {
        let mut mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 99], // index 99 is out of range, max valid is 2
        );
        let fixed = fix_out_of_range_indices(&mut mesh);
        assert_eq!(fixed, 1);
        assert_eq!(mesh.indices[2], 2); // clamped to last valid index
    }

    #[test]
    fn has_valid_indices_true_for_clean_mesh() {
        let mesh = triangle_mesh();
        assert!(has_valid_indices(&mesh));
    }

    #[test]
    fn has_valid_indices_false_for_bad_index() {
        let mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 100],
        );
        assert!(!has_valid_indices(&mesh));
    }

    #[test]
    fn flip_winding_reverses_order() {
        let mut mesh = triangle_mesh();
        flip_winding(&mut mesh);
        assert_eq!(mesh.indices, vec![0, 2, 1]);
    }

    #[test]
    fn repair_mesh_cleans_degenerate() {
        // One valid triangle, one degenerate (same index)
        let mut mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2, 0, 0, 2],
        );
        let report = repair_mesh(&mut mesh);
        assert_eq!(report.degenerate_faces_removed, 1);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn ensure_complete_triangles_trims_partial() {
        let mut mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2, 0, 1], // 5 indices: not a multiple of 3
        );
        ensure_complete_triangles(&mut mesh);
        assert_eq!(mesh.indices.len(), 3);
        assert_eq!(&mesh.indices[..], &[0u32, 1, 2]);
    }

    #[test]
    fn has_degenerate_faces_detects_bad() {
        let mesh = make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 0, 2], // degenerate: a==b
        );
        assert!(has_degenerate_faces(&mesh));
    }

    #[test]
    fn count_zero_length_edges_zero_for_clean() {
        let mesh = triangle_mesh();
        let count = count_zero_length_edges(&mesh, 1e-6);
        assert_eq!(count, 0);
    }

    #[test]
    fn repair_report_counts_correctly() {
        // Two degenerate faces, one duplicate, one out-of-range index
        let mut mesh = make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![
                0, 1, 2, // valid unique
                1, 3, 2, // valid unique
                0, 0, 2, // degenerate (a==b)
                1, 3, 2, // duplicate of second tri
                0, 1, 99, // out-of-range index (will be clamped first)
            ],
        );
        let report = repair_mesh(&mut mesh);
        // fix_out_of_range runs first: index 99 clamped to 3 (last valid = positions.len()-1 = 3)
        assert_eq!(report.out_of_range_indices_fixed, 1);
        // degenerate: one face with a==b
        assert_eq!(report.degenerate_faces_removed, 1);
        // duplicate: tri [1,3,2] appears twice
        assert_eq!(report.duplicate_faces_removed, 1);
    }
}
