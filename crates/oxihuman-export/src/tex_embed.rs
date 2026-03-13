// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GLB texture embedding: embed a texture into a GLB file alongside the mesh,
//! wired to the material's `baseColorTexture`.

use std::io::Write;
use std::path::Path;

use bytemuck::cast_slice;
use oxihuman_mesh::MeshBuffers;
use serde_json::json;

use crate::material::PbrMaterial;
use crate::texture::PixelBuffer;

// GLB magic constants (same as glb.rs)
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

/// An embedded texture.
pub struct EmbeddedTexture {
    pub name: String,
    /// Raw or encoded bytes (row-major RGBA, TGA, etc.)
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// MIME type hint, e.g. "image/x-raw-rgba" or "image/x-tga".
    pub mime_type: String,
}

impl EmbeddedTexture {
    /// Create from a raw RGBA `PixelBuffer` (no encoding).
    pub fn from_pixel_buffer(name: impl Into<String>, buf: &PixelBuffer) -> Self {
        Self {
            name: name.into(),
            pixels: buf.pixels.clone(),
            width: buf.width,
            height: buf.height,
            mime_type: "image/x-raw-rgba".into(),
        }
    }

    /// Create from a `PixelBuffer`, encoding the pixels to TGA bytes.
    pub fn from_tga(name: impl Into<String>, buf: &PixelBuffer) -> Self {
        Self {
            name: name.into(),
            pixels: buf.to_tga_bytes(),
            width: buf.width,
            height: buf.height,
            mime_type: "image/x-tga".into(),
        }
    }

    /// Total byte length of the texture payload.
    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }
}

/// Export a GLB 2.0 file with an embedded texture wired to `baseColorTexture`.
///
/// BIN layout: `[POSITION][NORMAL][TEXCOORD_0][indices][texture_bytes]`
///
/// The GLTF JSON includes:
/// - A material with `pbrMetallicRoughness.baseColorTexture` pointing at the texture.
/// - A texture, image, and sampler entry.
/// - A bufferView for the texture bytes appended after the mesh bufferViews.
pub fn export_glb_with_texture(
    mesh: &MeshBuffers,
    material: &PbrMaterial,
    texture: &EmbeddedTexture,
    path: &Path,
) -> anyhow::Result<()> {
    // ── 1. Build BIN chunk ────────────────────────────────────────────────────
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
    let mesh_bin_len = idx_offset + idx_bytes.len();

    // Texture bytes appended right after the mesh binary data.
    let tex_offset = mesh_bin_len;

    let mut bin_data: Vec<u8> = Vec::with_capacity(mesh_bin_len + texture.byte_len() + 3);
    bin_data.extend_from_slice(pos_bytes);
    bin_data.extend_from_slice(norm_bytes);
    bin_data.extend_from_slice(uv_bytes);
    bin_data.extend_from_slice(idx_bytes);
    bin_data.extend_from_slice(&texture.pixels);

    // Pad BIN to 4-byte boundary.
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }
    let total_bin = bin_data.len() as u32;

    // ── 2. Build GLTF JSON ────────────────────────────────────────────────────
    // bufferViews: 0=POSITION, 1=NORMAL, 2=TEXCOORD_0, 3=indices, 4=texture
    // accessors:   0=POSITION, 1=NORMAL, 2=TEXCOORD_0, 3=indices
    let accessors: Vec<serde_json::Value> = vec![
        json!({ "bufferView": 0, "componentType": 5126, "count": n_verts, "type": "VEC3" }),
        json!({ "bufferView": 1, "componentType": 5126, "count": n_verts, "type": "VEC3" }),
        json!({ "bufferView": 2, "componentType": 5126, "count": n_verts, "type": "VEC2" }),
        json!({ "bufferView": 3, "componentType": 5125, "count": n_idx,   "type": "SCALAR" }),
    ];

    let buffer_views: Vec<serde_json::Value> = vec![
        json!({ "buffer": 0, "byteOffset": pos_offset,  "byteLength": pos_bytes.len()  }),
        json!({ "buffer": 0, "byteOffset": norm_offset, "byteLength": norm_bytes.len() }),
        json!({ "buffer": 0, "byteOffset": uv_offset,   "byteLength": uv_bytes.len()   }),
        json!({ "buffer": 0, "byteOffset": idx_offset,  "byteLength": idx_bytes.len()  }),
        // bufferView index 4: texture data
        json!({ "buffer": 0, "byteOffset": tex_offset,  "byteLength": texture.byte_len() }),
    ];

    let gltf = json!({
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
                "mode": 4,
                "material": 0
            }]
        }],
        "materials": [{
            "name": material.name,
            "pbrMetallicRoughness": {
                "baseColorFactor": [
                    material.base_color[0],
                    material.base_color[1],
                    material.base_color[2],
                    material.base_color[3]
                ],
                "baseColorTexture": { "index": 0 },
                "metallicFactor": material.metallic,
                "roughnessFactor": material.roughness
            },
            "emissiveFactor": [
                material.emissive[0],
                material.emissive[1],
                material.emissive[2]
            ],
            "doubleSided": material.double_sided,
            "alphaMode": material.alpha_mode
        }],
        "textures": [{ "source": 0, "sampler": 0 }],
        "images": [{
            "name": texture.name,
            "bufferView": 4,
            "mimeType": texture.mime_type
        }],
        "samplers": [{
            "magFilter": 9729,
            "minFilter": 9987,
            "wrapS": 10497,
            "wrapT": 10497
        }],
        "accessors":   accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": total_bin }]
    });

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    // Pad JSON to 4-byte boundary with spaces.
    while json_bytes.len() % 4 != 0 {
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
    use crate::glb::export_glb;
    use crate::material::PbrMaterial;
    use crate::texture::generate_skin_texture;

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

    fn small_tex() -> EmbeddedTexture {
        let buf = generate_skin_texture(4, 4, 200, 160, 140);
        EmbeddedTexture::from_tga("skin", &buf)
    }

    #[test]
    fn embedded_texture_byte_len_positive() {
        assert!(small_tex().byte_len() > 0);
    }

    #[test]
    fn from_pixel_buffer_dimensions() {
        let buf = generate_skin_texture(8, 6, 180, 140, 120);
        let tex = EmbeddedTexture::from_pixel_buffer("t", &buf);
        assert_eq!(tex.width, buf.width);
        assert_eq!(tex.height, buf.height);
    }

    #[test]
    fn export_glb_with_texture_creates_file() {
        let mesh = tri_mesh();
        let mat = PbrMaterial::skin();
        let tex = small_tex();
        let path = std::path::PathBuf::from("/tmp/test_tex_embed_create.glb");
        export_glb_with_texture(&mesh, &mat, &tex, &path).expect("export failed");
        assert!(path.exists(), "GLB file was not created");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn glb_with_texture_valid_header() {
        let mesh = tri_mesh();
        let mat = PbrMaterial::skin();
        let tex = small_tex();
        let path = std::path::PathBuf::from("/tmp/test_tex_embed_header.glb");
        export_glb_with_texture(&mesh, &mat, &tex, &path).expect("export failed");
        let bytes = std::fs::read(&path).expect("should succeed");
        assert!(bytes.len() >= 4, "file too short to contain GLB magic");
        let magic = u32::from_le_bytes(bytes[0..4].try_into().expect("should succeed"));
        assert_eq!(magic, 0x46546C67u32, "first 4 bytes are not GLB magic");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn glb_with_texture_larger_than_without() {
        let mesh = tri_mesh();
        let mat = PbrMaterial::skin();
        let tex = small_tex();

        let path_plain = std::path::PathBuf::from("/tmp/test_tex_embed_plain.glb");
        let path_tex = std::path::PathBuf::from("/tmp/test_tex_embed_with_tex.glb");

        // Export plain GLB (no texture) using the existing export_glb function.
        export_glb(&mesh, &path_plain).expect("plain export failed");
        export_glb_with_texture(&mesh, &mat, &tex, &path_tex).expect("texture export failed");

        let size_plain = std::fs::metadata(&path_plain).expect("should succeed").len();
        let size_tex = std::fs::metadata(&path_tex).expect("should succeed").len();

        assert!(
            size_tex > size_plain,
            "GLB with texture ({} bytes) should be larger than plain ({} bytes)",
            size_tex,
            size_plain
        );

        std::fs::remove_file(&path_plain).ok();
        std::fs::remove_file(&path_tex).ok();
    }
}
