// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Unit-level tests for CLI packaging/distribution subcommands: variant-pack,
//! report, asset-bundle, sign-pack/verify-sign, batch-chars, and anim-bake.

use oxihuman_cli::commands;

// ── variant-pack ──────────────────────────────────────────────────────────────

#[test]
fn variant_pack_missing_params_list_errors() {
    assert!(commands::export::cmd_variant_pack(&[
        "--base".to_string(),
        "/tmp/dummy.obj".to_string(),
        "--output-dir".to_string(),
        "/tmp/vpack_out".to_string()
    ])
    .is_err());
}

#[test]
fn variant_pack_missing_base_errors() {
    assert!(commands::export::cmd_variant_pack(&[
        "--params-list".to_string(),
        "/tmp/dummy_params.json".to_string(),
        "--output-dir".to_string(),
        "/tmp/vpack_out".to_string()
    ])
    .is_err());
}

#[test]
fn variant_pack_missing_output_dir_errors() {
    assert!(commands::export::cmd_variant_pack(&[
        "--params-list".to_string(),
        "/tmp/dummy_params.json".to_string(),
        "--base".to_string(),
        "/tmp/dummy.obj".to_string()
    ])
    .is_err());
}

#[test]
fn variant_pack_nonexistent_params_list_errors() {
    let obj_path = "/tmp/test_vpack_nonexistent_params_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::export::cmd_variant_pack(&[
        "--params-list".to_string(),
        "/nonexistent_params_list.json".to_string(),
        "--base".to_string(),
        obj_path.to_string(),
        "--output-dir".to_string(),
        "/tmp/vpack_out".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj_path);
}

#[test]
fn variant_pack_success() {
    let obj_path = "/tmp/test_vpack_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let params_list_path = "/tmp/test_vpack_params.json";
    std::fs::write(params_list_path, r#"[{"height":0.5,"weight":0.5,"muscle":0.5,"age":0.5},{"height":0.7,"weight":0.3,"muscle":0.6,"age":0.4}]"#).expect("should succeed");
    let out_dir = "/tmp/test_vpack_out";
    std::fs::create_dir_all(out_dir).expect("should succeed");
    assert!(commands::export::cmd_variant_pack(&[
        "--params-list".to_string(),
        params_list_path.to_string(),
        "--base".to_string(),
        obj_path.to_string(),
        "--output-dir".to_string(),
        out_dir.to_string(),
        "--pack-name".to_string(),
        "TestPack".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_dir).join("manifest.json").exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(params_list_path);
    let _ = std::fs::remove_dir_all(out_dir);
}

#[test]
fn variant_pack_unknown_option_errors() {
    assert!(commands::export::cmd_variant_pack(&["--unknown-flag".to_string()]).is_err());
}

// ── report ────────────────────────────────────────────────────────────────────

#[test]
fn report_missing_base_errors() {
    assert!(commands::export::cmd_report(&[
        "--output".to_string(),
        "/tmp/test_report_out.html".to_string()
    ])
    .is_err());
}

#[test]
fn report_nonexistent_base_errors() {
    assert!(commands::export::cmd_report(&[
        "--base".to_string(),
        "/nonexistent_base_report.obj".to_string(),
        "--output".to_string(),
        "/tmp/test_report_out.html".to_string()
    ])
    .is_err());
}

#[test]
fn report_valid_inputs_succeeds() {
    let obj_path = "/tmp/test_report_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_report_out.html";
    assert!(commands::export::cmd_report(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
        "--title".to_string(),
        "Test Report".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let html = std::fs::read_to_string(out_path).expect("should succeed");
    assert!(html.contains("Test Report"));
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

// ── asset-bundle ──────────────────────────────────────────────────────────────

#[test]
fn asset_bundle_missing_base_errors() {
    assert!(commands::pack::cmd_asset_bundle(&[
        "--targets".to_string(),
        "/tmp".to_string(),
        "--output".to_string(),
        "/tmp/test_bundle_out.oxb".to_string()
    ])
    .is_err());
}

#[test]
fn asset_bundle_nonexistent_targets_errors() {
    let obj_path = "/tmp/test_bundle_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::pack::cmd_asset_bundle(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--targets".to_string(),
        "/no_such_targets_dir_bundle".to_string(),
        "--output".to_string(),
        "/tmp/test_bundle_out.oxb".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj_path);
}

#[test]
fn asset_bundle_valid_inputs_succeeds() {
    let obj_path = "/tmp/test_bundle_valid_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = "/tmp/test_bundle_valid_targets";
    std::fs::create_dir_all(targets_dir).expect("should succeed");
    std::fs::write(
        format!("{}/height-up.target", targets_dir),
        "0 0.1 0.2 0.3\n1 0.0 0.1 0.0\n",
    )
    .expect("should succeed");
    let out_path = "/tmp/test_bundle_valid_out.oxb";
    assert!(commands::pack::cmd_asset_bundle(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--targets".to_string(),
        targets_dir.to_string(),
        "--output".to_string(),
        out_path.to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let bytes = std::fs::read(out_path).expect("should succeed");
    assert_eq!(&bytes[0..4], b"OXB1");
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
    let _ = std::fs::remove_dir_all(targets_dir);
}

#[test]
fn asset_bundle_filters_blocked_targets_by_policy() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_bundle_policy_base.obj");
    std::fs::write(&obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = std::env::temp_dir().join("oxihuman_test_bundle_policy_targets");
    std::fs::create_dir_all(&targets_dir).expect("should succeed");
    std::fs::write(
        targets_dir.join("height-up.target"),
        "0 0.1 0.2 0.3\n1 0.0 0.1 0.0\n",
    )
    .expect("should succeed");
    std::fs::write(
        targets_dir.join("sexual-content.target"),
        "0 0.1 0.2 0.3\n1 0.0 0.1 0.0\n",
    )
    .expect("should succeed");
    let out_path = std::env::temp_dir().join("oxihuman_test_bundle_policy_out.oxb");

    assert!(commands::pack::cmd_asset_bundle(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().into_owned(),
        "--output".to_string(),
        out_path.to_string_lossy().into_owned(),
    ])
    .is_ok());

    let bytes = std::fs::read(&out_path).expect("should succeed");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("height-up"), "allowed target must be bundled");
    assert!(
        !text.contains("sexual-content"),
        "blocked-tag target must not be bundled"
    );

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_dir_all(&targets_dir);
}

// ── sign-pack / verify-sign ───────────────────────────────────────────────────

#[test]
fn sign_pack_missing_pack_dir_errors() {
    assert!(commands::pack::cmd_sign_pack(&[
        "--key".to_string(),
        "secret".to_string(),
        "--output".to_string(),
        "/tmp/sig.txt".to_string()
    ])
    .is_err());
}

#[test]
fn sign_pack_missing_key_errors() {
    assert!(commands::pack::cmd_sign_pack(&[
        "--pack-dir".to_string(),
        "/tmp".to_string(),
        "--output".to_string(),
        "/tmp/sig.txt".to_string()
    ])
    .is_err());
}

#[test]
fn sign_pack_nonexistent_dir_errors() {
    assert!(commands::pack::cmd_sign_pack(&[
        "--pack-dir".to_string(),
        "/no_such_dir_sign_pack_test".to_string(),
        "--key".to_string(),
        "k".to_string(),
        "--output".to_string(),
        "/tmp/sig.txt".to_string()
    ])
    .is_err());
}

#[test]
fn sign_pack_succeeds_and_writes_file() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);
    let pack_dir = format!("/tmp/oxihuman_cli_sign_test_{}", nanos);
    std::fs::create_dir_all(&pack_dir).expect("should succeed");
    std::fs::write(format!("{}/asset.bin", pack_dir), b"data").expect("should succeed");
    let sig_path = format!("/tmp/oxihuman_cli_sig_{}.txt", nanos);
    assert!(commands::pack::cmd_sign_pack(&[
        "--pack-dir".to_string(),
        pack_dir.clone(),
        "--key".to_string(),
        "mysecretkey".to_string(),
        "--signer-id".to_string(),
        "ci".to_string(),
        "--output".to_string(),
        sig_path.clone()
    ])
    .is_ok());
    assert!(std::path::Path::new(&sig_path).exists());
    let _ = std::fs::remove_dir_all(&pack_dir);
    let _ = std::fs::remove_file(&sig_path);
}

#[test]
fn verify_sign_missing_pack_dir_errors() {
    assert!(commands::pack::cmd_verify_sign(&[
        "--sig-file".to_string(),
        "/tmp/sig.txt".to_string(),
        "--key".to_string(),
        "k".to_string()
    ])
    .is_err());
}

#[test]
fn verify_sign_valid_signature_prints_valid() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(99);
    let pack_dir = format!("/tmp/oxihuman_cli_verify_test_{}", nanos);
    std::fs::create_dir_all(&pack_dir).expect("should succeed");
    std::fs::write(format!("{}/f.bin", pack_dir), b"hello").expect("should succeed");
    let sig_path = format!("/tmp/oxihuman_cli_verify_sig_{}.txt", nanos);
    commands::pack::cmd_sign_pack(&[
        "--pack-dir".to_string(),
        pack_dir.clone(),
        "--key".to_string(),
        "verifykey".to_string(),
        "--output".to_string(),
        sig_path.clone(),
    ])
    .expect("should succeed");
    assert!(commands::pack::cmd_verify_sign(&[
        "--pack-dir".to_string(),
        pack_dir.clone(),
        "--sig-file".to_string(),
        sig_path.clone(),
        "--key".to_string(),
        "verifykey".to_string()
    ])
    .is_ok());
    let _ = std::fs::remove_dir_all(&pack_dir);
    let _ = std::fs::remove_file(&sig_path);
}

// ── batch-chars ───────────────────────────────────────────────────────────────

#[test]
fn batch_chars_missing_out_dir_errors() {
    assert!(
        commands::misc::cmd_batch_chars(&["--format".to_string(), "json".to_string()]).is_err()
    );
}

#[test]
fn batch_chars_unknown_format_errors() {
    assert!(commands::misc::cmd_batch_chars(&[
        "--out-dir".to_string(),
        "/tmp".to_string(),
        "--format".to_string(),
        "badformat".to_string()
    ])
    .is_err());
}

#[test]
fn batch_chars_json_format_succeeds() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(77);
    let out_dir = format!("/tmp/oxihuman_cli_batch_{}", nanos);
    assert!(commands::misc::cmd_batch_chars(&[
        "--out-dir".to_string(),
        out_dir.clone(),
        "--format".to_string(),
        "json".to_string(),
        "--height-steps".to_string(),
        "2".to_string(),
        "--weight-steps".to_string(),
        "2".to_string(),
        "--age-steps".to_string(),
        "1".to_string()
    ])
    .is_ok());
    let count = std::fs::read_dir(&out_dir).expect("should succeed").count();
    assert_eq!(count, 4);
    let _ = std::fs::remove_dir_all(&out_dir);
}

// ── anim-bake ─────────────────────────────────────────────────────────────────

#[test]
fn anim_bake_missing_input_errors() {
    assert!(commands::anim::cmd_anim_bake(&[
        "--params-json".to_string(),
        "/tmp/p.json".to_string(),
        "--output".to_string(),
        "/tmp/out.pc2".to_string()
    ])
    .is_err());
}

#[test]
fn anim_bake_missing_params_json_errors() {
    let obj = "/tmp/test_ab_base.obj";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::anim::cmd_anim_bake(&[
        "--input".to_string(),
        obj.to_string(),
        "--output".to_string(),
        "/tmp/out.pc2".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj);
}

#[test]
fn anim_bake_invalid_format_errors() {
    let obj = "/tmp/test_ab_fmt.obj";
    let params = "/tmp/test_ab_params.json";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    std::fs::write(params, "[{\"height\":0.5}]").expect("should succeed");
    assert!(commands::anim::cmd_anim_bake(&[
        "--input".to_string(),
        obj.to_string(),
        "--params-json".to_string(),
        params.to_string(),
        "--output".to_string(),
        "/tmp/out.xyz".to_string(),
        "--format".to_string(),
        "badformat".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(params);
}

#[test]
fn anim_bake_pc2_succeeds() {
    let obj = "/tmp/test_ab_pc2.obj";
    let params = "/tmp/test_ab_pc2_params.json";
    let out = "/tmp/test_ab_out.pc2";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    std::fs::write(params, "[{\"height\":0.0},{\"height\":0.5}]").expect("should succeed");
    assert!(commands::anim::cmd_anim_bake(&[
        "--input".to_string(),
        obj.to_string(),
        "--params-json".to_string(),
        params.to_string(),
        "--output".to_string(),
        out.to_string(),
        "--format".to_string(),
        "pc2".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out).exists());
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(params);
    let _ = std::fs::remove_file(out);
}

#[test]
fn anim_bake_mdd_succeeds() {
    let obj = "/tmp/test_ab_mdd.obj";
    let params = "/tmp/test_ab_mdd_params.json";
    let out = "/tmp/test_ab_out.mdd";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    std::fs::write(params, "[{\"height\":0.0}]").expect("should succeed");
    assert!(commands::anim::cmd_anim_bake(&[
        "--input".to_string(),
        obj.to_string(),
        "--params-json".to_string(),
        params.to_string(),
        "--output".to_string(),
        out.to_string(),
        "--format".to_string(),
        "mdd".to_string()
    ])
    .is_ok());
    assert!(std::path::Path::new(out).exists());
    let _ = std::fs::remove_file(obj);
    let _ = std::fs::remove_file(params);
    let _ = std::fs::remove_file(out);
}
