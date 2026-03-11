// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use bytemuck::cast_slice;
use oxihuman_mesh::mesh::MeshBuffers;
use serde_json::json;
use std::io::Write;
use std::path::Path;

// GLB constants (same as glb.rs)
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

/// A named blend shape: positional deltas from the base mesh.
#[allow(dead_code)]
pub struct BlendShape {
    pub name: String,
    /// Per-vertex position deltas [dx, dy, dz]. Must be same length as mesh.positions.
    pub position_deltas: Vec<[f32; 3]>,
}

impl BlendShape {
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>, deltas: Vec<[f32; 3]>) -> Self {
        Self {
            name: name.into(),
            position_deltas: deltas,
        }
    }

    /// Create a zero blend shape (no deformation) — useful as a neutral reference.
    #[allow(dead_code)]
    pub fn zero(name: impl Into<String>, n_verts: usize) -> Self {
        Self {
            name: name.into(),
            position_deltas: vec![[0.0, 0.0, 0.0]; n_verts],
        }
    }
}

/// Compute min and max for a slice of VEC3 deltas.
fn min_max_vec3(deltas: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::INFINITY; 3];
    let mut mx = [f32::NEG_INFINITY; 3];
    for d in deltas {
        for i in 0..3 {
            mn[i] = mn[i].min(d[i]);
            mx[i] = mx[i].max(d[i]);
        }
    }
    (mn, mx)
}

/// Export a GLB file with one mesh primitive and multiple morph targets (blend shapes).
///
/// The base mesh uses `mesh.positions`; each blend shape provides position deltas.
/// The resulting GLB has:
///   - One base POSITION accessor
///   - One INDEX accessor
///   - One morph target per BlendShape, each with a POSITION_delta accessor
///   - mesh.weights = [0.0, 0.0, ...] (blend weights start at zero)
///   - mesh.extras.targetNames = ["name1", "name2", ...]
#[allow(dead_code)]
pub fn export_glb_blend_shapes(
    mesh: &MeshBuffers,
    shapes: &[BlendShape],
    path: &Path,
) -> Result<()> {
    let n_verts = mesh.positions.len();
    let n_idx = mesh.indices.len();

    // ── 1. Build BIN chunk ────────────────────────────────────────────────────
    // Layout:
    //   [base POSITION: f32*3*n_verts]
    //   [INDEX: u32*n_idx]
    //   [for each shape: POSITION delta f32*3*n_verts]

    let pos_bytes: &[u8] = cast_slice(&mesh.positions);
    let idx_bytes: &[u8] = cast_slice(&mesh.indices);

    let pos_offset = 0usize;
    let idx_offset = pos_offset + pos_bytes.len();
    let mut bin_data: Vec<u8> = Vec::new();
    bin_data.extend_from_slice(pos_bytes);
    bin_data.extend_from_slice(idx_bytes);

    // Track byte offsets for each morph delta accessor
    let mut morph_offsets: Vec<usize> = Vec::with_capacity(shapes.len());
    for shape in shapes {
        morph_offsets.push(bin_data.len());
        let delta_bytes: &[u8] = cast_slice(&shape.position_deltas);
        bin_data.extend_from_slice(delta_bytes);
    }

    // Pad BIN to 4-byte boundary
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }
    let total_bin = bin_data.len() as u32;

    // ── 2. Build GLTF JSON ────────────────────────────────────────────────────
    // Accessor indices:
    //   0: base POSITION (VEC3, f32)
    //   1: INDEX (SCALAR, u32)
    //   2..2+shapes.len(): morph POSITION deltas

    let mut accessors: Vec<serde_json::Value> = Vec::new();
    let mut buffer_views: Vec<serde_json::Value> = Vec::new();

    // bufferView 0: base positions
    buffer_views.push(json!({
        "buffer": 0,
        "byteOffset": pos_offset,
        "byteLength": pos_bytes.len()
    }));
    // accessor 0: base POSITION
    accessors.push(json!({
        "bufferView": 0,
        "componentType": 5126,
        "count": n_verts,
        "type": "VEC3",
        "byteOffset": 0
    }));

    // bufferView 1: indices
    buffer_views.push(json!({
        "buffer": 0,
        "byteOffset": idx_offset,
        "byteLength": idx_bytes.len()
    }));
    // accessor 1: INDEX
    accessors.push(json!({
        "bufferView": 1,
        "componentType": 5125,
        "count": n_idx,
        "type": "SCALAR"
    }));

    // Morph delta accessors and buffer views
    let mut targets: Vec<serde_json::Value> = Vec::with_capacity(shapes.len());
    for (i, shape) in shapes.iter().enumerate() {
        let bv_idx = buffer_views.len();
        let delta_byte_len = shape.position_deltas.len() * std::mem::size_of::<[f32; 3]>();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": morph_offsets[i],
            "byteLength": delta_byte_len
        }));

        let acc_idx = accessors.len();
        let mut acc = json!({
            "bufferView": bv_idx,
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC3"
        });

        if !shape.position_deltas.is_empty() {
            let (mn, mx) = min_max_vec3(&shape.position_deltas);
            acc["min"] = json!([mn[0], mn[1], mn[2]]);
            acc["max"] = json!([mx[0], mx[1], mx[2]]);
        }

        accessors.push(acc);
        targets.push(json!({ "POSITION": acc_idx }));
    }

    // Build mesh.weights and mesh.extras.targetNames
    let weights: Vec<f32> = vec![0.0; shapes.len()];
    let target_names: Vec<&str> = shapes.iter().map(|s| s.name.as_str()).collect();

    let gltf = json!({
        "asset": { "version": "2.0", "generator": "oxihuman-export" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{
            "extras": { "targetNames": target_names },
            "weights": weights,
            "primitives": [{
                "attributes": { "POSITION": 0 },
                "indices": 1,
                "targets": targets
            }]
        }],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": total_bin }]
    });

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    // Pad JSON to 4-byte boundary with spaces
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }

    // ── 3. Write GLB ──────────────────────────────────────────────────────────
    let json_chunk_len = json_bytes.len() as u32;
    let bin_chunk_len = bin_data.len() as u32;
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut file = std::fs::File::create(path)?;

    // GLB header (12 bytes)
    file.write_all(&GLB_MAGIC.to_le_bytes())?;
    file.write_all(&GLB_VERSION.to_le_bytes())?;
    file.write_all(&total_len.to_le_bytes())?;

    // JSON chunk
    file.write_all(&json_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_JSON.to_le_bytes())?;
    file.write_all(&json_bytes)?;

    // BIN chunk
    file.write_all(&bin_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_BIN.to_le_bytes())?;
    file.write_all(&bin_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    #[test]
    fn blend_shape_zero_has_correct_len() {
        let shape = BlendShape::zero("neutral", 10);
        assert_eq!(shape.position_deltas.len(), 10);
    }

    #[test]
    fn export_glb_blend_shapes_creates_file() {
        let mesh = triangle_mesh();
        let shapes = vec![
            BlendShape::new("smile", vec![[0.1, 0.0, 0.0]; 3]),
            BlendShape::zero("neutral", 3),
        ];
        let path = std::path::Path::new("/tmp/test_blend_shapes_export.glb");
        export_glb_blend_shapes(&mesh, &shapes, path).expect("export failed");
        assert!(path.exists(), "GLB file was not created");
        let meta = std::fs::metadata(path).unwrap();
        assert!(meta.len() > 0, "GLB file is empty");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn glb_blend_shapes_valid_header() {
        let mesh = triangle_mesh();
        let shapes = vec![BlendShape::new("blink", vec![[0.0, 0.05, 0.0]; 3])];
        let path = std::path::Path::new("/tmp/test_blend_shapes_header.glb");
        export_glb_blend_shapes(&mesh, &shapes, path).expect("export failed");

        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() >= 4, "file too short");
        assert_eq!(
            &bytes[0..4],
            &[0x67, 0x6C, 0x54, 0x46],
            "wrong GLB magic bytes"
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn single_zero_shape_export() {
        let mesh = triangle_mesh();
        let shapes = vec![BlendShape::zero("rest", 3)];
        let path = std::path::Path::new("/tmp/test_blend_shapes_single_zero.glb");
        export_glb_blend_shapes(&mesh, &shapes, path).expect("single zero shape export failed");
        assert!(path.exists());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn empty_shapes_export() {
        let mesh = triangle_mesh();
        let path = std::path::Path::new("/tmp/test_blend_shapes_empty.glb");
        export_glb_blend_shapes(&mesh, &[], path).expect("empty shapes export failed");
        assert!(path.exists());
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() >= 12, "GLB too short");
        std::fs::remove_file(path).ok();
    }
}
