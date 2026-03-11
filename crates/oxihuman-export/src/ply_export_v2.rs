// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! PLY format export v2 with per-vertex colors.

#[allow(dead_code)]
pub struct PlyVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[allow(dead_code)]
pub struct PlyFace {
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
pub struct PlyExportV2 {
    pub vertices: Vec<PlyVertex>,
    pub faces: Vec<PlyFace>,
}

#[allow(dead_code)]
pub fn new_ply_export_v2() -> PlyExportV2 {
    PlyExportV2 { vertices: Vec::new(), faces: Vec::new() }
}

#[allow(dead_code)]
pub fn ply2_add_vertex(e: &mut PlyExportV2, x: f32, y: f32, z: f32, r: u8, g: u8, b: u8) {
    e.vertices.push(PlyVertex { x, y, z, r, g, b });
}

#[allow(dead_code)]
pub fn ply2_add_face(e: &mut PlyExportV2, indices: Vec<u32>) {
    e.faces.push(PlyFace { indices });
}

#[allow(dead_code)]
pub fn ply2_vertex_count(e: &PlyExportV2) -> usize {
    e.vertices.len()
}

#[allow(dead_code)]
pub fn ply2_to_ascii(e: &PlyExportV2) -> String {
    let mut out = String::from("ply\nformat ascii 1.0\n");
    out.push_str(&format!("element vertex {}\n", e.vertices.len()));
    out.push_str("property float x\nproperty float y\nproperty float z\n");
    out.push_str("property uchar red\nproperty uchar green\nproperty uchar blue\n");
    out.push_str(&format!("element face {}\n", e.faces.len()));
    out.push_str("property list uchar int vertex_indices\n");
    out.push_str("end_header\n");
    for v in &e.vertices {
        out.push_str(&format!("{} {} {} {} {} {}\n", v.x, v.y, v.z, v.r, v.g, v.b));
    }
    for f in &e.faces {
        let idx_str: Vec<String> = f.indices.iter().map(|i| i.to_string()).collect();
        out.push_str(&format!("{} {}\n", f.indices.len(), idx_str.join(" ")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let e = new_ply_export_v2();
        assert_eq!(ply2_vertex_count(&e), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut e = new_ply_export_v2();
        ply2_add_vertex(&mut e, 1.0, 2.0, 3.0, 255, 0, 0);
        assert_eq!(ply2_vertex_count(&e), 1);
    }

    #[test]
    fn test_add_face() {
        let mut e = new_ply_export_v2();
        ply2_add_face(&mut e, vec![0, 1, 2]);
        assert_eq!(e.faces.len(), 1);
    }

    #[test]
    fn test_to_ascii_starts_with_ply() {
        let e = new_ply_export_v2();
        let s = ply2_to_ascii(&e);
        assert!(s.starts_with("ply"));
    }

    #[test]
    fn test_to_ascii_contains_end_header() {
        let e = new_ply_export_v2();
        let s = ply2_to_ascii(&e);
        assert!(s.contains("end_header"));
    }

    #[test]
    fn test_vertex_count_multiple() {
        let mut e = new_ply_export_v2();
        for i in 0..4 {
            ply2_add_vertex(&mut e, i as f32, 0.0, 0.0, 0, 0, 0);
        }
        assert_eq!(ply2_vertex_count(&e), 4);
    }

    #[test]
    fn test_to_ascii_vertex_data() {
        let mut e = new_ply_export_v2();
        ply2_add_vertex(&mut e, 1.0, 2.0, 3.0, 100, 200, 50);
        let s = ply2_to_ascii(&e);
        assert!(s.contains("100"));
    }

    #[test]
    fn test_face_count() {
        let mut e = new_ply_export_v2();
        ply2_add_face(&mut e, vec![0, 1, 2]);
        ply2_add_face(&mut e, vec![1, 2, 3]);
        assert_eq!(e.faces.len(), 2);
    }
}
