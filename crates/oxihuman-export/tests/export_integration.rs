// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the oxihuman-export crate.
//!
//! Each test exercises a distinct export format, verifying the output binary
//! or text structure at the byte / string level without any round-trip
//! parsing library.  All file I/O uses `std::env::temp_dir()`.

use oxihuman_export::{
    export_3mf, export_glb, export_mesh_fbx_binary, export_obj, export_stl_binary, run_batch,
    BatchCharacterSpec, BatchConfig, BatchOutputFormat, BlendShapeTimeSamples, ThreeMfOptions,
    UsdaWriter, VrmBoneName, VrmExporter, VrmHumanBone, VrmHumanoid10, VrmMeta10,
};
use oxihuman_mesh::MeshBuffers;
use oxihuman_morph::engine::MeshBuffers as MorphMB;

// ── Shared test fixture ───────────────────────────────────────────────────────

/// A minimal one-triangle mesh, with `has_suit = true` so GLB export is allowed.
fn minimal_mesh() -> MeshBuffers {
    let mut m = MeshBuffers::from_morph(MorphMB {
        positions: vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        normals: vec![[0.0_f32, 0.0, 1.0]; 3],
        uvs: vec![[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]],
        indices: vec![0u32, 1, 2],
        has_suit: false,
    });
    // Mark suit present so GLB exporter does not refuse the mesh.
    m.tangents = vec![[1.0_f32, 0.0, 0.0, 1.0]; 3];
    m.has_suit = true;
    m
}

/// Return a unique path inside the system temp directory.
fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("oxihuman_export_integration_{}", name))
}

// ── 1. GLB round-trip ─────────────────────────────────────────────────────────

/// Export a minimal mesh → GLB file → read back bytes → verify GLB magic.
#[test]
fn test_glb_roundtrip() {
    let mesh = minimal_mesh();
    let path = tmp_path("test_glb_roundtrip.glb");

    export_glb(&mesh, &path).expect("export_glb should succeed");

    let bytes = std::fs::read(&path).expect("reading exported GLB file");
    assert!(
        bytes.len() >= 4,
        "GLB output must be at least 4 bytes, got {}",
        bytes.len()
    );
    // GLB magic "glTF" — little-endian u32 0x46546C67
    assert_eq!(
        &bytes[..4],
        b"glTF",
        "GLB file must start with the 'glTF' magic bytes"
    );
}

// ── 2. OBJ round-trip ────────────────────────────────────────────────────────

/// Export a minimal mesh → OBJ file → read back string → verify vertex lines.
#[test]
fn test_obj_roundtrip() {
    let mesh = minimal_mesh();
    let path = tmp_path("test_obj_roundtrip.obj");

    export_obj(&mesh, &path).expect("export_obj should succeed");

    let content = std::fs::read_to_string(&path).expect("reading exported OBJ file");
    assert!(
        content.contains("v "),
        "OBJ file must contain at least one vertex line ('v ')"
    );
    // All three vertex positions must appear.
    let vertex_line_count = content.lines().filter(|l| l.starts_with("v ")).count();
    assert_eq!(
        vertex_line_count, 3,
        "expected 3 vertex lines in OBJ, found {}",
        vertex_line_count
    );
}

// ── 3. Binary STL header ─────────────────────────────────────────────────────

/// Export a minimal mesh → binary STL file → verify 80-byte header and
/// the triangle count u32 stored at byte offset 80.
#[test]
fn test_stl_binary_header() {
    let mesh = minimal_mesh();
    let path = tmp_path("test_stl_binary_header.stl");

    export_stl_binary(&mesh, &path).expect("export_stl_binary should succeed");

    let bytes = std::fs::read(&path).expect("reading exported STL file");
    assert!(
        bytes.len() >= 84,
        "binary STL must be at least 84 bytes (header + count), got {}",
        bytes.len()
    );

    // The 80-byte header is the first 80 bytes; no specific content required
    // by spec beyond being 80 bytes long.
    let header = &bytes[..80];
    assert_eq!(header.len(), 80, "STL header must be exactly 80 bytes");

    // Triangle count is a little-endian u32 at offset 80.
    let tri_count = u32::from_le_bytes(
        bytes[80..84]
            .try_into()
            .expect("4 bytes for triangle count"),
    );
    assert_eq!(
        tri_count, 1,
        "expected 1 triangle in the STL, found {}",
        tri_count
    );
}

// ── 4. USDA blend-shape animation ────────────────────────────────────────────

/// Build a BlendShapeTimeSamples → write via UsdaWriter → verify the USDA
/// string contains `"timeSamples"`.
#[test]
fn test_usda_blend_shape_animation() {
    let mut writer = UsdaWriter::new();
    writer.write_header("Y", 1.0);

    let samples = vec![
        BlendShapeTimeSamples {
            shape_name: "smile".to_string(),
            time_weight_pairs: vec![(0.0, 0.0), (10.0, 1.0), (20.0, 0.0)],
        },
        BlendShapeTimeSamples {
            shape_name: "frown".to_string(),
            time_weight_pairs: vec![(5.0, 0.5), (15.0, 1.0)],
        },
    ];

    writer
        .write_blend_shape_animation("/Root/Body", &samples)
        .expect("write_blend_shape_animation should succeed");

    let usda = writer.finish();
    assert!(
        usda.contains("timeSamples"),
        "USDA output must contain 'timeSamples', got:\n{}",
        usda
    );
    // The blend shape names must also appear in the output.
    assert!(
        usda.contains("smile"),
        "USDA output must contain blend shape name 'smile'"
    );
    assert!(
        usda.contains("frown"),
        "USDA output must contain blend shape name 'frown'"
    );
}

// ── 5. FBX binary magic ───────────────────────────────────────────────────────

/// Export a minimal mesh → FBX binary bytes → verify the 18-byte
/// `"Kaydara FBX Binary"` prefix.
#[test]
fn test_fbx_binary_magic() {
    let mesh = minimal_mesh();
    let bytes = export_mesh_fbx_binary(&mesh).expect("export_mesh_fbx_binary should succeed");

    assert!(
        bytes.len() >= 23,
        "FBX output must be at least 23 bytes for the magic header, got {}",
        bytes.len()
    );
    // The full FBX magic is b"Kaydara FBX Binary  \x00\x1a\x00" (23 bytes).
    // The spec header prefix we verify is the human-readable 18-byte substring.
    assert_eq!(
        &bytes[..18],
        b"Kaydara FBX Binary",
        "FBX binary file must start with 'Kaydara FBX Binary'"
    );
}

// ── 6. VRM JSON structure ─────────────────────────────────────────────────────

/// Build a minimal VrmExporter → export → parse the embedded JSON chunk →
/// verify the outer JSON object contains `"extensionsUsed"`.
#[test]
fn test_vrm_json_structure() {
    let mut exporter = VrmExporter::new();

    // Set a minimal one-triangle mesh (no skeleton required).
    exporter
        .set_mesh(
            &[[0.0_f64, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0.0_f64, 0.0, 1.0]; 3],
            &[[0.0_f64, 0.0], [1.0, 0.0], [0.0, 1.0]],
            &[[0usize, 1, 2]],
        )
        .expect("VrmExporter::set_mesh should succeed");

    // Build a minimal humanoid with all required bones (17 required).
    let required_bones = VrmBoneName::all_required();
    let bones: Vec<VrmHumanBone> = required_bones
        .iter()
        .enumerate()
        .map(|(i, &name)| VrmHumanBone {
            name,
            node_index: i,
        })
        .collect();
    let humanoid = VrmHumanoid10 { bones };
    exporter
        .set_humanoid(&humanoid)
        .expect("VrmExporter::set_humanoid should succeed");

    // Set a minimal meta.
    let meta = VrmMeta10::default_cc_by("IntegrationTestAvatar");
    exporter
        .set_meta(&meta)
        .expect("VrmExporter::set_meta should succeed");

    let glb_bytes = exporter
        .export()
        .expect("VrmExporter::export should succeed");

    // A GLB JSON chunk starts at byte 20 (12-byte header + 4-byte chunk length
    // + 4-byte chunk type).  The JSON chunk length is at bytes 12..16 (LE u32).
    assert!(
        glb_bytes.len() >= 20,
        "VRM GLB output must be at least 20 bytes"
    );
    let json_chunk_len = u32::from_le_bytes(
        glb_bytes[12..16]
            .try_into()
            .expect("4 bytes for JSON chunk length"),
    ) as usize;
    let json_start = 20usize;
    let json_end = json_start + json_chunk_len;
    assert!(
        glb_bytes.len() >= json_end,
        "GLB bytes too short for declared JSON chunk"
    );

    let json_str = std::str::from_utf8(&glb_bytes[json_start..json_end])
        .expect("JSON chunk must be valid UTF-8");

    // Trim whitespace padding added to align to 4 bytes.
    let json_str = json_str.trim_end();

    let json_val: serde_json::Value =
        serde_json::from_str(json_str).expect("JSON chunk must be valid JSON");

    assert!(
        json_val.get("extensionsUsed").is_some(),
        "VRM glTF JSON must contain 'extensionsUsed' key; keys present: {:?}",
        json_val
            .as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>())
    );
}

// ── 7. 3MF ZIP structure ──────────────────────────────────────────────────────

/// Export a minimal mesh to 3MF → verify the ZIP local-file-header magic
/// `PK\x03\x04` at the start of the byte stream.
#[test]
fn test_3mf_zip_structure() {
    let mesh = minimal_mesh();
    let opts = ThreeMfOptions::default();
    let result = export_3mf(&mesh, &opts);

    assert!(
        result.zip_bytes.len() >= 4,
        "3MF ZIP output must be at least 4 bytes"
    );
    assert_eq!(
        &result.zip_bytes[..4],
        b"PK\x03\x04",
        "3MF file must start with ZIP local-file-header magic 'PK\\x03\\x04'"
    );
}

// ── 8. Batch pipeline — multiple formats ─────────────────────────────────────

/// Build a BatchPipeline that exports the same stub mesh to OBJ, STL, and GLB
/// (via the batch_pipeline API) and verify all three specs succeed.
#[test]
fn test_batch_pipeline_multi_format() {
    let out_dir = std::env::temp_dir().join("oxihuman_batch_multi_format_test");
    std::fs::create_dir_all(&out_dir).expect("creating temp output directory");

    let specs = vec![
        BatchCharacterSpec {
            id: "multi_obj".to_string(),
            params: std::collections::HashMap::new(),
            output_format: BatchOutputFormat::Obj,
            output_path: out_dir.join("multi_obj.obj"),
        },
        BatchCharacterSpec {
            id: "multi_stl".to_string(),
            params: std::collections::HashMap::new(),
            output_format: BatchOutputFormat::Stl,
            output_path: out_dir.join("multi_stl.stl"),
        },
        BatchCharacterSpec {
            id: "multi_glb".to_string(),
            params: std::collections::HashMap::new(),
            output_format: BatchOutputFormat::Glb,
            output_path: out_dir.join("multi_glb.glb"),
        },
    ];

    let cfg = BatchConfig {
        base_obj_path: None,
        max_parallel: 1,
        skip_existing: false,
        verbose: false,
    };

    let result = run_batch(&specs, &cfg);

    assert_eq!(result.total, 3, "batch should have processed 3 specs");
    assert_eq!(
        result.succeeded, 3,
        "all 3 batch specs should succeed; errors: {:?}",
        result.errors
    );
    assert_eq!(result.failed, 0, "no batch spec should fail");

    // Verify the OBJ output contains vertex lines.
    let obj_content =
        std::fs::read_to_string(out_dir.join("multi_obj.obj")).expect("reading batch OBJ output");
    assert!(
        obj_content.contains("v "),
        "batch OBJ output must contain vertex lines"
    );

    // Verify the STL output starts with "solid" (ASCII STL stub).
    let stl_content =
        std::fs::read_to_string(out_dir.join("multi_stl.stl")).expect("reading batch STL output");
    assert!(
        stl_content.starts_with("solid"),
        "batch STL output must start with 'solid'"
    );

    // Verify the GLB stub was written (non-empty file).
    let glb_metadata =
        std::fs::metadata(out_dir.join("multi_glb.glb")).expect("checking batch GLB output");
    assert!(glb_metadata.len() > 0, "batch GLB output must be non-empty");
}
