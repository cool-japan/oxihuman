// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! JSON mesh data export.

#[allow(dead_code)]
pub struct JsonMeshExport {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
pub fn new_json_mesh_export() -> JsonMeshExport {
    JsonMeshExport { vertices: Vec::new(), normals: Vec::new(), uvs: Vec::new(), indices: Vec::new() }
}

#[allow(dead_code)]
pub fn jme_add_vertex(e: &mut JsonMeshExport, pos: [f32; 3], normal: [f32; 3], uv: [f32; 2]) {
    e.vertices.push(pos);
    e.normals.push(normal);
    e.uvs.push(uv);
}

#[allow(dead_code)]
pub fn jme_add_triangle(e: &mut JsonMeshExport, a: u32, b: u32, c: u32) {
    e.indices.push(a);
    e.indices.push(b);
    e.indices.push(c);
}

#[allow(dead_code)]
pub fn jme_vertex_count(e: &JsonMeshExport) -> usize {
    e.vertices.len()
}

#[allow(dead_code)]
pub fn jme_triangle_count(e: &JsonMeshExport) -> usize {
    e.indices.len() / 3
}

#[allow(dead_code)]
pub fn jme_to_json(e: &JsonMeshExport) -> String {
    let verts: Vec<String> = e.vertices.iter()
        .map(|v| format!("[{},{},{}]", v[0], v[1], v[2]))
        .collect();
    let idxs: Vec<String> = e.indices.iter().map(|i| i.to_string()).collect();
    format!(
        r#"{{"vertices":[{}],"indices":[{}],"vertex_count":{},"triangle_count":{}}}"#,
        verts.join(","),
        idxs.join(","),
        e.vertices.len(),
        e.indices.len() / 3
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let e = new_json_mesh_export();
        assert_eq!(jme_vertex_count(&e), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut e = new_json_mesh_export();
        jme_add_vertex(&mut e, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0], [0.5, 0.5]);
        assert_eq!(jme_vertex_count(&e), 1);
    }

    #[test]
    fn test_add_triangle() {
        let mut e = new_json_mesh_export();
        jme_add_vertex(&mut e, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]);
        jme_add_vertex(&mut e, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0]);
        jme_add_vertex(&mut e, [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0]);
        jme_add_triangle(&mut e, 0, 1, 2);
        assert_eq!(jme_triangle_count(&e), 1);
    }

    #[test]
    fn test_vertex_count() {
        let mut e = new_json_mesh_export();
        for _ in 0..5 {
            jme_add_vertex(&mut e, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]);
        }
        assert_eq!(jme_vertex_count(&e), 5);
    }

    #[test]
    fn test_triangle_count() {
        let mut e = new_json_mesh_export();
        jme_add_triangle(&mut e, 0, 1, 2);
        jme_add_triangle(&mut e, 1, 2, 3);
        assert_eq!(jme_triangle_count(&e), 2);
    }

    #[test]
    fn test_to_json_contains_vertices() {
        let mut e = new_json_mesh_export();
        jme_add_vertex(&mut e, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]);
        let json = jme_to_json(&e);
        assert!(json.contains("vertices"));
    }

    #[test]
    fn test_to_json_contains_indices() {
        let e = new_json_mesh_export();
        let json = jme_to_json(&e);
        assert!(json.contains("indices"));
    }

    #[test]
    fn test_normals_stored() {
        let mut e = new_json_mesh_export();
        jme_add_vertex(&mut e, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]);
        assert_eq!(e.normals.len(), 1);
    }
}
