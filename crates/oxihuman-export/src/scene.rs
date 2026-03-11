// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::Write;
use std::path::Path;

use anyhow::Result;
use bytemuck::cast_slice;
use oxihuman_mesh::MeshBuffers;
use serde_json::json;

use crate::material::PbrMaterial;

// GLB magic constants (same as glb.rs)
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

/// A named mesh entry in a multi-mesh scene.
pub struct SceneMesh {
    /// Display name for this mesh in the scene hierarchy.
    pub name: String,
    pub mesh: MeshBuffers,
    pub material: Option<PbrMaterial>,
    /// Translation offset [x, y, z] from scene origin.
    pub translation: [f32; 3],
}

impl SceneMesh {
    pub fn new(name: impl Into<String>, mesh: MeshBuffers) -> Self {
        Self {
            name: name.into(),
            mesh,
            material: None,
            translation: [0.0, 0.0, 0.0],
        }
    }

    pub fn with_material(mut self, m: PbrMaterial) -> Self {
        self.material = Some(m);
        self
    }

    pub fn with_translation(mut self, t: [f32; 3]) -> Self {
        self.translation = t;
        self
    }
}

/// A scene containing multiple meshes.
pub struct Scene {
    pub name: String,
    pub meshes: Vec<SceneMesh>,
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            meshes: Vec::new(),
        }
    }

    pub fn add_mesh(mut self, mesh: SceneMesh) -> Self {
        self.meshes.push(mesh);
        self
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Convenience method: export scene as GLB.
    pub fn export(&self, path: &Path) -> Result<()> {
        export_scene_glb(self, path)
    }
}

/// Per-mesh BIN layout info computed during BIN construction.
struct MeshBinLayout {
    pos_offset: usize,
    norm_offset: usize,
    uv_offset: usize,
    idx_offset: usize,
    n_verts: usize,
    n_idx: usize,
    pos_bytes_len: usize,
    norm_bytes_len: usize,
    uv_bytes_len: usize,
    idx_bytes_len: usize,
}

/// Export a multi-mesh scene as a single GLB file.
/// Each mesh becomes a separate GLTF mesh node under the scene root.
/// Materials are per-mesh (if provided).
pub fn export_scene_glb(scene: &Scene, path: &Path) -> Result<()> {
    // ── 1. Build BIN chunk by concatenating all mesh data ───────────────────
    // For each mesh: POSITION, NORMAL, TEXCOORD_0, indices (u32)
    let mut bin_data: Vec<u8> = Vec::new();
    let mut layouts: Vec<MeshBinLayout> = Vec::new();

    for sm in &scene.meshes {
        let mesh = &sm.mesh;
        let pos_bytes: &[u8] = cast_slice(&mesh.positions);
        let norm_bytes: &[u8] = cast_slice(&mesh.normals);
        let uv_bytes: &[u8] = cast_slice(&mesh.uvs);
        let idx_bytes: &[u8] = cast_slice(&mesh.indices);

        let pos_offset = bin_data.len();
        bin_data.extend_from_slice(pos_bytes);
        let norm_offset = bin_data.len();
        bin_data.extend_from_slice(norm_bytes);
        let uv_offset = bin_data.len();
        bin_data.extend_from_slice(uv_bytes);
        let idx_offset = bin_data.len();
        bin_data.extend_from_slice(idx_bytes);

        layouts.push(MeshBinLayout {
            pos_offset,
            norm_offset,
            uv_offset,
            idx_offset,
            n_verts: mesh.positions.len(),
            n_idx: mesh.indices.len(),
            pos_bytes_len: pos_bytes.len(),
            norm_bytes_len: norm_bytes.len(),
            uv_bytes_len: uv_bytes.len(),
            idx_bytes_len: idx_bytes.len(),
        });
    }

    // Pad BIN to 4-byte boundary
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }

    // ── 2. Build materials array + mesh→material index mapping ──────────────
    let mut materials_json: Vec<serde_json::Value> = Vec::new();
    // mesh_material_idx[i] = Some(mat_idx) if scene.meshes[i] has a material
    let mut mesh_material_idx: Vec<Option<usize>> = Vec::new();

    for sm in &scene.meshes {
        if let Some(ref mat) = sm.material {
            let mat_idx = materials_json.len();
            materials_json.push(mat.to_gltf_json());
            mesh_material_idx.push(Some(mat_idx));
        } else {
            mesh_material_idx.push(None);
        }
    }

    // ── 3. Build accessors + bufferViews + meshes + nodes ───────────────────
    let mut accessors: Vec<serde_json::Value> = Vec::new();
    let mut buffer_views: Vec<serde_json::Value> = Vec::new();
    let mut meshes_json: Vec<serde_json::Value> = Vec::new();
    let mut nodes_json: Vec<serde_json::Value> = Vec::new();

    for (mesh_idx, (sm, layout)) in scene.meshes.iter().zip(layouts.iter()).enumerate() {
        // Each mesh gets 4 accessors: POSITION(VEC3), NORMAL(VEC3), TEXCOORD_0(VEC2), indices(SCALAR)
        // and 4 bufferViews corresponding to them.

        let pos_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.pos_offset,
            "byteLength": layout.pos_bytes_len
        }));

        let norm_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.norm_offset,
            "byteLength": layout.norm_bytes_len
        }));

        let uv_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.uv_offset,
            "byteLength": layout.uv_bytes_len
        }));

        let idx_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.idx_offset,
            "byteLength": layout.idx_bytes_len
        }));

        let pos_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": pos_bv_idx,
            "componentType": 5126,
            "count": layout.n_verts,
            "type": "VEC3"
        }));

        let norm_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": norm_bv_idx,
            "componentType": 5126,
            "count": layout.n_verts,
            "type": "VEC3"
        }));

        let uv_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": uv_bv_idx,
            "componentType": 5126,
            "count": layout.n_verts,
            "type": "VEC2"
        }));

        let idx_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": idx_bv_idx,
            "componentType": 5125,
            "count": layout.n_idx,
            "type": "SCALAR"
        }));

        // Build primitive
        let attributes = json!({
            "POSITION":   pos_acc_idx,
            "NORMAL":     norm_acc_idx,
            "TEXCOORD_0": uv_acc_idx
        });

        let primitive = if let Some(mat_idx) = mesh_material_idx[mesh_idx] {
            json!({
                "attributes": attributes,
                "indices": idx_acc_idx,
                "material": mat_idx
            })
        } else {
            json!({
                "attributes": attributes,
                "indices": idx_acc_idx
            })
        };

        meshes_json.push(json!({
            "name": sm.name,
            "primitives": [primitive]
        }));

        // Build node
        let t = sm.translation;
        nodes_json.push(json!({
            "name": sm.name,
            "mesh": mesh_idx,
            "translation": [t[0], t[1], t[2]]
        }));
    }

    // Node indices for the scene: [0, 1, 2, ...]
    let node_indices: Vec<usize> = (0..scene.meshes.len()).collect();

    // ── 4. Build GLTF JSON ───────────────────────────────────────────────────
    let total_bin = bin_data.len() as u32;

    let gltf = if materials_json.is_empty() {
        json!({
            "asset": { "version": "2.0", "generator": "oxihuman-export" },
            "scene": 0,
            "scenes": [{ "name": scene.name, "nodes": node_indices }],
            "nodes": nodes_json,
            "meshes": meshes_json,
            "accessors": accessors,
            "bufferViews": buffer_views,
            "buffers": [{ "byteLength": total_bin }]
        })
    } else {
        json!({
            "asset": { "version": "2.0", "generator": "oxihuman-export" },
            "scene": 0,
            "scenes": [{ "name": scene.name, "nodes": node_indices }],
            "nodes": nodes_json,
            "meshes": meshes_json,
            "materials": materials_json,
            "accessors": accessors,
            "bufferViews": buffer_views,
            "buffers": [{ "byteLength": total_bin }]
        })
    };

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    // Pad JSON to 4-byte boundary with spaces
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }

    // ── 5. Write GLB ─────────────────────────────────────────────────────────
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
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn tri_mesh(y_offset: f32) -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, y_offset, 0.0],
                [1.0, y_offset, 0.0],
                [0.0, y_offset + 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    #[test]
    fn scene_empty_export() {
        let path = std::path::PathBuf::from("/tmp/test_scene_empty.glb");
        let scene = Scene::new("test");
        assert_eq!(scene.mesh_count(), 0);
        scene.export(&path).expect("export failed");
        assert!(path.exists(), "GLB file should be created");
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() >= 12, "GLB must have at least 12 bytes");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn scene_single_mesh_export() {
        let path = std::path::PathBuf::from("/tmp/test_scene_single.glb");
        let scene = Scene::new("single").add_mesh(SceneMesh::new("body", tri_mesh(0.0)));
        export_scene_glb(&scene, &path).expect("export failed");
        assert!(path.exists(), "file should exist");
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() >= 12, "valid GLB header required");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn scene_two_meshes_export() {
        let path = std::path::PathBuf::from("/tmp/test_scene_two.glb");
        let scene = Scene::new("two_meshes")
            .add_mesh(SceneMesh::new("body", tri_mesh(0.0)))
            .add_mesh(SceneMesh::new("clothing", tri_mesh(1.0)));
        export_scene_glb(&scene, &path).expect("export failed");
        assert!(path.exists(), "file should exist");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn scene_glb_header_valid() {
        let path = std::path::PathBuf::from("/tmp/test_scene_header.glb");
        let scene = Scene::new("header_test").add_mesh(SceneMesh::new("mesh0", tri_mesh(0.0)));
        export_scene_glb(&scene, &path).expect("export failed");
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() >= 4);
        // Magic: "glTF" = 0x46546C67 in LE = [0x67, 0x6C, 0x54, 0x46]
        assert_eq!(
            &bytes[0..4],
            &[0x67u8, 0x6Cu8, 0x54u8, 0x46u8],
            "GLB magic must be glTF"
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn scene_with_material() {
        let path = std::path::PathBuf::from("/tmp/test_scene_material.glb");
        let scene = Scene::new("with_material")
            .add_mesh(SceneMesh::new("body", tri_mesh(0.0)).with_material(PbrMaterial::skin()));
        export_scene_glb(&scene, &path).expect("export failed");
        assert!(path.exists(), "file should exist");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn scene_mesh_count() {
        let scene = Scene::new("x")
            .add_mesh(SceneMesh::new("a", tri_mesh(0.0)))
            .add_mesh(SceneMesh::new("b", tri_mesh(1.0)));
        assert_eq!(scene.mesh_count(), 2);
    }
}
