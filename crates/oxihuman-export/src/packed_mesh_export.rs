// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export packed mesh binary format with interleaved vertex attributes.

/// Vertex attribute layout.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VertexAttrib { Position, Normal, Uv, Color, Tangent }

/// Packed mesh export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PackedMeshExport {
    pub layout: Vec<VertexAttrib>,
    pub vertex_data: Vec<u8>,
    pub index_data: Vec<u32>,
    pub vertex_count: u32,
}

#[allow(dead_code)]
pub fn attrib_size(a: VertexAttrib) -> usize {
    match a {
        VertexAttrib::Position => 12, VertexAttrib::Normal => 12,
        VertexAttrib::Uv => 8, VertexAttrib::Color => 16, VertexAttrib::Tangent => 16,
    }
}

#[allow(dead_code)]
pub fn stride(layout: &[VertexAttrib]) -> usize {
    layout.iter().map(|a| attrib_size(*a)).sum()
}

#[allow(dead_code)]
pub fn new_packed_mesh(layout: &[VertexAttrib]) -> PackedMeshExport {
    PackedMeshExport { layout: layout.to_vec(), vertex_data: Vec::new(), index_data: Vec::new(), vertex_count: 0 }
}

#[allow(dead_code)]
pub fn packed_add_vertex(mesh: &mut PackedMeshExport, data: &[u8]) {
    let s = stride(&mesh.layout);
    if data.len() >= s {
        mesh.vertex_data.extend_from_slice(&data[..s]);
        mesh.vertex_count += 1;
    }
}

#[allow(dead_code)]
pub fn packed_add_triangle(mesh: &mut PackedMeshExport, a: u32, b: u32, c: u32) {
    mesh.index_data.extend_from_slice(&[a, b, c]);
}

#[allow(dead_code)]
pub fn packed_vertex_count(mesh: &PackedMeshExport) -> u32 { mesh.vertex_count }

#[allow(dead_code)]
pub fn packed_face_count(mesh: &PackedMeshExport) -> usize { mesh.index_data.len() / 3 }

#[allow(dead_code)]
pub fn packed_data_size(mesh: &PackedMeshExport) -> usize {
    mesh.vertex_data.len() + mesh.index_data.len() * 4
}

#[allow(dead_code)]
pub fn attrib_name(a: VertexAttrib) -> &'static str {
    match a {
        VertexAttrib::Position => "position", VertexAttrib::Normal => "normal",
        VertexAttrib::Uv => "uv", VertexAttrib::Color => "color", VertexAttrib::Tangent => "tangent",
    }
}

#[allow(dead_code)]
pub fn packed_to_json(mesh: &PackedMeshExport) -> String {
    let attrs: Vec<String> = mesh.layout.iter().map(|a| format!(r#""{}""#, attrib_name(*a))).collect();
    format!(r#"{{"layout":[{}],"vertices":{},"faces":{},"bytes":{}}}"#,
        attrs.join(","), mesh.vertex_count, packed_face_count(mesh), packed_data_size(mesh))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attrib_size() {
        assert_eq!(attrib_size(VertexAttrib::Position), 12);
        assert_eq!(attrib_size(VertexAttrib::Uv), 8);
    }

    #[test]
    fn test_stride() {
        let layout = vec![VertexAttrib::Position, VertexAttrib::Normal];
        assert_eq!(stride(&layout), 24);
    }

    #[test]
    fn test_new_packed() {
        let mesh = new_packed_mesh(&[VertexAttrib::Position]);
        assert_eq!(packed_vertex_count(&mesh), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut mesh = new_packed_mesh(&[VertexAttrib::Position]);
        let data = vec![0u8; 12];
        packed_add_vertex(&mut mesh, &data);
        assert_eq!(packed_vertex_count(&mesh), 1);
    }

    #[test]
    fn test_add_triangle() {
        let mut mesh = new_packed_mesh(&[VertexAttrib::Position]);
        packed_add_triangle(&mut mesh, 0, 1, 2);
        assert_eq!(packed_face_count(&mesh), 1);
    }

    #[test]
    fn test_data_size() {
        let mut mesh = new_packed_mesh(&[VertexAttrib::Position]);
        packed_add_vertex(&mut mesh, &[0u8; 12]);
        packed_add_triangle(&mut mesh, 0, 0, 0);
        assert!(packed_data_size(&mesh) > 0);
    }

    #[test]
    fn test_attrib_name() {
        assert_eq!(attrib_name(VertexAttrib::Tangent), "tangent");
    }

    #[test]
    fn test_to_json() {
        let mesh = new_packed_mesh(&[VertexAttrib::Position, VertexAttrib::Uv]);
        let json = packed_to_json(&mesh);
        assert!(json.contains("position"));
    }

    #[test]
    fn test_short_data_rejected() {
        let mut mesh = new_packed_mesh(&[VertexAttrib::Position]);
        packed_add_vertex(&mut mesh, &[0u8; 5]); // too short
        assert_eq!(packed_vertex_count(&mesh), 0);
    }

    #[test]
    fn test_multiple_attribs() {
        let layout = vec![VertexAttrib::Position, VertexAttrib::Normal, VertexAttrib::Uv];
        let s = stride(&layout);
        assert_eq!(s, 32);
    }

}
