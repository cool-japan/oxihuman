// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cross-crate integration tests for the OxiHuman workspace.
//!
//! These tests exercise workflows that span multiple crates:
//!   1. Core -> Morph -> Mesh -> Export pipeline
//!   2. Core -> Physics (collision proxies, cloth sim)
//!   3. Morph -> Measurements
//!   4. Morph -> Export -> Multiple formats (FBX ASCII, VRM, USDA, 3MF)
//!   5. Core -> Morph -> Viewer (MeshUploadBuffer)

use oxihuman_core::parser::obj::ObjMesh;
use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_morph::engine::HumanEngine;
use oxihuman_morph::engine::MeshBuffers as MorphMeshBuffers;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a permissive policy for tests.
fn test_policy() -> Policy {
    Policy::new(PolicyProfile::Standard)
}

/// Build a minimal humanoid-ish mesh (a rectangular prism standing on Y)
/// suitable for feeding into the morph engine and downstream crates.
fn make_test_obj_mesh() -> ObjMesh {
    // 8 vertices of a box: width=0.4 (X), height=1.7 (Y), depth=0.3 (Z)
    let positions = vec![
        [-0.2, 0.0, -0.15],
        [0.2, 0.0, -0.15],
        [0.2, 1.7, -0.15],
        [-0.2, 1.7, -0.15],
        [-0.2, 0.0, 0.15],
        [0.2, 0.0, 0.15],
        [0.2, 1.7, 0.15],
        [-0.2, 1.7, 0.15],
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 8];
    let uvs = vec![[0.0, 0.0]; 8];
    // 12 triangles (2 per face of the box)
    let indices = vec![
        // front
        0, 1, 2, 0, 2, 3, // back
        4, 6, 5, 4, 7, 6, // left
        0, 3, 7, 0, 7, 4, // right
        1, 5, 6, 1, 6, 2, // top
        3, 2, 6, 3, 6, 7, // bottom
        0, 4, 5, 0, 5, 1,
    ];
    ObjMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Convert morph engine output into the mesh crate's MeshBuffers.
fn morph_to_mesh(morph: MorphMeshBuffers) -> MeshBuffers {
    MeshBuffers::from_morph(morph)
}

/// Build a slightly larger mesh that looks more "body-like" by adding vertices
/// at various height fractions so that width-at-height sampling in
/// `compute_measurements` produces non-zero shoulder/waist/hip widths.
fn make_body_like_mesh() -> MeshBuffers {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let slices: &[(f32, f32)] = &[
        (0.00, 0.10), // feet
        (0.10, 0.12),
        (0.26, 0.14), // lower leg
        (0.35, 0.18), // hip
        (0.44, 0.17),
        (0.55, 0.14), // waist
        (0.75, 0.20), // shoulder
        (0.84, 0.12),
        (0.92, 0.10), // head
        (1.00, 0.08), // top of head
    ];
    let height = 1.7_f32;

    for &(frac, half_w) in slices {
        let y = frac * height;
        let base = positions.len() as u32;
        // 4 verts per slice (quad ring)
        positions.push([-half_w, y, -0.10]);
        positions.push([half_w, y, -0.10]);
        positions.push([half_w, y, 0.10]);
        positions.push([-half_w, y, 0.10]);

        // Connect to previous slice with triangles
        if base >= 4 {
            let prev = base - 4;
            for i in 0..4u32 {
                let next = (i + 1) % 4;
                indices.push(prev + i);
                indices.push(base + i);
                indices.push(base + next);

                indices.push(prev + i);
                indices.push(base + next);
                indices.push(prev + next);
            }
        }
    }

    // Cap top and bottom
    let n_slices = slices.len() as u32;
    // bottom cap
    indices.push(0);
    indices.push(1);
    indices.push(2);
    indices.push(0);
    indices.push(2);
    indices.push(3);
    // top cap
    let top_base = (n_slices - 1) * 4;
    indices.push(top_base);
    indices.push(top_base + 2);
    indices.push(top_base + 1);
    indices.push(top_base);
    indices.push(top_base + 3);
    indices.push(top_base + 2);

    let n = positions.len();
    let normals = vec![[0.0, 1.0, 0.0]; n];
    let tangents = vec![[1.0, 0.0, 0.0, 1.0]; n];
    let uvs = vec![[0.0, 0.0]; n];

    MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        colors: None,
        has_suit: false,
    }
}

// ===========================================================================
// 1. Core -> Morph -> Mesh -> Export pipeline
// ===========================================================================

mod core_morph_mesh_export {
    use super::*;

    /// End-to-end: create engine, build mesh, convert to mesh crate buffers,
    /// export to GLB file.
    #[test]
    fn pipeline_glb_export() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());

        let morph_buf = engine.build_mesh();
        assert!(!morph_buf.positions.is_empty(), "morph must produce verts");

        let mut mesh = morph_to_mesh(morph_buf);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);

        // GLB export requires has_suit = true (safety check)
        mesh.has_suit = true;

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.glb");
        oxihuman_export::export_glb(&mesh, &tmp)?;
        assert!(tmp.exists(), "GLB file should be created");
        let meta = std::fs::metadata(&tmp)?;
        assert!(meta.len() > 0, "GLB file should not be empty");
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Export to OBJ format.
    #[test]
    fn pipeline_obj_export() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.obj");
        oxihuman_export::export_obj(&mesh, &tmp)?;
        assert!(tmp.exists());
        let content = std::fs::read_to_string(&tmp)?;
        assert!(content.contains("v "), "OBJ should contain vertex lines");
        assert!(content.contains("f "), "OBJ should contain face lines");
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Export to STL ASCII format.
    #[test]
    fn pipeline_stl_ascii_export() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.stl");
        oxihuman_export::export_stl_ascii(&mesh, &tmp, "test_body")?;
        assert!(tmp.exists());
        let content = std::fs::read_to_string(&tmp)?;
        assert!(
            content.starts_with("solid"),
            "STL ASCII should start with 'solid'"
        );
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Export to STL binary format.
    #[test]
    fn pipeline_stl_binary_export() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test_bin.stl");
        oxihuman_export::export_stl_binary(&mesh, &tmp)?;
        assert!(tmp.exists());
        let meta = std::fs::metadata(&tmp)?;
        // STL binary header = 80 bytes + 4 bytes (tri count) + 50 bytes per tri
        assert!(meta.len() >= 84, "STL binary should have header");
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Verify the mesh_to_stl_ascii string builder works across crate boundary.
    #[test]
    fn pipeline_stl_string_builder() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let stl_str = oxihuman_export::mesh_to_stl_ascii(&mesh, "integ_test")?;
        assert!(stl_str.contains("facet normal"));
        assert!(stl_str.contains("endsolid"));

        Ok(())
    }

    /// Parallel build path produces same vertex count.
    #[test]
    fn parallel_build_matches_sequential() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let seq = engine.build_mesh();
        let par = engine.build_mesh_parallel();
        assert_eq!(seq.positions.len(), par.positions.len());
        assert_eq!(seq.indices.len(), par.indices.len());
    }

    /// Export to GLB with skeleton metadata.
    #[test]
    fn pipeline_glb_with_skeleton() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let skeleton = oxihuman_mesh::Skeleton {
            joints: vec![
                oxihuman_mesh::Joint {
                    name: "root".to_string(),
                    parent: None,
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                oxihuman_mesh::Joint {
                    name: "spine".to_string(),
                    parent: Some(0),
                    translation: [0.0, 0.5, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
            ],
        };

        let tmp = std::env::temp_dir().join("oxihuman_integ_skel.glb");
        oxihuman_export::export_glb_with_skeleton(&mesh, &skeleton, &tmp)?;
        assert!(tmp.exists());
        let meta = std::fs::metadata(&tmp)?;
        assert!(meta.len() > 0);
        std::fs::remove_file(&tmp)?;

        Ok(())
    }
}

// ===========================================================================
// 2. Core -> Physics (collision proxies, cloth sim)
// ===========================================================================

mod core_physics {
    use super::*;

    /// Generate collision proxies from a body-like mesh.
    #[test]
    fn generate_collision_proxies() {
        let mesh = make_body_like_mesh();
        let proxies = oxihuman_physics::generate_proxies(&mesh);
        assert!(proxies.is_some(), "proxies should be generated");
        let proxies = proxies.expect("should succeed");
        assert!(
            proxies.total_count() > 0,
            "should have at least one proxy primitive"
        );
        assert!(
            !proxies.spheres.is_empty(),
            "should have a head sphere proxy"
        );
        assert!(
            !proxies.capsules.is_empty(),
            "should have capsule proxies for body parts"
        );
    }

    /// Serialize proxies to JSON and verify structure.
    #[test]
    fn proxies_to_json_roundtrip() {
        let mesh = make_body_like_mesh();
        let proxies = oxihuman_physics::generate_proxies(&mesh).expect("should succeed");
        let json = oxihuman_physics::proxies_to_json(&proxies);
        assert!(json.contains("\"capsules\""));
        assert!(json.contains("\"spheres\""));
        assert!(json.contains("\"boxes\""));
    }

    /// Cloth simulation: build from mesh positions, step forward.
    #[test]
    fn cloth_sim_step() {
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 2];

        let mut cloth = oxihuman_physics::ClothSim::from_mesh(&positions, &indices, 0.8);
        assert_eq!(cloth.particles.len(), 4);
        assert!(!cloth.springs.is_empty());

        // Pin top-left corner
        cloth.particles[0].pinned = true;

        let initial_y1 = cloth.particles[1].position[1];
        cloth.gravity = [0.0, -9.81, 0.0];
        cloth.damping = 0.99;
        cloth.step(1.0 / 60.0, 4);

        let after_y1 = cloth.particles[1].position[1];
        assert!(
            after_y1 < initial_y1,
            "particle should fall under gravity: before={initial_y1}, after={after_y1}"
        );
        assert!(
            (cloth.particles[0].position[1] - 1.0).abs() < 1e-6,
            "pinned particle should stay put"
        );
    }

    /// Collision detection primitives work across crate boundaries.
    #[test]
    fn collision_sphere_sphere() {
        let s1 = oxihuman_physics::Sphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        };
        let s2 = oxihuman_physics::Sphere {
            center: [1.5, 0.0, 0.0],
            radius: 1.0,
        };
        let contact = oxihuman_physics::sphere_sphere(&s1, &s2);
        assert!(
            contact.is_some(),
            "overlapping spheres should produce a contact"
        );
        let c = contact.expect("should succeed");
        assert!(c.depth > 0.0);
    }

    /// Build a physics rig from body proxies.
    #[test]
    fn build_physics_rig() {
        let mesh = make_body_like_mesh();
        let proxies = oxihuman_physics::generate_proxies(&mesh).expect("should succeed");
        let rig = oxihuman_physics::build_rig(&proxies);
        assert!(
            !rig.joints.is_empty(),
            "rig should have at least some joints"
        );
    }
}

// ===========================================================================
// 3. Morph -> Measurements
// ===========================================================================

mod morph_measurements {
    use super::*;

    /// Apply morph parameters and compute body measurements from the result.
    #[test]
    fn morph_to_body_measurements() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let morph_buf = engine.build_mesh();
        let mesh = morph_to_mesh(morph_buf);

        let meas = oxihuman_mesh::compute_measurements(&mesh);
        assert!(meas.is_some(), "measurements should be computable");
        let meas = meas.expect("should succeed");
        assert!(
            meas.total_height > 0.0,
            "height should be positive: {}",
            meas.total_height
        );
        assert!(
            (meas.total_height - 1.7).abs() < 0.01,
            "height should be ~1.7, got {}",
            meas.total_height
        );
    }

    /// The body-like mesh produces realistic width measurements.
    #[test]
    fn body_like_mesh_measurements() {
        let mesh = make_body_like_mesh();
        let meas = oxihuman_mesh::compute_measurements(&mesh).expect("should succeed");
        assert!(meas.total_height > 1.0, "height should be over 1m");
        assert!(
            meas.shoulder_width > meas.waist_width,
            "shoulders should be wider than waist"
        );
        assert!(
            meas.hip_width > meas.waist_width,
            "hips should be wider than waist"
        );
    }

    /// AABB computation from mesh buffers.
    #[test]
    fn aabb_from_morph_mesh() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let aabb = oxihuman_mesh::compute_aabb(&mesh).expect("should succeed");
        assert!(aabb.height() > 0.0);
        assert!(aabb.width() > 0.0);
        assert!(aabb.depth() > 0.0);
        let center = aabb.center();
        assert!(center[0].abs() < 0.5, "center X should be near 0");
        assert!(center[2].abs() < 0.5, "center Z should be near 0");
    }

    /// Morph engine mesh -> mesh crate normals computation.
    #[test]
    fn normals_computed_after_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let morph_buf = engine.build_mesh();
        let mut mesh = morph_to_mesh(morph_buf);

        oxihuman_mesh::compute_normals(&mut mesh);

        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(
                len < 1e-6 || (len - 1.0).abs() < 1e-4,
                "normal should be unit or zero, got len={len}"
            );
        }
    }

    /// Mesh stats computation across crate boundary.
    #[test]
    fn mesh_stats_from_morph_output() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let stats = oxihuman_mesh::compute_stats(&mesh);
        assert_eq!(stats.vertex_count, 8);
        assert_eq!(stats.face_count, 12);
        assert!(stats.surface_area > 0.0);
    }
}

// ===========================================================================
// 4. Morph -> Export -> Multiple formats
// ===========================================================================

mod morph_export_formats {
    use super::*;

    /// Export to USDA (Universal Scene Description ASCII).
    #[test]
    fn export_usda_from_morph() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.usda");
        let opts = oxihuman_export::UsdExportOptions::default();
        let stats = oxihuman_export::export_usda(&mesh, &tmp, &opts)?;
        assert!(tmp.exists());
        let content = std::fs::read_to_string(&tmp)?;
        assert!(content.contains("Xform"), "USDA should contain Xform prim");
        assert!(stats.vertex_count > 0);
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Build USDA string directly (no file I/O).
    #[test]
    fn build_usda_string() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let opts = oxihuman_export::UsdExportOptions::default();
        let usda = oxihuman_export::build_usda(&mesh, &opts);
        assert!(!usda.is_empty());
        assert!(usda.contains("float3[] points"));
    }

    /// Export to 3MF format (3D printing).
    #[test]
    fn export_3mf_from_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let opts = oxihuman_export::ThreeMfOptions::default();
        let result = oxihuman_export::export_3mf(&mesh, &opts);
        assert!(
            !result.zip_bytes.is_empty(),
            "3MF zip archive should not be empty"
        );
        assert!(
            result.model_xml_size > 0,
            "model XML should have non-zero size"
        );
    }

    /// Export to FBX binary format.
    #[test]
    fn export_fbx_binary_from_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let morph_buf = engine.build_mesh();
        let mesh_buf = MeshBuffers::from_morph(morph_buf);

        let result = oxihuman_export::export_mesh_fbx_binary(&mesh_buf);
        assert!(result.is_ok(), "FBX binary export should succeed");
        let bytes = result.expect("should succeed");
        assert!(!bytes.is_empty(), "FBX binary content should not be empty");
        // FBX binary magic: "Kaydara FBX Binary  \0"
        assert!(
            bytes.starts_with(b"Kaydara FBX Binary"),
            "FBX binary should start with magic header"
        );
    }

    /// VRM metadata builder works across crate boundary.
    #[test]
    fn vrm_metadata_builder() {
        let meta = oxihuman_export::default_vrm_meta("TestAvatar");
        let opts = oxihuman_export::VrmExportOptions {
            meta,
            humanoid: oxihuman_export::VrmHumanoid {
                hips_node: 0,
                head_node: 1,
                left_hand_node: Some(2),
                right_hand_node: Some(3),
            },
            spec_version: "1.0".to_string(),
        };
        let result = oxihuman_export::validate_vrm_options(&opts);
        assert!(result.is_ok(), "default VRM options should be valid");

        let json = oxihuman_export::build_vrm_extensions_json(&opts);
        assert!(json.contains("VRMC_vrm"));
        assert!(json.contains("specVersion"));
    }

    /// Export to COLLADA format.
    #[test]
    fn export_collada_from_morph() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.dae");
        let opts = oxihuman_export::ColladaExportOptions::default();
        let stats = oxihuman_export::export_collada(&mesh, &tmp, &opts)?;
        assert!(tmp.exists());
        assert!(stats.vertex_count > 0);
        let content = std::fs::read_to_string(&tmp)?;
        assert!(content.contains("COLLADA"));
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Export to X3D format.
    #[test]
    fn export_x3d_from_morph() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.x3d");
        let opts = oxihuman_export::X3dExportOptions::default();
        let stats = oxihuman_export::export_x3d(&mesh, &tmp, &opts)?;
        assert!(tmp.exists());
        assert!(stats.vertex_count > 0);
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Export to PLY format.
    #[test]
    fn export_ply_from_morph() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let tmp = std::env::temp_dir().join("oxihuman_integ_test.ply");
        oxihuman_export::export_ply(&mesh, &tmp, oxihuman_export::PlyFormat::Ascii)?;
        assert!(tmp.exists());
        let content = std::fs::read_to_string(&tmp)?;
        assert!(content.contains("ply"));
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// JSON mesh export from morph output.
    #[test]
    fn export_json_mesh_from_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let json = oxihuman_export::export_json_mesh(&mesh);
        let json_str = json.to_string();
        assert!(json_str.contains("vertex_count"));
        assert!(json_str.contains("face_count"));
    }

    /// CSV export of vertices.
    #[test]
    fn export_csv_vertices_from_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let csv = oxihuman_export::vertices_to_csv_string(&mesh);
        let lines: Vec<&str> = csv.lines().collect();
        // header + 8 vertices
        assert!(lines.len() > 1, "CSV should have header + data rows");
    }
}

// ===========================================================================
// 5. Core -> Morph -> Viewer (MeshUploadBuffer)
// ===========================================================================

mod core_morph_viewer {
    use super::*;
    use oxihuman_viewer::MeshUploadBuffer;

    /// Build mesh from morph engine, convert to viewer upload buffer format.
    #[test]
    fn morph_to_upload_buffer() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let morph_buf = engine.build_mesh();

        let upload = MeshUploadBuffer {
            positions: morph_buf.positions.clone(),
            normals: morph_buf.normals.clone(),
            uvs: morph_buf.uvs.clone(),
            indices: morph_buf.indices.clone(),
            timestamp: 0,
        };

        assert_eq!(upload.positions.len(), morph_buf.positions.len());
        assert_eq!(upload.normals.len(), morph_buf.normals.len());
        assert_eq!(upload.indices.len(), morph_buf.indices.len());
    }

    /// Serialize upload buffer to binary, deserialize back.
    #[test]
    fn upload_buffer_binary_roundtrip() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let morph_buf = engine.build_mesh();

        let n_verts = morph_buf.positions.len() as u32;
        let n_idx = morph_buf.indices.len() as u32;

        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&1u32.to_le_bytes()); // version
        bytes.extend_from_slice(&n_verts.to_le_bytes());
        bytes.extend_from_slice(&n_idx.to_le_bytes());

        for p in &morph_buf.positions {
            bytes.extend_from_slice(&p[0].to_le_bytes());
            bytes.extend_from_slice(&p[1].to_le_bytes());
            bytes.extend_from_slice(&p[2].to_le_bytes());
        }
        for n in &morph_buf.normals {
            bytes.extend_from_slice(&n[0].to_le_bytes());
            bytes.extend_from_slice(&n[1].to_le_bytes());
            bytes.extend_from_slice(&n[2].to_le_bytes());
        }
        for uv in &morph_buf.uvs {
            bytes.extend_from_slice(&uv[0].to_le_bytes());
            bytes.extend_from_slice(&uv[1].to_le_bytes());
        }
        for &i in &morph_buf.indices {
            bytes.extend_from_slice(&i.to_le_bytes());
        }

        let upload = MeshUploadBuffer::from_raw_bytes(&bytes);
        assert!(upload.is_some(), "should parse binary buffer");
        let upload = upload.expect("should succeed");
        assert_eq!(upload.positions.len(), morph_buf.positions.len());
        assert_eq!(upload.indices.len(), morph_buf.indices.len());

        for (orig, parsed) in morph_buf.positions.iter().zip(upload.positions.iter()) {
            assert!((orig[0] - parsed[0]).abs() < 1e-6);
            assert!((orig[1] - parsed[1]).abs() < 1e-6);
            assert!((orig[2] - parsed[2]).abs() < 1e-6);
        }
    }

    /// Viewer scene creation from morph data.
    #[test]
    fn viewer_scene_from_morph() {
        let scene = oxihuman_viewer::default_scene();
        assert!(!scene.nodes.is_empty(), "default scene should have nodes");

        let camera = oxihuman_viewer::CameraState::default();
        assert!(camera.fov_deg > 0.0, "camera should have positive FOV");
    }

    /// Material and pipeline descriptors work across crate boundary.
    #[test]
    fn viewer_material_pipeline() {
        let mat = oxihuman_viewer::PbrMaterial::default_skin();
        let json = oxihuman_viewer::material_to_gltf_json(&mat);
        assert!(json.contains("pbrMetallicRoughness"));

        let pipeline = oxihuman_viewer::default_mesh_pipeline();
        let valid = oxihuman_viewer::validate_pipeline(&pipeline);
        assert!(valid.is_ok(), "default pipeline should be valid");

        let layout = oxihuman_viewer::standard_vertex_layout();
        assert!(
            !layout.attributes.is_empty(),
            "standard layout should have attributes"
        );
    }
}

// ===========================================================================
// Cross-cutting: multiple crates working together
// ===========================================================================

mod cross_cutting {
    use super::*;

    /// Full pipeline: morph -> mesh -> physics proxies -> export proxies JSON.
    #[test]
    fn full_pipeline_morph_physics_export() {
        let mesh = make_body_like_mesh();

        let proxies = oxihuman_physics::generate_proxies(&mesh).expect("should succeed");
        let json = oxihuman_physics::proxies_to_json(&proxies);

        for cap in &proxies.capsules {
            assert!(
                json.contains(&cap.label),
                "JSON should contain label '{}'",
                cap.label
            );
        }
        for sph in &proxies.spheres {
            assert!(
                json.contains(&sph.label),
                "JSON should contain label '{}'",
                sph.label
            );
        }

        let stl =
            oxihuman_export::mesh_to_stl_ascii(&mesh, "physics_body").expect("should succeed");
        assert!(!stl.is_empty());
    }

    /// Morph -> mesh processing pipeline (subdivide, smooth).
    #[test]
    fn morph_mesh_processing_pipeline() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let stats_before = oxihuman_mesh::compute_stats(&mesh);
        assert!(stats_before.vertex_count > 0);

        let sub_mesh = oxihuman_mesh::midpoint_subdivide(&mesh, 1);
        assert!(
            sub_mesh.positions.len() > mesh.positions.len(),
            "subdivision should increase vertex count"
        );
        assert!(
            sub_mesh.indices.len() > mesh.indices.len(),
            "subdivision should increase triangle count"
        );

        let original_positions = sub_mesh.positions.clone();

        let config = oxihuman_mesh::SmoothConfig {
            iterations: 3,
            factor: 0.5,
            preserve_boundary: false,
        };
        let smoothed = oxihuman_mesh::laplacian_smooth(&sub_mesh, &config);
        let mut any_diff = false;
        for (a, b) in original_positions.iter().zip(smoothed.positions.iter()) {
            if (a[0] - b[0]).abs() > 1e-8
                || (a[1] - b[1]).abs() > 1e-8
                || (a[2] - b[2]).abs() > 1e-8
            {
                any_diff = true;
                break;
            }
        }
        assert!(any_diff, "smoothing should modify at least some positions");
    }

    /// Mesh integrity check after morph output.
    #[test]
    fn mesh_integrity_after_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let report = oxihuman_mesh::check_integrity(&mesh);
        assert!(!report.has_nan_positions, "no NaN positions expected");
        assert!(!report.has_inf_positions, "no Inf positions expected");
        assert!(
            report.out_of_bounds_indices.is_empty(),
            "all indices should be within bounds"
        );
    }

    /// Convex hull from morph mesh vertices.
    #[test]
    fn convex_hull_from_morph() {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let hull = oxihuman_mesh::convex_hull(&mesh.positions);
        assert!(hull.is_some(), "convex hull should succeed for a box");
        let hull = hull.expect("should succeed");
        assert!(!hull.vertices.is_empty(), "hull should have vertices");
        assert!(!hull.indices.is_empty(), "hull should have faces");
        assert_eq!(hull.vertices.len(), 8);
    }

    /// Core event bus + morph parameter changes trigger events.
    #[test]
    fn event_bus_morph_param_change() {
        let mut bus = oxihuman_core::EventBus::new();

        let event = oxihuman_core::make_param_changed_event("height", 0.75);
        bus.publish(event);

        let events = bus.history();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, oxihuman_core::EventKind::ParamChanged);
        assert!(events[0].payload.contains("height"));
        assert!(events[0].payload.contains("0.75"));
    }

    /// Core undo/redo stack with morph-like commands.
    #[test]
    fn undo_redo_morph_workflow() {
        let mut stack = oxihuman_core::new_undo_stack(50);

        oxihuman_core::push_command(&mut stack, "set_height", b"0.5".to_vec(), b"0.75".to_vec());
        oxihuman_core::push_command(&mut stack, "set_weight", b"0.5".to_vec(), b"0.6".to_vec());

        assert_eq!(oxihuman_core::history_depth(&stack), 2);
        assert!(oxihuman_core::can_undo(&stack));

        let undone = oxihuman_core::undo(&mut stack);
        assert!(undone.is_some());
        assert_eq!(undone.expect("should succeed").name, "set_weight");

        assert!(oxihuman_core::can_redo(&stack));
        let redone = oxihuman_core::redo(&mut stack);
        assert!(redone.is_some());
    }

    /// Export pipeline with scene from morph data.
    #[test]
    fn scene_export() -> anyhow::Result<()> {
        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let scene_mesh = oxihuman_export::SceneMesh::new("body", mesh);
        let mut scene = oxihuman_export::Scene::new("IntegrationTest");
        scene.meshes.push(scene_mesh);

        let tmp = std::env::temp_dir().join("oxihuman_integ_scene.glb");
        oxihuman_export::export_scene_glb(&scene, &tmp)?;
        assert!(tmp.exists());
        std::fs::remove_file(&tmp)?;

        Ok(())
    }

    /// Morph body preset -> engine -> mesh -> measurements -> export.
    #[test]
    fn preset_to_export_pipeline() -> anyhow::Result<()> {
        let athletic_preset = oxihuman_morph::preset_athletic();
        assert!(!athletic_preset.name.is_empty());

        let obj = make_test_obj_mesh();
        let engine = HumanEngine::new(obj, test_policy());
        let mesh = morph_to_mesh(engine.build_mesh());

        let meas = oxihuman_mesh::compute_measurements(&mesh).expect("should succeed");
        assert!(meas.total_height > 0.0);

        let meas_json = oxihuman_export::export_mesh_measurements(&mesh);
        let meas_str = meas_json.to_string();
        assert!(
            meas_str.contains("height") || meas_str.contains("width"),
            "measurements JSON should contain height or width"
        );

        Ok(())
    }
}
