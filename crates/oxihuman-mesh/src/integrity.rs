// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh integrity and manifold checks.

use crate::mesh::MeshBuffers;
use std::collections::HashMap;

/// Result of an integrity check.
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub vertex_count: usize,
    pub face_count: usize,
    pub degenerate_faces: Vec<usize>, // face indices with zero area
    pub out_of_bounds_indices: Vec<usize>, // face indices referencing invalid verts
    /// Edges shared by more than 2 faces (true non-manifold; boundary edges are OK).
    pub non_manifold_edges: usize,
    pub has_nan_positions: bool,
    pub has_inf_positions: bool,
}

impl IntegrityReport {
    /// Returns true if the mesh passed all checks.
    pub fn is_valid(&self) -> bool {
        self.degenerate_faces.is_empty()
            && self.out_of_bounds_indices.is_empty()
            && self.non_manifold_edges == 0
            && !self.has_nan_positions
            && !self.has_inf_positions
    }

    /// Returns a human-readable summary.
    pub fn summary(&self) -> String {
        if self.is_valid() {
            format!("OK: {} verts, {} faces", self.vertex_count, self.face_count)
        } else {
            format!(
                "INVALID: {} degenerate, {} oob, {} non-manifold, nan={}, inf={}",
                self.degenerate_faces.len(),
                self.out_of_bounds_indices.len(),
                self.non_manifold_edges,
                self.has_nan_positions,
                self.has_inf_positions
            )
        }
    }
}

/// Run all integrity checks on a mesh and return a report.
pub fn check_integrity(buf: &MeshBuffers) -> IntegrityReport {
    let n = buf.positions.len();
    let face_count = buf.indices.len() / 3;

    // NaN / Inf check
    let mut has_nan = false;
    let mut has_inf = false;
    for pos in &buf.positions {
        for &c in pos {
            if c.is_nan() {
                has_nan = true;
            }
            if c.is_infinite() {
                has_inf = true;
            }
        }
    }

    let mut degenerate_faces = Vec::new();
    let mut out_of_bounds_indices = Vec::new();

    // Edge → face count map for manifold check.
    // Boundary edges (count == 1) are acceptable for open meshes.
    // Only edges with count > 2 are true non-manifold violations.
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();

    for (fi, tri) in buf.indices.chunks_exact(3).enumerate() {
        let (i0, i1, i2) = (tri[0], tri[1], tri[2]);

        // Index bounds check
        if i0 as usize >= n || i1 as usize >= n || i2 as usize >= n {
            out_of_bounds_indices.push(fi);
            continue;
        }

        // Degenerate triangle (zero area)
        let p0 = buf.positions[i0 as usize];
        let p1 = buf.positions[i1 as usize];
        let p2 = buf.positions[i2 as usize];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let cross_len_sq = {
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            cx * cx + cy * cy + cz * cz
        };
        if cross_len_sq < 1e-20 {
            degenerate_faces.push(fi);
        }

        // Manifold edge tracking (canonical: min,max)
        for &(a, b) in &[(i0, i1), (i1, i2), (i2, i0)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Only count edges shared by MORE than 2 faces as non-manifold.
    // Edges shared by exactly 1 face are boundary edges (valid for open meshes).
    let non_manifold_edges = edge_count.values().filter(|&&c| c > 2).count();

    IntegrityReport {
        vertex_count: n,
        face_count,
        degenerate_faces,
        out_of_bounds_indices,
        non_manifold_edges,
        has_nan_positions: has_nan,
        has_inf_positions: has_inf,
    }
}

/// Check index bounds only (fast path).
pub fn check_index_bounds(buf: &MeshBuffers) -> bool {
    let n = buf.positions.len() as u32;
    buf.indices.iter().all(|&i| i < n)
}

/// Check for NaN or Inf in vertex positions.
pub fn check_positions_finite(buf: &MeshBuffers) -> bool {
    buf.positions
        .iter()
        .all(|p| p.iter().all(|c| c.is_finite()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers as MyMesh;
    use oxihuman_morph::engine::MeshBuffers as MB;
    use proptest::prelude::*;

    fn valid_triangle() -> MyMesh {
        MyMesh::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn valid_mesh_passes() {
        let m = valid_triangle();
        let report = check_integrity(&m);
        assert!(report.is_valid(), "{}", report.summary());
        assert_eq!(report.face_count, 1);
        assert_eq!(report.vertex_count, 3);
    }

    #[test]
    fn out_of_bounds_index_detected() {
        let mut m = valid_triangle();
        m.indices.push(0);
        m.indices.push(1);
        m.indices.push(99); // out of bounds
        let report = check_integrity(&m);
        assert!(!report.out_of_bounds_indices.is_empty());
    }

    #[test]
    fn degenerate_triangle_detected() {
        // Collinear points
        let m = MyMesh::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0], // collinear!
            ],
            normals: vec![[0.0, 1.0, 0.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        });
        let report = check_integrity(&m);
        assert!(!report.degenerate_faces.is_empty());
    }

    #[test]
    fn nan_position_detected() {
        let m = MyMesh::from_morph(MB {
            positions: vec![[f32::NAN, 0.0, 0.0]],
            normals: vec![[0.0, 1.0, 0.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        });
        let report = check_integrity(&m);
        assert!(report.has_nan_positions);
        assert!(!report.is_valid());
    }

    #[test]
    fn index_bounds_fast_path() {
        let m = valid_triangle();
        assert!(check_index_bounds(&m));
    }

    #[test]
    fn finite_positions_check() {
        let m = valid_triangle();
        assert!(check_positions_finite(&m));
    }

    #[test]
    fn non_manifold_edge_detected() {
        // Three triangles sharing the same edge (i0=0, i1=1) → count=3, non-manifold.
        let m = MyMesh::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.5, 1.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 5],
            uvs: vec![[0.0, 0.0]; 5],
            // Edge (0,1) appears in all three triangles → count 3 → non-manifold
            indices: vec![0, 1, 2, 0, 1, 3, 0, 1, 4],
            has_suit: false,
        });
        let report = check_integrity(&m);
        assert!(report.non_manifold_edges > 0, "expected non-manifold edge");
        assert!(!report.is_valid());
    }

    #[test]
    fn real_base_mesh_integrity() {
        use crate::normals::compute_normals;
        use crate::suit::apply_suit_flag;
        use oxihuman_core::parser::obj::parse_obj;
        let path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
        if let Ok(src) = std::fs::read_to_string(path) {
            if let Ok(obj) = parse_obj(&src) {
                let morph_buf = oxihuman_morph::engine::MeshBuffers {
                    positions: obj.positions,
                    normals: obj.normals,
                    uvs: obj.uvs,
                    indices: obj.indices,
                    has_suit: false,
                };
                let mut mesh = MyMesh::from_morph(morph_buf);
                compute_normals(&mut mesh);
                apply_suit_flag(&mut mesh);
                let report = check_integrity(&mesh);
                assert!(
                    check_index_bounds(&mesh),
                    "index out of bounds in real mesh"
                );
                assert!(check_positions_finite(&mesh), "non-finite in real mesh");
                assert!(report.out_of_bounds_indices.is_empty());
                assert!(!report.has_nan_positions);
            }
        }
    }

    proptest! {
        #[test]
        fn random_triangles_no_false_oob(
            i0 in 0u32..100u32,
            i1 in 0u32..100u32,
            i2 in 0u32..100u32,
            n_verts in 100usize..200usize,
        ) {
            let positions = vec![[0.0f32, 0.0, 0.0]; n_verts];
            let mesh = MyMesh::from_morph(MB {
                positions,
                normals: vec![[0.0, 1.0, 0.0]; n_verts],
                uvs: vec![[0.0, 0.0]; n_verts],
                indices: vec![i0 % n_verts as u32, i1 % n_verts as u32, i2 % n_verts as u32],
                has_suit: false,
            });
            // Indices are mod n_verts so always in bounds
            prop_assert!(check_index_bounds(&mesh));
        }

        #[test]
        fn finite_positions_always_pass_finite_check(
            x in -1000.0f32..1000.0f32,
            y in -1000.0f32..1000.0f32,
            z in -1000.0f32..1000.0f32,
        ) {
            let mesh = MyMesh::from_morph(MB {
                positions: vec![[x, y, z]],
                normals: vec![[0.0, 1.0, 0.0]],
                uvs: vec![[0.0, 0.0]],
                indices: vec![],
                has_suit: false,
            });
            prop_assert!(check_positions_finite(&mesh));
        }
    }
}
