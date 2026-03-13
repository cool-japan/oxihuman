// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use bytemuck::cast_slice;
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::suit::ensure_suit_mesh;
use serde_json::json;
use std::io::Write;
use std::path::Path;

// GLB magic constants
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

/// Export a MeshBuffers to a GLB 2.0 file.
/// Returns Err if the mesh has no suit applied (safety check).
/// If `mesh.colors` is Some, a COLOR_0 accessor is included in the output.
/// If `mesh.tangents.len() == mesh.positions.len()`, a TANGENT accessor is included.
pub fn export_glb(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    ensure_suit_mesh(mesh)?;

    // ── 1. Build BIN chunk data ──────────────────────────────────────────────
    // Layout: [positions f32*3*n] [normals f32*3*n] [uvs f32*2*n] [indices u32*m]
    //         [colors f32*4*n (optional)] [tangents f32*4*n (optional)]
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
    let mut bin_len = idx_offset + idx_bytes.len();

    // Optional colors
    let has_color = mesh.colors.is_some();
    let color_offset;
    let color_bytes_opt: Option<&[u8]> = if let Some(ref cols) = mesh.colors {
        let cb: &[u8] = cast_slice(cols.as_slice());
        color_offset = bin_len;
        bin_len += cb.len();
        Some(cb)
    } else {
        color_offset = 0;
        None
    };

    // Optional tangents (only if count matches vertex count)
    let has_tangents = n_verts > 0 && mesh.tangents.len() == n_verts;
    let tangent_offset;
    let tangent_bytes_opt: Option<&[u8]> = if has_tangents {
        let tb: &[u8] = cast_slice(mesh.tangents.as_slice());
        tangent_offset = bin_len;
        bin_len += tb.len();
        Some(tb)
    } else {
        tangent_offset = 0;
        None
    };

    let mut bin_data: Vec<u8> = Vec::with_capacity(bin_len + 3);
    bin_data.extend_from_slice(pos_bytes);
    bin_data.extend_from_slice(norm_bytes);
    bin_data.extend_from_slice(uv_bytes);
    bin_data.extend_from_slice(idx_bytes);
    if let Some(cb) = color_bytes_opt {
        bin_data.extend_from_slice(cb);
    }
    if let Some(tb) = tangent_bytes_opt {
        bin_data.extend_from_slice(tb);
    }
    // Pad BIN to 4-byte boundary
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }

    // ── 2. Build GLTF JSON ───────────────────────────────────────────────────
    // Accessor and bufferView indices are assigned dynamically:
    //   0: POSITION (VEC3)
    //   1: NORMAL   (VEC3)
    //   2: TEXCOORD_0 (VEC2)
    //   3: indices  (SCALAR)
    //   4: COLOR_0  (VEC4) [if has_color]
    //   4 or 5: TANGENT (VEC4) [if has_tangents]
    let total_bin = bin_data.len() as u32;

    let mut accessors: Vec<serde_json::Value> = vec![
        json!({ "bufferView": 0, "componentType": 5126, "count": n_verts, "type": "VEC3" }),
        json!({ "bufferView": 1, "componentType": 5126, "count": n_verts, "type": "VEC3" }),
        json!({ "bufferView": 2, "componentType": 5126, "count": n_verts, "type": "VEC2" }),
        json!({ "bufferView": 3, "componentType": 5125, "count": n_idx,   "type": "SCALAR" }),
    ];
    let mut buffer_views: Vec<serde_json::Value> = vec![
        json!({ "buffer": 0, "byteOffset": pos_offset,  "byteLength": pos_bytes.len()  }),
        json!({ "buffer": 0, "byteOffset": norm_offset, "byteLength": norm_bytes.len() }),
        json!({ "buffer": 0, "byteOffset": uv_offset,   "byteLength": uv_bytes.len()   }),
        json!({ "buffer": 0, "byteOffset": idx_offset,  "byteLength": idx_bytes.len()  }),
    ];

    let mut attributes = json!({
        "POSITION":   0,
        "NORMAL":     1,
        "TEXCOORD_0": 2
    });

    if has_color {
        let color_byte_len =
            mesh.colors.as_ref().map_or(0, |c| c.len()) * std::mem::size_of::<[f32; 4]>();
        let color_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": buffer_views.len(),
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC4"
        }));
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": color_offset,
            "byteLength": color_byte_len
        }));
        attributes["COLOR_0"] = json!(color_acc_idx);
    }

    if has_tangents {
        let tangent_byte_len = mesh.tangents.len() * std::mem::size_of::<[f32; 4]>();
        let tangent_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": buffer_views.len(),
            "componentType": 5126,
            "count": n_verts,
            "type": "VEC4"
        }));
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": tangent_offset,
            "byteLength": tangent_byte_len
        }));
        attributes["TANGENT"] = json!(tangent_acc_idx);
    }

    let gltf = json!({
        "asset": { "version": "2.0", "generator": "OxiHuman 0.1.0" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{
            "name": "human",
            "primitives": [{
                "attributes": attributes,
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

/// Read the first 12 bytes of a GLB file and verify the magic/version header.
pub fn verify_glb_header(path: &Path) -> Result<()> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut header = [0u8; 12];
    f.read_exact(&mut header)?;
    let magic = u32::from_le_bytes(
        header[0..4]
            .try_into()
            .map_err(|_| anyhow::anyhow!("failed to read GLB magic bytes"))?,
    );
    let version = u32::from_le_bytes(
        header[4..8]
            .try_into()
            .map_err(|_| anyhow::anyhow!("failed to read GLB version bytes"))?,
    );
    if magic != GLB_MAGIC {
        bail!("invalid GLB magic: 0x{:08X}", magic);
    }
    if version != GLB_VERSION {
        bail!("unexpected GLB version: {}", version);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::mesh::MeshBuffers;
    use oxihuman_mesh::normals::compute_tangents;
    use oxihuman_mesh::set_uniform_color;
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
    fn export_glb_creates_valid_file() {
        let mesh = suited_mesh();
        let path = std::path::PathBuf::from("/tmp/test_oxihuman.glb");
        export_glb(&mesh, &path).expect("export failed");
        verify_glb_header(&path).expect("header invalid");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn export_refuses_unsuited_mesh() {
        let mesh = unsuited_mesh();
        let path = std::path::PathBuf::from("/tmp/test_unsuited.glb");
        let result = export_glb(&mesh, &path);
        assert!(result.is_err(), "should refuse unsuited mesh");
    }

    #[test]
    fn glb_header_magic() {
        let mesh = suited_mesh();
        let path = std::path::PathBuf::from("/tmp/test_magic.glb");
        export_glb(&mesh, &path).expect("should succeed");
        // Read raw bytes
        let bytes = std::fs::read(&path).expect("should succeed");
        assert!(bytes.len() >= 12, "GLB too short");
        let magic = u32::from_le_bytes(bytes[0..4].try_into().expect("should succeed"));
        assert_eq!(magic, 0x46546C67u32, "wrong magic");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn glb_with_colors_has_larger_bin() {
        let mesh_no_color = suited_mesh();
        let mut mesh_with_color = suited_mesh();
        set_uniform_color(&mut mesh_with_color, [1.0, 0.0, 0.0, 1.0]);

        let path_no = std::path::PathBuf::from("/tmp/test_no_color.glb");
        let path_col = std::path::PathBuf::from("/tmp/test_with_color.glb");

        export_glb(&mesh_no_color, &path_no).expect("should succeed");
        export_glb(&mesh_with_color, &path_col).expect("should succeed");

        let size_no = std::fs::metadata(&path_no).expect("should succeed").len();
        let size_col = std::fs::metadata(&path_col).expect("should succeed").len();

        assert!(
            size_col > size_no,
            "colored GLB ({} bytes) should be larger than plain ({} bytes)",
            size_col,
            size_no
        );

        std::fs::remove_file(&path_no).ok();
        std::fs::remove_file(&path_col).ok();
    }

    #[test]
    fn glb_with_colors_header_still_valid() {
        let mut mesh = suited_mesh();
        set_uniform_color(&mut mesh, [0.5, 0.5, 0.5, 1.0]);
        let path = std::path::PathBuf::from("/tmp/test_color_header.glb");
        export_glb(&mesh, &path).expect("should succeed");

        let bytes = std::fs::read(&path).expect("should succeed");
        assert!(bytes.len() >= 12, "GLB too short");
        let magic = u32::from_le_bytes(bytes[0..4].try_into().expect("should succeed"));
        assert_eq!(magic, 0x46546C67u32, "wrong GLB magic in colored file");

        verify_glb_header(&path).expect("should succeed");
        std::fs::remove_file(&path).ok();
    }

    // ── Tangent-specific tests ─────────────────────────────────────────────

    #[test]
    fn tangent_glb_has_larger_bin() {
        // Base mesh from from_morph has tangents already (same count as positions),
        // so we need a mesh WITHOUT tangents to compare against.
        // We create a mesh and clear its tangents to simulate "no tangent" export.
        let mut mesh_no_tang = suited_mesh();
        mesh_no_tang.tangents = Vec::new(); // force no-tangent path

        let mut mesh_with_tang = suited_mesh();
        compute_tangents(&mut mesh_with_tang);

        let path_no = std::path::PathBuf::from("/tmp/test_no_tangent.glb");
        let path_tang = std::path::PathBuf::from("/tmp/test_with_tangent.glb");

        export_glb(&mesh_no_tang, &path_no).expect("should succeed");
        export_glb(&mesh_with_tang, &path_tang).expect("should succeed");

        let size_no = std::fs::metadata(&path_no).expect("should succeed").len();
        let size_tang = std::fs::metadata(&path_tang).expect("should succeed").len();

        assert!(
            size_tang > size_no,
            "tangent GLB ({} bytes) should be larger than plain ({} bytes)",
            size_tang,
            size_no
        );

        std::fs::remove_file(&path_no).ok();
        std::fs::remove_file(&path_tang).ok();
    }

    #[test]
    fn glb_tangent_header_valid() {
        let mut mesh = suited_mesh();
        compute_tangents(&mut mesh);

        let path = std::path::PathBuf::from("/tmp/test_tangent_header.glb");
        export_glb(&mesh, &path).expect("should succeed");

        let bytes = std::fs::read(&path).expect("should succeed");
        assert!(bytes.len() >= 12, "GLB too short");
        let magic = u32::from_le_bytes(bytes[0..4].try_into().expect("should succeed"));
        assert_eq!(magic, 0x46546C67u32, "wrong GLB magic after tangent export");

        verify_glb_header(&path).expect("should succeed");
        std::fs::remove_file(&path).ok();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Skinned GLB export
// ─────────────────────────────────────────────────────────────────────────────

use oxihuman_mesh::skeleton::Skeleton;

/// Export a mesh with a skeleton as a skinned GLB 2.0 file.
/// Produces a GLB with a `skins` node containing the joint hierarchy.
/// Vertex skinning data (JOINTS_0, WEIGHTS_0) is not included in this
/// stub — the mesh uses the skeleton purely for the node hierarchy.
pub fn export_glb_with_skeleton(
    mesh: &MeshBuffers,
    skeleton: &Skeleton,
    path: &Path,
) -> anyhow::Result<()> {
    // ── 1. Build BIN chunk (same layout as export_glb, no suit check) ────────
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
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }
    let total_bin = bin_data.len() as u32;

    // ── 2. Build nodes array ─────────────────────────────────────────────────
    // One node per joint, then one mesh node at the end.
    let n_joints = skeleton.joints.len();
    let mesh_node_idx = n_joints; // index of the mesh node in the nodes array

    let mut nodes: Vec<serde_json::Value> = skeleton
        .joints
        .iter()
        .enumerate()
        .map(|(i, joint)| {
            let children: Vec<usize> = skeleton.children_of(i);
            let mut node = serde_json::json!({
                "name":        joint.name,
                "translation": joint.translation,
                "rotation":    joint.rotation,
                "scale":       joint.scale
            });
            if !children.is_empty() {
                node["children"] = serde_json::json!(children);
            }
            node
        })
        .collect();

    // Mesh node — references mesh 0 and skin 0
    nodes.push(serde_json::json!({
        "mesh": 0,
        "skin": 0
    }));

    // scene[0].nodes = root joint indices + mesh node index
    let mut scene_nodes: Vec<usize> = skeleton.roots();
    scene_nodes.push(mesh_node_idx);

    // all joint node indices (0 .. n_joints-1)
    let all_joint_indices: Vec<usize> = (0..n_joints).collect();
    // skeleton root = first root joint (or 0 if no roots somehow)
    let skeleton_root = skeleton.roots().into_iter().next().unwrap_or(0);

    // ── 3. Build GLTF JSON ───────────────────────────────────────────────────
    let gltf = serde_json::json!({
        "asset": { "version": "2.0", "generator": "OxiHuman 0.1.0" },
        "scene": 0,
        "scenes": [{ "nodes": scene_nodes }],
        "nodes": nodes,
        "skins": [{
            "joints":   all_joint_indices,
            "skeleton": skeleton_root
        }],
        "meshes": [{
            "name": "human",
            "primitives": [{
                "attributes": {
                    "POSITION":   0,
                    "NORMAL":     1,
                    "TEXCOORD_0": 2
                },
                "indices": 3,
                "mode":    4
            }]
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC3"
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC3"
            },
            {
                "bufferView": 2,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC2"
            },
            {
                "bufferView": 3,
                "componentType": 5125,
                "count": n_idx,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": pos_offset,  "byteLength": pos_bytes.len()  },
            { "buffer": 0, "byteOffset": norm_offset, "byteLength": norm_bytes.len() },
            { "buffer": 0, "byteOffset": uv_offset,   "byteLength": uv_bytes.len()   },
            { "buffer": 0, "byteOffset": idx_offset,  "byteLength": idx_bytes.len()  }
        ],
        "buffers": [{ "byteLength": total_bin }]
    });

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    // ── 4. Write GLB ─────────────────────────────────────────────────────────
    let json_chunk_len = json_bytes.len() as u32;
    let bin_chunk_len = bin_data.len() as u32;
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut file = std::fs::File::create(path)?;
    file.write_all(&GLB_MAGIC.to_le_bytes())?;
    file.write_all(&GLB_VERSION.to_le_bytes())?;
    file.write_all(&total_len.to_le_bytes())?;
    file.write_all(&json_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_JSON.to_le_bytes())?;
    file.write_all(&json_bytes)?;
    file.write_all(&bin_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_BIN.to_le_bytes())?;
    file.write_all(&bin_data)?;

    Ok(())
}

#[cfg(test)]
mod skeleton_glb_tests {
    use super::*;
    use oxihuman_mesh::skeleton::Skeleton;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn suited_mesh_for_skin() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    #[test]
    fn export_glb_with_skeleton_creates_file() {
        let mesh = suited_mesh_for_skin();
        let skeleton = Skeleton::human_body();
        let path = std::path::Path::new("/tmp/test_skeleton.glb");
        export_glb_with_skeleton(&mesh, &skeleton, path).expect("export_glb_with_skeleton failed");

        assert!(path.exists(), "GLB file was not created");

        // Verify GLB header
        verify_glb_header(path).expect("GLB header invalid");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn skeleton_glb_has_nodes_array() {
        let mesh = suited_mesh_for_skin();
        let skeleton = Skeleton::human_body();
        let path = std::path::Path::new("/tmp/test_skeleton_nodes.glb");
        export_glb_with_skeleton(&mesh, &skeleton, path).expect("export_glb_with_skeleton failed");

        // Read the file and parse the JSON chunk
        use std::io::Read;
        let mut f = std::fs::File::open(path).expect("should succeed");

        // Skip 12-byte GLB header
        let mut buf12 = [0u8; 12];
        f.read_exact(&mut buf12).expect("should succeed");

        // Read JSON chunk header (8 bytes: length + type)
        let mut chunk_hdr = [0u8; 8];
        f.read_exact(&mut chunk_hdr).expect("should succeed");
        let json_len = u32::from_le_bytes(chunk_hdr[0..4].try_into().expect("should succeed")) as usize;

        // Read JSON bytes
        let mut json_buf = vec![0u8; json_len];
        f.read_exact(&mut json_buf).expect("should succeed");

        let json_str = std::str::from_utf8(&json_buf)
            .expect("should succeed")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("should succeed");

        let nodes = parsed["nodes"].as_array().expect("nodes must be an array");
        let expected_len = skeleton.joints.len() + 1; // joints + mesh node
        assert_eq!(
            nodes.len(),
            expected_len,
            "expected {} nodes (joints + mesh node), got {}",
            expected_len,
            nodes.len()
        );

        std::fs::remove_file(path).ok();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Material-aware GLB export
// ─────────────────────────────────────────────────────────────────────────────

use crate::material::PbrMaterial;

/// Export a MeshBuffers to a GLB 2.0 file with an explicit PBR material.
/// The material is embedded in the `materials` array and referenced from the
/// mesh primitive.
#[allow(dead_code)]
pub fn export_glb_with_material(
    mesh: &MeshBuffers,
    material: &PbrMaterial,
    path: &Path,
) -> anyhow::Result<()> {
    ensure_suit_mesh(mesh)?;

    // ── 1. Build BIN chunk ───────────────────────────────────────────────────
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
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }
    let total_bin = bin_data.len() as u32;

    // ── 2. Build GLTF JSON ───────────────────────────────────────────────────
    let material_json = material.to_gltf_json();

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
        "materials": [material_json],
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
        "buffers": [{ "byteLength": total_bin }]
    });

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    // ── 3. Write GLB ─────────────────────────────────────────────────────────
    let json_chunk_len = json_bytes.len() as u32;
    let bin_chunk_len = bin_data.len() as u32;
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut file = std::fs::File::create(path)?;
    file.write_all(&GLB_MAGIC.to_le_bytes())?;
    file.write_all(&GLB_VERSION.to_le_bytes())?;
    file.write_all(&total_len.to_le_bytes())?;
    file.write_all(&json_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_JSON.to_le_bytes())?;
    file.write_all(&json_bytes)?;
    file.write_all(&bin_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_BIN.to_le_bytes())?;
    file.write_all(&bin_data)?;

    Ok(())
}

#[cfg(test)]
mod material_glb_tests {
    use super::*;
    use crate::material::PbrMaterial;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn suited_mesh_for_material() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    #[test]
    fn export_glb_with_material_creates_file() {
        let mesh = suited_mesh_for_material();
        let path = std::path::Path::new("/tmp/test_material.glb");
        export_glb_with_material(&mesh, &PbrMaterial::skin(), path)
            .expect("export_glb_with_material failed");
        assert!(path.exists(), "GLB file was not created");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn glb_with_material_references_material_zero() {
        let mesh = suited_mesh_for_material();
        let path = std::path::Path::new("/tmp/test_material_ref.glb");
        export_glb_with_material(&mesh, &PbrMaterial::skin(), path)
            .expect("export_glb_with_material failed");

        // Parse the JSON chunk from the GLB
        use std::io::Read;
        let mut f = std::fs::File::open(path).expect("should succeed");

        // Skip 12-byte GLB header
        let mut buf12 = [0u8; 12];
        f.read_exact(&mut buf12).expect("should succeed");

        // Read JSON chunk header (8 bytes: length + type)
        let mut chunk_hdr = [0u8; 8];
        f.read_exact(&mut chunk_hdr).expect("should succeed");
        let json_len = u32::from_le_bytes(chunk_hdr[0..4].try_into().expect("should succeed")) as usize;

        // Read JSON bytes
        let mut json_buf = vec![0u8; json_len];
        f.read_exact(&mut json_buf).expect("should succeed");

        let json_str = std::str::from_utf8(&json_buf)
            .expect("should succeed")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("should succeed");

        let material_idx = parsed["meshes"][0]["primitives"][0]["material"]
            .as_u64()
            .expect("material field should be an integer");
        assert_eq!(material_idx, 0, "primitive should reference material 0");

        std::fs::remove_file(path).ok();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Metadata-aware GLB export
// ─────────────────────────────────────────────────────────────────────────────

/// Export a GLB with OxiHuman metadata embedded in `asset.extras`.
///
/// Calls the same BIN/JSON construction as [`export_glb`] and then patches
/// `gltf_json["asset"]["extras"]` with the serialised [`crate::metadata::OxiHumanMeta`].
pub fn export_glb_with_meta(
    mesh: &MeshBuffers,
    meta: &crate::metadata::OxiHumanMeta,
    path: &Path,
) -> anyhow::Result<()> {
    ensure_suit_mesh(mesh)?;

    // ── 1. Build BIN chunk ───────────────────────────────────────────────────
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
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }
    let total_bin = bin_data.len() as u32;

    // ── 2. Build GLTF JSON and patch asset.extras ────────────────────────────
    let mut gltf_json = json!({
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
        "buffers": [{ "byteLength": total_bin }]
    });

    gltf_json["asset"]["extras"] = meta.to_json();

    let mut json_bytes = serde_json::to_vec(&gltf_json)?;
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    // ── 3. Write GLB ─────────────────────────────────────────────────────────
    let json_chunk_len = json_bytes.len() as u32;
    let bin_chunk_len = bin_data.len() as u32;
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut file = std::fs::File::create(path)?;
    file.write_all(&GLB_MAGIC.to_le_bytes())?;
    file.write_all(&GLB_VERSION.to_le_bytes())?;
    file.write_all(&total_len.to_le_bytes())?;
    file.write_all(&json_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_JSON.to_le_bytes())?;
    file.write_all(&json_bytes)?;
    file.write_all(&bin_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_BIN.to_le_bytes())?;
    file.write_all(&bin_data)?;

    Ok(())
}

#[cfg(test)]
mod meta_glb_tests {
    use super::*;
    use crate::metadata::OxiHumanMeta;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn suited_mesh_for_meta() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    #[test]
    fn export_glb_with_meta_creates_file() {
        let mesh = suited_mesh_for_meta();
        let meta = OxiHumanMeta::minimal();
        let path = std::path::Path::new("/tmp/test_meta.glb");
        export_glb_with_meta(&mesh, &meta, path).expect("export_glb_with_meta failed");
        assert!(path.exists(), "GLB file was not created");
        verify_glb_header(path).expect("GLB header invalid");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn glb_with_meta_has_extras_in_json() {
        let mesh = suited_mesh_for_meta();
        let meta = OxiHumanMeta::minimal();
        let path = std::path::Path::new("/tmp/test_meta_extras.glb");
        export_glb_with_meta(&mesh, &meta, path).expect("export_glb_with_meta failed");

        use std::io::Read;
        let mut f = std::fs::File::open(path).expect("should succeed");

        // Skip 12-byte GLB header
        let mut buf12 = [0u8; 12];
        f.read_exact(&mut buf12).expect("should succeed");

        // Read JSON chunk header (8 bytes)
        let mut chunk_hdr = [0u8; 8];
        f.read_exact(&mut chunk_hdr).expect("should succeed");
        let json_len = u32::from_le_bytes(chunk_hdr[0..4].try_into().expect("should succeed")) as usize;

        let mut json_buf = vec![0u8; json_len];
        f.read_exact(&mut json_buf).expect("should succeed");

        let json_str = std::str::from_utf8(&json_buf)
            .expect("should succeed")
            .trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("should succeed");

        let generator = parsed["asset"]["extras"]["generator"]
            .as_str()
            .expect("asset.extras.generator must be a string");
        assert_eq!(generator, "oxihuman-export");

        std::fs::remove_file(path).ok();
    }
}
