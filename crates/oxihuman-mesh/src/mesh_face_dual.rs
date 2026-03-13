// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face dual mesh construction (dual graph where each face becomes a vertex).

use std::collections::HashMap;

/// Dual mesh: vertices at face centroids, edges between adjacent faces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceDual {
    pub vertices: Vec<[f32; 3]>,
    pub edges: Vec<(usize, usize)>,
}

/// Compute the centroid of a triangle.
#[allow(dead_code)]
pub fn triangle_centroid(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ]
}

/// Build the face dual mesh.
#[allow(dead_code)]
pub fn build_face_dual(positions: &[[f32; 3]], indices: &[u32]) -> FaceDual {
    let tri_count = indices.len() / 3;
    let mut vertices = Vec::with_capacity(tri_count);

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        vertices.push(triangle_centroid(
            positions[i0],
            positions[i1],
            positions[i2],
        ));
    }

    // Build edge adjacency
    let mut edge_face: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tri_count {
        let verts = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for &(a, b) in &[
            (verts[0], verts[1]),
            (verts[1], verts[2]),
            (verts[2], verts[0]),
        ] {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_face.entry(key).or_default().push(t);
        }
    }

    let mut edges = Vec::new();
    for faces in edge_face.values() {
        if faces.len() == 2 {
            let a = faces[0].min(faces[1]);
            let b = faces[0].max(faces[1]);
            edges.push((a, b));
        }
    }
    edges.sort_unstable();
    edges.dedup();

    FaceDual { vertices, edges }
}

/// Vertex count in dual mesh.
#[allow(dead_code)]
pub fn dual_vertex_count(dual: &FaceDual) -> usize {
    dual.vertices.len()
}

/// Edge count in dual mesh.
#[allow(dead_code)]
pub fn dual_edge_count(dual: &FaceDual) -> usize {
    dual.edges.len()
}

/// Compute degree (number of adjacent faces) for a dual vertex.
#[allow(dead_code)]
pub fn dual_degree(dual: &FaceDual, vertex: usize) -> usize {
    dual.edges
        .iter()
        .filter(|&&(a, b)| a == vertex || b == vertex)
        .count()
}

/// Average dual edge length.
#[allow(dead_code)]
pub fn avg_dual_edge_length(dual: &FaceDual) -> f32 {
    if dual.edges.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for &(a, b) in &dual.edges {
        let va = dual.vertices[a];
        let vb = dual.vertices[b];
        let dx = va[0] - vb[0];
        let dy = va[1] - vb[1];
        let dz = va[2] - vb[2];
        sum += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    sum / dual.edges.len() as f32
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn face_dual_to_json(dual: &FaceDual) -> String {
    format!(
        "{{\"vertices\":{},\"edges\":{}}}",
        dual_vertex_count(dual),
        dual_edge_count(dual)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn test_triangle_centroid() {
        let c = triangle_centroid([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_build_dual_two_tri() {
        let (pos, idx) = two_tri_mesh();
        let dual = build_face_dual(&pos, &idx);
        assert_eq!(dual_vertex_count(&dual), 2);
        assert_eq!(dual_edge_count(&dual), 1);
    }

    #[test]
    fn test_build_dual_single() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let dual = build_face_dual(&pos, &idx);
        assert_eq!(dual_vertex_count(&dual), 1);
        assert_eq!(dual_edge_count(&dual), 0);
    }

    #[test]
    fn test_build_dual_empty() {
        let dual = build_face_dual(&[], &[]);
        assert_eq!(dual_vertex_count(&dual), 0);
    }

    #[test]
    fn test_dual_degree() {
        let (pos, idx) = two_tri_mesh();
        let dual = build_face_dual(&pos, &idx);
        assert_eq!(dual_degree(&dual, 0), 1);
    }

    #[test]
    fn test_avg_edge_length() {
        let (pos, idx) = two_tri_mesh();
        let dual = build_face_dual(&pos, &idx);
        let avg = avg_dual_edge_length(&dual);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_avg_edge_empty() {
        let dual = FaceDual {
            vertices: vec![],
            edges: vec![],
        };
        assert!((avg_dual_edge_length(&dual)).abs() < 1e-9);
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = two_tri_mesh();
        let dual = build_face_dual(&pos, &idx);
        let j = face_dual_to_json(&dual);
        assert!(j.contains("\"vertices\":2"));
    }

    #[test]
    fn test_edges_sorted() {
        let (pos, idx) = two_tri_mesh();
        let dual = build_face_dual(&pos, &idx);
        for &(a, b) in &dual.edges {
            assert!(a < b);
        }
    }

    #[test]
    fn test_centroid_positions() {
        let (pos, idx) = two_tri_mesh();
        let dual = build_face_dual(&pos, &idx);
        // Both centroids should have z=0
        for v in &dual.vertices {
            assert!((v[2]).abs() < 1e-6);
        }
    }
}
