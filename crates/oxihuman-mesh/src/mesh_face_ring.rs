// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face ring: find all faces adjacent to a given vertex (the face one-ring).

/// Face ring for a single vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceRing {
    pub vertex: usize,
    pub face_indices: Vec<usize>,
}

/// Compute the face ring for a given vertex.
#[allow(dead_code)]
pub fn face_ring_for_vertex(indices: &[u32], vertex: usize) -> FaceRing {
    let tri_count = indices.len() / 3;
    let mut faces = Vec::new();
    for t in 0..tri_count {
        if indices[t * 3] as usize == vertex
            || indices[t * 3 + 1] as usize == vertex
            || indices[t * 3 + 2] as usize == vertex
        {
            faces.push(t);
        }
    }
    FaceRing {
        vertex,
        face_indices: faces,
    }
}

/// Compute face rings for all vertices.
#[allow(dead_code)]
pub fn all_face_rings(vertex_count: usize, indices: &[u32]) -> Vec<FaceRing> {
    let tri_count = indices.len() / 3;
    let mut rings: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
    for t in 0..tri_count {
        for k in 0..3 {
            let vi = indices[t * 3 + k] as usize;
            if vi < vertex_count {
                rings[vi].push(t);
            }
        }
    }
    rings
        .into_iter()
        .enumerate()
        .map(|(vi, face_indices)| FaceRing {
            vertex: vi,
            face_indices,
        })
        .collect()
}

/// Number of faces in a ring.
#[allow(dead_code)]
pub fn ring_face_count(ring: &FaceRing) -> usize {
    ring.face_indices.len()
}

/// Check if a face is in the ring.
#[allow(dead_code)]
pub fn ring_contains_face(ring: &FaceRing, face: usize) -> bool {
    ring.face_indices.contains(&face)
}

/// Average face ring size across all vertices.
#[allow(dead_code)]
pub fn avg_ring_size(rings: &[FaceRing]) -> f32 {
    if rings.is_empty() {
        return 0.0;
    }
    let sum: usize = rings.iter().map(|r| r.face_indices.len()).sum();
    sum as f32 / rings.len() as f32
}

/// Maximum ring size.
#[allow(dead_code)]
pub fn max_ring_size(rings: &[FaceRing]) -> usize {
    rings
        .iter()
        .map(|r| r.face_indices.len())
        .max()
        .unwrap_or(0)
}

/// Find vertices with a given ring size.
#[allow(dead_code)]
pub fn vertices_with_ring_size(rings: &[FaceRing], size: usize) -> Vec<usize> {
    rings
        .iter()
        .filter(|r| r.face_indices.len() == size)
        .map(|r| r.vertex)
        .collect()
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn face_ring_to_json(ring: &FaceRing) -> String {
    format!(
        "{{\"vertex\":{},\"face_count\":{}}}",
        ring.vertex,
        ring.face_indices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> Vec<u32> {
        vec![0, 1, 2]
    }
    fn two_tris() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_face_ring_single_tri() {
        let ring = face_ring_for_vertex(&single_tri(), 0);
        assert_eq!(ring_face_count(&ring), 1);
    }

    #[test]
    fn test_face_ring_shared_vertex() {
        let ring = face_ring_for_vertex(&two_tris(), 1);
        assert_eq!(ring_face_count(&ring), 2);
    }

    #[test]
    fn test_all_face_rings() {
        let rings = all_face_rings(4, &two_tris());
        assert_eq!(rings.len(), 4);
    }

    #[test]
    fn test_ring_contains_face() {
        let ring = face_ring_for_vertex(&single_tri(), 0);
        assert!(ring_contains_face(&ring, 0));
        assert!(!ring_contains_face(&ring, 1));
    }

    #[test]
    fn test_avg_ring_size() {
        let rings = all_face_rings(3, &single_tri());
        let avg = avg_ring_size(&rings);
        assert!((avg - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_ring_size() {
        let rings = all_face_rings(4, &two_tris());
        let max = max_ring_size(&rings);
        assert_eq!(max, 2);
    }

    #[test]
    fn test_vertices_with_ring_size() {
        let rings = all_face_rings(4, &two_tris());
        let v = vertices_with_ring_size(&rings, 2);
        assert!(v.contains(&1));
        assert!(v.contains(&2));
    }

    #[test]
    fn test_empty() {
        let rings = all_face_rings(0, &[]);
        assert!(rings.is_empty());
    }

    #[test]
    fn test_face_ring_to_json() {
        let ring = FaceRing {
            vertex: 0,
            face_indices: vec![0, 1],
        };
        let json = face_ring_to_json(&ring);
        assert!(json.contains("\"face_count\":2"));
    }

    #[test]
    fn test_vertex_not_in_mesh() {
        let ring = face_ring_for_vertex(&single_tri(), 10);
        assert_eq!(ring_face_count(&ring), 0);
    }
}
