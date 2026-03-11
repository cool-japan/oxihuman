// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Binary PLY format export (little-endian).

/// Binary PLY export container.
#[derive(Clone, Debug, Default)]
pub struct PlyBinaryExport {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Create a new binary PLY export.
pub fn new_ply_binary_export() -> PlyBinaryExport {
    PlyBinaryExport::default()
}

/// Set mesh data.
pub fn ply_binary_set_mesh(
    doc: &mut PlyBinaryExport,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
) {
    doc.positions = positions;
    doc.normals = normals;
    doc.indices = indices;
}

/// Return the vertex count.
pub fn ply_binary_vertex_count(doc: &PlyBinaryExport) -> usize {
    doc.positions.len()
}

/// Return the face count.
pub fn ply_binary_face_count(doc: &PlyBinaryExport) -> usize {
    doc.indices.len() / 3
}

/// Build the PLY ASCII header string.
pub fn ply_binary_header(doc: &PlyBinaryExport) -> String {
    let has_normals = !doc.normals.is_empty();
    let mut h = String::from("ply\nformat binary_little_endian 1.0\n");
    h.push_str(&format!(
        "element vertex {}\n",
        ply_binary_vertex_count(doc)
    ));
    h.push_str("property float x\nproperty float y\nproperty float z\n");
    if has_normals {
        h.push_str("property float nx\nproperty float ny\nproperty float nz\n");
    }
    h.push_str(&format!("element face {}\n", ply_binary_face_count(doc)));
    h.push_str("property list uchar uint vertex_indices\n");
    h.push_str("end_header\n");
    h
}

/// Serialize vertex data to little-endian bytes.
pub fn ply_binary_vertex_bytes(doc: &PlyBinaryExport) -> Vec<u8> {
    let has_normals = !doc.normals.is_empty();
    let mut out = Vec::new();
    for (i, &p) in doc.positions.iter().enumerate() {
        out.extend_from_slice(&p[0].to_le_bytes());
        out.extend_from_slice(&p[1].to_le_bytes());
        out.extend_from_slice(&p[2].to_le_bytes());
        if has_normals {
            if let Some(&n) = doc.normals.get(i) {
                out.extend_from_slice(&n[0].to_le_bytes());
                out.extend_from_slice(&n[1].to_le_bytes());
                out.extend_from_slice(&n[2].to_le_bytes());
            }
        }
    }
    out
}

/// Serialize face data to little-endian bytes.
pub fn ply_binary_face_bytes(doc: &PlyBinaryExport) -> Vec<u8> {
    let mut out = Vec::new();
    let tri_count = doc.indices.len() / 3;
    for t in 0..tri_count {
        out.push(3u8); // vertex count per face
        for k in 0..3 {
            let idx = doc.indices[t * 3 + k];
            out.extend_from_slice(&idx.to_le_bytes());
        }
    }
    out
}

/// Build the full binary PLY file as bytes (header + vertex data + face data).
pub fn export_ply_binary(doc: &PlyBinaryExport) -> Vec<u8> {
    let header = ply_binary_header(doc);
    let mut out = header.into_bytes();
    out.extend(ply_binary_vertex_bytes(doc));
    out.extend(ply_binary_face_bytes(doc));
    out
}

/// Estimate the file size in bytes.
pub fn ply_binary_size_estimate(doc: &PlyBinaryExport) -> usize {
    let has_normals = !doc.normals.is_empty();
    let verts_per_vertex = if has_normals { 6 } else { 3 };
    let vertex_bytes = ply_binary_vertex_count(doc) * verts_per_vertex * 4;
    let face_bytes = ply_binary_face_count(doc) * (1 + 3 * 4);
    ply_binary_header(doc).len() + vertex_bytes + face_bytes
}

/// Validate the export.
pub fn validate_ply_binary(doc: &PlyBinaryExport) -> bool {
    if doc.positions.is_empty() {
        return false;
    }
    let n = doc.positions.len() as u32;
    doc.indices.iter().all(|&i| i < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple() -> PlyBinaryExport {
        let mut d = new_ply_binary_export();
        ply_binary_set_mesh(
            &mut d,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![],
            vec![0, 1, 2],
        );
        d
    }

    #[test]
    fn vertex_count() {
        let d = simple();
        assert_eq!(ply_binary_vertex_count(&d), 3);
    }

    #[test]
    fn face_count() {
        let d = simple();
        assert_eq!(ply_binary_face_count(&d), 1);
    }

    #[test]
    fn header_contains_ply() {
        let d = simple();
        let h = ply_binary_header(&d);
        assert!(h.starts_with("ply\n"));
    }

    #[test]
    fn header_contains_binary() {
        let d = simple();
        let h = ply_binary_header(&d);
        assert!(h.contains("binary_little_endian"));
    }

    #[test]
    fn vertex_bytes_correct_size() {
        let d = simple();
        let bytes = ply_binary_vertex_bytes(&d);
        assert_eq!(bytes.len(), 3 * 3 * 4); // 3 verts × 3 floats × 4 bytes
    }

    #[test]
    fn face_bytes_correct_size() {
        let d = simple();
        let bytes = ply_binary_face_bytes(&d);
        assert_eq!(bytes.len(), 1 + 3 * 4); // 1 count byte + 3 uint32
    }

    #[test]
    fn export_ply_binary_non_empty() {
        let d = simple();
        let bytes = export_ply_binary(&d);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn validate_valid() {
        let d = simple();
        assert!(validate_ply_binary(&d));
    }
}
