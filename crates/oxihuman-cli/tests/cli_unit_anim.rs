// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Unit-level tests for CLI animation export (pc2/mdd de-stub), streaming
//! export, plugin/camera info, remesh, and physics export.

use oxihuman_cli::commands;
use oxihuman_core::default_builtin_plugins;

// ── pc2 / mdd de-stub (real frame evaluation) ───────────────────────────────

/// A minimal 3-vertex triangle OBJ, shared by the pc2/mdd de-stub tests.
fn tri_obj_src() -> &'static str {
    "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n"
}

/// Write a `.target` file that displaces all 3 triangle vertices by `+X`,
/// weighted by the `height` param (via `auto_weight_fn_for_target`'s
/// substring-matching on the filename `height.target`).
fn write_height_target_dir(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).expect("should succeed");
    std::fs::write(
        dir.join("height.target"),
        "0 1.0 0.0 0.0\n1 1.0 0.0 0.0\n2 1.0 0.0 0.0\n",
    )
    .expect("should succeed");
}

#[test]
fn pc2_header_layout_bytes_exact() {
    let dir = std::env::temp_dir().join("oxihuman_test_pc2_header");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let out = dir.join("out.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");

    assert!(commands::anim::cmd_pc2(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--frames".to_string(),
        "3".to_string(),
        "--fps".to_string(),
        "24".to_string(),
        "--start-time".to_string(),
        "0.5".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    // 32-byte header: 12-byte magic + i32 version + i32 numPoints + f32
    // startFrame + f32 sampleRate + i32 numSamples, then f32 xyz per point
    // per frame, little-endian throughout.
    assert_eq!(&bytes[..12], b"POINTCACHE2\0");
    assert_eq!(
        i32::from_le_bytes(bytes[12..16].try_into().expect("4 bytes")),
        1
    );
    assert_eq!(
        i32::from_le_bytes(bytes[16..20].try_into().expect("4 bytes")),
        3
    ); // numPoints
    assert!((f32::from_le_bytes(bytes[20..24].try_into().expect("4 bytes")) - 0.5).abs() < 1e-6);
    assert!((f32::from_le_bytes(bytes[24..28].try_into().expect("4 bytes")) - 24.0).abs() < 1e-6);
    assert_eq!(
        i32::from_le_bytes(bytes[28..32].try_into().expect("4 bytes")),
        3
    ); // numSamples
    assert_eq!(bytes.len(), 32 + 3 /*points*/ * 3 /*frames*/ * 12);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mdd_header_layout_bytes_exact() {
    let dir = std::env::temp_dir().join("oxihuman_test_mdd_header");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let out = dir.join("out.mdd");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");

    assert!(commands::anim::cmd_mdd(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--frames".to_string(),
        "4".to_string(),
        "--fps".to_string(),
        "10".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    // big-endian: i32 numFrames, i32 numPoints, f32 times[numFrames], then
    // f32 xyz per point per frame.
    assert_eq!(
        i32::from_be_bytes(bytes[0..4].try_into().expect("4 bytes")),
        4
    ); // numFrames
    assert_eq!(
        i32::from_be_bytes(bytes[4..8].try_into().expect("4 bytes")),
        3
    ); // numPoints
    let t1 = f32::from_be_bytes(bytes[12..16].try_into().expect("4 bytes"));
    assert!((t1 - 0.1).abs() < 1e-5); // frame 1 at 1/fps seconds
    let expected_len = 8 + 4 * 4 /*times*/ + 4 * 3 * 3 * 4 /*frames*points*xyz*/;
    assert_eq!(bytes.len(), expected_len);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pc2_frames_differ_when_params_animate_with_targets() {
    let dir = std::env::temp_dir().join("oxihuman_test_pc2_animated");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let targets_dir = dir.join("targets");
    let anim_json = dir.join("anim.json");
    let out = dir.join("out.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");
    write_height_target_dir(&targets_dir);
    std::fs::write(
        &anim_json,
        r#"{"interp":"linear","keyframes":[
            {"time":0.0,"params":{"height":0.0}},
            {"time":1.0,"params":{"height":1.0}}]}"#,
    )
    .expect("should succeed");

    assert!(commands::anim::cmd_pc2(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().to_string(),
        "--anim".to_string(),
        anim_json.to_string_lossy().to_string(),
        "--frames".to_string(),
        "3".to_string(),
        "--fps".to_string(),
        "2".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    let cache = oxihuman_export::read_pc2(&bytes).expect("should parse back");
    assert_eq!(cache.header.point_count, 3);
    assert_eq!(cache.header.sample_count, 3);
    // height goes 0.0 -> 0.5 -> 1.0 across the three frames (t=0,0.5,1 at 2fps)
    // so the +X-displaced vertex 0 must differ frame-to-frame.
    assert_ne!(cache.frames[0][0], cache.frames[1][0]);
    assert_ne!(cache.frames[1][0], cache.frames[2][0]);
    assert_ne!(cache.frames[0][0], cache.frames[2][0]);
    // First frame (height=0) must equal the untouched base position.
    assert!((cache.frames[0][0][0] - 0.0).abs() < 1e-5);
    // Last frame (height=1) must be displaced by +1 in X.
    assert!((cache.frames[2][0][0] - 1.0).abs() < 1e-5);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mdd_frames_differ_when_params_animate_with_targets() {
    let dir = std::env::temp_dir().join("oxihuman_test_mdd_animated");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let targets_dir = dir.join("targets");
    let anim_json = dir.join("anim.json");
    let out = dir.join("out.mdd");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");
    write_height_target_dir(&targets_dir);
    std::fs::write(
        &anim_json,
        r#"[{"height":0.0},{"height":1.0}]"#, // dense snapshot form
    )
    .expect("should succeed");

    assert!(commands::anim::cmd_mdd(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().to_string(),
        "--anim".to_string(),
        anim_json.to_string_lossy().to_string(),
        "--fps".to_string(),
        "24".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    let cache = oxihuman_export::read_mdd(&bytes).expect("should parse back");
    assert_eq!(cache.point_count, 3);
    // dense snapshot form fixes frame count to the array length (2), even
    // though no --frames flag was given.
    assert_eq!(cache.frames.len(), 2);
    assert_ne!(cache.frames[0][0], cache.frames[1][0]);
    assert!((cache.frames[0][0][0] - 0.0).abs() < 1e-5);
    assert!((cache.frames[1][0][0] - 1.0).abs() < 1e-5);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pc2_without_anim_source_is_honestly_static_not_fake() {
    // No --anim / --targets at all: every frame must equal the base mesh
    // exactly (this is the mathematically correct output, not a stub).
    let dir = std::env::temp_dir().join("oxihuman_test_pc2_static");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let out = dir.join("out.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");

    assert!(commands::anim::cmd_pc2(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--frames".to_string(),
        "3".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    let cache = oxihuman_export::read_pc2(&bytes).expect("should parse back");
    assert_eq!(cache.frames[0], cache.frames[1]);
    assert_eq!(cache.frames[1], cache.frames[2]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pc2_anim_without_targets_errors_is_avoided_but_stays_honest() {
    // --anim without --targets: still succeeds (there's simply nothing to
    // morph), and every frame stays identical to the base mesh — an honest
    // degenerate case rather than a silently-ignored animation input.
    let dir = std::env::temp_dir().join("oxihuman_test_pc2_anim_no_targets");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let anim_json = dir.join("anim.json");
    let out = dir.join("out.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");
    std::fs::write(
        &anim_json,
        r#"{"keyframes":[{"time":0.0,"params":{"height":0.0}},{"time":1.0,"params":{"height":1.0}}]}"#,
    )
    .expect("should succeed");

    assert!(commands::anim::cmd_pc2(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--anim".to_string(),
        anim_json.to_string_lossy().to_string(),
        "--frames".to_string(),
        "2".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    let cache = oxihuman_export::read_pc2(&bytes).expect("should parse back");
    assert_eq!(cache.frames[0], cache.frames[1]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pc2_empty_targets_dir_errors() {
    let dir = std::env::temp_dir().join("oxihuman_test_pc2_empty_targets");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let targets_dir = dir.join("targets_empty");
    let out = dir.join("out.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");
    std::fs::create_dir_all(&targets_dir).expect("should succeed");

    assert!(commands::anim::cmd_pc2(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().to_string(),
    ])
    .is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pc2_deterministic_same_output_twice() {
    let dir = std::env::temp_dir().join("oxihuman_test_pc2_determinism");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let targets_dir = dir.join("targets");
    let anim_json = dir.join("anim.json");
    let out1 = dir.join("out1.pc2");
    let out2 = dir.join("out2.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");
    write_height_target_dir(&targets_dir);
    std::fs::write(
        &anim_json,
        r#"[{"height":0.2},{"height":0.6},{"height":0.9}]"#,
    )
    .expect("should succeed");

    for out in [&out1, &out2] {
        assert!(commands::anim::cmd_pc2(&[
            "--input".to_string(),
            obj.to_string_lossy().to_string(),
            "--output".to_string(),
            out.to_string_lossy().to_string(),
            "--targets".to_string(),
            targets_dir.to_string_lossy().to_string(),
            "--anim".to_string(),
            anim_json.to_string_lossy().to_string(),
            "--fps".to_string(),
            "5".to_string(),
        ])
        .is_ok());
    }

    let bytes1 = std::fs::read(&out1).expect("should succeed");
    let bytes2 = std::fs::read(&out2).expect("should succeed");
    assert_eq!(
        bytes1, bytes2,
        "same inputs must produce byte-identical output"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn anim_bake_frames_differ_with_targets() {
    let dir = std::env::temp_dir().join("oxihuman_test_anim_bake_animated");
    std::fs::create_dir_all(&dir).expect("should succeed");
    let obj = dir.join("base.obj");
    let targets_dir = dir.join("targets");
    let params = dir.join("params.json");
    let out = dir.join("out.pc2");
    std::fs::write(&obj, tri_obj_src()).expect("should succeed");
    write_height_target_dir(&targets_dir);
    std::fs::write(&params, r#"[{"height":0.0},{"height":1.0}]"#).expect("should succeed");

    assert!(commands::anim::cmd_anim_bake(&[
        "--input".to_string(),
        obj.to_string_lossy().to_string(),
        "--params-json".to_string(),
        params.to_string_lossy().to_string(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().to_string(),
        "--output".to_string(),
        out.to_string_lossy().to_string(),
        "--format".to_string(),
        "pc2".to_string(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out).expect("should succeed");
    let cache = oxihuman_export::read_pc2(&bytes).expect("should parse back");
    assert_eq!(cache.header.sample_count, 2);
    assert_ne!(cache.frames[0][0], cache.frames[1][0]);

    let _ = std::fs::remove_dir_all(&dir);
}

// ── stream-export ─────────────────────────────────────────────────────────────

#[test]
fn stream_export_missing_input_errors() {
    assert!(commands::anim::cmd_stream_export(&[
        "--output".to_string(),
        "/tmp/se_out.bin".to_string()
    ])
    .is_err());
}

#[test]
fn stream_export_invalid_format_errors() {
    let obj = "/tmp/test_se_fmt.obj";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::anim::cmd_stream_export(&[
        "--input".to_string(),
        obj.to_string(),
        "--output".to_string(),
        "/tmp/se_out.bin".to_string(),
        "--format".to_string(),
        "badformat".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj);
}

#[test]
fn stream_export_f32_succeeds() {
    let obj = "/tmp/test_se_f32.obj";
    let out = "/tmp/test_se_f32.bin";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::anim::cmd_stream_export(&[
        "--input".to_string(),
        obj.to_string(),
        "--output".to_string(),
        out.to_string(),
        "--format".to_string(),
        "f32".to_string()
    ])
    .is_ok());
    let bytes = std::fs::read(out).expect("should succeed");
    assert_eq!(bytes.len(), 36);
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(out);
}

#[test]
fn stream_export_csv_succeeds() {
    let obj = "/tmp/test_se_csv.obj";
    let out = "/tmp/test_se_csv.csv";
    std::fs::write(obj, "v 1 2 3\nv 4 5 6\nf 1 2 1\n").expect("should succeed");
    assert!(commands::anim::cmd_stream_export(&[
        "--input".to_string(),
        obj.to_string(),
        "--output".to_string(),
        out.to_string(),
        "--format".to_string(),
        "csv".to_string()
    ])
    .is_ok());
    let content = std::fs::read_to_string(out).expect("should succeed");
    assert_eq!(content.lines().count(), 2);
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(out);
}

// ── plugin-list / camera-info ─────────────────────────────────────────────────

#[test]
fn plugin_list_runs_without_panic() {
    commands::info::cmd_plugin_list();
}

#[test]
fn default_builtin_plugins_has_six() {
    assert!(default_builtin_plugins().len() >= 6);
}

#[test]
fn camera_info_runs_without_panic() {
    commands::info::cmd_camera_info();
}

// ── remesh ────────────────────────────────────────────────────────────────────

/// A small closed tetrahedron (4 vertices, 4 triangular faces) — gives the
/// voxel remesh path a genuine, non-degenerate solid to reconstruct.
const TETRAHEDRON_OBJ: &str =
    "v 0 0 0\nv 1 0 0\nv 0 1 0\nv 0 0 1\nf 1 2 3\nf 1 2 4\nf 1 3 4\nf 2 3 4\n";

#[test]
fn remesh_missing_input_errors() {
    assert!(commands::misc::cmd_remesh(&["--voxel-size".to_string(), "0.1".to_string()]).is_err());
}

#[test]
fn remesh_nonexistent_input_errors() {
    let missing = std::env::temp_dir().join("oxihuman_remesh_nonexistent_xyz.obj");
    assert!(commands::misc::cmd_remesh(&[missing.to_string_lossy().into_owned()]).is_err());
}

#[test]
fn remesh_succeeds_with_existing_file() {
    let obj = std::env::temp_dir().join("oxihuman_test_remesh_input.obj");
    std::fs::write(&obj, TETRAHEDRON_OBJ).expect("should succeed");
    assert!(commands::misc::cmd_remesh(&[
        obj.to_string_lossy().into_owned(),
        "--voxel-size".to_string(),
        "0.3".to_string(),
        "--iters".to_string(),
        "1".to_string()
    ])
    .is_ok());
    let _ = std::fs::remove_file(&obj);
}

#[test]
fn remesh_writes_output_obj_when_requested() {
    let obj = std::env::temp_dir().join("oxihuman_test_remesh_output_in.obj");
    let out = std::env::temp_dir().join("oxihuman_test_remesh_output_out.obj");
    std::fs::write(&obj, TETRAHEDRON_OBJ).expect("should succeed");
    assert!(commands::misc::cmd_remesh(&[
        obj.to_string_lossy().into_owned(),
        "--voxel-size".to_string(),
        "0.3".to_string(),
        "--output".to_string(),
        out.to_string_lossy().into_owned(),
    ])
    .is_ok());
    let content = std::fs::read_to_string(&out).expect("output OBJ should be written");
    assert!(content.contains("v "), "output OBJ should contain vertices");
    assert!(content.contains("f "), "output OBJ should contain faces");
    let _ = std::fs::remove_file(&obj);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn remesh_rejects_non_positive_voxel_size() {
    let obj = std::env::temp_dir().join("oxihuman_test_remesh_bad_voxel.obj");
    std::fs::write(&obj, TETRAHEDRON_OBJ).expect("should succeed");
    assert!(commands::misc::cmd_remesh(&[
        obj.to_string_lossy().into_owned(),
        "--voxel-size".to_string(),
        "0".to_string(),
    ])
    .is_err());
    let _ = std::fs::remove_file(&obj);
}

#[test]
fn remesh_unknown_option_errors() {
    assert!(commands::misc::cmd_remesh(&["--badopt".to_string()]).is_err());
}

// ── physics-export ────────────────────────────────────────────────────────────

#[test]
fn physics_export_missing_input_errors() {
    assert!(commands::misc::cmd_physics_export(&[
        "--format".to_string(),
        "gltf-physics".to_string()
    ])
    .is_err());
}

#[test]
fn physics_export_unknown_format_errors() {
    let obj = "/tmp/test_phys_export.obj";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::misc::cmd_physics_export(&[
        obj.to_string(),
        "--format".to_string(),
        "badformat".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj);
}

#[test]
fn physics_export_gltf_physics_to_file() {
    let obj = "/tmp/test_phys_gltf.obj";
    let out = "/tmp/test_phys_gltf_out.json";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::misc::cmd_physics_export(&[
        obj.to_string(),
        "--format".to_string(),
        "gltf-physics".to_string(),
        "--output".to_string(),
        out.to_string()
    ])
    .is_ok());
    let content = std::fs::read_to_string(out).expect("should succeed");
    assert!(content.contains("KHR_physics_rigid_bodies"));
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(out);
}

#[test]
fn physics_export_gltf_physics_depends_on_input_mesh() {
    // A small mesh vs. the same mesh scaled up 20x must produce different
    // rigid-body geometry (radii/positions), proving the physics scene is
    // built from the actual input mesh rather than a fixed hardcoded biped
    // regardless of input.
    let small_obj = std::env::temp_dir().join("oxihuman_test_phys_scale_small.obj");
    let big_obj = std::env::temp_dir().join("oxihuman_test_phys_scale_big.obj");
    std::fs::write(
        &small_obj,
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nv 0 0 1\nf 1 2 3\nf 1 2 4\nf 1 3 4\nf 2 3 4\n",
    )
    .expect("should succeed");
    std::fs::write(
        &big_obj,
        "v 0 0 0\nv 20 0 0\nv 0 20 0\nv 0 0 20\nf 1 2 3\nf 1 2 4\nf 1 3 4\nf 2 3 4\n",
    )
    .expect("should succeed");
    let small_out = std::env::temp_dir().join("oxihuman_test_phys_scale_small_out.json");
    let big_out = std::env::temp_dir().join("oxihuman_test_phys_scale_big_out.json");

    assert!(commands::misc::cmd_physics_export(&[
        small_obj.to_string_lossy().into_owned(),
        "--format".to_string(),
        "gltf-physics".to_string(),
        "--output".to_string(),
        small_out.to_string_lossy().into_owned(),
    ])
    .is_ok());
    assert!(commands::misc::cmd_physics_export(&[
        big_obj.to_string_lossy().into_owned(),
        "--format".to_string(),
        "gltf-physics".to_string(),
        "--output".to_string(),
        big_out.to_string_lossy().into_owned(),
    ])
    .is_ok());

    let small_json = std::fs::read_to_string(&small_out).expect("should read output");
    let big_json = std::fs::read_to_string(&big_out).expect("should read output");
    assert!(small_json.contains("KHR_physics_rigid_bodies"));
    assert!(big_json.contains("KHR_physics_rigid_bodies"));
    assert_ne!(
        small_json, big_json,
        "physics scene must reflect the actual input mesh, not a fixed hardcoded biped"
    );

    let _ = std::fs::remove_file(&small_obj);
    let _ = std::fs::remove_file(&big_obj);
    let _ = std::fs::remove_file(&small_out);
    let _ = std::fs::remove_file(&big_out);
}

#[test]
fn physics_export_openxr_to_file() {
    let obj = "/tmp/test_phys_xr.obj";
    let out = "/tmp/test_phys_xr_out.json";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::misc::cmd_physics_export(&[
        obj.to_string(),
        "--format".to_string(),
        "openxr".to_string(),
        "--output".to_string(),
        out.to_string()
    ])
    .is_ok());
    let content = std::fs::read_to_string(out).expect("should succeed");
    assert!(content.contains("OxiHuman"));
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(out);
}
