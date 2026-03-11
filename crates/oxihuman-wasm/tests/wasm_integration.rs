// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for oxihuman-wasm
//! These tests verify the WASM API surface works correctly in native mode.
//! For actual browser testing, use wasm-pack test.

/// Minimal OBJ with a single triangle (position + UV + normal per vertex).
fn minimal_obj() -> &'static str {
    "v 0.0 0.0 0.0\n\
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

// ─── Buffer round-trip tests ────────────────────────────────────────────────

#[test]
fn test_buffer_roundtrip() {
    use oxihuman_wasm::BUFFER_FORMAT_VERSION;

    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    let mesh_bytes = engine.build_mesh_bytes();
    assert!(!mesh_bytes.is_empty(), "mesh bytes should not be empty");

    // Parse header: the first 4 bytes are the format version
    let header = oxihuman_wasm::parse_mesh_bytes_header(&mesh_bytes);
    assert!(header.is_some(), "header should parse successfully");

    let (version, n_verts, n_idx) = header.unwrap();
    assert_eq!(version, BUFFER_FORMAT_VERSION, "version mismatch");
    assert!(n_verts > 0, "should have at least one vertex");
    assert!(n_idx > 0, "should have at least one index");

    // Expected size: header(12) + positions(n*12) + normals(n*12) + uvs(n*8) + indices(m*4)
    let expected_size = 12
        + (n_verts as usize) * 12
        + (n_verts as usize) * 12
        + (n_verts as usize) * 8
        + (n_idx as usize) * 4;
    assert_eq!(
        mesh_bytes.len(),
        expected_size,
        "buffer size should match expected layout"
    );
}

#[test]
fn test_buffer_header_too_short() {
    // Less than 12 bytes should return None
    let short = [0u8; 8];
    assert!(oxihuman_wasm::parse_mesh_bytes_header(&short).is_none());
}

#[test]
fn test_buffer_header_exactly_12_bytes() {
    let mut buf = [0u8; 12];
    buf[0..4].copy_from_slice(&42u32.to_le_bytes()); // version
    buf[4..8].copy_from_slice(&10u32.to_le_bytes()); // n_verts
    buf[8..12].copy_from_slice(&36u32.to_le_bytes()); // n_idx

    let header = oxihuman_wasm::parse_mesh_bytes_header(&buf);
    assert!(header.is_some());
    let (v, nv, ni) = header.unwrap();
    assert_eq!(v, 42);
    assert_eq!(nv, 10);
    assert_eq!(ni, 36);
}

// ─── Compressed/quantized target round-trip ─────────────────────────────────

#[test]
fn test_compressed_target_roundtrip() {
    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    // Export quantized bytes from the base mesh
    let q_bytes = engine.export_quantized_bytes();
    assert!(!q_bytes.is_empty(), "quantized bytes should not be empty");

    // Verify QMSH header magic
    assert_eq!(&q_bytes[0..4], b"QMSH", "should start with QMSH magic");

    // Verify version field
    let version = u32::from_le_bytes(q_bytes[4..8].try_into().unwrap());
    assert_eq!(version, 1, "quantized mesh version should be 1");

    // Verify vertex/index counts are sensible
    let vc = u32::from_le_bytes(q_bytes[8..12].try_into().unwrap());
    let ic = u32::from_le_bytes(q_bytes[12..16].try_into().unwrap());
    assert!(vc > 0, "vertex count should be positive");
    assert!(ic > 0, "index count should be positive");
    assert_eq!(
        ic % 3,
        0,
        "index count should be a multiple of 3 (triangles)"
    );

    // Verify total size matches expected layout:
    //   header: 16 bytes
    //   pos_range: 24 bytes (3 axes * 2 floats * 4 bytes)
    //   positions: vc * 3 * 2 bytes (u16)
    //   normals: vc * 3 * 1 byte (i8)
    //   uvs: vc * 2 * 2 bytes (u16)
    //   indices: ic * 4 bytes (u32)
    //   has_suit: 1 byte
    let expected =
        16 + 24 + (vc as usize) * 6 + (vc as usize) * 3 + (vc as usize) * 4 + (ic as usize) * 4 + 1;
    assert_eq!(q_bytes.len(), expected, "quantized buffer size mismatch");
}

// ─── Lite-pack serialize / deserialize ──────────────────────────────────────

#[test]
fn test_lite_pack_serialize_deserialize() {
    // Build a minimal STORE-only ZIP in memory with a single text entry
    let filename = b"hello.txt";
    let content = b"world";

    let mut zip_bytes: Vec<u8> = Vec::new();

    // Local file header
    zip_bytes.extend_from_slice(&0x04034B50u32.to_le_bytes()); // signature
    zip_bytes.extend_from_slice(&20u16.to_le_bytes()); // version needed
    zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // flags
    zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // compression (STORE)
    zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // mod time
    zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // mod date
    zip_bytes.extend_from_slice(&0u32.to_le_bytes()); // crc-32
    zip_bytes.extend_from_slice(&(content.len() as u32).to_le_bytes()); // compressed size
    zip_bytes.extend_from_slice(&(content.len() as u32).to_le_bytes()); // uncompressed size
    zip_bytes.extend_from_slice(&(filename.len() as u16).to_le_bytes()); // filename len
    zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // extra field len
    zip_bytes.extend_from_slice(filename);
    zip_bytes.extend_from_slice(content);

    // Use the pack scanner to read back entries
    let entries = oxihuman_wasm::pack::scan_zip_local_entries(&zip_bytes).unwrap();
    assert_eq!(entries.len(), 1, "should find exactly one entry");
    assert_eq!(entries[0].0, "hello.txt");
    assert_eq!(entries[0].1, b"world");
}

#[test]
fn test_lite_pack_empty_zip() {
    // An empty byte slice should yield zero entries
    let entries = oxihuman_wasm::pack::scan_zip_local_entries(&[]).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_lite_pack_truncated_entry() {
    // ZIP signature followed by truncated header should not panic
    let mut data = Vec::new();
    data.extend_from_slice(&0x04034B50u32.to_le_bytes());
    // Only 4 bytes after signature, not enough for a full 30-byte header
    data.extend_from_slice(&[0u8; 10]);

    // Should not panic, just return empty or partial results
    let entries = oxihuman_wasm::pack::scan_zip_local_entries(&data).unwrap();
    assert!(entries.is_empty(), "truncated entry should be skipped");
}

// ─── Engine API tests ───────────────────────────────────────────────────────

#[test]
fn test_engine_creation_and_basic_ops() {
    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    // Set some parameters
    engine.set_height(0.5);
    engine.set_weight(0.3);
    engine.set_muscle(0.7);
    engine.set_age(0.2);

    // Build mesh -- should succeed
    let mesh = engine.build_mesh_bytes();
    assert!(!mesh.is_empty());

    // Vertex count should be positive
    assert!(engine.vertex_count() > 0);
}

#[test]
fn test_engine_strict_mode() {
    let obj_bytes = minimal_obj().as_bytes();
    let engine = oxihuman_wasm::WasmEngine::new_strict(obj_bytes);
    assert!(engine.is_ok(), "strict engine creation should succeed");
}

#[test]
fn test_engine_params_json_roundtrip() {
    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    engine.set_height(0.8);
    engine.set_weight(0.4);

    let json = engine.export_params_json();
    assert!(!json.is_empty());

    // Parse as JSON to verify it's valid
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object(), "params JSON should be an object");
}

#[test]
fn test_engine_reset_params() {
    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    engine.set_height(0.9);
    engine.reset_params();

    // After reset, export should still produce valid JSON
    let json = engine.export_params_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_engine_target_operations() {
    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    // Initially no targets loaded
    assert_eq!(engine.target_count(), 0);

    // Load a JSON target with deltas for vertex 0
    let target_json = r#"{"deltas":[[0, 0.1, 0.2, 0.3]]}"#;
    let loaded = engine.load_target_from_json("test_target", target_json);
    assert!(loaded, "loading a valid JSON target should succeed");

    // Unload
    let removed = engine.unload_target("test_target");
    assert!(removed, "unloading an existing target should succeed");

    let removed_again = engine.unload_target("nonexistent");
    assert!(
        !removed_again,
        "unloading a non-existent target should return false"
    );
}

#[test]
fn test_engine_animation_basics() {
    let obj_bytes = minimal_obj().as_bytes();
    let mut engine = oxihuman_wasm::WasmEngine::new_from_obj_bytes(obj_bytes).unwrap();

    // Record a frame
    engine.set_height(0.3);
    engine.record_anim_frame();
    assert_eq!(engine.anim_frame_count(), 1);

    // Record another
    engine.set_height(0.7);
    engine.record_anim_frame();
    assert_eq!(engine.anim_frame_count(), 2);

    // Export animation JSON (returns a JSON array of frame objects)
    let anim_json = engine.export_anim_json();
    let parsed: serde_json::Value = serde_json::from_str(&anim_json).unwrap();
    assert!(parsed.is_array());

    // Clear
    engine.clear_anim_frames();
    assert_eq!(engine.anim_frame_count(), 0);
}

#[test]
fn test_engine_invalid_obj() {
    let bad_obj = b"this is not valid OBJ data at all";
    let result = oxihuman_wasm::WasmEngine::new_from_obj_bytes(bad_obj);
    // Should either succeed with empty mesh or fail gracefully
    // The parser may accept any text without faces as a zero-vertex mesh
    // Either way, it should not panic
    let _ = result;
}
