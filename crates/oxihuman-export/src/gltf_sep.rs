// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GLTF 2.0 separated export: produces .gltf JSON + .bin binary file.
//!
//! Unlike GLB (embedded), this format keeps JSON and geometry separate.
//! Useful for web tooling where the .bin can be cached independently.

use anyhow::{bail, Result};
use bytemuck::cast_slice;
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::suit::ensure_suit_mesh;
use serde_json::json;
use std::path::Path;

/// Export a mesh as separated GLTF 2.0 files.
///
/// Creates two files:
/// - `gltf_path` (.gltf): JSON manifest referencing the .bin
/// - `bin_path` (.bin): raw binary vertex/index data
///
/// Returns Err if `mesh.has_suit` is false (safety check).
/// If `mesh.colors` is Some, a COLOR_0 accessor is included in the output.
pub fn export_gltf_sep(mesh: &MeshBuffers, gltf_path: &Path, bin_path: &Path) -> Result<()> {
    ensure_suit_mesh(mesh)?;

    // Validate extensions
    if gltf_path.extension().and_then(|e| e.to_str()) != Some("gltf") {
        bail!("gltf_path must have .gltf extension");
    }
    if bin_path.extension().and_then(|e| e.to_str()) != Some("bin") {
        bail!("bin_path must have .bin extension");
    }

    let n_verts = mesh.positions.len();
    let n_idx = mesh.indices.len();

    // ── Build BIN data ───────────────────────────────────────────────────────
    let pos_bytes: &[u8] = cast_slice(&mesh.positions);
    let norm_bytes: &[u8] = cast_slice(&mesh.normals);
    let uv_bytes: &[u8] = cast_slice(&mesh.uvs);
    let idx_bytes: &[u8] = cast_slice(&mesh.indices);

    let pos_offset = 0usize;
    let norm_offset = pos_offset + pos_bytes.len();
    let uv_offset = norm_offset + norm_bytes.len();
    let idx_offset = uv_offset + uv_bytes.len();
    let mut total_bin = idx_offset + idx_bytes.len();

    // Optional color data
    let color_offset;
    let color_bytes_opt: Option<&[u8]> = if let Some(ref cols) = mesh.colors {
        let cb: &[u8] = cast_slice(cols.as_slice());
        color_offset = total_bin;
        total_bin += cb.len();
        Some(cb)
    } else {
        color_offset = 0;
        None
    };

    let mut bin_data: Vec<u8> = Vec::with_capacity(total_bin + 3);
    bin_data.extend_from_slice(pos_bytes);
    bin_data.extend_from_slice(norm_bytes);
    bin_data.extend_from_slice(uv_bytes);
    bin_data.extend_from_slice(idx_bytes);
    if let Some(cb) = color_bytes_opt {
        bin_data.extend_from_slice(cb);
    }
    // Pad to 4-byte alignment
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }

    // Write .bin file
    std::fs::write(bin_path, &bin_data)?;

    // ── Build GLTF JSON ──────────────────────────────────────────────────────
    let bin_filename = bin_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mesh.bin");

    let gltf = if let Some(ref cols) = mesh.colors {
        let color_byte_len = cols.len() * std::mem::size_of::<[f32; 4]>();
        json!({
            "asset": { "version": "2.0", "generator": "OxiHuman 0.1.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "mesh": 0 }],
            "meshes": [{
                "name": "human",
                "primitives": [{
                    "attributes": {
                        "POSITION":   0,
                        "NORMAL":     1,
                        "TEXCOORD_0": 2,
                        "COLOR_0":    4
                    },
                    "indices": 3,
                    "mode": 4
                }]
            }],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": n_verts, "type": "VEC3" },
                { "bufferView": 1, "componentType": 5126, "count": n_verts, "type": "VEC3" },
                { "bufferView": 2, "componentType": 5126, "count": n_verts, "type": "VEC2" },
                { "bufferView": 3, "componentType": 5125, "count": n_idx,   "type": "SCALAR" },
                { "bufferView": 4, "componentType": 5126, "count": n_verts, "type": "VEC4"  }
            ],
            "bufferViews": [
                { "buffer": 0, "byteOffset": pos_offset,   "byteLength": pos_bytes.len()  },
                { "buffer": 0, "byteOffset": norm_offset,  "byteLength": norm_bytes.len() },
                { "buffer": 0, "byteOffset": uv_offset,    "byteLength": uv_bytes.len()   },
                { "buffer": 0, "byteOffset": idx_offset,   "byteLength": idx_bytes.len()  },
                { "buffer": 0, "byteOffset": color_offset, "byteLength": color_byte_len   }
            ],
            "buffers": [{
                "uri": bin_filename,
                "byteLength": bin_data.len()
            }]
        })
    } else {
        json!({
            "asset": { "version": "2.0", "generator": "OxiHuman 0.1.0" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "mesh": 0 }],
            "meshes": [{
                "name": "human",
                "primitives": [{
                    "attributes": {
                        "POSITION":   0,
                        "NORMAL":     1,
                        "TEXCOORD_0": 2
                    },
                    "indices": 3,
                    "mode": 4
                }]
            }],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": n_verts, "type": "VEC3" },
                { "bufferView": 1, "componentType": 5126, "count": n_verts, "type": "VEC3" },
                { "bufferView": 2, "componentType": 5126, "count": n_verts, "type": "VEC2" },
                { "bufferView": 3, "componentType": 5125, "count": n_idx,   "type": "SCALAR" }
            ],
            "bufferViews": [
                { "buffer": 0, "byteOffset": pos_offset,  "byteLength": pos_bytes.len()  },
                { "buffer": 0, "byteOffset": norm_offset, "byteLength": norm_bytes.len() },
                { "buffer": 0, "byteOffset": uv_offset,   "byteLength": uv_bytes.len()   },
                { "buffer": 0, "byteOffset": idx_offset,  "byteLength": idx_bytes.len()  }
            ],
            "buffers": [{
                "uri": bin_filename,
                "byteLength": bin_data.len()
            }]
        })
    };

    let json_str = serde_json::to_string_pretty(&gltf)?;
    std::fs::write(gltf_path, json_str)?;

    Ok(())
}

/// Verify that a .gltf JSON file references a valid .bin and has correct structure.
pub fn verify_gltf_sep(gltf_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(gltf_path)?;
    let val: serde_json::Value = serde_json::from_str(&content)?;
    let version = val["asset"]["version"].as_str().unwrap_or("");
    if version != "2.0" {
        bail!("expected GLTF version 2.0, got '{}'", version);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn suited_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    fn unsuited_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0]],
            normals: vec![[0.0, 1.0, 0.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        })
    }

    #[test]
    fn export_gltf_sep_creates_both_files() {
        let mesh = suited_mesh();
        let gltf = std::path::PathBuf::from("/tmp/test_oxihuman.gltf");
        let bin = std::path::PathBuf::from("/tmp/test_oxihuman.bin");
        export_gltf_sep(&mesh, &gltf, &bin).expect("should succeed");
        assert!(gltf.exists(), ".gltf file should exist");
        assert!(bin.exists(), ".bin file should exist");
        verify_gltf_sep(&gltf).expect("should succeed");
        std::fs::remove_file(&gltf).ok();
        std::fs::remove_file(&bin).ok();
    }

    #[test]
    fn gltf_json_references_bin_filename() {
        let mesh = suited_mesh();
        let gltf = std::path::PathBuf::from("/tmp/ref_test.gltf");
        let bin = std::path::PathBuf::from("/tmp/ref_test.bin");
        export_gltf_sep(&mesh, &gltf, &bin).expect("should succeed");
        let content = std::fs::read_to_string(&gltf).expect("should succeed");
        assert!(
            content.contains("ref_test.bin"),
            "gltf should reference bin filename"
        );
        std::fs::remove_file(&gltf).ok();
        std::fs::remove_file(&bin).ok();
    }

    #[test]
    fn bin_size_matches_vertex_data() {
        let mesh = suited_mesh();
        let gltf = std::path::PathBuf::from("/tmp/size_test.gltf");
        let bin = std::path::PathBuf::from("/tmp/size_test.bin");
        export_gltf_sep(&mesh, &gltf, &bin).expect("should succeed");
        let bin_size = std::fs::metadata(&bin).expect("should succeed").len() as usize;
        // 3 verts: pos(12) + norm(12) + uv(8) = 32 bytes/vert × 3 = 96 + idx(4×3=12) = 108, padded to 112
        let expected = (3 * (12 + 12 + 8) + 3 * 4 + 3) & !3; // pad to 4
        assert_eq!(
            bin_size, expected,
            "bin size {} != expected {}",
            bin_size, expected
        );
        std::fs::remove_file(&gltf).ok();
        std::fs::remove_file(&bin).ok();
    }

    #[test]
    fn export_refuses_unsuited_mesh() {
        let mesh = unsuited_mesh();
        let gltf = std::path::PathBuf::from("/tmp/bad.gltf");
        let bin = std::path::PathBuf::from("/tmp/bad.bin");
        assert!(export_gltf_sep(&mesh, &gltf, &bin).is_err());
    }

    #[test]
    fn wrong_extension_errors() {
        let mesh = suited_mesh();
        let result = export_gltf_sep(
            &mesh,
            &std::path::PathBuf::from("/tmp/bad.json"),
            &std::path::PathBuf::from("/tmp/good.bin"),
        );
        assert!(result.is_err());
    }
}
