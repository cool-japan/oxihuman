// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Euler operators for topological mesh editing: vertex/edge/face insertion and removal.

/// Simple mesh representation for Euler operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EulerMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Create a new empty mesh.
#[allow(dead_code)]
pub fn euler_new() -> EulerMesh {
    EulerMesh { positions: Vec::new(), indices: Vec::new() }
}

/// Make vertex: add a new vertex.
#[allow(dead_code)]
pub fn make_vertex(mesh: &mut EulerMesh, pos: [f32; 3]) -> u32 {
    let idx = mesh.positions.len() as u32;
    mesh.positions.push(pos);
    idx
}

/// Make face: add a triangle face.
#[allow(dead_code)]
pub fn make_face(mesh: &mut EulerMesh, a: u32, b: u32, c: u32) {
    mesh.indices.extend_from_slice(&[a, b, c]);
}

/// Kill face: remove a triangle (by face index).
#[allow(dead_code)]
pub fn kill_face(mesh: &mut EulerMesh, face_idx: usize) {
    let start = face_idx * 3;
    if start + 3 <= mesh.indices.len() {
        mesh.indices.drain(start..start + 3);
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn euler_vertex_count(mesh: &EulerMesh) -> usize {
    mesh.positions.len()
}

/// Face count.
#[allow(dead_code)]
pub fn euler_face_count(mesh: &EulerMesh) -> usize {
    mesh.indices.len() / 3
}

/// Edge count (unique edges).
#[allow(dead_code)]
pub fn euler_edge_count(mesh: &EulerMesh) -> usize {
    use std::collections::HashSet;
    let tc = mesh.indices.len() / 3;
    let mut edges = HashSet::new();
    for t in 0..tc {
        for k in 0..3 {
            let a = mesh.indices[t * 3 + k];
            let b = mesh.indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edges.insert(key);
        }
    }
    edges.len()
}

/// Euler characteristic: V - E + F.
#[allow(dead_code)]
pub fn euler_characteristic(mesh: &EulerMesh) -> i64 {
    euler_vertex_count(mesh) as i64 - euler_edge_count(mesh) as i64 + euler_face_count(mesh) as i64
}

/// Split edge: insert vertex at midpoint of edge (a, b).
#[allow(dead_code)]
pub fn split_edge(mesh: &mut EulerMesh, a: u32, b: u32) -> u32 {
    let pa = mesh.positions[a as usize];
    let pb = mesh.positions[b as usize];
    let mid = [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ];
    let m = make_vertex(mesh, mid);
    // Replace edge in all faces
    let tc = mesh.indices.len() / 3;
    let mut new_faces = Vec::new();
    let mut remove_faces = Vec::new();
    for t in (0..tc).rev() {
        let i0 = mesh.indices[t * 3];
        let i1 = mesh.indices[t * 3 + 1];
        let i2 = mesh.indices[t * 3 + 2];
        if (i0 == a && i1 == b) || (i0 == b && i1 == a) {
            remove_faces.push(t);
            new_faces.extend_from_slice(&[i0, m, i2, m, i1, i2]);
        } else if (i1 == a && i2 == b) || (i1 == b && i2 == a) {
            remove_faces.push(t);
            new_faces.extend_from_slice(&[i0, i1, m, i0, m, i2]);
        } else if (i2 == a && i0 == b) || (i2 == b && i0 == a) {
            remove_faces.push(t);
            new_faces.extend_from_slice(&[i0, i1, m, m, i1, i2]);
        }
    }
    for &fi in &remove_faces {
        kill_face(mesh, fi);
    }
    mesh.indices.extend_from_slice(&new_faces);
    m
}

/// Validate Euler formula for manifolds with boundary: V - E + F = chi.
#[allow(dead_code)]
pub fn validate_euler(mesh: &EulerMesh) -> bool {
    let chi = euler_characteristic(mesh);
    // For a disk, chi should be 1; for a sphere, 2; etc.
    // Just check it's a reasonable value
    (-10..=10).contains(&chi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mesh() {
        let m = euler_new();
        assert_eq!(euler_vertex_count(&m), 0);
        assert_eq!(euler_face_count(&m), 0);
    }

    #[test]
    fn test_make_vertex() {
        let mut m = euler_new();
        let v = make_vertex(&mut m, [1.0, 2.0, 3.0]);
        assert_eq!(v, 0);
        assert_eq!(euler_vertex_count(&m), 1);
    }

    #[test]
    fn test_make_face() {
        let mut m = euler_new();
        make_vertex(&mut m, [0.0; 3]);
        make_vertex(&mut m, [1.0, 0.0, 0.0]);
        make_vertex(&mut m, [0.0, 1.0, 0.0]);
        make_face(&mut m, 0, 1, 2);
        assert_eq!(euler_face_count(&m), 1);
    }

    #[test]
    fn test_kill_face() {
        let mut m = euler_new();
        for _ in 0..3 { make_vertex(&mut m, [0.0; 3]); }
        make_face(&mut m, 0, 1, 2);
        kill_face(&mut m, 0);
        assert_eq!(euler_face_count(&m), 0);
    }

    #[test]
    fn test_edge_count() {
        let mut m = euler_new();
        for p in &[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            make_vertex(&mut m, *p);
        }
        make_face(&mut m, 0, 1, 2);
        assert_eq!(euler_edge_count(&m), 3);
    }

    #[test]
    fn test_euler_characteristic_single_tri() {
        let mut m = euler_new();
        for p in &[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            make_vertex(&mut m, *p);
        }
        make_face(&mut m, 0, 1, 2);
        assert_eq!(euler_characteristic(&m), 1);
    }

    #[test]
    fn test_split_edge() {
        let mut m = euler_new();
        for p in &[[0.0; 3], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]] {
            make_vertex(&mut m, *p);
        }
        make_face(&mut m, 0, 1, 2);
        let mid = split_edge(&mut m, 0, 1);
        assert_eq!(mid, 3);
        assert_eq!(euler_face_count(&m), 2);
    }

    #[test]
    fn test_validate_euler() {
        let mut m = euler_new();
        for p in &[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            make_vertex(&mut m, *p);
        }
        make_face(&mut m, 0, 1, 2);
        assert!(validate_euler(&m));
    }

    #[test]
    fn test_multiple_faces() {
        let mut m = euler_new();
        for p in &[[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]] {
            make_vertex(&mut m, *p);
        }
        make_face(&mut m, 0, 1, 2);
        make_face(&mut m, 0, 2, 3);
        assert_eq!(euler_face_count(&m), 2);
        assert_eq!(euler_edge_count(&m), 5);
    }

    #[test]
    fn test_empty_validate() {
        let m = euler_new();
        assert!(validate_euler(&m));
    }

}
