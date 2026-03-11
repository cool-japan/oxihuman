// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DirectX .x mesh format export (text format).

/// A DirectX .x mesh object.
#[derive(Clone, Debug, Default)]
pub struct XMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// A DirectX .x document.
#[derive(Clone, Debug, Default)]
pub struct XDocument {
    pub meshes: Vec<XMesh>,
}

/// Create a new .x document.
pub fn new_x_document() -> XDocument {
    XDocument::default()
}

/// Add a mesh to the document.
pub fn x_add_mesh(doc: &mut XDocument, mesh: XMesh) {
    doc.meshes.push(mesh);
}

/// Return the mesh count.
pub fn x_mesh_count(doc: &XDocument) -> usize {
    doc.meshes.len()
}

/// Return total vertex count.
pub fn x_total_vertex_count(doc: &XDocument) -> usize {
    doc.meshes.iter().map(|m| m.positions.len()).sum()
}

/// Return total face count.
pub fn x_total_face_count(doc: &XDocument) -> usize {
    doc.meshes.iter().map(|m| m.indices.len() / 3).sum()
}

/// Render the .x file header.
pub fn x_file_header() -> &'static str {
    "xof 0303txt 0032\n"
}

/// Render a mesh to .x text format.
fn render_x_mesh(mesh: &XMesh) -> String {
    let mut out = format!("Mesh {} {{\n", mesh.name);
    out.push_str(&format!(" {};\n", mesh.positions.len()));
    for (i, p) in mesh.positions.iter().enumerate() {
        let sep = if i + 1 < mesh.positions.len() {
            ","
        } else {
            ";"
        };
        out.push_str(&format!(" {:.6};{:.6};{:.6};{}\n", p[0], p[1], p[2], sep));
    }
    // Faces
    let tri_count = mesh.indices.len() / 3;
    out.push_str(&format!(" {};\n", tri_count));
    for t in 0..tri_count {
        let a = mesh.indices[t * 3];
        let b = mesh.indices[t * 3 + 1];
        let c = mesh.indices[t * 3 + 2];
        let sep = if t + 1 < tri_count { "," } else { ";" };
        out.push_str(&format!(" 3;{},{},{};{}\n", a, b, c, sep));
    }
    // Normals section
    if !mesh.normals.is_empty() {
        out.push_str(" MeshNormals {\n");
        out.push_str(&format!("  {};\n", mesh.normals.len()));
        for (i, n) in mesh.normals.iter().enumerate() {
            let sep = if i + 1 < mesh.normals.len() { "," } else { ";" };
            out.push_str(&format!("  {:.6};{:.6};{:.6};{}\n", n[0], n[1], n[2], sep));
        }
        out.push_str(" }\n");
    }
    out.push_str("}\n");
    out
}

/// Render the full .x document.
pub fn render_x_document(doc: &XDocument) -> String {
    let mut out = String::from(x_file_header());
    out.push_str("template Mesh { <3d82ab44-62da-11cf-ab39-0020af71e433> ... }\n\n");
    for mesh in &doc.meshes {
        out.push_str(&render_x_mesh(mesh));
    }
    out
}

/// Estimate file size.
pub fn x_size_estimate(doc: &XDocument) -> usize {
    render_x_document(doc).len()
}

/// Validate the document.
pub fn validate_x_document(doc: &XDocument) -> bool {
    doc.meshes.iter().all(|m| {
        !m.positions.is_empty() && {
            let n = m.positions.len() as u32;
            m.indices.iter().all(|&i| i < n)
        }
    })
}

/// Create a mesh from raw geometry.
pub fn x_mesh_from_geometry(
    name: &str,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
) -> XMesh {
    XMesh {
        name: name.to_string(),
        positions,
        normals,
        uvs: Vec::new(),
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_doc() -> XDocument {
        let mut doc = new_x_document();
        let mesh = x_mesh_from_geometry(
            "Mesh01",
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![],
            vec![0, 1, 2],
        );
        x_add_mesh(&mut doc, mesh);
        doc
    }

    #[test]
    fn mesh_count() {
        let d = simple_doc();
        assert_eq!(x_mesh_count(&d), 1);
    }

    #[test]
    fn total_vertex_count() {
        let d = simple_doc();
        assert_eq!(x_total_vertex_count(&d), 3);
    }

    #[test]
    fn total_face_count() {
        let d = simple_doc();
        assert_eq!(x_total_face_count(&d), 1);
    }

    #[test]
    fn header_starts_with_xof() {
        assert!(x_file_header().starts_with("xof "));
    }

    #[test]
    fn render_contains_mesh_name() {
        let d = simple_doc();
        let s = render_x_document(&d);
        assert!(s.contains("Mesh01"));
    }

    #[test]
    fn validate_valid() {
        let d = simple_doc();
        assert!(validate_x_document(&d));
    }

    #[test]
    fn x_size_estimate_positive() {
        let d = simple_doc();
        assert!(x_size_estimate(&d) > 0);
    }

    #[test]
    fn empty_doc_valid() {
        let d = new_x_document();
        assert!(validate_x_document(&d));
    }
}
