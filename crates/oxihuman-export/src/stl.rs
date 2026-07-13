// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ASCII and binary STL exporter for 3D printing workflows.
//!
//! All entry points refuse a mesh whose `has_suit` flag is false (safety
//! check via [`crate::export_gate::ensure_export_allowed`]).  The historical
//! bypass ("STL is for geometry tools") has been removed.

use anyhow::Result;
use oxihuman_mesh::mesh::MeshBuffers;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use crate::export_gate::ensure_export_allowed;

/// Compute the unit face normal of a triangle (falls back to +Z-ish scaling
/// guard for degenerate triangles).
#[inline]
fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
    [nx / len, ny / len, nz / len]
}

/// Export mesh as ASCII STL text file.
/// Returns Err if the mesh has no suit applied (safety check).
pub fn export_stl_ascii(mesh: &MeshBuffers, path: &Path, solid_name: &str) -> Result<()> {
    let content = mesh_to_stl_ascii(mesh, solid_name)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Convert mesh to ASCII STL string.
/// Returns Err if the mesh has no suit applied (safety check).
pub fn mesh_to_stl_ascii(mesh: &MeshBuffers, solid_name: &str) -> Result<String> {
    ensure_export_allowed(mesh)?;

    let mut out = String::new();
    let name = solid_name.replace(char::is_whitespace, "_");
    writeln!(out, "solid {}", name)?;

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= mesh.positions.len() || i1 >= mesh.positions.len() || i2 >= mesh.positions.len() {
            continue;
        }

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];
        let [nx, ny, nz] = face_normal(p0, p1, p2);

        writeln!(out, "  facet normal {:.6e} {:.6e} {:.6e}", nx, ny, nz)?;
        writeln!(out, "    outer loop")?;
        writeln!(
            out,
            "      vertex {:.6e} {:.6e} {:.6e}",
            p0[0], p0[1], p0[2]
        )?;
        writeln!(
            out,
            "      vertex {:.6e} {:.6e} {:.6e}",
            p1[0], p1[1], p1[2]
        )?;
        writeln!(
            out,
            "      vertex {:.6e} {:.6e} {:.6e}",
            p2[0], p2[1], p2[2]
        )?;
        writeln!(out, "    endloop")?;
        writeln!(out, "  endfacet")?;
    }

    writeln!(out, "endsolid {}", name)?;
    Ok(out)
}

/// Encode mesh as binary STL bytes, entirely in memory (no filesystem —
/// safe on `wasm32-unknown-unknown`).
///
/// Binary STL format:
///   80-byte header | uint32 triangle_count | [normal f32x3 | v0 f32x3 | v1 f32x3 | v2 f32x3 | attr u16] x N
///
/// Returns Err if the mesh has no suit applied (safety check).
pub fn encode_stl_binary(mesh: &MeshBuffers) -> Result<Vec<u8>> {
    ensure_export_allowed(mesh)?;

    // Count only the triangles we will actually write (in-bounds indices) so
    // the header count always matches the payload.
    let valid_tris: Vec<&[u32]> = mesh
        .indices
        .chunks_exact(3)
        .filter(|tri| {
            tri.iter()
                .all(|&i| (i as usize) < mesh.positions.len())
        })
        .collect();

    let mut out: Vec<u8> = Vec::with_capacity(84 + valid_tris.len() * 50);

    // 80-byte header
    let mut header = [0u8; 80];
    let msg = b"OxiHuman binary STL";
    header[..msg.len()].copy_from_slice(msg);
    out.extend_from_slice(&header);

    // Triangle count
    out.extend_from_slice(&(valid_tris.len() as u32).to_le_bytes());

    for tri in valid_tris {
        let p0 = mesh.positions[tri[0] as usize];
        let p1 = mesh.positions[tri[1] as usize];
        let p2 = mesh.positions[tri[2] as usize];
        let n = face_normal(p0, p1, p2);

        for c in n {
            out.extend_from_slice(&c.to_le_bytes());
        }
        for p in [p0, p1, p2] {
            for c in p {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        // Attribute byte count (0)
        out.extend_from_slice(&0u16.to_le_bytes());
    }

    Ok(out)
}

/// Export mesh as binary STL file (thin file-writing wrapper around
/// [`encode_stl_binary`]).
/// Returns Err if the mesh has no suit applied (safety check).
pub fn export_stl_binary(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let bytes = encode_stl_binary(mesh)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Verify a binary STL file's header and triangle count.
pub fn verify_stl_binary(path: &Path) -> Result<u32> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut header = [0u8; 84];
    f.read_exact(&mut header)?;
    let tri_count = u32::from_le_bytes(
        header[80..84]
            .try_into()
            .map_err(|_| anyhow::anyhow!("failed to read STL triangle count"))?,
    );
    Ok(tri_count)
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

    fn unsuited_mesh() -> MeshBuffers {
        let mut m = triangle_mesh();
        m.has_suit = false;
        m
    }

    #[test]
    fn ascii_stl_contains_solid_name() {
        let m = triangle_mesh();
        let s = mesh_to_stl_ascii(&m, "test_human").expect("should succeed");
        assert!(s.starts_with("solid test_human"));
        assert!(s.contains("endsolid test_human"));
    }

    #[test]
    fn ascii_stl_has_one_facet() {
        let m = triangle_mesh();
        let s = mesh_to_stl_ascii(&m, "h").expect("should succeed");
        let facets = s.matches("facet normal").count();
        assert_eq!(facets, 1);
    }

    #[test]
    fn ascii_stl_writes_file() {
        let m = triangle_mesh();
        let path = std::env::temp_dir().join("test_oxihuman_gated.stl");
        export_stl_ascii(&m, &path, "oxihuman").expect("should succeed");
        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn binary_stl_correct_triangle_count() {
        let m = triangle_mesh();
        let path = std::env::temp_dir().join("test_oxihuman_bin_gated.stl");
        export_stl_binary(&m, &path).expect("should succeed");
        let count = verify_stl_binary(&path).expect("should succeed");
        assert_eq!(count, 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn binary_stl_file_size() {
        let m = triangle_mesh();
        let path = std::env::temp_dir().join("test_size_gated.stl");
        export_stl_binary(&m, &path).expect("should succeed");
        let size = std::fs::metadata(&path).expect("should succeed").len();
        // 80 (header) + 4 (count) + 1 * 50 (per triangle) = 134
        assert_eq!(size, 134);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn encode_stl_binary_in_memory_matches_layout() {
        let m = triangle_mesh();
        let bytes = encode_stl_binary(&m).expect("should succeed");
        assert_eq!(bytes.len(), 134);
        let count = u32::from_le_bytes(bytes[80..84].try_into().expect("count"));
        assert_eq!(count, 1);
    }

    #[test]
    fn ascii_stl_refuses_unsuited_mesh() {
        let m = unsuited_mesh();
        assert!(mesh_to_stl_ascii(&m, "x").is_err());
        let path = std::env::temp_dir().join("test_unsuited_ascii.stl");
        assert!(export_stl_ascii(&m, &path, "x").is_err());
        assert!(!path.exists());
    }

    #[test]
    fn binary_stl_refuses_unsuited_mesh() {
        let m = unsuited_mesh();
        assert!(encode_stl_binary(&m).is_err());
        let path = std::env::temp_dir().join("test_unsuited_bin.stl");
        assert!(export_stl_binary(&m, &path).is_err());
        assert!(!path.exists());
    }
}
