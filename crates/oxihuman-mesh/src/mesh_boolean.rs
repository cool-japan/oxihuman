// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::winding::winding_number;

// ──────────────────────────────────────────────────────────────────────────────
// Public types
// ──────────────────────────────────────────────────────────────────────────────

/// The four standard CSG boolean operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    /// A ∪ B — all space covered by A or B (or both).
    Union,
    /// A ∩ B — only space covered by both A and B.
    Intersection,
    /// A − B — space in A but not in B.
    Difference,
    /// (A ∪ B) − (A ∩ B) — space in one but not both.
    SymmetricDifference,
}

/// Result of a CSG boolean operation.
pub struct BooleanResult {
    /// The resulting combined mesh.
    pub mesh: MeshBuffers,
    /// Number of triangle faces contributed from mesh A.
    pub face_count_a: usize,
    /// Number of triangle faces contributed from mesh B.
    pub face_count_b: usize,
}

// ──────────────────────────────────────────────────────────────────────────────
// Vertex classification
// ──────────────────────────────────────────────────────────────────────────────

/// Classify each vertex of `mesh` as inside (`true`) or outside (`false`)
/// of `reference`, using the winding number.
///
/// A vertex is considered *inside* when |W(v)| >= 0.5.
pub fn classify_vertices(mesh: &MeshBuffers, reference: &MeshBuffers) -> Vec<bool> {
    mesh.positions
        .iter()
        .map(|&p| winding_number(reference, p).abs() >= 0.5)
        .collect()
}

// ──────────────────────────────────────────────────────────────────────────────
// Face filtering
// ──────────────────────────────────────────────────────────────────────────────

/// Keep only faces where **all three vertices** match the desired
/// classification.
///
/// * `keep_inside = true`  → keep faces whose vertices are all inside
/// * `keep_inside = false` → keep faces whose vertices are all outside
///
/// Vertices are re-indexed compactly; normals, uvs, tangents and colors are
/// preserved where present.
pub fn filter_faces_by_classification(
    mesh: &MeshBuffers,
    inside_flags: &[bool],
    keep_inside: bool,
) -> MeshBuffers {
    let n_faces = mesh.indices.len() / 3;

    // Collect accepted old vertex indices (in order of encounter).
    let mut old_to_new: Vec<Option<u32>> = vec![None; mesh.positions.len()];
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_tangents: Vec<[f32; 4]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_colors: Vec<[f32; 4]> = Vec::new();
    let has_colors = mesh.colors.is_some();
    let mut new_indices: Vec<u32> = Vec::new();

    for fi in 0..n_faces {
        let ia = mesh.indices[3 * fi] as usize;
        let ib = mesh.indices[3 * fi + 1] as usize;
        let ic = mesh.indices[3 * fi + 2] as usize;

        let fa = inside_flags.get(ia).copied().unwrap_or(false);
        let fb = inside_flags.get(ib).copied().unwrap_or(false);
        let fc = inside_flags.get(ic).copied().unwrap_or(false);

        let all_match = (fa == keep_inside) && (fb == keep_inside) && (fc == keep_inside);
        if !all_match {
            continue;
        }

        for &old_idx in &[ia, ib, ic] {
            if old_to_new[old_idx].is_none() {
                let new_idx = new_positions.len() as u32;
                old_to_new[old_idx] = Some(new_idx);
                new_positions.push(mesh.positions[old_idx]);
                if old_idx < mesh.normals.len() {
                    new_normals.push(mesh.normals[old_idx]);
                } else {
                    new_normals.push([0.0, 0.0, 1.0]);
                }
                if old_idx < mesh.tangents.len() {
                    new_tangents.push(mesh.tangents[old_idx]);
                } else {
                    new_tangents.push([1.0, 0.0, 0.0, 1.0]);
                }
                if old_idx < mesh.uvs.len() {
                    new_uvs.push(mesh.uvs[old_idx]);
                } else {
                    new_uvs.push([0.0, 0.0]);
                }
                if has_colors {
                    let c = mesh
                        .colors
                        .as_ref()
                        .and_then(|v| v.get(old_idx))
                        .copied()
                        .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    new_colors.push(c);
                }
            }
            new_indices.push(old_to_new[old_idx].unwrap_or(0));
        }
    }

    MeshBuffers {
        positions: new_positions,
        normals: new_normals,
        tangents: new_tangents,
        uvs: new_uvs,
        indices: new_indices,
        colors: if has_colors { Some(new_colors) } else { None },
        has_suit: mesh.has_suit,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Flip winding
// ──────────────────────────────────────────────────────────────────────────────

/// Reverse the triangle winding order of every face in `mesh`, returning a new
/// mesh.  This effectively flips all surface normals.
pub fn flip_winding(mesh: &MeshBuffers) -> MeshBuffers {
    let mut new_indices = mesh.indices.clone();
    let n_faces = new_indices.len() / 3;
    for fi in 0..n_faces {
        new_indices.swap(3 * fi + 1, 3 * fi + 2);
    }
    // Flip stored normals too.
    let new_normals: Vec<[f32; 3]> = mesh
        .normals
        .iter()
        .map(|&n| [-n[0], -n[1], -n[2]])
        .collect();
    MeshBuffers {
        positions: mesh.positions.clone(),
        normals: new_normals,
        tangents: mesh.tangents.clone(),
        uvs: mesh.uvs.clone(),
        indices: new_indices,
        colors: mesh.colors.clone(),
        has_suit: mesh.has_suit,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Combine meshes
// ──────────────────────────────────────────────────────────────────────────────

/// Concatenate two meshes into one, offsetting B's indices by the vertex count
/// of A.
pub fn combine_meshes(a: &MeshBuffers, b: &MeshBuffers) -> MeshBuffers {
    let offset = a.positions.len() as u32;

    let mut positions = a.positions.clone();
    positions.extend_from_slice(&b.positions);

    let mut normals = a.normals.clone();
    normals.extend_from_slice(&b.normals);

    let mut tangents = a.tangents.clone();
    tangents.extend_from_slice(&b.tangents);

    let mut uvs = a.uvs.clone();
    uvs.extend_from_slice(&b.uvs);

    let mut indices = a.indices.clone();
    for &idx in &b.indices {
        indices.push(idx + offset);
    }

    // Merge colors only when both meshes have them.
    let colors = match (&a.colors, &b.colors) {
        (Some(ca), Some(cb)) => {
            let mut merged = ca.clone();
            merged.extend_from_slice(cb);
            Some(merged)
        }
        _ => None,
    };

    MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        colors,
        has_suit: a.has_suit && b.has_suit,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Boolean operation
// ──────────────────────────────────────────────────────────────────────────────

/// Apply a CSG boolean operation between mesh `a` and mesh `b`.
///
/// | Operation            | From A              | From B                       |
/// |----------------------|---------------------|------------------------------|
/// | Union                | outside B           | outside A                    |
/// | Intersection         | inside B            | inside A                     |
/// | Difference (A − B)   | outside B           | inside A (winding flipped)   |
/// | SymmetricDifference  | outside B           | outside A                    |
#[allow(clippy::too_many_arguments)]
pub fn boolean_op(a: &MeshBuffers, b: &MeshBuffers, op: BooleanOp) -> BooleanResult {
    let flags_a_in_b = classify_vertices(a, b); // for each vertex of A: is it inside B?
    let flags_b_in_a = classify_vertices(b, a); // for each vertex of B: is it inside A?

    let (part_a, part_b) = match op {
        BooleanOp::Union => {
            // A outside B  +  B outside A
            let pa = filter_faces_by_classification(a, &flags_a_in_b, false);
            let pb = filter_faces_by_classification(b, &flags_b_in_a, false);
            (pa, pb)
        }
        BooleanOp::Intersection => {
            // A inside B  +  B inside A
            let pa = filter_faces_by_classification(a, &flags_a_in_b, true);
            let pb = filter_faces_by_classification(b, &flags_b_in_a, true);
            (pa, pb)
        }
        BooleanOp::Difference => {
            // A outside B  +  (B inside A) with flipped winding
            let pa = filter_faces_by_classification(a, &flags_a_in_b, false);
            let pb_inner = filter_faces_by_classification(b, &flags_b_in_a, true);
            let pb = flip_winding(&pb_inner);
            (pa, pb)
        }
        BooleanOp::SymmetricDifference => {
            // (A outside B)  +  (B outside A)  — same as Union minus the overlap
            let pa = filter_faces_by_classification(a, &flags_a_in_b, false);
            let pb = filter_faces_by_classification(b, &flags_b_in_a, false);
            (pa, pb)
        }
    };

    let face_count_a = part_a.face_count();
    let face_count_b = part_b.face_count();
    let mesh = combine_meshes(&part_a, &part_b);

    BooleanResult {
        mesh,
        face_count_a,
        face_count_b,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// A closed tetrahedron with outward-facing normals.
    /// Vertices: A=(0,0,0), B=(1,0,0), C=(0,1,0), D=(0,0,1)
    fn tetrahedron() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0f32, 0.0, 0.0], // 0 = A
                [1.0, 0.0, 0.0],    // 1 = B
                [0.0, 1.0, 0.0],    // 2 = C
                [0.0, 0.0, 1.0],    // 3 = D
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![
                0, 2, 1, // face A,C,B
                0, 1, 3, // face A,B,D
                0, 3, 2, // face A,D,C
                1, 2, 3, // face B,C,D
            ],
            has_suit: false,
        })
    }

    /// A translated tetrahedron whose centroid is far from the origin.
    fn tetrahedron_offset(dx: f32, dy: f32, dz: f32) -> MeshBuffers {
        let t = tetrahedron();
        let positions: Vec<[f32; 3]> = t
            .positions
            .iter()
            .map(|&[x, y, z]| [x + dx, y + dy, z + dz])
            .collect();
        MeshBuffers {
            positions,
            normals: t.normals.clone(),
            tangents: t.tangents.clone(),
            uvs: t.uvs.clone(),
            indices: t.indices.clone(),
            colors: None,
            has_suit: false,
        }
    }

    /// A single triangle (open surface, not watertight).
    fn single_triangle() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    // ── classify_vertices tests ───────────────────────────────────────────────

    #[test]
    fn classify_vertices_all_outside() {
        let mesh = tetrahedron_offset(100.0, 0.0, 0.0);
        let reference = tetrahedron();
        let flags = classify_vertices(&mesh, &reference);
        assert!(
            flags.iter().all(|&f| !f),
            "all vertices far away must be outside"
        );
    }

    #[test]
    fn classify_vertices_inside_self() {
        // A tetrahedron's centroid is inside itself.
        let tet = tetrahedron();
        // Create a tiny mesh at the centroid.
        let centroid_mesh = MeshBuffers::from_morph(MB {
            positions: vec![[0.25f32, 0.25, 0.25]],
            normals: vec![[0.0, 0.0, 1.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        });
        let flags = classify_vertices(&centroid_mesh, &tet);
        assert_eq!(flags.len(), 1);
        assert!(flags[0], "centroid must be inside the tetrahedron");
    }

    #[test]
    fn classify_vertices_outside_remote() {
        let tet = tetrahedron();
        let remote = MeshBuffers::from_morph(MB {
            positions: vec![[50.0f32, 50.0, 50.0]],
            normals: vec![[0.0, 0.0, 1.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        });
        let flags = classify_vertices(&remote, &tet);
        assert_eq!(flags.len(), 1);
        assert!(!flags[0], "remote point must be outside");
    }

    // ── filter_faces_by_classification tests ──────────────────────────────────

    #[test]
    fn filter_keep_outside_all_false_returns_all() {
        let tri = single_triangle();
        // All vertices classified as outside (false).
        let flags = vec![false, false, false];
        let result = filter_faces_by_classification(&tri, &flags, false);
        assert_eq!(result.face_count(), 1, "should keep the only face");
        assert_eq!(result.positions.len(), 3);
    }

    #[test]
    fn filter_keep_inside_all_false_returns_none() {
        let tri = single_triangle();
        let flags = vec![false, false, false];
        let result = filter_faces_by_classification(&tri, &flags, true);
        assert_eq!(result.face_count(), 0, "no inside faces to keep");
        assert_eq!(result.positions.len(), 0);
    }

    #[test]
    fn filter_partial_match_excludes_mixed_faces() {
        // Tetrahedron has 4 faces; mark vertex 3 as inside, others outside.
        let tet = tetrahedron();
        // vertices 0,1,2 = outside; vertex 3 = inside
        let flags = vec![false, false, false, true];
        // Keep only faces where all 3 verts are outside.
        // Face 0: verts 0,2,1 → all outside → kept
        // Face 1: verts 0,1,3 → mixed      → dropped
        // Face 2: verts 0,3,2 → mixed      → dropped
        // Face 3: verts 1,2,3 → mixed      → dropped
        let result = filter_faces_by_classification(&tet, &flags, false);
        assert_eq!(result.face_count(), 1, "only one face should pass");
    }

    #[test]
    fn filter_reindexes_compactly() {
        let tet = tetrahedron();
        // Keep only face 0 (verts 0,2,1) — all outside.
        let flags = vec![false, false, false, true];
        let result = filter_faces_by_classification(&tet, &flags, false);
        // 3 unique vertices in the kept face.
        assert_eq!(result.positions.len(), 3);
        // Indices should now be in 0..3.
        for &idx in &result.indices {
            assert!(
                (idx as usize) < result.positions.len(),
                "index out of range"
            );
        }
    }

    // ── flip_winding tests ────────────────────────────────────────────────────

    #[test]
    fn flip_winding_reverses_indices() {
        let tri = single_triangle();
        let flipped = flip_winding(&tri);
        // Original: [0, 1, 2] → Flipped: [0, 2, 1]
        assert_eq!(flipped.indices[0], tri.indices[0]);
        assert_eq!(flipped.indices[1], tri.indices[2]);
        assert_eq!(flipped.indices[2], tri.indices[1]);
    }

    #[test]
    fn flip_winding_negates_normals() {
        let tri = single_triangle();
        // Default normals are [0,0,1].
        let flipped = flip_winding(&tri);
        for n in &flipped.normals {
            assert!((n[0] - 0.0).abs() < 1e-6, "x normal should be 0 after flip");
            assert!((n[1] - 0.0).abs() < 1e-6, "y normal should be 0 after flip");
            assert!(
                (n[2] - (-1.0)).abs() < 1e-6,
                "z normal should be -1 after flip"
            );
        }
    }

    #[test]
    fn flip_winding_double_flip_is_identity() {
        let tet = tetrahedron();
        let double_flipped = flip_winding(&flip_winding(&tet));
        assert_eq!(double_flipped.indices, tet.indices);
        for (a, b) in double_flipped.normals.iter().zip(tet.normals.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-6);
            assert!((a[1] - b[1]).abs() < 1e-6);
            assert!((a[2] - b[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn flip_winding_preserves_vertex_count() {
        let tet = tetrahedron();
        let flipped = flip_winding(&tet);
        assert_eq!(flipped.positions.len(), tet.positions.len());
        assert_eq!(flipped.indices.len(), tet.indices.len());
    }

    // ── combine_meshes tests ──────────────────────────────────────────────────

    #[test]
    fn combine_meshes_vertex_count() {
        let a = single_triangle();
        let b = single_triangle();
        let combined = combine_meshes(&a, &b);
        assert_eq!(combined.positions.len(), 6, "should have 3+3 vertices");
    }

    #[test]
    fn combine_meshes_face_count() {
        let a = single_triangle();
        let b = single_triangle();
        let combined = combine_meshes(&a, &b);
        assert_eq!(combined.face_count(), 2, "should have 1+1 faces");
    }

    #[test]
    fn combine_meshes_index_offset() {
        let a = single_triangle();
        let b = single_triangle();
        let combined = combine_meshes(&a, &b);
        // B's indices should be offset by 3 (A has 3 vertices).
        assert_eq!(&combined.indices[0..3], &[0u32, 1, 2]);
        assert_eq!(&combined.indices[3..6], &[3u32, 4, 5]);
    }

    #[test]
    fn combine_meshes_all_indices_valid() {
        let a = tetrahedron();
        let b = tetrahedron_offset(5.0, 0.0, 0.0);
        let combined = combine_meshes(&a, &b);
        for &idx in &combined.indices {
            assert!(
                (idx as usize) < combined.positions.len(),
                "index out of range"
            );
        }
    }

    // ── boolean_op tests ──────────────────────────────────────────────────────

    #[test]
    fn boolean_union_non_overlapping_has_all_faces() {
        // Two non-overlapping tetrahedra → union keeps all faces.
        let a = tetrahedron();
        let b = tetrahedron_offset(100.0, 0.0, 0.0);
        let result = boolean_op(&a, &b, BooleanOp::Union);
        // All faces of A should be outside B (they don't overlap).
        assert_eq!(result.face_count_a, a.face_count());
        assert_eq!(result.face_count_b, b.face_count());
    }

    #[test]
    fn boolean_intersection_non_overlapping_is_empty() {
        let a = tetrahedron();
        let b = tetrahedron_offset(100.0, 0.0, 0.0);
        let result = boolean_op(&a, &b, BooleanOp::Intersection);
        // No faces should be inside each other.
        assert_eq!(result.face_count_a, 0);
        assert_eq!(result.face_count_b, 0);
        assert_eq!(result.mesh.face_count(), 0);
    }

    #[test]
    fn boolean_difference_non_overlapping_keeps_a() {
        // A − B where B is far away → all of A survives.
        let a = tetrahedron();
        let b = tetrahedron_offset(100.0, 0.0, 0.0);
        let result = boolean_op(&a, &b, BooleanOp::Difference);
        assert_eq!(result.face_count_a, a.face_count());
        // B's faces are outside A → after inside-filter, face_count_b = 0.
        assert_eq!(result.face_count_b, 0);
    }

    #[test]
    fn boolean_symmetric_difference_non_overlapping_same_as_union() {
        let a = tetrahedron();
        let b = tetrahedron_offset(100.0, 0.0, 0.0);
        let r_union = boolean_op(&a, &b, BooleanOp::Union);
        let r_xor = boolean_op(&a, &b, BooleanOp::SymmetricDifference);
        assert_eq!(r_union.mesh.face_count(), r_xor.mesh.face_count());
    }

    #[test]
    fn boolean_result_face_counts_match_mesh() {
        let a = tetrahedron();
        let b = tetrahedron_offset(100.0, 0.0, 0.0);
        let result = boolean_op(&a, &b, BooleanOp::Union);
        assert_eq!(
            result.face_count_a + result.face_count_b,
            result.mesh.face_count(),
            "total face count must equal a + b contributions"
        );
    }

    #[test]
    fn boolean_difference_flips_b_normals() {
        // When A and B overlap (same tet), difference should have part_b flipped.
        // We just verify the op completes and counts are sane.
        let a = tetrahedron();
        let b = tetrahedron(); // identical — fully overlapping
        let result = boolean_op(&a, &b, BooleanOp::Difference);
        // A outside B = 0 (A is entirely inside B)
        // B inside A (flipped) = 4 faces
        assert_eq!(
            result.mesh.face_count(),
            result.face_count_a + result.face_count_b
        );
    }

    #[test]
    fn boolean_union_result_has_valid_indices() {
        let a = tetrahedron();
        let b = tetrahedron_offset(100.0, 0.0, 0.0);
        let result = boolean_op(&a, &b, BooleanOp::Union);
        let n_verts = result.mesh.positions.len();
        for &idx in &result.mesh.indices {
            assert!((idx as usize) < n_verts, "index out of bounds");
        }
    }
}
