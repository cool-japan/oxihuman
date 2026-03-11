// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use bytemuck::cast_slice;
use oxihuman_mesh::MeshBuffers;
use serde_json::json;
use std::f32::consts::PI;
use std::io::Write;
use std::path::Path;

// GLB magic constants (same as glb.rs)
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

/// A transform for a single instance: translation, rotation (quaternion), scale.
#[derive(Debug, Clone)]
pub struct InstanceTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // quaternion [x, y, z, w]
    pub scale: [f32; 3],
}

impl InstanceTransform {
    pub fn identity() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    pub fn at(translation: [f32; 3]) -> Self {
        Self {
            translation,
            ..Self::identity()
        }
    }

    pub fn scaled(mut self, s: f32) -> Self {
        self.scale = [s, s, s];
        self
    }
}

/// Export a mesh with N instances as a GLB.
/// Each instance is a separate GLTF node sharing mesh index 0.
/// `instances`: list of transforms for each instance.
pub fn export_instanced_glb(
    mesh: &MeshBuffers,
    instances: &[InstanceTransform],
    path: &Path,
) -> Result<()> {
    // ── 1. Build BIN chunk data ──────────────────────────────────────────────
    // Layout: [positions f32*3*n] [normals f32*3*n] [uvs f32*2*n] [indices u32*m]
    let n_verts = mesh.positions.len();
    let n_idx = mesh.indices.len();

    let pos_bytes: &[u8] = cast_slice(&mesh.positions);
    let norm_bytes: &[u8] = cast_slice(&mesh.normals);
    let uv_bytes: &[u8] = cast_slice(&mesh.uvs);
    let idx_bytes: &[u8] = cast_slice(&mesh.indices);

    let pos_offset = 0usize;
    let norm_offset = pos_offset + pos_bytes.len();
    let uv_offset = norm_offset + norm_bytes.len();
    let idx_offset = uv_offset + uv_bytes.len();
    let bin_len = idx_offset + idx_bytes.len();

    let mut bin_data: Vec<u8> = Vec::with_capacity(bin_len + 3);
    bin_data.extend_from_slice(pos_bytes);
    bin_data.extend_from_slice(norm_bytes);
    bin_data.extend_from_slice(uv_bytes);
    bin_data.extend_from_slice(idx_bytes);
    // Pad BIN to 4-byte boundary
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }

    // ── 2. Build GLTF JSON ───────────────────────────────────────────────────
    let total_bin = bin_data.len() as u32;

    let accessors = vec![
        json!({ "bufferView": 0, "componentType": 5126, "count": n_verts, "type": "VEC3" }),
        json!({ "bufferView": 1, "componentType": 5126, "count": n_verts, "type": "VEC3" }),
        json!({ "bufferView": 2, "componentType": 5126, "count": n_verts, "type": "VEC2" }),
        json!({ "bufferView": 3, "componentType": 5125, "count": n_idx,   "type": "SCALAR" }),
    ];

    let buffer_views = vec![
        json!({ "buffer": 0, "byteOffset": pos_offset,  "byteLength": pos_bytes.len()  }),
        json!({ "buffer": 0, "byteOffset": norm_offset, "byteLength": norm_bytes.len() }),
        json!({ "buffer": 0, "byteOffset": uv_offset,   "byteLength": uv_bytes.len()   }),
        json!({ "buffer": 0, "byteOffset": idx_offset,  "byteLength": idx_bytes.len()  }),
    ];

    // Build nodes: one per instance, each references mesh 0
    let nodes: Vec<serde_json::Value> = instances
        .iter()
        .map(|inst| {
            json!({
                "mesh": 0,
                "translation": inst.translation,
                "rotation": inst.rotation,
                "scale": inst.scale
            })
        })
        .collect();

    // Scene lists all node indices 0..N
    let node_indices: Vec<usize> = (0..instances.len()).collect();

    let gltf = json!({
        "asset": { "version": "2.0", "generator": "OxiHuman 0.1.0" },
        "scene": 0,
        "scenes": [{ "nodes": node_indices }],
        "nodes": nodes,
        "meshes": [{
            "name": "instanced_mesh",
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
        "accessors":   accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": total_bin }]
    });

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    // Pad JSON to 4-byte boundary with spaces
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    // ── 3. Write GLB ─────────────────────────────────────────────────────────
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

/// Build a row of N instances spaced `spacing` apart along the X axis.
pub fn row_instances(count: usize, spacing: f32) -> Vec<InstanceTransform> {
    (0..count)
        .map(|i| InstanceTransform::at([i as f32 * spacing, 0.0, 0.0]))
        .collect()
}

/// Build a grid of instances: `rows` x `cols`, spaced `spacing` apart in XZ plane.
pub fn grid_instances(rows: usize, cols: usize, spacing: f32) -> Vec<InstanceTransform> {
    let mut out = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            out.push(InstanceTransform::at([
                c as f32 * spacing,
                0.0,
                r as f32 * spacing,
            ]));
        }
    }
    out
}

/// Build instances in a circle of radius `r`, evenly distributed.
pub fn circle_instances(count: usize, radius: f32) -> Vec<InstanceTransform> {
    (0..count)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / count as f32;
            InstanceTransform::at([radius * angle.cos(), 0.0, radius * angle.sin()])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::path::PathBuf;

    fn tri_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: true,
        }
    }

    fn tmp_path(name: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/{}", name))
    }

    #[test]
    fn row_instances_count() {
        assert_eq!(row_instances(5, 2.0).len(), 5);
    }

    #[test]
    fn grid_instances_count() {
        assert_eq!(grid_instances(3, 4, 1.0).len(), 12);
    }

    #[test]
    fn circle_instances_count() {
        assert_eq!(circle_instances(8, 2.0).len(), 8);
    }

    #[test]
    fn export_single_instance() {
        let mesh = tri_mesh();
        let path = tmp_path("test_instancing_single.glb");
        export_instanced_glb(&mesh, &[InstanceTransform::identity()], &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn export_five_instances() {
        let mesh = tri_mesh();
        let path = tmp_path("test_instancing_five.glb");
        let instances = row_instances(5, 1.5);
        export_instanced_glb(&mesh, &instances, &path).unwrap();
        assert!(path.exists());
        // Verify file has content (valid GLB has at least 12 byte header)
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 12);
    }

    #[test]
    fn glb_header_valid() {
        let mesh = tri_mesh();
        let path = tmp_path("test_instancing_header.glb");
        export_instanced_glb(&mesh, &[InstanceTransform::identity()], &path).unwrap();
        let mut f = std::fs::File::open(&path).unwrap();
        let mut buf = [0u8; 4];
        f.read_exact(&mut buf).unwrap();
        // "glTF" magic: 0x46546C67 in little-endian = [0x67, 0x6C, 0x54, 0x46]
        assert_eq!(buf, [0x67, 0x6C, 0x54, 0x46]);
    }

    #[test]
    fn identity_transform_fields() {
        let t = InstanceTransform::identity();
        assert_eq!(t.rotation[3], 1.0);
    }
}
