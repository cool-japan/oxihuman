// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Unit-level tests for CLI subcommand handlers.
//!
//! These tests exercise the command functions directly (not via the binary),
//! previously they lived inline in `main.rs`.

use oxihuman_cli::commands;
use oxihuman_cli::utils::load_params;
use oxihuman_core::default_builtin_plugins;
use oxihuman_morph::params::ParamState;
use oxihuman_morph::presets::BodyPreset;
use oxihuman_morph::session::MorphSession;

// ── utils ─────────────────────────────────────────────────────────────────────

#[test]
fn load_params_from_inline_json() {
    let src = r#"{"height": 0.7, "weight": 0.3, "muscle": 0.5, "age": 0.2}"#;
    let p = load_params(src).expect("should succeed");
    assert!((p.height - 0.7).abs() < 1e-5);
}

// ── presets ───────────────────────────────────────────────────────────────────

#[test]
fn all_presets_parse() {
    for name in BodyPreset::all_names() {
        assert!(
            BodyPreset::from_name(name).is_some(),
            "preset '{}' not found",
            name
        );
    }
}

// ── generate ──────────────────────────────────────────────────────────────────

#[test]
fn generate_missing_base_errors() {
    let args: Vec<String> = vec!["--base", "/nonexistent.obj", "--output", "/tmp/out.glb"]
        .into_iter()
        .map(String::from)
        .collect();
    assert!(commands::generate::cmd_generate(&args).is_err());
}

#[test]
fn generate_unknown_expression_errors() {
    let base_path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
    if !std::path::Path::new(base_path).exists() {
        return;
    }
    let args: Vec<String> = vec![
        "--base",
        base_path,
        "--output",
        "/tmp/test_expr_unknown.glb",
        "--expression",
        "xyzzy_unknown_expression",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert!(
        commands::generate::cmd_generate(&args).is_err(),
        "unknown expression should error"
    );
}

#[test]
fn generate_with_expression_no_targets_dir() {
    let base_path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
    if !std::path::Path::new(base_path).exists() {
        return;
    }
    let args: Vec<String> = vec![
        "--base",
        base_path,
        "--output",
        "/tmp/test_expr_no_targets.glb",
        "--expression",
        "neutral",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert!(
        commands::generate::cmd_generate(&args).is_ok(),
        "neutral expression without targets dir should succeed"
    );
    let _ = std::fs::remove_file("/tmp/test_expr_no_targets.glb");
}

#[test]
fn save_session_creates_file() {
    let base_path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
    if !std::path::Path::new(base_path).exists() {
        return;
    }
    let session_path = "/tmp/test_save_session_cli.json";
    let _ = std::fs::remove_file(session_path);
    let args: Vec<String> = vec![
        "--base",
        base_path,
        "--output",
        "/tmp/test_save_session_out.glb",
        "--params",
        r#"{"height":0.6,"weight":0.4,"muscle":0.7,"age":0.1}"#,
        "--save-session",
        session_path,
    ]
    .into_iter()
    .map(String::from)
    .collect();
    commands::generate::cmd_generate(&args).expect("generate with --save-session should succeed");
    assert!(
        std::path::Path::new(session_path).exists(),
        "session file was not created"
    );
    let session = MorphSession::load(std::path::Path::new(session_path))
        .expect("session file should be valid JSON");
    assert!(
        (session.params.height - 0.6).abs() < 1e-4,
        "height mismatch"
    );
    assert!(
        (session.params.weight - 0.4).abs() < 1e-4,
        "weight mismatch"
    );
    assert!(
        (session.params.muscle - 0.7).abs() < 1e-4,
        "muscle mismatch"
    );
    assert!((session.params.age - 0.1).abs() < 1e-4, "age mismatch");
    let _ = std::fs::remove_file(session_path);
    let _ = std::fs::remove_file("/tmp/test_save_session_out.glb");
}

#[test]
fn generate_load_session_nonexistent_errors() {
    let base_path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/3dobjs/base.obj";
    if !std::path::Path::new(base_path).exists() {
        return;
    }
    let args: Vec<String> = vec![
        "--base",
        base_path,
        "--output",
        "/tmp/test_load_nonexistent.glb",
        "--load-session",
        "/tmp/totally_nonexistent_session.json",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert!(commands::generate::cmd_generate(&args).is_err());
}

// ── validate ──────────────────────────────────────────────────────────────────

#[test]
fn validate_nonexistent_errors() {
    let args: Vec<String> = vec!["/nonexistent.target".to_string()];
    assert!(commands::info::cmd_validate(&args).is_err());
}

#[test]
fn validate_real_target_file() {
    let path = "/media/kitasan/Backup/resource/makehuman/makehuman/data/targets/armslegs";
    let entries: Vec<_> = std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "target").unwrap_or(false))
        .take(1)
        .collect();
    if let Some(entry) = entries.first() {
        let args = vec![entry.path().to_string_lossy().into_owned()];
        commands::info::cmd_validate(&args).expect("should succeed");
    }
}

#[test]
fn validate_pack_missing_manifest_errors() {
    let args: Vec<String> = vec![
        "--pack".to_string(),
        "/tmp/nonexistent_manifest.toml".to_string(),
    ];
    assert!(commands::info::cmd_validate(&args).is_err());
}

#[test]
fn validate_pack_flag_requires_path() {
    let args: Vec<String> = vec!["--pack".to_string()];
    assert!(commands::info::cmd_validate(&args).is_err());
}

// ── session ───────────────────────────────────────────────────────────────────

#[test]
fn load_session_overrides_params() {
    let session_path = "/tmp/test_load_session_cli.json";
    let p = ParamState::new(0.8, 0.2, 0.9, 0.3);
    let session = MorphSession::new(&p).with_label("override-test");
    session
        .save(std::path::Path::new(session_path))
        .expect("should save session");
    let loaded = MorphSession::load(std::path::Path::new(session_path)).expect("session must load");
    let params = loaded.to_param_state();
    assert!((params.height - 0.8).abs() < 1e-4);
    assert!((params.weight - 0.2).abs() < 1e-4);
    assert!((params.muscle - 0.9).abs() < 1e-4);
    assert!((params.age - 0.3).abs() < 1e-4);
    assert_eq!(loaded.label, Some("override-test".to_string()));
    let _ = std::fs::remove_file(session_path);
}

#[test]
fn session_subcommand_prints_info() {
    let session_path = "/tmp/test_session_subcommand.json";
    let mut p = ParamState::new(0.5, 0.5, 0.5, 0.5);
    p.extra.insert("expression".to_string(), 0.25);
    let session = MorphSession::new(&p)
        .with_label("subcommand-test")
        .with_targets_dir("/tmp/targets");
    session
        .save(std::path::Path::new(session_path))
        .expect("should save");
    let args: Vec<String> = vec![session_path.to_string()];
    commands::info::cmd_session(&args).expect("session subcommand should succeed");
    let _ = std::fs::remove_file(session_path);
}

#[test]
fn session_subcommand_missing_file_errors() {
    let args: Vec<String> = vec!["/tmp/nonexistent_session_xyz.json".to_string()];
    assert!(commands::info::cmd_session(&args).is_err());
}

#[test]
fn session_subcommand_no_args_errors() {
    let args: Vec<String> = vec![];
    assert!(commands::info::cmd_session(&args).is_err());
}

#[test]
fn session_json_round_trip_via_file() {
    let session_path = "/tmp/test_session_round_trip.json";
    let p = ParamState::new(0.3, 0.7, 0.4, 0.6);
    let orig = MorphSession::new(&p);
    orig.save(std::path::Path::new(session_path))
        .expect("should succeed");
    let restored = MorphSession::load(std::path::Path::new(session_path)).expect("should succeed");
    let rp = restored.to_param_state();
    assert!((rp.height - 0.3).abs() < 1e-4);
    assert!((rp.weight - 0.7).abs() < 1e-4);
    assert!((rp.muscle - 0.4).abs() < 1e-4);
    assert!((rp.age - 0.6).abs() < 1e-4);
    let _ = std::fs::remove_file(session_path);
}

// ── workspace / stats ─────────────────────────────────────────────────────────

#[test]
fn workspace_info_subcommand_runs() {
    commands::info::cmd_workspace_info();
}

#[test]
fn stats_on_nonexistent_file_errors() {
    assert!(commands::info::cmd_stats("/nonexistent/file.obj", false, false).is_err());
}

#[test]
fn stats_json_flag_produces_json() {
    let path = "/tmp/test_stats_cli.obj";
    std::fs::write(path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::info::cmd_stats(path, false, true).is_ok());
    let _ = std::fs::remove_file(path);
}

#[test]
fn stats_human_readable_produces_output() {
    let path = "/tmp/test_stats_human.obj";
    std::fs::write(path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::info::cmd_stats(path, false, false).is_ok());
    let _ = std::fs::remove_file(path);
}

#[test]
fn stats_full_flag_works() {
    let path = "/tmp/test_stats_full.obj";
    std::fs::write(path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::info::cmd_stats(path, true, false).is_ok());
    let _ = std::fs::remove_file(path);
}

#[test]
fn parse_stats_args_missing_path_errors() {
    let args: Vec<String> = vec!["--json".to_string()];
    assert!(commands::info::parse_stats_args(&args).is_err());
}

#[test]
fn parse_stats_args_unknown_option_errors() {
    let args: Vec<String> = vec!["--unknown".to_string(), "/tmp/x.obj".to_string()];
    assert!(commands::info::parse_stats_args(&args).is_err());
}

// ── proxies ───────────────────────────────────────────────────────────────────

#[test]
fn proxies_missing_base_errors() {
    assert!(commands::misc::cmd_proxies(&[]).is_err());
}

#[test]
fn proxies_nonexistent_base_errors() {
    let args: Vec<String> = vec!["--base".to_string(), "/nonexistent_mesh.obj".to_string()];
    assert!(commands::misc::cmd_proxies(&args).is_err());
}

#[test]
fn proxies_small_mesh_errors() {
    let path = "/tmp/test_proxies_tiny.obj";
    std::fs::write(path, "v 0 0 0\n").expect("should succeed");
    let args: Vec<String> = vec!["--base".to_string(), path.to_string()];
    let _ = commands::misc::cmd_proxies(&args);
    let _ = std::fs::remove_file(path);
}

#[test]
fn proxies_valid_mesh_outputs_json() {
    let path = "/tmp/test_proxies_human.obj";
    let mut obj = String::new();
    for v in &[
        [-0.2f32, 0.0, -0.1],
        [0.2, 0.0, -0.1],
        [0.2, 0.0, 0.1],
        [-0.2, 0.0, 0.1],
        [-0.2, 1.8, -0.1],
        [0.2, 1.8, -0.1],
        [0.2, 1.8, 0.1],
        [-0.2, 1.8, 0.1],
    ] {
        obj.push_str(&format!("v {} {} {}\n", v[0], v[1], v[2]));
    }
    for f in &[
        [1u32, 2, 3],
        [1, 3, 4],
        [5, 6, 7],
        [5, 7, 8],
        [1, 2, 6],
        [1, 6, 5],
        [4, 3, 7],
        [4, 7, 8],
        [1, 4, 8],
        [1, 8, 5],
        [2, 3, 7],
        [2, 7, 6],
    ] {
        obj.push_str(&format!("f {} {} {}\n", f[0], f[1], f[2]));
    }
    std::fs::write(path, &obj).expect("should succeed");
    let out_path = "/tmp/test_proxies_out.json";
    let args: Vec<String> = vec![
        "--base".to_string(),
        path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
    ];
    let result = commands::misc::cmd_proxies(&args);
    if result.is_ok() {
        let text = std::fs::read_to_string(out_path).expect("should succeed");
        let v: serde_json::Value = serde_json::from_str(&text).expect("output must be valid JSON");
        assert!(v.get("total").is_some());
    }
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn proxies_unknown_option_errors() {
    assert!(commands::misc::cmd_proxies(&["--unknown-flag".to_string()]).is_err());
}

// ── quantize ──────────────────────────────────────────────────────────────────

#[test]
fn quantize_missing_base_errors() {
    assert!(
        commands::pack::cmd_quantize(&["--output".to_string(), "/tmp/out.qmsh".to_string()])
            .is_err()
    );
}

#[test]
fn quantize_missing_output_errors() {
    assert!(
        commands::pack::cmd_quantize(&["--base".to_string(), "/tmp/dummy.obj".to_string()])
            .is_err()
    );
}

#[test]
fn quantize_nonexistent_base_errors() {
    assert!(commands::pack::cmd_quantize(&[
        "--base".to_string(),
        "/nonexistent_base.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.qmsh".to_string()
    ])
    .is_err());
}

#[test]
fn quantize_valid_obj_succeeds() {
    let obj_path = "/tmp/test_quantize_input.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nvn 0 0 1\nvn 0 0 1\nvt 0 0\nvt 1 0\nvt 0 1\nf 1/1/1 2/2/2 3/3/3\n").expect("should succeed");
    let out_path = "/tmp/test_quantize_output.qmsh";
    let args: Vec<String> = vec![
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
    ];
    assert!(commands::pack::cmd_quantize(&args).is_ok());
    assert!(std::path::Path::new(out_path).exists());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn quantize_with_stats_flag_succeeds() {
    let obj_path = "/tmp/test_quantize_stats.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = "/tmp/test_quantize_stats.qmsh";
    let args: Vec<String> = vec![
        "--base".to_string(),
        obj_path.to_string(),
        "--output".to_string(),
        out_path.to_string(),
        "--stats".to_string(),
    ];
    assert!(commands::pack::cmd_quantize(&args).is_ok());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn quantize_unknown_option_errors() {
    assert!(commands::pack::cmd_quantize(&["--unknown-flag".to_string()]).is_err());
}

// ── morph-export ──────────────────────────────────────────────────────────────

#[test]
fn morph_export_missing_base_errors() {
    assert!(commands::pack::cmd_morph_export(&[
        "--targets".to_string(),
        "/tmp".to_string(),
        "--output".to_string(),
        "/tmp/out.oxmd".to_string()
    ])
    .is_err());
}

#[test]
fn morph_export_missing_targets_errors() {
    assert!(commands::pack::cmd_morph_export(&[
        "--base".to_string(),
        "/tmp/dummy.obj".to_string(),
        "--output".to_string(),
        "/tmp/out.oxmd".to_string()
    ])
    .is_err());
}

#[test]
fn morph_export_missing_output_errors() {
    assert!(commands::pack::cmd_morph_export(&[
        "--base".to_string(),
        "/tmp/dummy.obj".to_string(),
        "--targets".to_string(),
        "/tmp".to_string()
    ])
    .is_err());
}

#[test]
fn morph_export_nonexistent_base_errors() {
    assert!(commands::pack::cmd_morph_export(&[
        "--base".to_string(),
        "/nonexistent_base.obj".to_string(),
        "--targets".to_string(),
        "/tmp".to_string(),
        "--output".to_string(),
        "/tmp/out.oxmd".to_string()
    ])
    .is_err());
}

#[test]
fn morph_export_nonexistent_targets_dir_errors() {
    let obj_path = "/tmp/test_morphexport_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::pack::cmd_morph_export(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--targets".to_string(),
        "/nonexistent_targets_dir".to_string(),
        "--output".to_string(),
        "/tmp/out.oxmd".to_string()
    ])
    .is_err());
    let _ = std::fs::remove_file(obj_path);
}

#[test]
fn morph_export_empty_targets_dir_succeeds() {
    let obj_path = "/tmp/test_morphexport_empty_base.obj";
    std::fs::write(obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = "/tmp/test_morphexport_empty_targets";
    std::fs::create_dir_all(targets_dir).expect("should succeed");
    let out_path = "/tmp/test_morphexport_empty_out.oxmd";
    assert!(commands::pack::cmd_morph_export(&[
        "--base".to_string(),
        obj_path.to_string(),
        "--targets".to_string(),
        targets_dir.to_string(),
        "--output".to_string(),
        out_path.to_string()
    ])
    .is_ok());
    let _ = std::fs::remove_file(obj_path);
    let _ = std::fs::remove_file(out_path);
    let _ = std::fs::remove_dir(targets_dir);
}

#[test]
fn morph_export_unknown_option_errors() {
    assert!(commands::pack::cmd_morph_export(&["--unknown-flag".to_string()]).is_err());
}

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

#[test]
fn remesh_missing_input_errors() {
    assert!(
        commands::misc::cmd_remesh(&["--target-edge-len".to_string(), "0.1".to_string()]).is_err()
    );
}

#[test]
fn remesh_nonexistent_input_errors() {
    assert!(commands::misc::cmd_remesh(&["/tmp/nonexistent_mesh_xyz.obj".to_string()]).is_err());
}

#[test]
fn remesh_succeeds_with_existing_file() {
    let obj = "/tmp/test_remesh_input.obj";
    std::fs::write(obj, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    assert!(commands::misc::cmd_remesh(&[
        obj.to_string(),
        "--target-edge-len".to_string(),
        "0.05".to_string(),
        "--iters".to_string(),
        "3".to_string()
    ])
    .is_ok());
    let _ = std::fs::remove_file(obj);
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
