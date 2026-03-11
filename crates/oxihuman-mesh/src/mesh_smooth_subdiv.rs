#![allow(dead_code)]

/// Result of smooth subdivision.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmoothSubdivResult {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 4]>,
    pub original_vertex_count: usize,
}

/// Perform one level of Catmull-Clark-like smooth subdivision.
#[allow(dead_code)]
pub fn smooth_subdivide(
    vertices: &[[f32; 3]],
    faces: &[[usize; 4]],
) -> SmoothSubdivResult {
    // Simplified: for each face, compute face point and add new vertices
    let mut new_verts = vertices.to_vec();
    let mut new_faces = Vec::new();

    for face in faces {
        let fp = smooth_face_point(vertices, face);
        let fp_idx = new_verts.len();
        new_verts.push(fp);

        // Create 4 sub-quads (simplified: using original vertices and face point)
        for i in 0..4 {
            let next = (i + 1) % 4;
            let ep = smooth_edge_point(vertices, face[i], face[next]);
            let ep_idx = new_verts.len();
            new_verts.push(ep);
            new_faces.push([face[i], ep_idx, fp_idx, ep_idx]);
        }
    }

    SmoothSubdivResult {
        original_vertex_count: vertices.len(),
        vertices: new_verts,
        faces: new_faces,
    }
}

/// Perform n levels of smooth subdivision.
#[allow(dead_code)]
pub fn smooth_subdivide_n(
    vertices: &[[f32; 3]],
    faces: &[[usize; 4]],
    levels: usize,
) -> SmoothSubdivResult {
    if levels == 0 {
        return SmoothSubdivResult {
            vertices: vertices.to_vec(),
            faces: faces.to_vec(),
            original_vertex_count: vertices.len(),
        };
    }
    let mut result = smooth_subdivide(vertices, faces);
    for _ in 1..levels {
        let next = smooth_subdivide(&result.vertices, &result.faces);
        result = next;
    }
    result
}

/// Compute the new position of an original vertex after smoothing.
#[allow(dead_code)]
pub fn smooth_vertex_position(
    vertex: [f32; 3],
    neighbor_avg: [f32; 3],
    face_avg: [f32; 3],
    valence: usize,
) -> [f32; 3] {
    let n = valence as f32;
    if n < 1.0 {
        return vertex;
    }
    [
        (face_avg[0] + 2.0 * neighbor_avg[0] + (n - 3.0) * vertex[0]) / n,
        (face_avg[1] + 2.0 * neighbor_avg[1] + (n - 3.0) * vertex[1]) / n,
        (face_avg[2] + 2.0 * neighbor_avg[2] + (n - 3.0) * vertex[2]) / n,
    ]
}

/// Compute the edge point (midpoint of edge endpoints and adjacent face points).
#[allow(dead_code)]
pub fn smooth_edge_point(vertices: &[[f32; 3]], a: usize, b: usize) -> [f32; 3] {
    [
        (vertices[a][0] + vertices[b][0]) * 0.5,
        (vertices[a][1] + vertices[b][1]) * 0.5,
        (vertices[a][2] + vertices[b][2]) * 0.5,
    ]
}

/// Compute the face point (centroid of face vertices).
#[allow(dead_code)]
pub fn smooth_face_point(vertices: &[[f32; 3]], face: &[usize; 4]) -> [f32; 3] {
    [
        (vertices[face[0]][0] + vertices[face[1]][0] + vertices[face[2]][0] + vertices[face[3]][0]) * 0.25,
        (vertices[face[0]][1] + vertices[face[1]][1] + vertices[face[2]][1] + vertices[face[3]][1]) * 0.25,
        (vertices[face[0]][2] + vertices[face[1]][2] + vertices[face[2]][2] + vertices[face[3]][2]) * 0.25,
    ]
}

/// Expected vertex count after one level of subdivision.
#[allow(dead_code)]
pub fn smooth_subdiv_vertex_count(original_verts: usize, face_count: usize, edge_count: usize) -> usize {
    original_verts + face_count + edge_count
}

/// Expected face count after one level of subdivision.
#[allow(dead_code)]
pub fn smooth_subdiv_face_count(original_face_count: usize) -> usize {
    original_face_count * 4
}

/// Compute boundary vertex position (simple average of neighbors).
#[allow(dead_code)]
pub fn smooth_boundary_vertex(vertex: [f32; 3], neighbor_a: [f32; 3], neighbor_b: [f32; 3]) -> [f32; 3] {
    [
        (vertex[0] * 2.0 + neighbor_a[0] + neighbor_b[0]) * 0.25,
        (vertex[1] * 2.0 + neighbor_a[1] + neighbor_b[1]) * 0.25,
        (vertex[2] * 2.0 + neighbor_a[2] + neighbor_b[2]) * 0.25,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_quad() -> (Vec<[f32; 3]>, Vec<[usize; 4]>) {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let faces = vec![[0, 1, 2, 3]];
        (verts, faces)
    }

    #[test]
    fn test_smooth_face_point() {
        let (v, f) = sample_quad();
        let fp = smooth_face_point(&v, &f[0]);
        assert!((fp[0] - 0.5).abs() < 1e-6);
        assert!((fp[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_edge_point() {
        let (v, _) = sample_quad();
        let ep = smooth_edge_point(&v, 0, 1);
        assert!((ep[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_subdivide() {
        let (v, f) = sample_quad();
        let result = smooth_subdivide(&v, &f);
        assert!(result.vertices.len() > v.len());
    }

    #[test]
    fn test_smooth_subdivide_n_zero() {
        let (v, f) = sample_quad();
        let result = smooth_subdivide_n(&v, &f, 0);
        assert_eq!(result.vertices.len(), v.len());
    }

    #[test]
    fn test_smooth_vertex_position() {
        let pos = smooth_vertex_position(
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
            4,
        );
        assert!(pos[0].is_finite());
    }

    #[test]
    fn test_smooth_subdiv_vertex_count() {
        assert_eq!(smooth_subdiv_vertex_count(4, 1, 4), 9);
    }

    #[test]
    fn test_smooth_subdiv_face_count() {
        assert_eq!(smooth_subdiv_face_count(1), 4);
        assert_eq!(smooth_subdiv_face_count(6), 24);
    }

    #[test]
    fn test_smooth_boundary_vertex() {
        let pos = smooth_boundary_vertex(
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
        );
        assert!((pos[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_subdivide_n_multiple() {
        let (v, f) = sample_quad();
        let result = smooth_subdivide_n(&v, &f, 2);
        assert!(result.vertices.len() > 10);
    }

    #[test]
    fn test_original_vertex_count_preserved() {
        let (v, f) = sample_quad();
        let result = smooth_subdivide(&v, &f);
        assert_eq!(result.original_vertex_count, 4);
    }
}
