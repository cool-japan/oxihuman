// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A face in a quad-dominant mesh.
#[allow(dead_code)]
#[derive(Clone)]
pub enum QdFace {
    Tri([u32; 3]),
    Quad([u32; 4]),
}

/// A quad-dominant mesh.
#[allow(dead_code)]
pub struct QuadDominantMesh {
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<QdFace>,
}

/// Count quad faces in a quad-dominant mesh.
#[allow(dead_code)]
pub fn quad_count(mesh: &QuadDominantMesh) -> usize {
    mesh.faces
        .iter()
        .filter(|f| matches!(f, QdFace::Quad(_)))
        .count()
}

/// Count triangle faces in a quad-dominant mesh.
#[allow(dead_code)]
pub fn tri_count(mesh: &QuadDominantMesh) -> usize {
    mesh.faces
        .iter()
        .filter(|f| matches!(f, QdFace::Tri(_)))
        .count()
}

/// Quad ratio of a mesh.
#[allow(dead_code)]
pub fn quad_ratio(mesh: &QuadDominantMesh) -> f32 {
    if mesh.faces.is_empty() {
        return 0.0;
    }
    quad_count(mesh) as f32 / mesh.faces.len() as f32
}

/// Triangulate all quads to produce a pure triangle mesh.
#[allow(dead_code)]
pub fn triangulate_quads_qd(mesh: &QuadDominantMesh) -> Vec<[u32; 3]> {
    let mut tris = Vec::new();
    for face in &mesh.faces {
        match face {
            QdFace::Tri(t) => tris.push(*t),
            QdFace::Quad([a, b, c, d]) => {
                tris.push([*a, *b, *c]);
                tris.push([*a, *c, *d]);
            }
        }
    }
    tris
}

/// Build a simple quad-dominant grid mesh.
#[allow(dead_code)]
pub fn build_qd_grid(rows: usize, cols: usize) -> QuadDominantMesh {
    let mut positions = Vec::new();
    for r in 0..=rows {
        for c in 0..=cols {
            positions.push([c as f32, r as f32, 0.0]);
        }
    }
    let mut faces = Vec::new();
    let stride = (cols + 1) as u32;
    for r in 0..rows {
        for c in 0..cols {
            let a = (r as u32) * stride + c as u32;
            let b = a + 1;
            let d = a + stride;
            let e = d + 1;
            faces.push(QdFace::Quad([a, b, e, d]));
        }
    }
    QuadDominantMesh { positions, faces }
}

/// Convert to JSON summary.
#[allow(dead_code)]
pub fn qd_to_json(mesh: &QuadDominantMesh) -> String {
    format!(
        r#"{{"vertices":{},"quads":{},"tris":{}}}"#,
        mesh.positions.len(),
        quad_count(mesh),
        tri_count(mesh)
    )
}

/// Check that all face indices are in bounds.
#[allow(dead_code)]
pub fn qd_indices_valid(mesh: &QuadDominantMesh) -> bool {
    let n = mesh.positions.len() as u32;
    mesh.faces.iter().all(|f| match f {
        QdFace::Tri(t) => t.iter().all(|&i| i < n),
        QdFace::Quad(q) => q.iter().all(|&i| i < n),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_all_quads() {
        let m = build_qd_grid(3, 4);
        assert_eq!(quad_count(&m), 12);
        assert_eq!(tri_count(&m), 0);
    }

    #[test]
    fn quad_ratio_full() {
        let m = build_qd_grid(2, 2);
        assert!((quad_ratio(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangulate_doubles_faces() {
        let m = build_qd_grid(2, 2);
        let tris = triangulate_quads_qd(&m);
        assert_eq!(tris.len(), 4 * 2);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_qd_grid(3, 3);
        assert!(qd_indices_valid(&m));
    }

    #[test]
    fn json_contains_quads() {
        let m = build_qd_grid(2, 3);
        let j = qd_to_json(&m);
        assert!(j.contains("\"quads\":6"));
    }

    #[test]
    fn mixed_mesh() {
        let mut m = build_qd_grid(1, 1);
        m.faces.push(QdFace::Tri([0, 1, 2]));
        assert_eq!(tri_count(&m), 1);
        assert_eq!(quad_count(&m), 1);
    }

    #[test]
    fn empty_faces_ratio() {
        let m = QuadDominantMesh {
            positions: vec![],
            faces: vec![],
        };
        assert!((quad_ratio(&m) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn triangulate_tri_passthrough() {
        let m = QuadDominantMesh {
            positions: vec![[0.0, 0.0, 0.0]; 3],
            faces: vec![QdFace::Tri([0, 1, 2])],
        };
        let tris = triangulate_quads_qd(&m);
        assert_eq!(tris.len(), 1);
    }

    #[test]
    fn grid_vertex_count() {
        let m = build_qd_grid(3, 4);
        assert_eq!(m.positions.len(), 4 * 5);
    }

    #[test]
    fn json_contains_tris_zero() {
        let m = build_qd_grid(1, 1);
        let j = qd_to_json(&m);
        assert!(j.contains("\"tris\":0"));
    }
}
