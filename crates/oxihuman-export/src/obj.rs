// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Wavefront OBJ exporter for OxiHuman meshes.

use anyhow::Result;
use oxihuman_mesh::mesh::MeshBuffers;
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// Export a MeshBuffers to a Wavefront OBJ text file.
/// Does NOT require `has_suit` (OBJ is for debugging/tools, not final export).
pub fn export_obj(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let content = mesh_to_obj_string(mesh)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Convert a MeshBuffers to OBJ format string.
pub fn mesh_to_obj_string(mesh: &MeshBuffers) -> Result<String> {
    let mut out = String::new();
    writeln!(out, "# OxiHuman exported mesh")?;
    writeln!(out, "# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)")?;
    writeln!(out, "# Vertices: {}", mesh.positions.len())?;
    writeln!(out, "# Faces: {}", mesh.indices.len() / 3)?;
    writeln!(out)?;

    // Vertex positions
    for p in &mesh.positions {
        writeln!(out, "v {:.6} {:.6} {:.6}", p[0], p[1], p[2])?;
    }
    writeln!(out)?;

    // UV coordinates
    for uv in &mesh.uvs {
        writeln!(out, "vt {:.6} {:.6}", uv[0], uv[1])?;
    }
    writeln!(out)?;

    // Vertex normals
    for n in &mesh.normals {
        writeln!(out, "vn {:.6} {:.6} {:.6}", n[0], n[1], n[2])?;
    }
    writeln!(out)?;

    // Faces (OBJ is 1-indexed; format: v/vt/vn)
    let has_uvs = !mesh.uvs.is_empty();
    let has_norms = !mesh.normals.is_empty();
    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] + 1, tri[1] + 1, tri[2] + 1); // 1-indexed
        if has_uvs && has_norms {
            writeln!(out, "f {0}/{0}/{0} {1}/{1}/{1} {2}/{2}/{2}", i0, i1, i2)?;
        } else if has_uvs {
            writeln!(out, "f {0}/{0} {1}/{1} {2}/{2}", i0, i1, i2)?;
        } else {
            writeln!(out, "f {} {} {}", i0, i1, i2)?;
        }
    }

    Ok(out)
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
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn obj_string_has_vertex_lines() {
        let m = triangle_mesh();
        let s = mesh_to_obj_string(&m).expect("should succeed");
        assert!(s.contains("v 0.000000 0.000000 0.000000"));
        assert!(s.contains("v 1.000000 0.000000 0.000000"));
        let v_count = s.lines().filter(|l| l.starts_with("v ")).count();
        assert_eq!(v_count, 3);
    }

    #[test]
    fn obj_string_has_face_lines() {
        let m = triangle_mesh();
        let s = mesh_to_obj_string(&m).expect("should succeed");
        let f_count = s.lines().filter(|l| l.starts_with("f ")).count();
        assert_eq!(f_count, 1);
    }

    #[test]
    fn export_obj_creates_file() {
        let m = triangle_mesh();
        let path = std::path::PathBuf::from("/tmp/test_oxihuman.obj");
        export_obj(&m, &path).expect("should succeed");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).expect("should succeed");
        assert!(content.contains("v "));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn obj_indices_are_one_based() {
        let m = triangle_mesh();
        let s = mesh_to_obj_string(&m).expect("should succeed");
        // OBJ is 1-indexed: first face should reference 1, not 0
        assert!(s.contains("f 1/1/1 2/2/2 3/3/3"));
    }
}
