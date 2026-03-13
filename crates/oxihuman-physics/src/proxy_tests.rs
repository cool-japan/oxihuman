// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Tests for proxy generation, JSON serialization, and voxelization.

#[cfg(test)]
mod tests {
    use oxihuman_mesh::measurements::compute_aabb;
    use oxihuman_mesh::measurements::compute_measurements;
    use oxihuman_mesh::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    use crate::proxy_gen::BODY_PART_BANDS;
    use crate::{
        generate_fitted_proxies, generate_proxies, proxies_from_json, proxies_to_json,
        voxelize_to_proxies, BodyProxies, CapsuleProxy, SphereProxy,
    };

    fn unit_body_mesh() -> MeshBuffers {
        // Minimal mesh spanning [0..0.5, 0..1.8, 0..0.3]
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.5, 0.0, 0.3],
            [0.0, 0.0, 0.3],
            [0.0, 1.8, 0.0],
            [0.5, 1.8, 0.0],
            [0.5, 1.8, 0.3],
            [0.0, 1.8, 0.3],
        ];
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; 8],
            uvs: vec![[0.0, 0.0]; 8],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn generate_proxies_produces_expected_count() {
        let mesh = unit_body_mesh();
        let proxies = generate_proxies(&mesh).expect("generate_proxies must succeed");
        // Should have: head sphere + torso + hips + 2 legs + 2 shins + 2 arms + 2 forearms
        // = 1 sphere + 10 capsules = 11 total
        assert_eq!(proxies.total_count(), 11);
        assert_eq!(proxies.spheres.len(), 1);
        assert_eq!(proxies.capsules.len(), 10);
        assert_eq!(proxies.spheres[0].label, "head");
    }

    #[test]
    fn proxy_radii_positive() {
        let mesh = unit_body_mesh();
        let proxies = generate_proxies(&mesh).expect("generate_proxies must succeed");
        for c in &proxies.capsules {
            assert!(
                c.radius > 0.0,
                "capsule {} has non-positive radius",
                c.label
            );
        }
        for s in &proxies.spheres {
            assert!(s.radius > 0.0, "sphere {} has non-positive radius", s.label);
        }
    }

    #[test]
    fn proxy_positions_within_mesh_height() {
        let mesh = unit_body_mesh();
        let aabb = compute_aabb(&mesh).expect("compute_aabb must succeed");
        let proxies = generate_proxies(&mesh).expect("generate_proxies must succeed");
        for c in &proxies.capsules {
            assert!(c.center_a[1] >= aabb.min[1], "{} below floor", c.label);
            assert!(
                c.center_b[1] <= aabb.max[1] + c.radius,
                "{} above ceiling",
                c.label
            );
        }
        for s in &proxies.spheres {
            assert!(s.center[1] >= aabb.min[1], "sphere {} below floor", s.label);
            assert!(
                s.center[1] <= aabb.max[1] + s.radius,
                "sphere {} above ceiling",
                s.label
            );
        }
    }

    #[test]
    fn real_base_mesh_proxies() {
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
                let mesh = MeshBuffers::from_morph(morph_buf);
                let proxies = generate_proxies(&mesh).expect("generate_proxies must succeed");
                assert_eq!(proxies.total_count(), 11);
                // Head should be near top of mesh
                let head = &proxies.spheres[0];
                let aabb = compute_aabb(&mesh).expect("compute_aabb must succeed");
                assert!(
                    head.center[1] > aabb.center()[1],
                    "head should be above center"
                );
            }
        }
    }

    // ── generate_fitted_proxies tests ─────────────────────────────────────────

    /// A denser body-shaped mesh for fitted proxy tests.
    fn body_mesh_dense() -> MeshBuffers {
        // Build a set of positions that cover the full human height range
        // so each band has some vertices.
        let mut positions = Vec::new();
        // Distribute 200 points evenly across y=0..1.8, with some x/z spread
        for i in 0..200usize {
            let t = i as f32 / 199.0;
            let y = t * 1.8;
            let angle = t * std::f32::consts::TAU * 10.0;
            positions.push([angle.cos() * 0.15, y, angle.sin() * 0.10]);
            positions.push([-angle.cos() * 0.15, y, angle.sin() * 0.10]);
        }
        MeshBuffers::from_morph(MB {
            normals: vec![[0.0, 1.0, 0.0]; positions.len()],
            uvs: vec![[0.0, 0.0]; positions.len()],
            indices: vec![0, 1, 2],
            has_suit: false,
            positions,
        })
    }

    #[test]
    fn generate_fitted_proxies_count() {
        let mesh = body_mesh_dense();
        let meas = compute_measurements(&mesh).expect("compute_measurements must succeed");
        let proxies = generate_fitted_proxies(&mesh, &meas);
        // One entry per body part band
        assert_eq!(
            proxies.len(),
            BODY_PART_BANDS.len(),
            "expected {} fitted proxies, got {}",
            BODY_PART_BANDS.len(),
            proxies.len()
        );
    }

    #[test]
    fn generate_fitted_proxies_names() {
        let mesh = body_mesh_dense();
        let meas = compute_measurements(&mesh).expect("compute_measurements must succeed");
        let proxies = generate_fitted_proxies(&mesh, &meas);
        let names: Vec<&str> = proxies.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"head"), "missing head");
        assert!(names.contains(&"torso"), "missing torso");
        assert!(names.contains(&"arm_l"), "missing arm_l");
        assert!(names.contains(&"forearm_r"), "missing forearm_r");
    }

    #[test]
    fn generate_fitted_proxies_radii_positive() {
        let mesh = body_mesh_dense();
        let meas = compute_measurements(&mesh).expect("compute_measurements must succeed");
        let proxies = generate_fitted_proxies(&mesh, &meas);
        for (name, cap) in &proxies {
            assert!(
                cap.radius > 0.0,
                "fitted proxy '{name}' has non-positive radius {}",
                cap.radius
            );
        }
    }

    #[test]
    fn generate_fitted_proxies_count_real_mesh() {
        use oxihuman_core::parser::obj::parse_obj;
        let path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
        if let Ok(src) = std::fs::read_to_string(path) {
            if let Ok(obj) = parse_obj(&src) {
                let morph_buf = MB {
                    positions: obj.positions,
                    normals: obj.normals,
                    uvs: obj.uvs,
                    indices: obj.indices,
                    has_suit: false,
                };
                let mesh = MeshBuffers::from_morph(morph_buf);
                let meas = compute_measurements(&mesh).expect("compute_measurements must succeed");
                let proxies = generate_fitted_proxies(&mesh, &meas);
                assert_eq!(
                    proxies.len(),
                    BODY_PART_BANDS.len(),
                    "real mesh proxy count should match body part band count"
                );
            }
        }
    }

    // ── Task 1: JSON serialization tests ─────────────────────────────────────

    fn sample_proxies() -> BodyProxies {
        let mut p = BodyProxies::new();
        p.capsules.push(CapsuleProxy::new(
            [0.0, 0.52, 0.0],
            [0.0, 0.84, 0.0],
            0.12,
            "torso",
        ));
        p.spheres
            .push(SphereProxy::new([0.0, 1.65, 0.0], 0.11, "head"));
        p
    }

    #[test]
    fn proxies_to_json_is_valid_json() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        // serde_json must be able to parse our hand-written output
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("proxies_to_json must produce valid JSON");
        assert!(parsed.is_object(), "top level must be an object");
    }

    #[test]
    fn proxies_to_json_contains_expected_keys() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        let v: serde_json::Value =
            serde_json::from_str(&json).expect("proxies_to_json must produce valid JSON");
        assert!(v["capsules"].is_array(), "must have capsules array");
        assert!(v["spheres"].is_array(), "must have spheres array");
        assert!(v["boxes"].is_array(), "must have boxes array");
    }

    #[test]
    fn proxies_to_json_capsule_label_present() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        let v: serde_json::Value =
            serde_json::from_str(&json).expect("proxies_to_json must produce valid JSON");
        let cap = &v["capsules"][0];
        assert_eq!(cap["label"].as_str(), Some("torso"));
    }

    #[test]
    fn proxies_to_json_sphere_label_present() {
        let proxies = sample_proxies();
        let json = proxies_to_json(&proxies);
        let v: serde_json::Value =
            serde_json::from_str(&json).expect("proxies_to_json must produce valid JSON");
        let sph = &v["spheres"][0];
        assert_eq!(sph["label"].as_str(), Some("head"));
    }

    #[test]
    fn proxies_from_json_round_trip() {
        let original = sample_proxies();
        let json = proxies_to_json(&original);
        let restored = proxies_from_json(&json).expect("round-trip must succeed");

        assert_eq!(restored.capsules.len(), original.capsules.len());
        assert_eq!(restored.spheres.len(), original.spheres.len());
        assert_eq!(restored.boxes.len(), original.boxes.len());

        let orig_cap = &original.capsules[0];
        let rest_cap = &restored.capsules[0];
        assert_eq!(rest_cap.label, orig_cap.label);
        assert!((rest_cap.radius - orig_cap.radius).abs() < 1e-4);
        for i in 0..3 {
            assert!((rest_cap.center_a[i] - orig_cap.center_a[i]).abs() < 1e-4);
            assert!((rest_cap.center_b[i] - orig_cap.center_b[i]).abs() < 1e-4);
        }

        let orig_sph = &original.spheres[0];
        let rest_sph = &restored.spheres[0];
        assert_eq!(rest_sph.label, orig_sph.label);
        assert!((rest_sph.radius - orig_sph.radius).abs() < 1e-4);
    }

    #[test]
    fn proxies_from_json_invalid_input_errors() {
        assert!(
            proxies_from_json("not json at all").is_err(),
            "invalid JSON must return Err"
        );
    }

    #[test]
    fn proxies_to_json_empty_proxies() {
        let empty = BodyProxies::new();
        let json = proxies_to_json(&empty);
        let v: serde_json::Value =
            serde_json::from_str(&json).expect("proxies_to_json must produce valid JSON");
        assert_eq!(
            v["capsules"]
                .as_array()
                .expect("capsules must be an array")
                .len(),
            0
        );
        assert_eq!(
            v["spheres"]
                .as_array()
                .expect("spheres must be an array")
                .len(),
            0
        );
        assert_eq!(
            v["boxes"].as_array().expect("boxes must be an array").len(),
            0
        );
    }

    // ── Task 2: voxelization tests ────────────────────────────────────────────

    #[test]
    fn voxelize_to_proxies_nonempty_for_body_mesh() {
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 8);
        assert!(
            !proxies.boxes.is_empty(),
            "voxelization must produce at least one box proxy"
        );
    }

    #[test]
    fn voxelize_to_proxies_labels_include_torso_or_head() {
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 8);
        let labels: Vec<&str> = proxies.boxes.iter().map(|b| b.label.as_str()).collect();
        let has_torso_or_head = labels.contains(&"torso") || labels.contains(&"head");
        assert!(
            has_torso_or_head,
            "expected 'torso' or 'head' label, got: {:?}",
            labels
        );
    }

    #[test]
    fn voxelize_to_proxies_half_extents_positive() {
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 6);
        for b in &proxies.boxes {
            assert!(
                b.half_extents[0] > 0.0 && b.half_extents[1] > 0.0 && b.half_extents[2] > 0.0,
                "box proxy '{}' has non-positive half_extents {:?}",
                b.label,
                b.half_extents
            );
        }
    }

    #[test]
    fn voxelize_to_proxies_empty_mesh_returns_empty() {
        let empty_mesh = MeshBuffers::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        });
        let proxies = voxelize_to_proxies(&empty_mesh, 8);
        assert!(
            proxies.boxes.is_empty(),
            "empty mesh must produce no box proxies"
        );
    }

    #[test]
    fn voxelize_to_proxies_json_round_trip() {
        // Voxelize the body mesh, serialize to JSON, parse back, and verify counts match.
        let mesh = body_mesh_dense();
        let proxies = voxelize_to_proxies(&mesh, 6);
        let json = proxies_to_json(&proxies);
        let restored = proxies_from_json(&json).expect("voxel proxy JSON round-trip must succeed");
        assert_eq!(
            proxies.boxes.len(),
            restored.boxes.len(),
            "box count must survive JSON round-trip"
        );
    }
}
