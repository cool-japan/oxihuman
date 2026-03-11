// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Property-based tests for oxihuman-mesh: normals, decimation, UV operations.

use proptest::prelude::*;

use oxihuman_mesh::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a valid triangle mesh from random vertices and triangle indices.
/// Generates `num_verts` random positions and `num_tris` random triangles
/// referencing valid vertex indices.
fn mesh_strategy(max_verts: usize, max_tris: usize) -> impl Strategy<Value = MeshBuffers> {
    (3..max_verts, 1..max_tris).prop_flat_map(move |(nv, nt)| {
        let positions = proptest::collection::vec(prop::array::uniform3(-10.0f32..10.0f32), nv);
        let indices = proptest::collection::vec((0..nv).prop_map(|i| i as u32), nt * 3);
        (positions, indices).prop_map(|(positions, indices)| {
            let normals = vec![[0.0f32, 1.0, 0.0]; positions.len()];
            let uvs = vec![[0.0f32, 0.0]; positions.len()];
            let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; positions.len()];
            MeshBuffers {
                positions,
                normals,
                tangents,
                uvs,
                indices,
                colors: None,
                has_suit: false,
            }
        })
    })
}

// ---------------------------------------------------------------------------
// Random valid triangle meshes always produce valid normals (no NaN)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn normals_never_nan(mesh in mesh_strategy(64, 32)) {
        let mut m = mesh;
        oxihuman_mesh::normals::compute_normals(&mut m);

        for (i, normal) in m.normals.iter().enumerate() {
            prop_assert!(!normal[0].is_nan(), "Normal X is NaN at vertex {}", i);
            prop_assert!(!normal[1].is_nan(), "Normal Y is NaN at vertex {}", i);
            prop_assert!(!normal[2].is_nan(), "Normal Z is NaN at vertex {}", i);
        }
    }

    #[test]
    fn normals_are_unit_length_or_default(mesh in mesh_strategy(64, 32)) {
        let mut m = mesh;
        oxihuman_mesh::normals::compute_normals(&mut m);

        for (i, normal) in m.normals.iter().enumerate() {
            let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            // Normals should be approximately unit length (default fallback is [0,1,0])
            prop_assert!(
                (len - 1.0).abs() < 1e-4,
                "Normal at vertex {} has non-unit length: {} (normal = {:?})",
                i, len, normal
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Mesh decimation never increases face count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn decimation_never_increases_faces(
        mesh in mesh_strategy(50, 20),
        target_ratio in 0.1f32..1.0,
    ) {
        let original_faces = mesh.face_count();
        let target_faces = ((original_faces as f32 * target_ratio) as usize).max(1);
        let decimated = oxihuman_mesh::decimate::decimate(&mesh, target_faces);
        let result_faces = decimated.face_count();

        prop_assert!(
            result_faces <= original_faces,
            "Decimation increased face count from {} to {} (target was {})",
            original_faces, result_faces, target_faces,
        );
    }

    #[test]
    fn decimation_with_zero_target_never_panics(mesh in mesh_strategy(30, 15)) {
        // Requesting 0 faces should not panic
        let decimated = oxihuman_mesh::decimate::decimate(&mesh, 0);
        let _ = decimated.face_count();
    }
}

// ---------------------------------------------------------------------------
// UV coordinates stay in [0, 1] range after UV operations
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn uv_projection_stays_in_unit_range(mesh in mesh_strategy(32, 16)) {
        // Test all projection methods
        let projections = [
            oxihuman_mesh::uvgen::UvProjection::Cylindrical,
            oxihuman_mesh::uvgen::UvProjection::Spherical,
            oxihuman_mesh::uvgen::UvProjection::PlanarTop,
            oxihuman_mesh::uvgen::UvProjection::PlanarFront,
            oxihuman_mesh::uvgen::UvProjection::Box,
        ];

        for proj in &projections {
            let projected = oxihuman_mesh::uvgen::project_uvs(&mesh, *proj);
            for (i, uv) in projected.uvs.iter().enumerate() {
                prop_assert!(
                    uv[0] >= -1e-6 && uv[0] <= 1.0 + 1e-6,
                    "UV u out of range at vertex {} with {:?}: u = {}",
                    i, proj, uv[0]
                );
                prop_assert!(
                    uv[1] >= -1e-6 && uv[1] <= 1.0 + 1e-6,
                    "UV v out of range at vertex {} with {:?}: v = {}",
                    i, proj, uv[1]
                );
            }
        }
    }

    #[test]
    fn normalize_uvs_in_unit_range(
        uvs in proptest::collection::vec(
            prop::array::uniform2(-100.0f32..100.0f32),
            1..64,
        )
    ) {
        let normalized = oxihuman_mesh::uvgen::normalize_uvs(&uvs);
        for (i, uv) in normalized.iter().enumerate() {
            prop_assert!(
                uv[0] >= -1e-6 && uv[0] <= 1.0 + 1e-6,
                "Normalized UV u out of range at index {}: u = {}",
                i, uv[0]
            );
            prop_assert!(
                uv[1] >= -1e-6 && uv[1] <= 1.0 + 1e-6,
                "Normalized UV v out of range at index {}: v = {}",
                i, uv[1]
            );
        }
    }

    #[test]
    fn flip_v_preserves_unit_range(
        uvs in proptest::collection::vec(
            prop::array::uniform2(0.0f32..1.0f32),
            1..64,
        )
    ) {
        let flipped = oxihuman_mesh::uvgen::flip_v(&uvs);
        for (i, uv) in flipped.iter().enumerate() {
            prop_assert!(
                uv[0] >= -1e-6 && uv[0] <= 1.0 + 1e-6,
                "Flipped UV u out of range at index {}: u = {}",
                i, uv[0]
            );
            prop_assert!(
                uv[1] >= -1e-6 && uv[1] <= 1.0 + 1e-6,
                "Flipped UV v out of range at index {}: v = {}",
                i, uv[1]
            );
        }
    }
}
