// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use oxihuman_mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

/// Report returned by [`export_mesh_csv`].
pub struct CsvExportReport {
    pub vertices_path: PathBuf,
    pub faces_path: PathBuf,
    pub normals_path: PathBuf,
    pub uvs_path: PathBuf,
    pub vertex_count: usize,
    pub face_count: usize,
}

// ---------------------------------------------------------------------------
// String helpers (no file I/O)
// ---------------------------------------------------------------------------

/// Export vertex positions as a CSV string: index,x,y,z
pub fn vertices_to_csv_string(mesh: &MeshBuffers) -> String {
    let mut out = String::from("index,x,y,z\n");
    for (i, p) in mesh.positions.iter().enumerate() {
        out.push_str(&format!("{},{},{},{}\n", i, p[0], p[1], p[2]));
    }
    out
}

/// Export face indices as a CSV string: face_index,v0,v1,v2
pub fn faces_to_csv_string(mesh: &MeshBuffers) -> String {
    let mut out = String::from("face_index,v0,v1,v2\n");
    for (fi, tri) in mesh.indices.chunks(3).enumerate() {
        if tri.len() == 3 {
            out.push_str(&format!("{},{},{},{}\n", fi, tri[0], tri[1], tri[2]));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Individual file exporters
// ---------------------------------------------------------------------------

/// Export vertex positions as CSV: index,x,y,z
pub fn export_vertices_csv(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    std::fs::write(path, vertices_to_csv_string(mesh))?;
    Ok(())
}

/// Export face indices as CSV: face_index,v0,v1,v2
pub fn export_faces_csv(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    std::fs::write(path, faces_to_csv_string(mesh))?;
    Ok(())
}

/// Export vertex normals as CSV: index,nx,ny,nz
pub fn export_normals_csv(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let mut out = String::from("index,nx,ny,nz\n");
    for (i, n) in mesh.normals.iter().enumerate() {
        out.push_str(&format!("{},{},{},{}\n", i, n[0], n[1], n[2]));
    }
    std::fs::write(path, out)?;
    Ok(())
}

/// Export UV coordinates as CSV: index,u,v
pub fn export_uvs_csv(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let mut out = String::from("index,u,v\n");
    for (i, uv) in mesh.uvs.iter().enumerate() {
        out.push_str(&format!("{},{},{}\n", i, uv[0], uv[1]));
    }
    std::fs::write(path, out)?;
    Ok(())
}

/// Export mesh statistics as a single-row CSV:
/// header: vertex_count,face_count,has_normals,has_uvs
pub fn export_stats_csv(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let has_normals = !mesh.normals.is_empty();
    let has_uvs = !mesh.uvs.is_empty();
    let out = format!(
        "vertex_count,face_count,has_normals,has_uvs\n{},{},{},{}\n",
        mesh.vertex_count(),
        mesh.face_count(),
        has_normals,
        has_uvs,
    );
    std::fs::write(path, out)?;
    Ok(())
}

/// Export a `HashMap<String, f32>` as CSV: key,value (sorted by key)
pub fn export_map_csv(data: &HashMap<String, f32>, path: &Path) -> Result<()> {
    let mut out = String::from("key,value\n");
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();
    for k in keys {
        out.push_str(&format!("{},{}\n", k, data[k]));
    }
    std::fs::write(path, out)?;
    Ok(())
}

/// Export all mesh data to multiple CSV files in a directory.
pub fn export_mesh_csv(mesh: &MeshBuffers, dir: &Path) -> Result<CsvExportReport> {
    std::fs::create_dir_all(dir)?;

    let vertices_path = dir.join("vertices.csv");
    let faces_path = dir.join("faces.csv");
    let normals_path = dir.join("normals.csv");
    let uvs_path = dir.join("uvs.csv");

    export_vertices_csv(mesh, &vertices_path)?;
    export_faces_csv(mesh, &faces_path)?;
    export_normals_csv(mesh, &normals_path)?;
    export_uvs_csv(mesh, &uvs_path)?;

    Ok(CsvExportReport {
        vertices_path,
        faces_path,
        normals_path,
        uvs_path,
        vertex_count: mesh.vertex_count(),
        face_count: mesh.face_count(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Simple 2-triangle (4-vertex) mesh.
    fn two_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    /// Empty mesh with no vertices or indices.
    fn empty_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        })
    }

    #[test]
    fn test_vertices_to_csv_string() {
        let mesh = two_tri_mesh();
        let csv = vertices_to_csv_string(&mesh);
        assert!(csv.starts_with("index,x,y,z\n"));
        assert!(csv.contains("0,0,0,0"));
        assert!(csv.contains("1,1,0,0"));
        assert!(csv.contains("2,1,1,0"));
        assert!(csv.contains("3,0,1,0"));
        let lines: Vec<&str> = csv.trim_end().lines().collect();
        // header + 4 data rows
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_faces_to_csv_string() {
        let mesh = two_tri_mesh();
        let csv = faces_to_csv_string(&mesh);
        assert!(csv.starts_with("face_index,v0,v1,v2\n"));
        assert!(csv.contains("0,0,1,2"));
        assert!(csv.contains("1,0,2,3"));
        let lines: Vec<&str> = csv.trim_end().lines().collect();
        // header + 2 face rows
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_export_vertices_csv() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_vertices.csv");
        export_vertices_csv(&mesh, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("index,x,y,z\n"));
        let lines: Vec<&str> = content.trim_end().lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_export_faces_csv() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_faces.csv");
        export_faces_csv(&mesh, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("face_index,v0,v1,v2\n"));
        let lines: Vec<&str> = content.trim_end().lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_export_normals_csv() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_normals.csv");
        export_normals_csv(&mesh, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("index,nx,ny,nz\n"));
        // 4 normals
        let lines: Vec<&str> = content.trim_end().lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(content.contains("0,0,0,1"));
    }

    #[test]
    fn test_export_uvs_csv() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_uvs.csv");
        export_uvs_csv(&mesh, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("index,u,v\n"));
        let lines: Vec<&str> = content.trim_end().lines().collect();
        // header + 4 uv rows
        assert_eq!(lines.len(), 5);
        assert!(content.contains("0,0,0"));
    }

    #[test]
    fn test_export_stats_csv() {
        let mesh = two_tri_mesh();
        let path = Path::new("/tmp/test_oxihuman_stats.csv");
        export_stats_csv(&mesh, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("vertex_count,face_count,has_normals,has_uvs\n"));
        assert!(content.contains("4,2,true,true"));
    }

    #[test]
    fn test_export_map_csv() {
        let mut data = HashMap::new();
        data.insert("height".to_string(), 1.75_f32);
        data.insert("weight".to_string(), 70.0_f32);
        let path = Path::new("/tmp/test_oxihuman_map.csv");
        export_map_csv(&data, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("key,value\n"));
        assert!(content.contains("height,"));
        assert!(content.contains("weight,"));
    }

    #[test]
    fn test_export_mesh_csv() {
        let mesh = two_tri_mesh();
        let dir = Path::new("/tmp/test_oxihuman_mesh_csv");
        let report = export_mesh_csv(&mesh, dir).unwrap();
        assert_eq!(report.vertex_count, 4);
        assert_eq!(report.face_count, 2);
        assert!(report.vertices_path.exists());
        assert!(report.faces_path.exists());
        assert!(report.normals_path.exists());
        assert!(report.uvs_path.exists());
    }

    #[test]
    fn test_csv_header_format() {
        let mesh = two_tri_mesh();
        let v_csv = vertices_to_csv_string(&mesh);
        let f_csv = faces_to_csv_string(&mesh);
        assert_eq!(v_csv.lines().next().unwrap(), "index,x,y,z");
        assert_eq!(f_csv.lines().next().unwrap(), "face_index,v0,v1,v2");
    }

    #[test]
    fn test_csv_empty_mesh() {
        let mesh = empty_mesh();
        let v_csv = vertices_to_csv_string(&mesh);
        let f_csv = faces_to_csv_string(&mesh);
        // Only header line when empty
        assert_eq!(v_csv.trim_end().lines().count(), 1);
        assert_eq!(f_csv.trim_end().lines().count(), 1);

        let stats_path = Path::new("/tmp/test_oxihuman_empty_stats.csv");
        export_stats_csv(&mesh, stats_path).unwrap();
        let content = std::fs::read_to_string(stats_path).unwrap();
        assert!(content.contains("0,0,false,false"));
    }

    #[test]
    fn test_export_map_csv_sorted() {
        let mut data = HashMap::new();
        data.insert("zebra".to_string(), 3.0_f32);
        data.insert("apple".to_string(), 1.0_f32);
        data.insert("mango".to_string(), 2.0_f32);
        let path = Path::new("/tmp/test_oxihuman_map_sorted.csv");
        export_map_csv(&data, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // lines[0] = header, [1] = apple, [2] = mango, [3] = zebra
        assert_eq!(lines.len(), 4);
        assert!(lines[1].starts_with("apple,"));
        assert!(lines[2].starts_with("mango,"));
        assert!(lines[3].starts_with("zebra,"));
    }
}
