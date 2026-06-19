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

fn mat4_identity() -> [f32; 16] {
    let mut m = [0.0f32; 16];
    m[0] = 1.0;
    m[5] = 1.0;
    m[10] = 1.0;
    m[15] = 1.0;
    m
}

fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut r = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut s = 0.0f32;
            for k in 0..4 {
                s += a[k * 4 + row] * b[col * 4 + k];
            }
            r[col * 4 + row] = s;
        }
    }
    r
}

fn quat_to_mat4(q: [f32; 4]) -> [f32; 16] {
    let [x, y, z, w] = q;
    let (xx, yy, zz) = (x * x, y * y, z * z);
    let (xy, xz, yz) = (x * y, x * z, y * z);
    let (wx, wy, wz) = (w * x, w * y, w * z);
    let mut m = mat4_identity();
    m[0] = 1.0 - 2.0 * (yy + zz);
    m[1] = 2.0 * (xy + wz);
    m[2] = 2.0 * (xz - wy);
    m[4] = 2.0 * (xy - wz);
    m[5] = 1.0 - 2.0 * (xx + zz);
    m[6] = 2.0 * (yz + wx);
    m[8] = 2.0 * (xz + wy);
    m[9] = 2.0 * (yz - wx);
    m[10] = 1.0 - 2.0 * (xx + yy);
    m
}

fn compose_trs(t: [f32; 3], r: [f32; 4], s: [f32; 3]) -> [f32; 16] {
    let mut m = quat_to_mat4(r);
    m[0] *= s[0];
    m[1] *= s[0];
    m[2] *= s[0];
    m[4] *= s[1];
    m[5] *= s[1];
    m[6] *= s[1];
    m[8] *= s[2];
    m[9] *= s[2];
    m[10] *= s[2];
    m[12] = t[0];
    m[13] = t[1];
    m[14] = t[2];
    m
}

fn mat4_inverse(m: &[f32; 16]) -> [f32; 16] {
    let mut inv = [0.0f32; 16];
    inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];
    inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];
    inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];
    inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];
    inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];
    inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];
    inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];
    inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];
    inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];
    inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];
    inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];
    inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];
    inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];
    inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];
    inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];
    inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];
    let det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
    if det.abs() < 1e-12 {
        return mat4_identity();
    }
    let inv_det = 1.0 / det;
    for v in inv.iter_mut() {
        *v *= inv_det;
    }
    inv
}

/// Compute world bind transforms for all joints (handles any parent ordering).
fn joint_world_transforms(skeleton: &Skeleton) -> Vec<[f32; 16]> {
    fn resolve(i: usize, sk: &Skeleton, world: &mut Vec<Option<[f32; 16]>>) -> [f32; 16] {
        if let Some(m) = world[i] {
            return m;
        }
        let j = &sk.joints[i];
        let local = compose_trs(j.translation, j.rotation, j.scale);
        let m = match j.parent {
            Some(p) if p < sk.joints.len() && p != i => {
                let pw = resolve(p, sk, world);
                mat4_mul(&pw, &local)
            }
            _ => local,
        };
        world[i] = Some(m);
        m
    }
    let n = skeleton.joints.len();
    let mut world: Vec<Option<[f32; 16]>> = vec![None; n];
    (0..n).map(|i| resolve(i, skeleton, &mut world)).collect()
}

/// Export a mesh with a skeleton as a skinned GLB 2.0 file.
///
/// Produces a fully skinned GLB: in addition to the joint node hierarchy and
/// `skins` array, it emits per-vertex `JOINTS_0` (u16x4) and `WEIGHTS_0`
/// (f32x4) attributes computed from distance-based automatic skin weights
/// (the top-4 nearest joints per vertex, inverse-distance weighted and
/// normalized to sum to 1), plus an `inverseBindMatrices` accessor holding one
/// inverse-bind matrix per joint (derived from the forward-kinematics world
/// bind transforms).
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

    // ── Skinning: FK world transforms, auto weights, inverse-bind matrices ────
    let world = joint_world_transforms(skeleton);
    let joint_pos: Vec<[f32; 3]> = world.iter().map(|m| [m[12], m[13], m[14]]).collect();
    let ibm: Vec<[f32; 16]> = world.iter().map(mat4_inverse).collect();

    let mut joints_data: Vec<[u16; 4]> = Vec::with_capacity(n_verts);
    let mut weights_data: Vec<[f32; 4]> = Vec::with_capacity(n_verts);
    for &p in &mesh.positions {
        let mut ranked: Vec<(usize, f32)> = joint_pos
            .iter()
            .enumerate()
            .map(|(j, jp)| {
                let dx = p[0] - jp[0];
                let dy = p[1] - jp[1];
                let dz = p[2] - jp[2];
                (j, dx * dx + dy * dy + dz * dz)
            })
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let k = ranked.len().min(4);
        let mut js = [0u16; 4];
        let mut ws = [0.0f32; 4];
        let mut sum = 0.0f32;
        for (slot, &(j, d2)) in ranked.iter().take(k).enumerate() {
            let w = 1.0 / (d2 + 1e-6);
            js[slot] = j as u16;
            ws[slot] = w;
            sum += w;
        }
        if sum > 0.0 {
            for w in ws.iter_mut() {
                *w /= sum;
            }
        } else {
            ws[0] = 1.0;
        }
        joints_data.push(js);
        weights_data.push(ws);
    }

    let joints_bytes: &[u8] = cast_slice(&joints_data);
    let weights_bytes: &[u8] = cast_slice(&weights_data);
    let ibm_bytes: &[u8] = cast_slice(&ibm);

    let pos_offset = 0usize;
    let norm_offset = pos_offset + pos_bytes.len();
    let uv_offset = norm_offset + norm_bytes.len();
    let idx_offset = uv_offset + uv_bytes.len();
    let joints_offset = idx_offset + idx_bytes.len();
    let weights_offset = joints_offset + joints_bytes.len();
    let ibm_offset = weights_offset + weights_bytes.len();
    let bin_len = ibm_offset + ibm_bytes.len();

    let mut bin_data: Vec<u8> = Vec::with_capacity(bin_len + 3);
    bin_data.extend_from_slice(pos_bytes);
    bin_data.extend_from_slice(norm_bytes);
    bin_data.extend_from_slice(uv_bytes);
    bin_data.extend_from_slice(idx_bytes);
    bin_data.extend_from_slice(joints_bytes);
    bin_data.extend_from_slice(weights_bytes);
    bin_data.extend_from_slice(ibm_bytes);
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
            "skeleton": skeleton_root,
            "inverseBindMatrices": 6
        }],
        "meshes": [{
            "name": "human",
            "primitives": [{
                "attributes": {
                    "POSITION":   0,
                    "NORMAL":     1,
                    "TEXCOORD_0": 2,
                    "JOINTS_0":   4,
                    "WEIGHTS_0":  5
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
            },
            {
                "bufferView": 4,
                "componentType": 5123,
                "count": n_verts,
                "type": "VEC4"
            },
            {
                "bufferView": 5,
                "componentType": 5126,
                "count": n_verts,
                "type": "VEC4"
            },
            {
                "bufferView": 6,
                "componentType": 5126,
                "count": n_joints,
                "type": "MAT4"
            }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": pos_offset,     "byteLength": pos_bytes.len()     },
            { "buffer": 0, "byteOffset": norm_offset,    "byteLength": norm_bytes.len()    },
            { "buffer": 0, "byteOffset": uv_offset,      "byteLength": uv_bytes.len()      },
            { "buffer": 0, "byteOffset": idx_offset,     "byteLength": idx_bytes.len()     },
            { "buffer": 0, "byteOffset": joints_offset,  "byteLength": joints_bytes.len()  },
            { "buffer": 0, "byteOffset": weights_offset, "byteLength": weights_bytes.len() },
            { "buffer": 0, "byteOffset": ibm_offset,     "byteLength": ibm_bytes.len()     }
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

    #[test]
    fn skeleton_glb_has_skinning_attributes() {
        let mesh = suited_mesh_for_skin();
        let skeleton = Skeleton::human_body();
        let path = std::path::Path::new("/tmp/test_skeleton_skin_attrs.glb");
        export_glb_with_skeleton(&mesh, &skeleton, path).expect("export failed");

        use std::io::Read;
        let mut f = std::fs::File::open(path).expect("open");
        let mut buf12 = [0u8; 12];
        f.read_exact(&mut buf12).expect("hdr");
        let mut chunk_hdr = [0u8; 8];
        f.read_exact(&mut chunk_hdr).expect("chunk hdr");
        let json_len = u32::from_le_bytes(chunk_hdr[0..4].try_into().expect("len")) as usize;
        let mut json_buf = vec![0u8; json_len];
        f.read_exact(&mut json_buf).expect("json");
        let json_str = std::str::from_utf8(&json_buf).expect("utf8").trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("parse");

        let attrs = &parsed["meshes"][0]["primitives"][0]["attributes"];
        assert!(attrs.get("JOINTS_0").is_some(), "JOINTS_0 missing");
        assert!(attrs.get("WEIGHTS_0").is_some(), "WEIGHTS_0 missing");
        assert!(
            parsed["skins"][0].get("inverseBindMatrices").is_some(),
            "inverseBindMatrices missing"
        );
        let accessors = parsed["accessors"].as_array().expect("accessors");
        assert_eq!(accessors.len(), 7, "expected 7 accessors, got {}", accessors.len());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn skeleton_glb_weights_normalized() {
        // Read WEIGHTS_0 from the BIN chunk and check each vertex's weights sum to ~1.
        let mesh = suited_mesh_for_skin();
        let skeleton = Skeleton::human_body();
        let path = std::path::Path::new("/tmp/test_skeleton_skin_weights.glb");
        export_glb_with_skeleton(&mesh, &skeleton, path).expect("export failed");

        let bytes = std::fs::read(path).expect("read");
        // Parse JSON chunk to locate WEIGHTS_0 accessor/bufferView.
        let json_len = u32::from_le_bytes(bytes[12..16].try_into().expect("len")) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len]).expect("utf8").trim_end_matches(' ');
        let parsed: serde_json::Value = serde_json::from_str(json_str).expect("parse");
        let w_acc = parsed["meshes"][0]["primitives"][0]["attributes"]["WEIGHTS_0"]
            .as_u64()
            .expect("WEIGHTS_0 idx") as usize;
        let bv_idx = parsed["accessors"][w_acc]["bufferView"].as_u64().expect("bv") as usize;
        let count = parsed["accessors"][w_acc]["count"].as_u64().expect("count") as usize;
        let byte_off = parsed["bufferViews"][bv_idx]["byteOffset"].as_u64().expect("off") as usize;

        // BIN chunk starts at 12 (header) + 8 (json chunk hdr) + json_len + 8 (bin chunk hdr).
        let bin_start = 12 + 8 + json_len + 8;
        for v in 0..count {
            let base = bin_start + byte_off + v * 16; // 4 f32 per vertex
            let mut sum = 0.0f32;
            for c in 0..4 {
                let o = base + c * 4;
                sum += f32::from_le_bytes(bytes[o..o + 4].try_into().expect("f32"));
            }
            assert!((sum - 1.0).abs() < 1e-4, "vertex {v} weights sum = {sum}");
        }
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
