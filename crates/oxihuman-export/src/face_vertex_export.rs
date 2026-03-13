// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A face-vertex mesh export.
#[allow(dead_code)]
#[derive(Default)]
pub struct FaceVertexExport {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub face_vertex_indices: Vec<u32>,
    pub face_sizes: Vec<usize>,
}

/// Create a new face-vertex export.
#[allow(dead_code)]
pub fn new_face_vertex_export() -> FaceVertexExport {
    FaceVertexExport::default()
}

/// Add a vertex.
#[allow(dead_code)]
pub fn fv_add_vertex(export: &mut FaceVertexExport, pos: [f32; 3], normal: [f32; 3], uv: [f32; 2]) {
    export.positions.push(pos);
    export.normals.push(normal);
    export.uvs.push(uv);
}

/// Add a face with given vertex indices.
#[allow(dead_code)]
pub fn fv_add_face(export: &mut FaceVertexExport, indices: &[u32]) {
    export.face_vertex_indices.extend_from_slice(indices);
    export.face_sizes.push(indices.len());
}

/// Count vertices.
#[allow(dead_code)]
pub fn fv_vertex_count(export: &FaceVertexExport) -> usize {
    export.positions.len()
}

/// Count faces.
#[allow(dead_code)]
pub fn fv_face_count(export: &FaceVertexExport) -> usize {
    export.face_sizes.len()
}

/// Average face size (valence).
#[allow(dead_code)]
pub fn fv_avg_face_size(export: &FaceVertexExport) -> f32 {
    if export.face_sizes.is_empty() {
        return 0.0;
    }
    export.face_sizes.iter().sum::<usize>() as f32 / export.face_sizes.len() as f32
}

/// Validate that all face indices are in bounds.
#[allow(dead_code)]
pub fn fv_indices_valid(export: &FaceVertexExport) -> bool {
    let n = export.positions.len() as u32;
    export.face_vertex_indices.iter().all(|&i| i < n)
}

/// Check normals are unit length.
#[allow(dead_code)]
pub fn fv_normals_unit(export: &FaceVertexExport) -> bool {
    export.normals.iter().all(|n| {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        (len - 1.0).abs() < 1e-3
    })
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn face_vertex_to_json(export: &FaceVertexExport) -> String {
    format!(
        r#"{{"vertices":{},"faces":{}}}"#,
        export.positions.len(),
        export.face_sizes.len()
    )
}

/// Flatten to triangle indices (fan triangulation).
#[allow(dead_code)]
pub fn fv_to_triangles(export: &FaceVertexExport) -> Vec<u32> {
    let mut tris = Vec::new();
    let mut offset = 0;
    for &size in &export.face_sizes {
        if size >= 3 {
            let base = export.face_vertex_indices[offset];
            for i in 1..(size - 1) {
                tris.push(base);
                tris.push(export.face_vertex_indices[offset + i]);
                tris.push(export.face_vertex_indices[offset + i + 1]);
            }
        }
        offset += size;
    }
    tris
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_vertex_count() {
        let mut e = new_face_vertex_export();
        fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        assert_eq!(fv_vertex_count(&e), 1);
    }

    #[test]
    fn add_face_count() {
        let mut e = new_face_vertex_export();
        for _ in 0..3 {
            fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        }
        fv_add_face(&mut e, &[0, 1, 2]);
        assert_eq!(fv_face_count(&e), 1);
    }

    #[test]
    fn avg_face_size_triangle() {
        let mut e = new_face_vertex_export();
        for _ in 0..3 {
            fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        }
        fv_add_face(&mut e, &[0, 1, 2]);
        assert!((fv_avg_face_size(&e) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn indices_valid() {
        let mut e = new_face_vertex_export();
        for _ in 0..3 {
            fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        }
        fv_add_face(&mut e, &[0, 1, 2]);
        assert!(fv_indices_valid(&e));
    }

    #[test]
    fn to_triangles_count() {
        let mut e = new_face_vertex_export();
        for _ in 0..4 {
            fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        }
        fv_add_face(&mut e, &[0, 1, 2, 3]); // quad -> 2 triangles
        let tris = fv_to_triangles(&e);
        assert_eq!(tris.len(), 6);
    }

    #[test]
    fn json_has_vertices() {
        let e = new_face_vertex_export();
        let j = face_vertex_to_json(&e);
        assert!(j.contains("\"vertices\":0"));
    }

    #[test]
    fn empty_avg_size() {
        let e = new_face_vertex_export();
        assert!((fv_avg_face_size(&e) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn normals_unit_check() {
        let mut e = new_face_vertex_export();
        fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        assert!(fv_normals_unit(&e));
    }

    #[test]
    fn normals_not_unit() {
        let mut e = new_face_vertex_export();
        fv_add_vertex(&mut e, [0.0; 3], [0.0, 0.0, 0.0], [0.0; 2]);
        assert!(!fv_normals_unit(&e));
    }

    #[test]
    fn tris_divisible_by_3() {
        let mut e = new_face_vertex_export();
        for _ in 0..3 {
            fv_add_vertex(&mut e, [0.0; 3], [0.0, 1.0, 0.0], [0.0; 2]);
        }
        fv_add_face(&mut e, &[0, 1, 2]);
        let tris = fv_to_triangles(&e);
        assert_eq!(tris.len() % 3, 0);
    }
}
