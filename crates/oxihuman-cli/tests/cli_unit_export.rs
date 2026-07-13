// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Unit-level tests for CLI mesh export subcommands: zip-pack, STL, Collada,
//! glTF-separate, SVG, and LOD export.

use oxihuman_cli::commands;

// ── zip-pack ──────────────────────────────────────────────────────────────────

#[test]
fn zip_pack_missing_base_errors() {
    assert!(commands::pack::cmd_zip_pack(&[
        "--targets".to_string(),
        "/tmp".to_string(),
        "--output".to_string(),
        "/tmp/out.zip".to_string()
    ])
    .is_err());
}

#[test]
fn zip_pack_missing_targets_errors() {
    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        "/tmp/dummy.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.zip".to_string()
    ])
    .is_err());
}

#[test]
fn zip_pack_missing_output_errors() {
    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        "/tmp/dummy.obj".to_string(),
        "--targets".to_string(),
        "/tmp".to_string()
    ])
    .is_err());
}

#[test]
fn zip_pack_nonexistent_base_errors() {
    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        "/nonexistent_base.obj".to_string(),
        "--targets".to_string(),
        "/tmp".to_string(),
        "--output".to_string(),
        "/tmp/out.zip".to_string()
    ])
    .is_err());
}

#[test]
fn zip_pack_nonexistent_targets_dir_errors() {
    let obj_path = "/tmp/test_zippack_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--targets".to_string(),
        "/nonexistent_targets_dir_zip".to_string(),
        "--output".to_string(),
        "/tmp/out.zip".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj_path);
}

#[test]
fn zip_pack_valid_inputs_succeeds() {
    let obj_path = "/tmp/test_zippack_valid_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = "/tmp/test_zippack_valid_targets";
    std::fs::create_dir_all(targets_dir).expect("should succeed");
    let out_path = "/tmp/test_zippack_valid_out.zip";
    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--targets".to_string(),
        targets_dir.to_string(),
        "--output".to_string(),
        out_path.to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
    let _ = std::fs::remove_dir(targets_dir);
}

#[test]
fn zip_pack_unknown_option_errors() {
    assert!(commands::pack::cmd_zip_pack(&["--unknown-flag".to_string()]).is_err());
}

#[test]
fn zip_pack_filters_blocked_targets_from_manifest() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_zippack_policy_base.obj");
    std::fs::write(&obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = std::env::temp_dir().join("oxihuman_test_zippack_policy_targets");
    std::fs::create_dir_all(&targets_dir).expect("should succeed");
    std::fs::write(targets_dir.join("height-up.target"), "0 0.1 0.2 0.3\n")
        .expect("should succeed");
    std::fs::write(targets_dir.join("nudity-test.target"), "0 0.1 0.2 0.3\n")
        .expect("should succeed");
    let out_path = std::env::temp_dir().join("oxihuman_test_zippack_policy_out.zip");

    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().into_owned(),
        "--output".to_string(),
        out_path.to_string_lossy().into_owned(),
    ])
    .is_ok());

    let names = oxihuman_export::read_zip_entry_names(&out_path).expect("should list entries");
    assert!(names.contains(&"manifest.json".to_string()));
    // The manifest.json entry embeds the (filtered) target file list; the
    // blocked-tag name must never appear anywhere in the archive.
    let raw = std::fs::read(&out_path).expect("should read zip bytes");
    let raw_str = String::from_utf8_lossy(&raw);
    assert!(
        !raw_str.contains("nudity-test"),
        "blocked-tag target name leaked into the pack"
    );

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_dir_all(&targets_dir);
}

#[test]
fn zip_pack_deflate_flag_produces_valid_readable_archive() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_zippack_deflate_base.obj");
    std::fs::write(&obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = std::env::temp_dir().join("oxihuman_test_zippack_deflate_targets");
    std::fs::create_dir_all(&targets_dir).expect("should succeed");
    let out_path = std::env::temp_dir().join("oxihuman_test_zippack_deflate_out.zip");

    assert!(commands::pack::cmd_zip_pack(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().into_owned(),
        "--output".to_string(),
        out_path.to_string_lossy().into_owned(),
        "--deflate".to_string(),
    ])
    .is_ok());

    let names = oxihuman_export::read_zip_entry_names(&out_path).expect("should list entries");
    assert!(names.contains(&"mesh.glb".to_string()));
    assert!(names.contains(&"params.json".to_string()));
    assert!(names.contains(&"manifest.json".to_string()));

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_dir_all(&targets_dir);
}

// ── stl ───────────────────────────────────────────────────────────────────────

#[test]
fn stl_missing_base_errors() {
    assert!(
        commands::export::cmd_stl(&["--output".to_string(), "/tmp/out.stl".to_string()]).is_err()
    );
}

#[test]
fn stl_missing_output_errors() {
    assert!(
        commands::export::cmd_stl(&["--base".to_string(), "/tmp/dummy.obj".to_string()]).is_err()
    );
}

#[test]
fn stl_nonexistent_base_errors() {
    assert!(commands::export::cmd_stl(&[
        "--base".to_string(),
        "/nonexistent_stl_base.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.stl".to_string()
    ])
    .is_err());
}

#[test]
fn stl_ascii_success() {
    let obj_path = "/tmp/test_stl_ascii_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_stl_ascii_out.stl";
    assert!(commands::export::cmd_stl(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn stl_binary_success() {
    let obj_path = "/tmp/test_stl_binary_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_stl_binary_out.stl";
    assert!(commands::export::cmd_stl(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
        "--binary".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn stl_unknown_option_errors() {
    assert!(commands::export::cmd_stl(&["--unknown-flag".to_string()]).is_err());
}

// ── collada ───────────────────────────────────────────────────────────────────

#[test]
fn collada_missing_base_errors() {
    assert!(
        commands::export::cmd_collada(&["--output".to_string(), "/tmp/out.dae".to_string()])
            .is_err()
    );
}

#[test]
fn collada_missing_output_errors() {
    assert!(
        commands::export::cmd_collada(&["--base".to_string(), "/tmp/dummy.obj".to_string()])
            .is_err()
    );
}

#[test]
fn collada_nonexistent_base_errors() {
    assert!(commands::export::cmd_collada(&[
        "--base".to_string(),
        "/nonexistent_collada_base.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.dae".to_string()
    ])
    .is_err());
}

#[test]
fn collada_success() {
    let obj_path = "/tmp/test_collada_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_collada_out.dae";
    assert!(commands::export::cmd_collada(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn collada_with_author_success() {
    let obj_path = "/tmp/test_collada_author_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_collada_author_out.dae";
    assert!(commands::export::cmd_collada(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
        "--author".to_string(),
        "TestAuthor".to_string()
    ])
    .is_ok());
    let content = std::fs::read_to_string(out_path).expect("should succeed");
    assert!(content.contains("TestAuthor"));
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn collada_unknown_option_errors() {
    assert!(commands::export::cmd_collada(&["--unknown-flag".to_string()]).is_err());
}

// ── gltf-sep ──────────────────────────────────────────────────────────────────

#[test]
fn gltf_sep_missing_base_errors() {
    assert!(
        commands::export::cmd_gltf_sep(&["--output".to_string(), "/tmp/out.gltf".to_string()])
            .is_err()
    );
}

#[test]
fn gltf_sep_missing_output_errors() {
    assert!(
        commands::export::cmd_gltf_sep(&["--base".to_string(), "/tmp/dummy.obj".to_string()])
            .is_err()
    );
}

#[test]
fn gltf_sep_nonexistent_base_errors() {
    assert!(commands::export::cmd_gltf_sep(&[
        "--base".to_string(),
        "/nonexistent_gltf_sep_base.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.gltf".to_string()
    ])
    .is_err());
}

#[test]
fn gltf_sep_success() {
    let obj_path = "/tmp/test_gltf_sep_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_gltf = "/tmp/test_gltf_sep_out.gltf";
    let out_bin = "/tmp/test_gltf_sep_out.bin";
    assert!(commands::export::cmd_gltf_sep(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_gltf.to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_gltf).exists());
    assert!(std::path::Path::new(out_bin).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_gltf);
    let _ = std::fs::remove_file(out_bin);
}

#[test]
fn gltf_sep_explicit_bin_path_success() {
    let obj_path = "/tmp/test_gltf_sep_explicitbin_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_gltf = "/tmp/test_gltf_sep_explicit.gltf";
    let out_bin = "/tmp/test_gltf_sep_custom.bin";
    assert!(commands::export::cmd_gltf_sep(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_gltf.to_string(),
        "--bin".to_string(),
        out_bin.to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_bin).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_gltf);
    let _ = std::fs::remove_file(out_bin);
}

#[test]
fn gltf_sep_unknown_option_errors() {
    assert!(commands::export::cmd_gltf_sep(&["--unknown-flag".to_string()]).is_err());
}

// ── svg ───────────────────────────────────────────────────────────────────────

#[test]
fn svg_missing_base_errors() {
    assert!(
        commands::export::cmd_svg(&["--output".to_string(), "/tmp/out.svg".to_string()]).is_err()
    );
}

#[test]
fn svg_missing_output_errors() {
    assert!(
        commands::export::cmd_svg(&["--base".to_string(), "/tmp/dummy.obj".to_string()]).is_err()
    );
}

#[test]
fn svg_nonexistent_base_errors() {
    assert!(commands::export::cmd_svg(&[
        "--base".to_string(),
        "/nonexistent_svg_base.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.svg".to_string()
    ])
    .is_err());
}

#[test]
fn svg_front_projection_success() {
    let obj_path = "/tmp/test_svg_front_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_svg_front_out.svg";
    assert!(commands::export::cmd_svg(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
        "--projection".to_string(),
        "front".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn svg_uv_mode_success() {
    let obj_path = "/tmp/test_svg_uv_base.obj";
    std::fs::write(
        obj_path,
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nf 1/1 2/2 3/3\n",
    )
    .expect("should succeed");
    let out_path = "/tmp/test_svg_uv_out.svg";
    assert!(commands::export::cmd_svg(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
        "--uv".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn svg_unknown_option_errors() {
    assert!(commands::export::cmd_svg(&["--unknown-flag".to_string()]).is_err());
}

#[test]
fn svg_invalid_projection_errors() {
    let obj_path = "/tmp/test_svg_badproj_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::export::cmd_svg(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        "/tmp/test_svg_badproj_out.svg".to_string(),
        "--projection".to_string(),
        "diagonal".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj_path);
}

// ── lod-export ────────────────────────────────────────────────────────────────

#[test]
fn lod_export_missing_base_errors() {
    assert!(commands::export::cmd_lod_export(&[
        "--output-dir".to_string(),
        "/tmp/lod_out".to_string()
    ])
    .is_err());
}

#[test]
fn lod_export_missing_output_dir_errors() {
    assert!(commands::export::cmd_lod_export(&[
        "--base".to_string(),
        "/tmp/dummy.obj".to_string()
    ])
    .is_err());
}

#[test]
fn lod_export_nonexistent_base_errors() {
    assert!(commands::export::cmd_lod_export(&[
        "--base".to_string(),
        "/nonexistent_lod_base.obj".to_string(),
        "--output-dir".to_string(),
        "/tmp/lod_out".to_string()
    ])
    .is_err());
}

#[test]
fn lod_export_success() {
    let obj_path = "/tmp/test_lod_export_base.obj";
    std::fs::write(
        obj_path,
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nv 0 0 1\nf 1 2 3\nf 1 2 4\nf 1 3 4\nf 2 3 4\n",
    )
    .expect("should succeed");
    let out_dir = "/tmp/test_lod_export_out";
    std::fs::create_dir_all(out_dir).expect("should succeed");
    assert!(commands::export::cmd_lod_export(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output-dir".to_string(),
        out_dir.to_string()
    ])
    .is_ok());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_dir_all(out_dir);
}

#[test]
fn lod_export_unknown_option_errors() {
    assert!(commands::export::cmd_lod_export(&["--unknown-flag".to_string()]).is_err());
}

#[test]
fn lod_export_honors_custom_levels_count() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_lod_levels_base.obj");
    std::fs::write(
        &obj_path,
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nv 0 0 1\nf 1 2 3\nf 1 2 4\nf 1 3 4\nf 2 3 4\n",
    )
    .expect("should succeed");
    let out_dir = std::env::temp_dir().join("oxihuman_test_lod_levels_out");
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).expect("should succeed");

    assert!(commands::export::cmd_lod_export(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--output-dir".to_string(),
        out_dir.to_string_lossy().into_owned(),
        "--levels".to_string(),
        "5".to_string(),
    ])
    .is_ok());

    let glb_count = std::fs::read_dir(&out_dir)
        .expect("should read output dir")
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "glb").unwrap_or(false))
        .count();
    assert_eq!(
        glb_count, 5,
        "--levels 5 must produce exactly 5 GLB files, got {}",
        glb_count
    );

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn lod_export_rejects_zero_levels() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_lod_zero_levels_base.obj");
    std::fs::write(&obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_dir = std::env::temp_dir().join("oxihuman_test_lod_zero_levels_out");

    assert!(commands::export::cmd_lod_export(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--output-dir".to_string(),
        out_dir.to_string_lossy().into_owned(),
        "--levels".to_string(),
        "0".to_string(),
    ])
    .is_err());

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_dir_all(&out_dir);
}
