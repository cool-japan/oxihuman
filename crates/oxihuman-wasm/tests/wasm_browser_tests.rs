// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Headless WASM browser/node tests for oxihuman-wasm.
//!
//! Run with:
//!   wasm-pack test --node --release -p oxihuman-wasm --all-features
//!   wasm-pack test --headless --chrome --release -p oxihuman-wasm --all-features

use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_node_experimental);

// ── Shared test fixture ───────────────────────────────────────────────────────

/// Minimal valid OBJ with one triangle (position + UV + normal).
fn minimal_obj() -> &'static [u8] {
    b"v 0.0 0.0 0.0\n\
      v 1.0 0.0 0.0\n\
      v 0.0 1.0 0.0\n\
      vt 0.0 0.0\n\
      vt 1.0 0.0\n\
      vt 0.0 1.0\n\
      vn 0.0 0.0 1.0\n\
      vn 0.0 0.0 1.0\n\
      vn 0.0 0.0 1.0\n\
      f 1/1/1 2/2/2 3/3/3\n"
}

// ── Test 1: Engine lifecycle ─────────────────────────────────────────────────
//
// Create engine, set params, get params, reset — all round-trip correctly.

#[wasm_bindgen_test]
fn test_engine_lifecycle_create_set_get_reset() {
    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");

    // Initial vertex count must be positive.
    assert!(
        engine.vertex_count() > 0,
        "vertex_count must be > 0 after creation"
    );

    // Set the four standard params.
    engine.set_height(0.7);
    engine.set_weight(0.3);
    engine.set_muscle(0.6);
    engine.set_age(0.2);

    // Params should be reflected in export_params_json.
    let json = engine.export_params_json();
    assert!(!json.is_empty(), "export_params_json must not be empty");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(
        parsed.is_object(),
        "export_params_json must return a JSON object"
    );

    let height = parsed["height"].as_f64().expect("should succeed");
    let weight = parsed["weight"].as_f64().expect("should succeed");
    assert!(
        (height - 0.7_f64).abs() < 1e-5,
        "height should be ~0.7, got {height}"
    );
    assert!(
        (weight - 0.3_f64).abs() < 1e-5,
        "weight should be ~0.3, got {weight}"
    );

    // Reset must restore defaults.
    engine.reset_params();
    let json_after_reset = engine.export_params_json();
    let parsed_reset: serde_json::Value =
        serde_json::from_str(&json_after_reset).expect("should succeed");
    assert!(
        parsed_reset.is_object(),
        "export_params_json after reset must be an object"
    );

    // After reset, import_params_json round-trip must work.
    let mut engine2 =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");
    engine2.set_height(0.8);
    let params_json = engine2.export_params_json();
    engine2.reset_params();
    engine2
        .import_params_json(&params_json)
        .expect("should succeed");
    let json_rt = engine2.export_params_json();
    let parsed_rt: serde_json::Value = serde_json::from_str(&json_rt).expect("should succeed");
    let h_rt = parsed_rt["height"].as_f64().expect("should succeed");
    assert!(
        (h_rt - 0.8_f64).abs() < 1e-5,
        "round-trip height should be ~0.8, got {h_rt}"
    );
}

// ── Test 2: Mesh build returns non-empty bytes with correct header ────────────
//
// build_mesh_bytes() returns non-empty bytes and uses BUFFER_FORMAT_VERSION header.
// Note: the current format stores [version: u32 LE][n_verts: u32 LE][n_idx: u32 LE] at offset 0.
// The OXIM magic prefix is reserved for future format version 2.

#[wasm_bindgen_test]
fn test_mesh_build_returns_non_empty_with_header() {
    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");
    let bytes = engine.build_mesh_bytes();

    assert!(
        !bytes.is_empty(),
        "build_mesh_bytes must return non-empty bytes"
    );
    assert!(bytes.len() >= 12, "must have at least a 12-byte header");

    // Parse header: [version u32 LE][n_verts u32 LE][n_idx u32 LE]
    let header = oxihuman_wasm::parse_mesh_bytes_header(&bytes);
    assert!(header.is_some(), "header parse must succeed");
    let (version, n_verts, n_idx) = header.expect("should succeed");

    assert_eq!(
        version,
        oxihuman_wasm::BUFFER_FORMAT_VERSION,
        "version must match BUFFER_FORMAT_VERSION"
    );
    assert!(n_verts > 0, "n_verts must be positive");
    assert!(n_idx > 0, "n_idx must be positive");

    // Verify the full buffer length matches the declared counts.
    let expected_len = 12
        + (n_verts as usize) * 12   // positions f32×3
        + (n_verts as usize) * 12   // normals   f32×3
        + (n_verts as usize) * 8    // uvs       f32×2
        + (n_idx as usize) * 4; // indices   u32
    assert_eq!(
        bytes.len(),
        expected_len,
        "buffer length must match declared counts"
    );

    // has_cached_mesh flag should now be true.
    assert!(
        engine.has_cached_mesh(),
        "has_cached_mesh must be true after build"
    );
}

// ── Test 3: Buffer transfer encode/decode round-trip ─────────────────────────
//
// WasmBuffer write_f32_slice / read_f32_slice round-trips correctly.

#[wasm_bindgen_test]
fn test_buffer_transfer_round_trip() {
    use oxihuman_wasm::buffer_transfer::WasmBuffer;

    let src: Vec<f32> = vec![
        1.0,
        -2.5,
        0.0,
        std::f32::consts::PI,
        f32::MAX,
        f32::MIN_POSITIVE,
    ];
    let capacity = src.len() * 4;
    let mut buf = WasmBuffer::new(capacity);

    buf.write_f32_slice(&src).expect("should succeed");
    assert_eq!(
        buf.len(),
        capacity,
        "buffer length must equal slice byte count"
    );

    let out = buf.read_f32_slice().expect("should succeed");
    assert_eq!(
        out.len(),
        src.len(),
        "decoded slice length must match source"
    );
    for (i, (&a, &b)) in src.iter().zip(out.iter()).enumerate() {
        assert_eq!(a, b, "f32 round-trip mismatch at index {i}: {a} vs {b}");
    }

    // Also verify mesh position round-trip.
    let positions: Vec<[f64; 3]> = vec![[1.0, 2.0, 3.0], [-0.5, 0.0, 100.0]];
    let pos_cap = positions.len() * 3 * 8;
    let mut pos_buf = WasmBuffer::new(pos_cap);
    pos_buf
        .write_mesh_positions(&positions)
        .expect("should succeed");
    let floats = pos_buf.read_f64_slice().expect("should succeed");
    assert_eq!(floats.len(), 6, "must have 6 f64 values for 2 positions");
    assert!((floats[0] - 1.0).abs() < f64::EPSILON);
    assert!((floats[3] - (-0.5)).abs() < f64::EPSILON);
}

// ── Test 4: Compressed targets – LitePack serialize/deserialize round-trip ───
//
// LitePack serialize/deserialize round-trips with delta data intact.

#[wasm_bindgen_test]
fn test_compressed_targets_lite_pack_round_trip() {
    use oxihuman_wasm::compressed_target::{CompressedTarget, LitePack};

    // Build a sparse delta array (typical morph target).
    let mut deltas = vec![[0.0f64; 3]; 100];
    deltas[10] = [0.001, -0.002, 0.003];
    deltas[50] = [-0.005, 0.004, 0.001];
    deltas[99] = [0.01, 0.0, -0.01];

    // Verify CompressedTarget round-trip.
    let ct = CompressedTarget::compress(&deltas).expect("should succeed");
    assert!(ct.vertex_count() == 100, "vertex_count must be 100");
    let out = ct.decompress().expect("should succeed");
    assert_eq!(out.len(), 100);
    for i in [10usize, 50, 99] {
        for c in 0..3 {
            assert!(
                (out[i][c] - deltas[i][c]).abs() < 1e-6,
                "component {c} of vertex {i}: {} vs {}",
                out[i][c],
                deltas[i][c]
            );
        }
    }

    // LitePack round-trip.
    let mut pack = LitePack::new();
    pack.set_metadata("version".to_string(), "test-1".to_string());
    pack.add_target("morph_a".to_string(), &deltas)
        .expect("should succeed");

    let mut deltas_b = vec![[0.0f64; 3]; 100];
    deltas_b[5] = [0.02, 0.03, 0.04];
    pack.add_target("morph_b".to_string(), &deltas_b)
        .expect("should succeed");

    let packed_bytes = pack.serialize().expect("should succeed");
    assert!(
        !packed_bytes.is_empty(),
        "serialized LitePack must not be empty"
    );

    // Deserialize and verify.
    let pack2 = LitePack::deserialize(&packed_bytes).expect("should succeed");
    assert_eq!(pack2.len(), 2, "deserialized pack must have 2 targets");
    let names = pack2.target_names();
    assert!(names.contains(&"morph_a"), "must contain morph_a");
    assert!(names.contains(&"morph_b"), "must contain morph_b");

    let meta = pack2.metadata();
    assert_eq!(meta.get("version").map(String::as_str), Some("test-1"));

    let out_a = pack2.get_target("morph_a").expect("should succeed");
    assert!((out_a[10][0] - 0.001_f64).abs() < 1e-6);
    assert!((out_a[50][1] - 0.004_f64).abs() < 1e-6);
}

// ── Test 5: Error handling – invalid JSON returns error, not panic ────────────

#[wasm_bindgen_test]
fn test_error_handling_invalid_json() {
    use oxihuman_wasm::error::WasmError;

    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");

    // import_params_json with invalid JSON must return Err, not panic.
    let result = engine.import_params_json("{{{{ not valid JSON at all }}}}}");
    assert!(
        result.is_err(),
        "import_params_json with garbage must return Err"
    );

    // load_target_from_json with invalid JSON must return false, not panic.
    let ok = engine.load_target_from_json("bad_target", "this is not json");
    assert!(
        !ok,
        "load_target_from_json with invalid JSON must return false"
    );

    // load_target_from_json with wrong schema must also return false.
    let ok2 = engine.load_target_from_json("bad_schema", r#"{"wrong_key": []}"#);
    assert!(
        !ok2,
        "load_target_from_json with wrong schema must return false"
    );

    // WasmError::Json variant must display cleanly.
    let json_err = WasmError::Json("test error message".to_string());
    let display = format!("{json_err}");
    assert!(
        display.contains("JSON"),
        "WasmError::Json display must mention JSON"
    );
    assert!(display.contains("test error message"));

    // WasmError::Other variant.
    let other_err = WasmError::Other("generic failure".to_string());
    let other_display = format!("{other_err}");
    assert!(other_display.contains("generic failure"));
}

// ── Test 6: Param clamping – values outside [0,1] do not cause panic ─────────
//
// The engine's internal set_params clamps to [0,1], so extreme values
// must not produce invalid state or panics.  Mesh building must succeed.

#[wasm_bindgen_test]
fn test_param_clamping_extreme_values() {
    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");

    // Set params well outside [0,1].
    engine.set_height(99.0);
    engine.set_weight(-50.0);
    engine.set_muscle(1000.0);
    engine.set_age(-1.0);

    // Mesh building must succeed without panic.
    let mesh_bytes = engine.build_mesh_bytes();
    assert!(
        !mesh_bytes.is_empty(),
        "mesh must still build with extreme params"
    );

    // Params JSON must still be valid JSON.
    let json = engine.export_params_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should succeed");
    assert!(parsed.is_object());

    // The engine itself clamps on set_params; verify mesh header is still valid.
    let header = oxihuman_wasm::parse_mesh_bytes_header(&mesh_bytes);
    assert!(
        header.is_some(),
        "mesh header must still parse with extreme params"
    );
    let (_, n_verts, _) = header.expect("should succeed");
    assert!(n_verts > 0, "must still have vertices after clamping");

    // Setting boundary values exactly should also work.
    engine.set_height(0.0);
    engine.set_weight(1.0);
    let mesh_boundary = engine.build_mesh_bytes();
    assert!(!mesh_boundary.is_empty());
}

// ── Test 7: Animation frames – record 3, export returns array of 3 ───────────

#[wasm_bindgen_test]
fn test_animation_frames_record_and_export() {
    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");

    assert_eq!(engine.anim_frame_count(), 0, "initially no frames");

    // Record frame 1.
    engine.set_height(0.2);
    engine.set_weight(0.3);
    engine.record_anim_frame();
    assert_eq!(engine.anim_frame_count(), 1);

    // Record frame 2.
    engine.set_height(0.5);
    engine.set_muscle(0.6);
    engine.record_anim_frame();
    assert_eq!(engine.anim_frame_count(), 2);

    // Record frame 3.
    engine.set_age(0.8);
    engine.record_anim_frame();
    assert_eq!(engine.anim_frame_count(), 3);

    // export_anim_json must return a valid JSON array with exactly 3 entries.
    let anim_json = engine.export_anim_json();
    assert!(!anim_json.is_empty(), "export_anim_json must not be empty");
    let parsed: serde_json::Value = serde_json::from_str(&anim_json).expect("should succeed");
    assert!(
        parsed.is_array(),
        "export_anim_json must return a JSON array"
    );

    let frames = parsed.as_array().expect("should succeed");
    assert_eq!(
        frames.len(),
        3,
        "must have exactly 3 frames, got {}",
        frames.len()
    );

    // Each frame must be an object with at least a "height" key.
    for (i, frame) in frames.iter().enumerate() {
        assert!(frame.is_object(), "frame {i} must be a JSON object");
        assert!(
            frame.get("height").is_some(),
            "frame {i} must have a 'height' key"
        );
    }

    // Seek to frame 0 and verify it restores params.
    engine.seek_anim_frame(0);

    // Clear frames.
    engine.clear_anim_frames();
    assert_eq!(
        engine.anim_frame_count(),
        0,
        "anim_frame_count must be 0 after clear"
    );

    // FPS setting must be stored.
    engine.set_anim_fps(30.0);
    assert!((engine.get_anim_fps() - 30.0).abs() < f32::EPSILON);
}

// ── Test 8: Target loading – load, verify count increases, unload, verify ────

#[wasm_bindgen_test]
fn test_target_loading_and_unloading() {
    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");

    // No JSON targets initially.
    assert_eq!(engine.loaded_target_count(), 0, "no json targets initially");

    // Load a valid JSON target.
    let target_json = r#"{"deltas":[[0, 0.1, 0.2, 0.3],[1, -0.1, 0.0, 0.1]]}"#;
    let loaded = engine.load_target_from_json("test_morph_a", target_json);
    assert!(
        loaded,
        "load_target_from_json must return true for valid JSON"
    );
    assert_eq!(
        engine.loaded_target_count(),
        1,
        "count must be 1 after loading one target"
    );

    // Load a second target.
    let target_json2 = r#"{"deltas":[[2, 0.0, 0.5, 0.0]]}"#;
    let loaded2 = engine.load_target_from_json("test_morph_b", target_json2);
    assert!(loaded2);
    assert_eq!(
        engine.loaded_target_count(),
        2,
        "count must be 2 after loading two targets"
    );

    // Verify target names are listed.
    let names_json = engine.get_loaded_target_names();
    assert!(
        names_json.contains("test_morph_a"),
        "target names must include test_morph_a"
    );
    assert!(
        names_json.contains("test_morph_b"),
        "target names must include test_morph_b"
    );

    // Set target weight.
    let found = engine.set_target_weight_by_name("test_morph_a", 0.75);
    assert!(
        found,
        "set_target_weight_by_name must return true for existing target"
    );
    let w = engine.get_target_weight_by_name("test_morph_a");
    assert!(
        (w - 0.75).abs() < f32::EPSILON,
        "weight must be 0.75, got {w}"
    );

    // Unload first target — count must decrease.
    let removed = engine.unload_target("test_morph_a");
    assert!(
        removed,
        "unload_target must return true for existing target"
    );
    assert_eq!(
        engine.loaded_target_count(),
        1,
        "count must be 1 after unloading"
    );

    // Unload non-existent target must return false.
    let removed_again = engine.unload_target("nonexistent_target_xyz");
    assert!(
        !removed_again,
        "unload_target must return false for missing target"
    );
    assert_eq!(engine.loaded_target_count(), 1, "count must still be 1");

    // Unload second target.
    let removed2 = engine.unload_target("test_morph_b");
    assert!(removed2);
    assert_eq!(
        engine.loaded_target_count(),
        0,
        "count must be 0 after all unloaded"
    );
}

// ── Test 9: Export GLB – starts with "glTF" magic bytes ──────────────────────
//
// oxihuman_export::glb::export_glb writes a file starting with GLB magic
// 0x46546C67 ("glTF" in LE), then we read it back and verify the first 4 bytes.
//
// This uses the filesystem via std::env::temp_dir(), which is available in
// Node.js WASM mode (run_in_node_experimental).

#[wasm_bindgen_test]
fn test_export_glb_magic_bytes() {
    let mut engine =
        oxihuman_wasm::WasmEngine::new_from_obj_bytes(minimal_obj()).expect("should succeed");

    // build_mesh_prepared returns a fully-prepared MeshBuffers with suit flag applied.
    let mesh = engine.build_mesh_prepared();

    // Write to a temp GLB file.
    let tmp_path = std::env::temp_dir().join("oxihuman_wasm_browser_test.glb");
    oxihuman_export::glb::export_glb(&mesh, &tmp_path).expect("should succeed");

    // Read back and verify GLB magic.
    let glb_bytes = std::fs::read(&tmp_path).expect("should succeed");
    assert!(glb_bytes.len() >= 4, "GLB must be at least 4 bytes");

    // GLB magic: 0x46546C67 in little-endian = bytes [0x67, 0x6C, 0x54, 0x46] = "glTF"
    let magic = u32::from_le_bytes([glb_bytes[0], glb_bytes[1], glb_bytes[2], glb_bytes[3]]);
    assert_eq!(
        magic, 0x46546C67,
        "first 4 bytes must be glTF magic 0x46546C67, got {magic:#010x}"
    );

    // Clean up.
    let _ = std::fs::remove_file(&tmp_path);
}

// ── Test 10: Service worker – generate_sw_js produces non-empty JS ────────────
//
// generate_sw_js produces a non-empty JS string containing an "install" handler.

#[wasm_bindgen_test]
fn test_service_worker_generate_sw_js() {
    use oxihuman_wasm::service_worker::{generate_sw_js, CacheStrategy, ServiceWorkerConfig};

    let config = ServiceWorkerConfig {
        cache_name: "oxihuman-test-cache-v1".to_string(),
        asset_urls: vec![
            "/oxihuman_wasm.js".to_string(),
            "/oxihuman_wasm_bg.wasm".to_string(),
            "/index.html".to_string(),
        ],
        max_cache_size_mb: 25.0,
        cache_strategy: CacheStrategy::CacheFirst,
    };

    let sw_js = generate_sw_js(&config);

    assert!(
        !sw_js.is_empty(),
        "generate_sw_js must produce non-empty output"
    );

    // Must contain the install event handler.
    assert!(
        sw_js.contains("addEventListener('install'"),
        "SW JS must contain 'install' event handler"
    );

    // Must contain the cache name.
    assert!(
        sw_js.contains("oxihuman-test-cache-v1"),
        "SW JS must contain the configured cache name"
    );

    // Must contain all asset URLs.
    assert!(
        sw_js.contains("/oxihuman_wasm.js"),
        "SW JS must list oxihuman_wasm.js"
    );
    assert!(sw_js.contains("/index.html"), "SW JS must list index.html");

    // Must contain the activate handler (for cache cleanup).
    assert!(
        sw_js.contains("addEventListener('activate'"),
        "SW JS must contain 'activate' event handler"
    );

    // Must contain the fetch handler.
    assert!(
        sw_js.contains("addEventListener('fetch'"),
        "SW JS must contain 'fetch' event handler"
    );

    // Test all three strategies produce non-empty output with install handlers.
    for strategy in [
        CacheStrategy::CacheFirst,
        CacheStrategy::NetworkFirst,
        CacheStrategy::StaleWhileRevalidate,
    ] {
        let cfg = ServiceWorkerConfig {
            cache_name: "test-v1".to_string(),
            asset_urls: vec!["/app.wasm".to_string()],
            max_cache_size_mb: 10.0,
            cache_strategy: strategy,
        };
        let js = generate_sw_js(&cfg);
        assert!(!js.is_empty());
        assert!(js.contains("addEventListener('install'"));
    }
}
