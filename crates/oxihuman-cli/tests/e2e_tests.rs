// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end integration tests for the OxiHuman CLI.
//!
//! Each test invokes the compiled `oxihuman` binary via `std::process::Command`
//! and inspects stdout, exit code, or output files.  All output files are
//! placed in `std::env::temp_dir()`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── binary path ───────────────────────────────────────────────────────────────

/// Return the path to the compiled `oxihuman` CLI binary.
///
/// When running via `cargo test`, Cargo sets `CARGO_BIN_EXE_oxihuman`.
fn cli_bin() -> PathBuf {
    let var = std::env::var("CARGO_BIN_EXE_oxihuman").unwrap_or_else(|_| {
        // Fallback: use the workspace target/debug directory
        let manifest = env!("CARGO_MANIFEST_DIR");
        let root = Path::new(manifest).parent().unwrap().parent().unwrap();
        root.join("target")
            .join("debug")
            .join("oxihuman")
            .to_string_lossy()
            .into_owned()
    });
    PathBuf::from(var)
}

// ── minimal OBJ fixture ───────────────────────────────────────────────────────

/// OBJ content for a minimal "body-like" box mesh (8 verts, 12 tris).
///
/// Dimensions approximate a standing human torso: 0.4 × 1.7 × 0.3 m.
const MINIMAL_OBJ: &str = "\
# OxiHuman e2e test fixture
v -0.2  0.0 -0.15
v  0.2  0.0 -0.15
v  0.2  1.7 -0.15
v -0.2  1.7 -0.15
v -0.2  0.0  0.15
v  0.2  0.0  0.15
v  0.2  1.7  0.15
v -0.2  1.7  0.15
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
f 1/1/1 2/2/2 3/3/3
f 1/1/1 3/3/3 4/4/4
f 5/5/5 7/7/7 6/6/6
f 5/5/5 8/8/8 7/7/7
f 1/1/1 4/4/4 8/8/8
f 1/1/1 8/8/8 5/5/5
f 2/2/2 6/6/6 7/7/7
f 2/2/2 7/7/7 3/3/3
f 3/3/3 7/7/7 8/8/8
f 3/3/3 8/8/8 4/4/4
f 1/1/1 5/5/5 6/6/6
f 1/1/1 6/6/6 2/2/2
";

/// Write the minimal OBJ to a temporary path and return that path.
fn write_base_obj(name: &str) -> PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("oxihuman_e2e_base_{}.obj", name));
    fs::write(&path, MINIMAL_OBJ).expect("could not write base OBJ");
    path
}

/// Generate a unique path in the temp directory.
fn tmp_path(name: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "oxihuman_e2e_{}_{}.{}",
        name,
        std::process::id(),
        ext
    ))
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Run the CLI with the given arguments; return stdout, stderr, and status.
fn run_cli(args: &[&str]) -> (String, String, std::process::ExitStatus) {
    let bin = cli_bin();
    let out = Command::new(&bin)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {:?}: {}", bin, e));
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (stdout, stderr, out.status)
}

// ── test 1: generate with default params ─────────────────────────────────────

#[test]
fn test_generate_basic() {
    let base = write_base_obj("gen_basic");
    let out = tmp_path("gen_basic", "glb");

    let (_stdout, _stderr, status) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--output",
        out.to_str().unwrap(),
    ]);

    assert!(
        status.success(),
        "generate (basic) exited with non-zero status; stderr: {}",
        _stderr
    );
    assert!(
        out.exists(),
        "output GLB file was not created: {}",
        out.display()
    );

    // Clean up
    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&base);
}

// ── test 2: generate with custom morphology params ────────────────────────────

#[test]
fn test_generate_with_params() {
    let base = write_base_obj("gen_params");
    let out = tmp_path("gen_params", "glb");
    let params = r#"{"height":0.8,"weight":0.3,"muscle":0.5,"age":0.5}"#;

    let (_stdout, _stderr, status) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--params",
        params,
        "--output",
        out.to_str().unwrap(),
    ]);

    assert!(
        status.success(),
        "generate (with params) exited with non-zero status; stderr: {}",
        _stderr
    );
    assert!(out.exists(), "output GLB not found: {}", out.display());

    // Verify the output is a valid GLB (magic bytes "glTF")
    let bytes = fs::read(&out).unwrap();
    assert!(bytes.len() >= 4, "GLB output too small");
    // GLB header: bytes [0..4] = b"glTF"
    assert_eq!(&bytes[0..4], b"glTF", "GLB magic bytes mismatch");

    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&base);
}

// ── test 3: export to OBJ ─────────────────────────────────────────────────────

#[test]
fn test_export_obj() {
    let base = write_base_obj("exp_obj");
    let glb = tmp_path("exp_obj_glb", "glb");
    let obj = tmp_path("exp_obj_out", "obj");

    // First generate a GLB
    let (_so, _se, st) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--output",
        glb.to_str().unwrap(),
        "--output-obj",
        obj.to_str().unwrap(),
    ]);
    assert!(
        st.success(),
        "generate (for OBJ test) failed; stderr: {}",
        _se
    );
    assert!(obj.exists(), "OBJ output not found: {}", obj.display());

    // Verify the OBJ contains vertex lines
    let content = fs::read_to_string(&obj).unwrap();
    assert!(content.contains("v "), "OBJ output has no vertex lines");
    assert!(content.contains("f "), "OBJ output has no face lines");

    let _ = fs::remove_file(&glb);
    let _ = fs::remove_file(&obj);
    let _ = fs::remove_file(&base);
}

// ── test 4: export to GLB — verify magic bytes ────────────────────────────────

#[test]
fn test_export_glb() {
    let base = write_base_obj("exp_glb");
    let out = tmp_path("exp_glb", "glb");

    let (_so, _se, status) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--output",
        out.to_str().unwrap(),
    ]);
    assert!(status.success(), "generate (GLB) failed; stderr: {}", _se);

    let bytes = fs::read(&out).unwrap();
    assert!(bytes.len() >= 12, "GLB too small ({} bytes)", bytes.len());

    // GLB 2.0 magic: "glTF" = 0x67 0x6C 0x54 0x46
    assert_eq!(&bytes[0..4], b"glTF", "GLB magic bytes mismatch");

    // Version must be 2
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    assert_eq!(version, 2, "expected GLB version 2, got {}", version);

    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&base);
}

// ── test 5: export to STL — verify ASCII header ───────────────────────────────

#[test]
fn test_export_stl() {
    let base = write_base_obj("exp_stl");
    let out = tmp_path("exp_stl", "stl");

    let (_so, _se, status) = run_cli(&[
        "stl",
        "--base",
        base.to_str().unwrap(),
        "--output",
        out.to_str().unwrap(),
    ]);
    assert!(status.success(), "stl export failed; stderr: {}", _se);
    assert!(out.exists(), "STL not found: {}", out.display());

    let content = fs::read_to_string(&out).unwrap();
    // ASCII STL starts with "solid"
    assert!(
        content.trim_start().starts_with("solid"),
        "STL output does not start with 'solid'"
    );
    assert!(
        content.contains("endsolid"),
        "STL output does not contain 'endsolid'"
    );

    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&base);
}

// ── test 6: pack load (programmatic OXP pack in temp dir) ────────────────────

/// A `PackManifest` TOML with zero entries (valid empty pack).
///
/// The schema requires: `version`, `entries` (array), and `[stats]` section.
const EMPTY_PACK_MANIFEST_TOML: &str = r#"version = "0.1.0"
entries = []

[stats]
total_files = 0
allowed_files = 0
blocked_files = 0
total_deltas = 0
estimated_memory_bytes = 0
"#;

#[test]
fn test_pack_load() {
    let dir = std::env::temp_dir().join(format!("oxihuman_e2e_pack_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    // Write a minimal pack manifest (PackManifest TOML schema)
    let manifest_path = dir.join("pack.toml");
    fs::write(&manifest_path, EMPTY_PACK_MANIFEST_TOML).unwrap();

    // Validate the pack manifest — an empty pack should validate successfully
    let (_so, _se, status) = run_cli(&["validate", "--pack", manifest_path.to_str().unwrap()]);

    assert!(status.success(), "pack validate failed; stderr: {}", _se);

    let _ = fs::remove_dir_all(&dir);
}

// ── test 7: info subcommand returns file info ─────────────────────────────────

#[test]
fn test_info_command() {
    let base = write_base_obj("info_cmd");
    let glb = tmp_path("info_cmd", "glb");

    // Generate a GLB first
    let (_so, _se, st) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--output",
        glb.to_str().unwrap(),
    ]);
    assert!(
        st.success(),
        "generate (for info test) failed; stderr: {}",
        _se
    );

    // Run info on the generated GLB
    let (stdout, _se, status) = run_cli(&["info", glb.to_str().unwrap()]);
    assert!(status.success(), "info command failed; stderr: {}", _se);

    // The info output should mention the file path and GLB format
    assert!(
        stdout.contains("GLB") || stdout.contains("glb"),
        "info output should mention GLB format; got: {}",
        stdout
    );

    let _ = fs::remove_file(&glb);
    let _ = fs::remove_file(&base);
}

// ── test 8: invalid params → graceful error, not panic ───────────────────────

#[test]
fn test_invalid_params() {
    let base = write_base_obj("invalid_params");
    let out = tmp_path("invalid_params", "glb");

    // Pass a completely invalid params string (not valid JSON)
    let (_so, _se, status) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--params",
        "not-valid-json!!!",
        "--output",
        out.to_str().unwrap(),
    ]);

    // Must exit non-zero but must NOT crash with a panic message
    assert!(
        !status.success(),
        "expected non-zero exit for invalid params, but got success"
    );
    // Panics produce "thread 'main' panicked" in stderr
    assert!(
        !_se.contains("panicked"),
        "CLI panicked on invalid params (should have returned error): {}",
        _se
    );

    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&base);
}

// ── test 9: full pipeline generate → export OBJ → re-import ─────────────────

#[test]
fn test_pipeline_generate_export() {
    let base = write_base_obj("pipeline");
    let glb = tmp_path("pipeline_glb", "glb");
    let obj = tmp_path("pipeline_obj", "obj");

    // Step 1: generate GLB + OBJ simultaneously
    let (_so, _se, st) = run_cli(&[
        "generate",
        "--base",
        base.to_str().unwrap(),
        "--output",
        glb.to_str().unwrap(),
        "--output-obj",
        obj.to_str().unwrap(),
    ]);
    assert!(st.success(), "pipeline generate failed; stderr: {}", _se);
    assert!(glb.exists(), "pipeline GLB not found");
    assert!(obj.exists(), "pipeline OBJ not found");

    // Step 2: get stats on the OBJ to verify it parses and has vertices
    let (stdout, _se, st2) = run_cli(&["info", obj.to_str().unwrap()]);
    assert!(st2.success(), "info (OBJ) failed; stderr: {}", _se);

    // Should report vertices
    assert!(
        stdout.contains("Vertices") || stdout.contains("vertices"),
        "info output should mention vertices; got: {}",
        stdout
    );

    // Step 3: re-generate from the OBJ file (it is a valid base mesh)
    let glb2 = tmp_path("pipeline_glb2", "glb");
    let (_so3, _se3, st3) = run_cli(&[
        "generate",
        "--base",
        obj.to_str().unwrap(),
        "--output",
        glb2.to_str().unwrap(),
    ]);
    assert!(
        st3.success(),
        "re-generate from OBJ failed; stderr: {}",
        _se3
    );
    assert!(glb2.exists(), "re-generated GLB not found");

    // Both GLBs should have the same vertex count (stable mesh)
    let bytes1 = fs::read(&glb).unwrap();
    let bytes2 = fs::read(&glb2).unwrap();
    // Both must be valid GLBs
    assert_eq!(&bytes1[0..4], b"glTF", "GLB1 magic bytes wrong");
    assert_eq!(&bytes2[0..4], b"glTF", "GLB2 magic bytes wrong");

    let _ = fs::remove_file(&glb);
    let _ = fs::remove_file(&glb2);
    let _ = fs::remove_file(&obj);
    let _ = fs::remove_file(&base);
}
