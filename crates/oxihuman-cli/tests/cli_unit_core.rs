// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Unit-level tests for CLI core commands: parameter loading, presets, mesh
//! generation, validation, sessions, workspace/stats, mesh proxies,
//! quantization, and morph-delta export.

use oxihuman_cli::commands;
use oxihuman_cli::utils::load_params;
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
    let base_path = oxihuman_test_utils::base_obj();
    if !base_path.exists() {
        return;
    }
    let args: Vec<String> = vec![
        "--base",
        base_path.to_str().unwrap_or_default(),
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
    let base_path = oxihuman_test_utils::base_obj();
    if !base_path.exists() {
        return;
    }
    let args: Vec<String> = vec![
        "--base",
        base_path.to_str().unwrap_or_default(),
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
    let base_path = oxihuman_test_utils::base_obj();
    if !base_path.exists() {
        return;
    }
    let session_path = "/tmp/test_save_session_cli.json";
    let _ = std::fs::remove_file(session_path);
    let args: Vec<String> = vec![
        "--base",
        base_path.to_str().unwrap_or_default(),
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
    let base_path = oxihuman_test_utils::base_obj();
    if !base_path.exists() {
        return;
    }
    let args: Vec<String> = vec![
        "--base",
        base_path.to_str().unwrap_or_default(),
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
    let path = oxihuman_test_utils::targets_dir().join("armslegs");
    let entries: Vec<_> = std::fs::read_dir(&path)
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

#[test]
fn quantize_rejects_blocked_tag_in_base_name() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_quantize_explicit_content.obj");
    std::fs::write(&obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let out_path = std::env::temp_dir().join("oxihuman_test_quantize_explicit_out.qmsh");
    let result = commands::pack::cmd_quantize(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--output".to_string(),
        out_path.to_string_lossy().into_owned(),
    ]);
    assert!(
        result.is_err(),
        "policy should reject a blocked-tag file name"
    );
    assert!(
        !out_path.exists(),
        "no output should be written for a rejected input"
    );
    let _ = std::fs::remove_file(&obj_path);
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

#[test]
fn morph_export_filters_blocked_targets_by_policy() {
    let obj_path = std::env::temp_dir().join("oxihuman_test_morphexport_policy_base.obj");
    std::fs::write(&obj_path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("should succeed");
    let targets_dir = std::env::temp_dir().join("oxihuman_test_morphexport_policy_targets");
    std::fs::create_dir_all(&targets_dir).expect("should succeed");
    std::fs::write(
        targets_dir.join("height-up.target"),
        "0 0.1 0.2 0.3\n1 0.0 0.1 0.0\n",
    )
    .expect("should succeed");
    std::fs::write(
        targets_dir.join("adult-content.target"),
        "0 0.1 0.2 0.3\n1 0.0 0.1 0.0\n",
    )
    .expect("should succeed");
    let out_path = std::env::temp_dir().join("oxihuman_test_morphexport_policy_out.oxmd");

    assert!(commands::pack::cmd_morph_export(&[
        "--base".to_string(),
        obj_path.to_string_lossy().into_owned(),
        "--targets".to_string(),
        targets_dir.to_string_lossy().into_owned(),
        "--output".to_string(),
        out_path.to_string_lossy().into_owned(),
    ])
    .is_ok());

    let bin = oxihuman_export::read_morph_delta_bin(&out_path).expect("should read OXMD");
    assert!(
        bin.targets.iter().any(|t| t.name == "height-up"),
        "allowed target must be included"
    );
    assert!(
        !bin.targets.iter().any(|t| t.name == "adult-content"),
        "blocked-tag target must be excluded"
    );

    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_dir_all(&targets_dir);
}
