// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An ordered loop of vertices forming a closed or open edge ring.
pub struct BridgeEdgeLoop {
    pub vertices: Vec<[f32; 3]>,
}

pub fn new_bridge_edge_loop(verts: Vec<[f32; 3]>) -> BridgeEdgeLoop {
    BridgeEdgeLoop { vertices: verts }
}

pub fn bridge_loops_quads(a: &BridgeEdgeLoop, b: &BridgeEdgeLoop) -> Vec<[usize; 4]> {
    let na = a.vertices.len();
    let nb = b.vertices.len();
    let n = na.min(nb);
    if n < 2 {
        return Vec::new();
    }
    let mut faces = Vec::with_capacity(n);
    for i in 0..n {
        let next = (i + 1) % n;
        faces.push([i, next, nb + next, nb + i]);
    }
    faces
}

pub fn bridge_vertex_count_bl(a: &BridgeEdgeLoop, b: &BridgeEdgeLoop) -> usize {
    a.vertices.len() + b.vertices.len()
}

pub fn bridge_face_count_bl(loop_len: usize) -> usize {
    if loop_len < 2 {
        0
    } else {
        loop_len
    }
}

pub fn loop_centroid_bl(l: &BridgeEdgeLoop) -> [f32; 3] {
    let n = l.vertices.len();
    if n == 0 {
        return [0.0; 3];
    }
    let sum = l.vertices.iter().fold([0.0f32; 3], |acc, v| {
        [acc[0] + v[0], acc[1] + v[1], acc[2] + v[2]]
    });
    [sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32]
}

pub fn loop_perimeter_bl(l: &BridgeEdgeLoop) -> f32 {
    let n = l.vertices.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n {
        let a = l.vertices[i];
        let b = l.vertices[(i + 1) % n];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bridge_edge_loop() {
        /* basic construction */
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let l = new_bridge_edge_loop(verts);
        assert_eq!(l.vertices.len(), 3);
    }

    #[test]
    fn test_bridge_loops_quads_count() {
        let a = new_bridge_edge_loop(vec![[0.0; 3]; 4]);
        let b = new_bridge_edge_loop(vec![[1.0; 3]; 4]);
        let faces = bridge_loops_quads(&a, &b);
        assert_eq!(faces.len(), 4);
    }

    #[test]
    fn test_bridge_vertex_count() {
        let a = new_bridge_edge_loop(vec![[0.0; 3]; 5]);
        let b = new_bridge_edge_loop(vec![[0.0; 3]; 3]);
        assert_eq!(bridge_vertex_count_bl(&a, &b), 8);
    }

    #[test]
    fn test_bridge_face_count() {
        assert_eq!(bridge_face_count_bl(6), 6);
        assert_eq!(bridge_face_count_bl(1), 0);
    }

    #[test]
    fn test_loop_centroid_bl() {
        let verts = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let l = new_bridge_edge_loop(verts);
        let c = loop_centroid_bl(&l);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_loop_perimeter_bl() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let l = new_bridge_edge_loop(verts);
        let p = loop_perimeter_bl(&l);
        assert!((p - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_bridge_loops_empty() {
        let a = new_bridge_edge_loop(vec![]);
        let b = new_bridge_edge_loop(vec![]);
        let faces = bridge_loops_quads(&a, &b);
        assert!(faces.is_empty());
    }
}
