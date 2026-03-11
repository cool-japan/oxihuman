// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ASCII and binary STL exporter for 3D printing workflows.

use anyhow::Result;
use oxihuman_mesh::mesh::MeshBuffers;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::Path;

/// Export mesh as ASCII STL text file.
/// Does not require `has_suit` (STL is for geometry tools, not final human export).
pub fn export_stl_ascii(mesh: &MeshBuffers, path: &Path, solid_name: &str) -> Result<()> {
    let content = mesh_to_stl_ascii(mesh, solid_name)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Convert mesh to ASCII STL string.
pub fn mesh_to_stl_ascii(mesh: &MeshBuffers, solid_name: &str) -> Result<String> {
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

        // Compute face normal
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
        let (nx, ny, nz) = (nx / len, ny / len, nz / len);

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

/// Export mesh as binary STL (more compact, no ASCII parsing overhead).
/// Binary STL format:
///   80-byte header | uint32 triangle_count | [normal f32x3 | v0 f32x3 | v1 f32x3 | v2 f32x3 | attr u16] x N
pub fn export_stl_binary(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let tri_count = mesh.indices.len() / 3;
    let mut file = std::fs::File::create(path)?;

    // 80-byte header
    let mut header = [0u8; 80];
    let msg = b"OxiHuman binary STL";
    header[..msg.len()].copy_from_slice(msg);
    file.write_all(&header)?;

    // Triangle count
    file.write_all(&(tri_count as u32).to_le_bytes())?;

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= mesh.positions.len() || i1 >= mesh.positions.len() || i2 >= mesh.positions.len() {
            continue;
        }

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];

        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);

        // Normal
        file.write_all(&(nx / len).to_le_bytes())?;
        file.write_all(&(ny / len).to_le_bytes())?;
        file.write_all(&(nz / len).to_le_bytes())?;
        // Vertices
        for p in &[p0, p1, p2] {
            file.write_all(&p[0].to_le_bytes())?;
            file.write_all(&p[1].to_le_bytes())?;
            file.write_all(&p[2].to_le_bytes())?;
        }
        // Attribute byte count (0)
        file.write_all(&0u16.to_le_bytes())?;
    }

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
            has_suit: false,
        })
    }

    #[test]
    fn ascii_stl_contains_solid_name() {
        let m = triangle_mesh();
        let s = mesh_to_stl_ascii(&m, "test_human").unwrap();
        assert!(s.starts_with("solid test_human"));
        assert!(s.contains("endsolid test_human"));
    }

    #[test]
    fn ascii_stl_has_one_facet() {
        let m = triangle_mesh();
        let s = mesh_to_stl_ascii(&m, "h").unwrap();
        let facets = s.matches("facet normal").count();
        assert_eq!(facets, 1);
    }

    #[test]
    fn ascii_stl_writes_file() {
        let m = triangle_mesh();
        let path = std::path::PathBuf::from("/tmp/test_oxihuman.stl");
        export_stl_ascii(&m, &path, "oxihuman").unwrap();
        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn binary_stl_correct_triangle_count() {
        let m = triangle_mesh();
        let path = std::path::PathBuf::from("/tmp/test_oxihuman_bin.stl");
        export_stl_binary(&m, &path).unwrap();
        let count = verify_stl_binary(&path).unwrap();
        assert_eq!(count, 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn binary_stl_file_size() {
        let m = triangle_mesh();
        let path = std::path::PathBuf::from("/tmp/test_size.stl");
        export_stl_binary(&m, &path).unwrap();
        let size = std::fs::metadata(&path).unwrap().len();
        // 80 (header) + 4 (count) + 1 * 50 (per triangle) = 134
        assert_eq!(size, 134);
        std::fs::remove_file(&path).ok();
    }
}
