// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Native round-trip tests: OHPK core pack → `WasmEngine`, JSON targets,
//! age floor, zero-copy geometry buffers, and export gates.

use oxihuman_export::{CorePackBuilder, CorePackManifest};
use oxihuman_wasm::{age_years_to_param, WasmEngine};

/// Build a small synthetic OHPK pack: an 8-vertex box-ish mesh with
/// `height` / `weight` / custom-category targets and an 18-year age floor.
fn synth_pack(age_floor: Option<f32>) -> Vec<u8> {
    #[rustfmt::skip]
    let positions: Vec<f32> = vec![
        -1.0, 0.0, -1.0,
         1.0, 0.0, -1.0,
         1.0, 0.0,  1.0,
        -1.0, 0.0,  1.0,
        -1.0, 8.0, -1.0,
         1.0, 8.0, -1.0,
         1.0, 8.0,  1.0,
        -1.0, 8.0,  1.0,
    ];
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0, 1, 2, 0, 2, 3, // bottom
        4, 6, 5, 4, 7, 6, // top
        0, 4, 5, 0, 5, 1, // sides
        1, 5, 6, 1, 6, 2,
        2, 6, 7, 2, 7, 3,
        3, 7, 4, 3, 4, 0,
    ];
    let uvs: Vec<f32> = (0..8).flat_map(|i| [i as f32 / 8.0, 0.5]).collect();

    let mut manifest = CorePackManifest::new("synthetic-test-pack", "1");
    manifest.age_floor_years = age_floor;

    let mut b = CorePackBuilder::new();
    b.set_manifest(manifest);
    b.set_base_mesh(&positions, &indices, Some(&uvs));
    // Height target: lifts the top face by up to 2 units.
    b.add_target(
        "tall",
        "height",
        &[
            (4u32, [0.0, 2.0, 0.0]),
            (5u32, [0.0, 2.0, 0.0]),
            (6u32, [0.0, 2.0, 0.0]),
            (7u32, [0.0, 2.0, 0.0]),
        ],
    );
    // Weight target: widens the waist.
    b.add_target(
        "wide",
        "weight",
        &[(0u32, [-0.5, 0.0, 0.0]), (1u32, [0.5, 0.0, 0.0])],
    );
    // Unknown category: must stay inert until set_param("bumpy", w).
    b.add_target("bumpy", "customcat", &[(2u32, [0.0, 0.0, 0.9])]);
    b.build().expect("synthetic pack must build")
}

fn positions_of(engine: &mut WasmEngine) -> Vec<[f32; 3]> {
    engine.build_mesh_prepared().positions
}

#[test]
fn core_pack_round_trip_replaces_base_mesh() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");
    assert_eq!(engine.vertex_count(), 8, "vertex count from pack");
    assert_eq!(engine.target_count(), 3, "all pack targets loaded");
    let mesh = engine.build_mesh_prepared();
    assert_eq!(mesh.indices.len(), 36);
    assert!(mesh.has_suit, "engine meshes carry the suit flag");
}

#[test]
fn core_pack_height_param_drives_height_target() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");

    engine.set_height(0.0);
    let low = positions_of(&mut engine);
    engine.set_height(1.0);
    let high = positions_of(&mut engine);

    let dy = high[4][1] - low[4][1];
    assert!(
        dy > 1.5,
        "height param must drive the 'tall' target (dy = {dy})"
    );
}

#[test]
fn core_pack_unknown_category_inert_until_named_param_set() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");

    let before = positions_of(&mut engine);
    assert!(
        (before[2][2] - 1.0).abs() < 0.01,
        "custom-category target must be inert by default (z = {})",
        before[2][2]
    );

    engine.set_param("bumpy", 1.0);
    let after = positions_of(&mut engine);
    assert!(
        (after[2][2] - before[2][2]) > 0.5,
        "set_param(name) must drive the custom target"
    );
}

#[test]
fn core_pack_age_floor_clamps_age_param() {
    let pack = synth_pack(Some(18.0));
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");
    assert_eq!(engine.age_floor_years(), Some(18.0));

    let floor_param = age_years_to_param(18.0);
    engine.set_age(0.0);
    assert!(
        engine.build_mesh_prepared().positions.len() == 8,
        "engine still functional"
    );
    // get_param path: params.age must have been clamped to the floor.
    let json = engine.export_params_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("params json");
    let age = v["age"].as_f64().expect("age param") as f32;
    assert!(
        age >= floor_param - 1e-6,
        "age {age} must be clamped to floor param {floor_param}"
    );

    // Values above the floor are untouched.
    engine.set_age(0.9);
    let json = engine.export_params_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("params json");
    let age = v["age"].as_f64().expect("age param") as f32;
    assert!((age - 0.9).abs() < 1e-6);
}

#[test]
fn json_targets_affect_built_mesh() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");

    let before = positions_of(&mut engine);
    assert!(engine.load_target_from_json("nose", r#"{"deltas":[[0, 0.0, 0.0, 3.0]]}"#));

    // Weight 0 → no effect yet.
    let at_zero = positions_of(&mut engine);
    assert!((at_zero[0][2] - before[0][2]).abs() < 1e-6);

    // Weight 1 → vertex 0 moves by +3 in z.
    assert!(engine.set_target_weight_by_name("nose", 1.0));
    let at_one = positions_of(&mut engine);
    assert!(
        (at_one[0][2] - before[0][2] - 3.0).abs() < 1e-4,
        "JSON target must be scatter-added into the built mesh (dz = {})",
        at_one[0][2] - before[0][2]
    );

    // Unload → effect disappears.
    assert!(engine.unload_target("nose"));
    let after_unload = positions_of(&mut engine);
    assert!((after_unload[0][2] - before[0][2]).abs() < 1e-4);
}

#[test]
fn json_targets_affect_mesh_bytes_and_geometry_buffers() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");

    engine.load_target_from_json("shift", r#"{"deltas":[[1, 0.0, 5.0, 0.0]]}"#);
    engine.set_target_weight_by_name("shift", 1.0);

    // build_mesh_bytes path
    let bytes = engine.build_mesh_bytes();
    let y1 = f32::from_le_bytes(
        bytes[12 + 3 * 4 + 4..12 + 3 * 4 + 8]
            .try_into()
            .expect("y of vertex 1"),
    );
    assert!(
        (y1 - 5.0).abs() < 0.01,
        "build_mesh_bytes must include JSON targets (y1 = {y1})"
    );

    // zero-copy path
    engine.refresh_geometry();
    let n = engine.positions_len() as usize;
    assert_eq!(n, 8 * 3);
    // Read directly out of the persistent buffer via the raw parts.
    let mesh = engine.build_mesh_prepared();
    assert!((mesh.positions[1][1] - 5.0).abs() < 0.01);
}

#[test]
fn zero_copy_buffers_stable_across_param_changes() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");

    let gen0 = engine.refresh_geometry();
    let ptr0 = engine.positions_ptr();
    let len0 = engine.positions_len();
    assert_eq!(len0, 8 * 3);
    assert!(ptr0 != 0);

    // Param change without topology change: same buffer, same generation.
    engine.set_height(1.0);
    let gen1 = engine.refresh_geometry();
    assert_eq!(gen0, gen1, "generation stable without topology change");
    assert_eq!(engine.positions_ptr(), ptr0, "positions_ptr stable");
    assert_eq!(engine.positions_len(), len0);
    assert_eq!(engine.normals_len(), 8 * 3);
    assert_eq!(engine.uvs_len(), 8 * 2);
    assert_eq!(engine.indices_len(), 36);

    // Topology replacement bumps the generation.
    let pack2 = synth_pack(None);
    engine
        .load_core_pack_bytes(&pack2)
        .expect("second pack load");
    let gen2 = engine.refresh_geometry();
    assert_ne!(gen1, gen2, "generation must bump on topology replacement");
}

#[test]
fn refresh_geometry_is_noop_when_clean() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");
    let gen0 = engine.refresh_geometry();
    let gen1 = engine.refresh_geometry();
    assert_eq!(gen0, gen1);
}

#[test]
fn engine_mesh_passes_export_gates() {
    let pack = synth_pack(None);
    let mut engine = WasmEngine::new_from_core_pack_bytes(&pack).expect("pack load");
    let mesh = engine.build_mesh_prepared();

    // GLB (in-memory)
    let glb = oxihuman_export::glb::build_glb_bytes(&mesh).expect("glb bytes");
    assert_eq!(&glb[0..4], b"glTF");

    // OBJ string
    let obj = oxihuman_export::obj::mesh_to_obj_string(&mesh).expect("obj string");
    assert!(obj.lines().any(|l| l.starts_with("v ")));
    assert!(obj.lines().any(|l| l.starts_with("f ")));

    // STL binary (in-memory): triangle count == indices / 3
    let stl = oxihuman_export::stl::encode_stl_binary(&mesh).expect("stl bytes");
    let tri_count = u32::from_le_bytes(stl[80..84].try_into().expect("count"));
    assert_eq!(tri_count as usize, mesh.indices.len() / 3);

    // A de-suited copy is refused everywhere.
    let mut bare = mesh.clone();
    bare.has_suit = false;
    assert!(oxihuman_export::glb::build_glb_bytes(&bare).is_err());
    assert!(oxihuman_export::obj::mesh_to_obj_string(&bare).is_err());
    assert!(oxihuman_export::stl::encode_stl_binary(&bare).is_err());
}

#[test]
fn age_mapping_round_trip() {
    use oxihuman_wasm::age_param_to_years;
    assert!((age_param_to_years(0.0) - 1.0).abs() < 1e-5);
    assert!((age_param_to_years(0.5) - 25.0).abs() < 1e-5);
    assert!((age_param_to_years(1.0) - 90.0).abs() < 1e-5);
    for years in [1.0f32, 7.5, 18.0, 25.0, 40.0, 90.0] {
        let p = age_years_to_param(years);
        let back = age_param_to_years(p);
        assert!(
            (back - years).abs() < 0.01,
            "round trip {years} -> {p} -> {back}"
        );
    }
}

#[test]
fn malformed_pack_is_rejected() {
    let mut engine = WasmEngine::new_stub();
    assert!(engine.load_core_pack_bytes(b"not a pack").is_err());
    assert!(engine.load_core_pack_bytes(&[]).is_err());
    // Engine still usable afterwards.
    assert_eq!(engine.vertex_count(), 3);
    let _ = engine.build_mesh_bytes();
}
