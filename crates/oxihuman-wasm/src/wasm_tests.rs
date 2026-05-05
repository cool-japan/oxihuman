// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use super::*;
use oxihuman_morph::params::ParamState;

const SIMPLE_OBJ: &[u8] = br#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
"#;

// -- original 9 tests --

#[test]
fn create_engine_from_obj_bytes() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    assert_eq!(e.vertex_count(), 3);
}

#[test]
fn build_mesh_bytes_format() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let bytes = e.build_mesh_bytes();
    let (version, n_verts, n_idx) = parse_mesh_bytes_header(&bytes).expect("should succeed");
    assert_eq!(version, BUFFER_FORMAT_VERSION);
    assert_eq!(n_verts, 3);
    assert_eq!(n_idx, 3); // 1 triangle
}

#[test]
fn build_mesh_bytes_correct_length() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let bytes = e.build_mesh_bytes();
    let (_, n_verts, n_idx) = parse_mesh_bytes_header(&bytes).expect("should succeed");
    let expected_len = 12 + (n_verts as usize) * (3 + 3 + 2) * 4 + (n_idx as usize) * 4;
    assert_eq!(bytes.len(), expected_len);
}

#[test]
fn load_target_bytes_works() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let target = b"# test\n0 0.1 0.2 0.3\n1 0.0 0.1 0.0\n";
    e.load_target_bytes("height", target)
        .expect("should succeed");
    let bytes = e.build_mesh_bytes();
    assert!(bytes.len() > 12);
}

#[test]
fn set_params_update_engine() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.8);
    e.set_weight(0.3);
    e.set_muscle(0.6);
    e.set_age(0.2);
    assert!(!e.has_cached_mesh());
    let bytes = e.build_mesh_bytes();
    assert!(bytes.len() > 12);
    assert!(e.has_cached_mesh());
}

#[test]
fn set_param_extra_works() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_param("expression", 0.7);
    let bytes = e.build_mesh_bytes();
    assert!(bytes.len() > 12);
}

#[test]
fn has_cached_mesh_clears_on_param_change() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _bytes = e.build_mesh_bytes();
    assert!(e.has_cached_mesh());
    e.set_height(0.9);
    assert!(!e.has_cached_mesh());
}

#[test]
fn strict_policy_engine_builds() {
    let e = WasmEngine::new_strict(SIMPLE_OBJ).expect("should succeed");
    assert_eq!(e.vertex_count(), 3);
}

#[test]
fn real_base_mesh_via_wasm_engine() {
    let path = std::env::var("MAKEHUMAN_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/oxihuman_nonexistent_data"))
        .join("3dobjs/base.obj");
    if let Ok(obj_bytes) = std::fs::read(&path) {
        let mut e = WasmEngine::new_from_obj_bytes(&obj_bytes).expect("should succeed");
        assert!(e.vertex_count() > 10_000);
        let bytes = e.build_mesh_bytes();
        let (version, n_verts, _) = parse_mesh_bytes_header(&bytes).expect("should succeed");
        assert_eq!(version, BUFFER_FORMAT_VERSION);
        assert_eq!(n_verts as usize, e.vertex_count());
    }
}

// -- reset_params --

#[test]
fn reset_params_restores_defaults() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.9);
    e.set_weight(0.1);
    e.reset_params();
    let default = ParamState::default();
    assert!((e.params.height - default.height).abs() < 1e-6);
    assert!((e.params.weight - default.weight).abs() < 1e-6);
    assert!((e.params.muscle - default.muscle).abs() < 1e-6);
    assert!((e.params.age - default.age).abs() < 1e-6);
}

#[test]
fn reset_params_invalidates_cache() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    assert!(e.has_cached_mesh());
    e.reset_params();
    assert!(!e.has_cached_mesh());
}

// -- target_count --

#[test]
fn target_count_zero_initially() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    assert_eq!(e.target_count(), 0);
}

#[test]
fn target_count_increases_on_load() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let target = b"# test\n0 0.1 0.2 0.3\n";
    e.load_target_bytes("height", target)
        .expect("should succeed");
    assert_eq!(e.target_count(), 1);
    e.load_target_bytes("weight", target)
        .expect("should succeed");
    assert_eq!(e.target_count(), 2);
}

// -- export_params_json --

#[test]
fn export_params_json_valid_json() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.export_params_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(v.get("height").is_some());
    assert!(v.get("weight").is_some());
    assert!(v.get("muscle").is_some());
    assert!(v.get("age").is_some());
}

#[test]
fn export_params_json_reflects_changes() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.9);
    let json = e.export_params_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    let h = v["height"].as_f64().expect("should succeed");
    assert!((h - 0.9f64).abs() < 1e-5, "expected 0.9, got {h}");
}

// -- import_params_json --

#[test]
fn import_params_json_round_trips() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.75);
    e.set_weight(0.25);
    let json = e.export_params_json();
    e.reset_params();
    e.import_params_json(&json).expect("should succeed");
    assert!((e.params.height - 0.75).abs() < 1e-5);
    assert!((e.params.weight - 0.25).abs() < 1e-5);
}

#[test]
fn import_params_json_invalid_returns_error() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let result = e.import_params_json("not valid json {{");
    assert!(result.is_err());
}

// -- get_measurements_json --

#[test]
fn get_measurements_json_contains_keys() {
    let obj = br#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 2.0 0.0
v 0.0 0.0 0.5
v 1.0 0.0 0.5
v 0.5 2.0 0.5
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 4/1/1 5/2/1 6/3/1
"#;
    let mut e = WasmEngine::new_from_obj_bytes(obj).expect("should succeed");
    let json = e.get_measurements_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(
        v.get("total_height").is_some(),
        "missing total_height in: {json}"
    );
    assert!(v.get("max_width").is_some(), "missing max_width in: {json}");
    assert!(
        v.get("shoulder_width").is_some(),
        "missing shoulder_width in: {json}"
    );
}

#[test]
fn get_measurements_json_values_non_negative() {
    let obj = br#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 2.0 0.0
v 0.0 0.0 0.5
v 1.0 0.0 0.5
v 0.5 2.0 0.5
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 4/1/1 5/2/1 6/3/1
"#;
    let mut e = WasmEngine::new_from_obj_bytes(obj).expect("should succeed");
    let json = e.get_measurements_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    for key in &["total_height", "max_width", "max_depth", "torso_height"] {
        let val = v[key].as_f64().unwrap_or(-1.0);
        assert!(val >= 0.0, "{key} should be non-negative, got {val}");
    }
}

// -- get_physics_proxies_json --

#[test]
fn get_physics_proxies_json_has_array_keys() {
    let obj = br#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 2.0 0.0
v 0.0 0.0 0.5
v 1.0 0.0 0.5
v 0.5 2.0 0.5
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 4/1/1 5/2/1 6/3/1
"#;
    let mut e = WasmEngine::new_from_obj_bytes(obj).expect("should succeed");
    let json = e.get_physics_proxies_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(v.get("capsules").is_some(), "missing capsules in: {json}");
    assert!(v.get("spheres").is_some(), "missing spheres in: {json}");
    assert!(v.get("boxes").is_some(), "missing boxes in: {json}");
}

#[test]
fn get_physics_proxies_json_capsules_have_label() {
    let obj = br#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 2.0 0.0
v 0.0 0.0 0.5
v 1.0 0.0 0.5
v 0.5 2.0 0.5
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 4/1/1 5/2/1 6/3/1
"#;
    let mut e = WasmEngine::new_from_obj_bytes(obj).expect("should succeed");
    let json = e.get_physics_proxies_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    let caps = v["capsules"].as_array().expect("should succeed");
    if !caps.is_empty() {
        for cap in caps {
            assert!(
                cap.get("label").and_then(|l| l.as_str()).is_some(),
                "capsule missing label: {cap}"
            );
        }
    }
}

// -- set_allowlist --

#[test]
fn set_allowlist_blocks_non_listed_targets() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_allowlist(&["height"]);
    let target = b"# test\n0 0.1 0.2 0.3\n";
    e.load_target_bytes("weight", target)
        .expect("should succeed");
    assert_eq!(e.target_count(), 0, "weight should be blocked by allowlist");
}

#[test]
fn set_allowlist_permits_listed_targets() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_allowlist(&["height", "muscle"]);
    let target = b"# test\n0 0.1 0.2 0.3\n";
    e.load_target_bytes("height", target)
        .expect("should succeed");
    assert_eq!(
        e.target_count(),
        1,
        "height should be permitted by allowlist"
    );
    e.load_target_bytes("muscle", target)
        .expect("should succeed");
    assert_eq!(
        e.target_count(),
        2,
        "muscle should be permitted by allowlist"
    );
    e.load_target_bytes("age", target).expect("should succeed");
    assert_eq!(e.target_count(), 2, "age should be blocked by allowlist");
}

// -- load_zip_pack_bytes --

fn make_test_zip(obj_name: &str, obj_data: &[u8], targets: &[(&str, &[u8])]) -> Vec<u8> {
    use oxihuman_export::zip_pack::{zip_bytes, ZipEntry};
    let mut entries = vec![ZipEntry {
        filename: obj_name.to_string(),
        data: obj_data.to_vec(),
    }];
    for (name, data) in targets {
        entries.push(ZipEntry {
            filename: name.to_string(),
            data: data.to_vec(),
        });
    }
    zip_bytes(&entries)
}

#[test]
fn load_zip_pack_bytes_no_targets() {
    let zip = make_test_zip("base.obj", SIMPLE_OBJ, &[]);
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let n = e.load_zip_pack_bytes(&zip).expect("should succeed");
    assert_eq!(n, 0, "expected 0 targets, got {n}");
    assert_eq!(e.vertex_count(), 3);
}

#[test]
fn load_zip_pack_bytes_with_targets() {
    let target_data = b"# t\n0 0.1 0.0 0.0\n";
    let zip = make_test_zip(
        "base.obj",
        SIMPLE_OBJ,
        &[
            ("height.target", target_data),
            ("weight.target", target_data),
        ],
    );
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let n = e.load_zip_pack_bytes(&zip).expect("should succeed");
    assert_eq!(n, 2, "expected 2 targets, got {n}");
    assert_eq!(e.target_count(), 2);
}

#[test]
fn load_zip_pack_bytes_reinitialises_engine() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let t = b"# t\n0 0.05 0.0 0.0\n";
    e.load_target_bytes("old_target", t)
        .expect("should succeed");
    assert_eq!(e.target_count(), 1);
    let zip = make_test_zip("base.obj", SIMPLE_OBJ, &[]);
    let n = e.load_zip_pack_bytes(&zip).expect("should succeed");
    assert_eq!(n, 0);
    assert_eq!(
        e.target_count(),
        0,
        "engine should be reset after zip reload"
    );
}

#[test]
fn load_zip_pack_bytes_missing_obj_returns_error() {
    use oxihuman_export::zip_pack::{zip_bytes, ZipEntry};
    let entries = vec![ZipEntry {
        filename: "readme.txt".to_string(),
        data: b"hello".to_vec(),
    }];
    let zip = zip_bytes(&entries);
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let result = e.load_zip_pack_bytes(&zip);
    assert!(result.is_err(), "expected error when no .obj present");
}

#[test]
fn load_zip_pack_bytes_invalidates_cache() {
    let zip = make_test_zip("base.obj", SIMPLE_OBJ, &[]);
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    assert!(e.has_cached_mesh());
    e.load_zip_pack_bytes(&zip).expect("should succeed");
    assert!(
        !e.has_cached_mesh(),
        "cache should be invalidated after zip load"
    );
}

#[test]
fn load_zip_pack_bytes_non_base_obj_name() {
    let zip = make_test_zip("model.obj", SIMPLE_OBJ, &[]);
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let n = e.load_zip_pack_bytes(&zip).expect("should succeed");
    assert_eq!(n, 0);
    assert_eq!(e.vertex_count(), 3);
}

// -- list_loaded_targets --

#[test]
fn list_loaded_targets_empty_is_array() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.list_loaded_targets();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(v.is_array(), "expected JSON array, got: {json}");
    assert_eq!(v.as_array().expect("should succeed").len(), 0);
}

#[test]
fn list_loaded_targets_contains_names() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let t = b"# t\n0 0.1 0.0 0.0\n";
    e.load_target_bytes("height", t).expect("should succeed");
    e.load_target_bytes("weight", t).expect("should succeed");
    let json = e.list_loaded_targets();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    let arr = v.as_array().expect("should succeed");
    assert_eq!(arr.len(), 2);
    let names: Vec<&str> = arr
        .iter()
        .map(|x| x.as_str().expect("should succeed"))
        .collect();
    assert!(names.contains(&"height"), "missing height: {json}");
    assert!(names.contains(&"weight"), "missing weight: {json}");
}

#[test]
fn list_loaded_targets_after_zip_load() {
    let target_data = b"# t\n0 0.1 0.0 0.0\n";
    let zip = make_test_zip(
        "base.obj",
        SIMPLE_OBJ,
        &[("muscle.target", target_data), ("age.target", target_data)],
    );
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.load_zip_pack_bytes(&zip).expect("should succeed");
    let json = e.list_loaded_targets();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    let arr = v.as_array().expect("should succeed");
    assert_eq!(arr.len(), 2, "expected 2 targets: {json}");
}

#[test]
fn list_loaded_targets_clears_after_zip_reload() {
    let t = b"# t\n0 0.1 0.0 0.0\n";
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.load_target_bytes("height", t).expect("should succeed");
    let zip = make_test_zip("base.obj", SIMPLE_OBJ, &[]);
    e.load_zip_pack_bytes(&zip).expect("should succeed");
    let json = e.list_loaded_targets();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(
        v.as_array().expect("should succeed").len(),
        0,
        "names should be cleared: {json}"
    );
}

// -- export_quantized_bytes --

#[test]
fn export_quantized_bytes_starts_with_qmsh() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let bytes = e.export_quantized_bytes();
    assert!(bytes.len() >= 16, "too short: {} bytes", bytes.len());
    assert_eq!(&bytes[0..4], b"QMSH", "expected QMSH magic");
}

#[test]
fn export_quantized_bytes_version_one() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let bytes = e.export_quantized_bytes();
    let version = u32::from_le_bytes(bytes[4..8].try_into().expect("should succeed"));
    assert_eq!(version, 1);
}

#[test]
fn export_quantized_bytes_correct_vertex_count() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let bytes = e.export_quantized_bytes();
    let vc = u32::from_le_bytes(bytes[8..12].try_into().expect("should succeed"));
    assert_eq!(vc, 3);
}

#[test]
fn export_quantized_bytes_readable_by_read_quantized_bin() {
    use oxihuman_export::mesh_quantize::read_quantized_bin;
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let bytes = e.export_quantized_bytes();
    let tmp = std::env::temp_dir().join("oxihuman_wasm_qmsh_test.bin");
    std::fs::write(&tmp, &bytes).expect("should succeed");
    let q = read_quantized_bin(&tmp).expect("read_quantized_bin failed");
    assert_eq!(q.positions.len(), 3);
    assert_eq!(q.indices.len(), 3);
}

#[test]
fn export_quantized_bytes_populates_cache() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    assert!(!e.has_cached_mesh());
    let _ = e.export_quantized_bytes();
    assert!(
        e.has_cached_mesh(),
        "export_quantized_bytes should populate cache"
    );
}

#[test]
fn export_quantized_bytes_smaller_than_float_mesh() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let qbytes = e.export_quantized_bytes();
    let fbytes = e.build_mesh_bytes();
    assert!(!qbytes.is_empty());
    assert!(!fbytes.is_empty());
}

// -- incremental build tests --

#[test]
fn incremental_mode_matches_full_build() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let first = e.build_mesh_bytes();
    e.reset_incremental_cache();
    let second = e.build_mesh_bytes();
    assert_eq!(
        first.len(),
        second.len(),
        "incremental rebuild should produce the same byte count"
    );
}

#[test]
fn reset_incremental_cache_works() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    assert!(e.has_cached_mesh(), "cache should be populated after build");
    e.reset_incremental_cache();
    assert!(
        !e.has_cached_mesh(),
        "cache should be cleared after reset_incremental_cache"
    );
}

// -- get_physics_rig_json --

#[test]
fn get_physics_rig_json_contains_joints_key() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_physics_rig_json();
    assert!(
        json.contains("\"joints\""),
        "expected 'joints' key in: {json}"
    );
}

#[test]
fn get_physics_rig_json_is_valid_json_object() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_physics_rig_json();
    let trimmed = json.trim();
    assert!(
        trimmed.starts_with('{'),
        "expected JSON object, got: {json}"
    );
    assert!(trimmed.ends_with('}'), "expected JSON object, got: {json}");
}

// -- get_body_proportions_json --

#[test]
fn get_body_proportions_json_is_non_empty_object() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_body_proportions_json();
    assert!(
        !json.is_empty(),
        "body proportions JSON should not be empty"
    );
    let trimmed = json.trim();
    assert!(
        trimmed.starts_with('{'),
        "expected JSON object, got: {json}"
    );
    assert!(trimmed.ends_with('}'), "expected JSON object, got: {json}");
}

#[test]
fn get_body_proportions_json_contains_colon() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_body_proportions_json();
    assert!(
        json.contains(':'),
        "expected at least one key-value pair with ':' in: {json}"
    );
}

// -- set_params_from_preset --

#[test]
fn set_params_from_preset_average_sets_midpoint() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_params_from_preset("average");
    assert!((e.params.height - 0.5).abs() < 1e-6, "height should be 0.5");
    assert!((e.params.weight - 0.5).abs() < 1e-6, "weight should be 0.5");
    assert!((e.params.muscle - 0.5).abs() < 1e-6, "muscle should be 0.5");
    assert!((e.params.age - 0.5).abs() < 1e-6, "age should be 0.5");
}

#[test]
fn set_params_from_preset_invalidates_cache() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    assert!(e.has_cached_mesh());
    e.set_params_from_preset("athletic");
    assert!(
        !e.has_cached_mesh(),
        "preset should invalidate the mesh cache"
    );
}

#[test]
fn set_params_from_preset_unknown_name_is_noop() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.99);
    e.set_params_from_preset("nonexistent_preset_xyz");
    assert!(
        (e.params.height - 0.99).abs() < 1e-6,
        "unknown preset should not change params"
    );
}

// -- get_capsule_chains_json --

#[test]
fn get_capsule_chains_json_contains_spine() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_capsule_chains_json();
    let trimmed = json.trim();
    assert!(
        trimmed.starts_with('[') && trimmed.ends_with(']'),
        "expected JSON array, got: {json}"
    );
    if trimmed != "[]" {
        assert!(
            json.contains("\"spine\""),
            "expected 'spine' chain in: {json}"
        );
    }
}

#[test]
fn get_capsule_chains_json_is_valid_json_array() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_capsule_chains_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(v.is_array(), "expected JSON array, got: {json}");
}

// -- get_param_summary_json --

#[test]
fn get_param_summary_json_contains_height_and_weight() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_param_summary_json();
    assert!(json.contains("\"height\""), "missing 'height' in: {json}");
    assert!(json.contains("\"weight\""), "missing 'weight' in: {json}");
}

#[test]
fn get_param_summary_json_reflects_set_params() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.77);
    e.set_param("extra1", 0.5);
    let json = e.get_param_summary_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    let h = v["height"].as_f64().expect("should succeed");
    assert!(
        (h - 0.77_f64).abs() < 1e-4,
        "expected height ~0.77, got {h}"
    );
    let ec = v["extra_count"].as_u64().expect("should succeed");
    assert_eq!(ec, 1, "extra_count should be 1");
}

// -- Animation streaming --

#[test]
fn record_anim_frame_stores_frame() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.8);
    e.record_anim_frame();
    assert_eq!(e.anim_frame_count(), 1);
}

#[test]
fn clear_anim_frames_empties_clip() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.record_anim_frame();
    e.record_anim_frame();
    assert_eq!(e.anim_frame_count(), 2);
    e.clear_anim_frames();
    assert_eq!(e.anim_frame_count(), 0);
}

#[test]
fn anim_frame_count_zero_initially() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    assert_eq!(e.anim_frame_count(), 0);
}

#[test]
fn set_anim_fps_and_get_anim_fps() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_anim_fps(30.0);
    assert!((e.get_anim_fps() - 30.0).abs() < 1e-6);
}

#[test]
fn play_anim_step_advances_frame() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.2);
    e.record_anim_frame();
    e.set_height(0.8);
    e.record_anim_frame();
    e.set_anim_fps(10.0);
    let frame = e.play_anim_step(0.1);
    assert_eq!(frame, 1);
}

#[test]
fn play_anim_step_wraps_around() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.record_anim_frame();
    e.record_anim_frame();
    e.set_anim_fps(10.0);
    e.play_anim_step(0.2);
    let frame = e.play_anim_step(0.0);
    assert_eq!(frame, 0);
}

#[test]
fn seek_anim_frame_sets_params() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.1);
    e.record_anim_frame();
    e.set_height(0.9);
    e.record_anim_frame();
    e.seek_anim_frame(0);
    assert!(
        (e.params.height - 0.1).abs() < 1e-5,
        "height should be 0.1 after seek to frame 0"
    );
    e.seek_anim_frame(1);
    assert!(
        (e.params.height - 0.9).abs() < 1e-5,
        "height should be 0.9 after seek to frame 1"
    );
}

#[test]
fn export_anim_json_valid_json_array() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.3);
    e.record_anim_frame();
    e.set_height(0.7);
    e.record_anim_frame();
    let json = e.export_anim_json();
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("export_anim_json must be valid JSON");
    assert!(v.is_array(), "expected array, got: {json}");
    assert_eq!(v.as_array().expect("should succeed").len(), 2);
}

// -- Scene export --

#[test]
fn get_scene_json_valid_json() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_scene_json();
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("get_scene_json must be valid JSON");
    assert!(v.get("params").is_some(), "missing 'params' in: {json}");
    assert!(v.get("rig").is_some(), "missing 'rig' in: {json}");
    assert!(
        v.get("vertex_count").is_some(),
        "missing 'vertex_count' in: {json}"
    );
}

#[test]
fn get_lod_scene_json_valid_json() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_lod_scene_json(1);
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("get_lod_scene_json must be valid JSON");
    assert!(
        v.get("vertex_count").is_some(),
        "missing vertex_count in: {json}"
    );
    let lod = v["lod_level"].as_u64().expect("should succeed");
    assert_eq!(lod, 1);
}

// -- Vertex / index count --

#[test]
fn get_vertex_count_non_zero() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    assert!(e.get_vertex_count() > 0, "vertex count should be > 0");
}

#[test]
fn get_index_count_after_build() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    assert!(
        e.get_index_count() > 0,
        "index count should be > 0 after build"
    );
}

// -- reset_all_weights --

#[test]
fn reset_all_weights_resets_params() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_height(0.9);
    e.set_weight(0.1);
    e.reset_all_weights();
    assert!((e.params.height - 0.5).abs() < 1e-6, "height should be 0.5");
    assert!((e.params.weight - 0.5).abs() < 1e-6, "weight should be 0.5");
}

// -- On-demand target streaming --

#[test]
fn load_target_from_json_success() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = r#"{"deltas":[[0,0.1,0.2,0.3],[1,0.0,0.1,0.0]]}"#;
    let ok = e.load_target_from_json("test_target", json);
    assert!(ok, "load_target_from_json should succeed");
    assert_eq!(e.loaded_target_count(), 1);
}

#[test]
fn load_target_from_json_bad_json_returns_false() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let ok = e.load_target_from_json("bad", "not valid json {{{");
    assert!(!ok, "load_target_from_json should return false on bad JSON");
}

#[test]
fn unload_target_removes_target() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = r#"{"deltas":[[0,0.1,0.0,0.0]]}"#;
    e.load_target_from_json("my_target", json);
    assert_eq!(e.loaded_target_count(), 1);
    let removed = e.unload_target("my_target");
    assert!(removed, "unload_target should return true");
    assert_eq!(e.loaded_target_count(), 0);
}

#[test]
fn unload_target_returns_false_when_missing() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let removed = e.unload_target("nonexistent");
    assert!(!removed, "unload_target of nonexistent should return false");
}

#[test]
fn get_loaded_target_names_json_array() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = r#"{"deltas":[[0,0.1,0.0,0.0]]}"#;
    e.load_target_from_json("alpha", json);
    e.load_target_from_json("beta", json);
    let names_json = e.get_loaded_target_names();
    let v: serde_json::Value =
        serde_json::from_str(&names_json).expect("get_loaded_target_names must return valid JSON");
    assert!(v.is_array(), "expected array");
    let arr = v.as_array().expect("should succeed");
    assert_eq!(arr.len(), 2, "should have 2 targets");
    let names: Vec<&str> = arr
        .iter()
        .map(|x| x.as_str().expect("should succeed"))
        .collect();
    assert!(names.contains(&"alpha"), "missing 'alpha'");
    assert!(names.contains(&"beta"), "missing 'beta'");
}

#[test]
fn set_target_weight_by_name_updates_weight() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = r#"{"deltas":[[0,0.1,0.0,0.0]]}"#;
    e.load_target_from_json("morph_a", json);
    let ok = e.set_target_weight_by_name("morph_a", 0.75);
    assert!(ok, "set_target_weight_by_name should return true");
    let w = e.get_target_weight_by_name("morph_a");
    assert!((w - 0.75).abs() < 1e-6, "weight should be 0.75, got {w}");
}

#[test]
fn get_target_weight_by_name_returns_neg1_when_missing() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let w = e.get_target_weight_by_name("nonexistent");
    assert!(
        (w - (-1.0)).abs() < 1e-6,
        "should return -1.0 for missing target"
    );
}

// -- apply_preset_by_name --

#[test]
fn apply_preset_by_name_known_preset_returns_true() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let ok = e.apply_preset_by_name("athletic");
    assert!(
        ok,
        "apply_preset_by_name should return true for known preset"
    );
}

#[test]
fn apply_preset_by_name_unknown_returns_false() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let ok = e.apply_preset_by_name("totally_unknown_xyz");
    assert!(
        !ok,
        "apply_preset_by_name should return false for unknown preset"
    );
}

// -- step_physics / get_cloth_state / get_physics_proxy_json / set_wind --

#[test]
fn step_physics_does_not_panic() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.step_physics(0.016);
}

#[test]
fn get_cloth_state_returns_valid_json() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_cloth_state();
    let v: serde_json::Value = serde_json::from_str(&json).expect("cloth state must be valid JSON");
    assert!(v.get("cloth_positions").is_some());
}

#[test]
fn get_physics_proxy_json_returns_valid_json() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_physics_proxy_json();
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("physics proxy must be valid JSON");
    assert!(v.get("proxies").is_some());
}

#[test]
fn set_wind_does_not_panic() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.set_wind(1.0, 0.0, -0.5);
}

// -- apply_expression_blend --

#[test]
fn apply_expression_blend_both_known_returns_true() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let ok = e.apply_expression_blend("athletic", "slender", 0.5);
    assert!(ok, "blend of two known presets should return true");
}

#[test]
fn apply_expression_blend_unknown_returns_false() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let ok = e.apply_expression_blend("athletic", "no_such_expr", 0.5);
    assert!(!ok, "blend with unknown preset should return false");
}

// -- get_curvature_map --

#[test]
fn get_curvature_map_returns_json_array() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_curvature_map();
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("curvature map must be valid JSON");
    assert!(v.is_array(), "curvature map must be an array");
}

#[test]
fn get_curvature_map_length_matches_vertex_count() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let n = e.vertex_count();
    let json = e.get_curvature_map();
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(v.as_array().expect("should succeed").len(), n);
}

// -- get_geodesic_distances --

#[test]
fn get_geodesic_distances_empty_before_build() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_geodesic_distances(0);
    assert_eq!(json, "[]");
}

#[test]
fn get_geodesic_distances_after_build_is_array() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    let json = e.get_geodesic_distances(0);
    let v: serde_json::Value =
        serde_json::from_str(&json).expect("geodesic distances must be valid JSON");
    assert!(v.is_array());
}

#[test]
fn get_geodesic_distances_source_zero() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    let json = e.get_geodesic_distances(0);
    let v: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    let arr = v.as_array().expect("should succeed");
    assert!((arr[0].as_f64().expect("should succeed") as f32).abs() < 1e-5);
}

#[test]
fn get_geodesic_distances_out_of_bounds_returns_empty() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    let json = e.get_geodesic_distances(999999);
    assert_eq!(json, "[]");
}

// -- query_sphere_near_point --

#[test]
fn query_sphere_near_point_empty_before_build() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.query_sphere_near_point(0.0, 0.0, 0.0, 100.0);
    assert_eq!(json, "[]", "no mesh built yet - should return empty array");
}

#[test]
fn query_sphere_near_point_after_build_returns_array() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    let json = e.query_sphere_near_point(0.0, 0.0, 0.0, 1000.0);
    let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(v.is_array(), "must be array");
}

// -- get_mesh_segments --

#[test]
fn get_mesh_segments_empty_before_build() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.get_mesh_segments("connected");
    let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert_eq!(v["segment_count"].as_u64().expect("should succeed"), 0);
}

#[test]
fn get_mesh_segments_connected_after_build() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let _ = e.build_mesh_bytes();
    let json = e.get_mesh_segments("connected");
    let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(v["segment_count"].as_u64().expect("should succeed") >= 1);
}

// -- create_particle_system / step_particles --

#[test]
fn create_particle_system_returns_true() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    assert!(e.create_particle_system(10.0, 2.0));
}

#[test]
fn step_particles_without_system_returns_empty() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.step_particles(0.016);
    let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert_eq!(v["active"].as_u64().expect("should succeed"), 0);
}

#[test]
fn step_particles_after_create_emits_particles() {
    let mut e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    e.create_particle_system(100.0, 5.0);
    let json = e.step_particles(0.1);
    let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    let active = v["active"].as_u64().expect("should succeed");
    assert!(
        active >= 1,
        "expected at least 1 particle after step, got {}",
        active
    );
}

// -- list_builtin_shaders --

#[test]
fn list_builtin_shaders_returns_json_array() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.list_builtin_shaders();
    let v: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(v.is_array());
}

#[test]
fn list_builtin_shaders_contains_pbr_shaders() {
    let e = WasmEngine::new_from_obj_bytes(SIMPLE_OBJ).expect("should succeed");
    let json = e.list_builtin_shaders();
    assert!(json.contains("pbr_vertex") && json.contains("pbr_fragment"));
}
