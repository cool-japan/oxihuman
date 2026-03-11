// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! K-ring neighbourhood queries on triangle meshes.

/// K-ring result for a single query vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KRingResult {
    pub center: u32,
    pub k: usize,
    pub vertices: Vec<u32>,
}

/// Build vertex adjacency list from triangle indices.
#[allow(dead_code)]
pub fn build_vertex_adjacency(indices: &[u32], vertex_count: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); vertex_count];
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let i0 = indices[f * 3];
        let i1 = indices[f * 3 + 1];
        let i2 = indices[f * 3 + 2];
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0), (i1, i0), (i2, i1), (i0, i2)] {
            let a = a as usize;
            let b_u = b;
            if !adj[a].contains(&b_u) {
                adj[a].push(b_u);
            }
        }
    }
    adj
}

/// Compute the k-ring neighbourhood of a vertex (BFS up to k hops).
#[allow(dead_code)]
pub fn k_ring(adj: &[Vec<u32>], center: u32, k: usize) -> KRingResult {
    use std::collections::VecDeque;
    let mut visited = vec![false; adj.len()];
    let mut queue = VecDeque::new();
    let mut vertices = Vec::new();
    if (center as usize) >= adj.len() {
        return KRingResult {
            center,
            k,
            vertices,
        };
    }
    visited[center as usize] = true;
    queue.push_back((center, 0usize));
    while let Some((v, depth)) = queue.pop_front() {
        if depth > 0 {
            vertices.push(v);
        }
        if depth < k {
            for &nb in &adj[v as usize] {
                if !visited[nb as usize] {
                    visited[nb as usize] = true;
                    queue.push_back((nb, depth + 1));
                }
            }
        }
    }
    KRingResult {
        center,
        k,
        vertices,
    }
}

/// 1-ring: immediate neighbours.
#[allow(dead_code)]
pub fn one_ring(adj: &[Vec<u32>], center: u32) -> KRingResult {
    k_ring(adj, center, 1)
}

/// Ring size (vertex count in ring).
#[allow(dead_code)]
pub fn ring_size(r: &KRingResult) -> usize {
    r.vertices.len()
}

/// Check if a vertex is in the k-ring.
#[allow(dead_code)]
pub fn ring_contains(r: &KRingResult, v: u32) -> bool {
    r.vertices.contains(&v)
}

/// Average k-ring size over all vertices.
#[allow(dead_code)]
pub fn avg_ring_size_all(adj: &[Vec<u32>], k: usize) -> f32 {
    if adj.is_empty() {
        return 0.0;
    }
    let total: usize = (0..adj.len())
        .map(|v| ring_size(&k_ring(adj, v as u32, k)))
        .sum();
    total as f32 / adj.len() as f32
}

/// Export to JSON.
#[allow(dead_code)]
pub fn k_ring_to_json(r: &KRingResult) -> String {
    format!(
        "{{\"center\":{},\"k\":{},\"ring_size\":{}}}",
        r.center,
        r.k,
        r.vertices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_strip_indices() -> (Vec<u32>, usize) {
        // 4 vertices, 2 triangles in a strip
        (vec![0, 1, 2, 1, 3, 2], 4)
    }

    #[test]
    fn test_build_adjacency_empty() {
        let adj = build_vertex_adjacency(&[], 0);
        assert!(adj.is_empty());
    }

    #[test]
    fn test_build_adjacency_triangle() {
        let adj = build_vertex_adjacency(&[0, 1, 2], 3);
        assert_eq!(adj.len(), 3);
        assert!(!adj[0].is_empty());
    }

    #[test]
    fn test_k_ring_zero() {
        let (idx, vc) = triangle_strip_indices();
        let adj = build_vertex_adjacency(&idx, vc);
        let r = k_ring(&adj, 0, 0);
        assert_eq!(ring_size(&r), 0);
    }

    #[test]
    fn test_one_ring() {
        let (idx, vc) = triangle_strip_indices();
        let adj = build_vertex_adjacency(&idx, vc);
        let r = one_ring(&adj, 1);
        assert!(ring_size(&r) >= 2);
    }

    #[test]
    fn test_ring_contains() {
        let (idx, vc) = triangle_strip_indices();
        let adj = build_vertex_adjacency(&idx, vc);
        let r = one_ring(&adj, 0);
        assert!(ring_contains(&r, 1));
        assert!(!ring_contains(&r, 3));
    }

    #[test]
    fn test_k_ring_invalid_center() {
        let adj = build_vertex_adjacency(&[0, 1, 2], 3);
        let r = k_ring(&adj, 99, 2);
        assert_eq!(ring_size(&r), 0);
    }

    #[test]
    fn test_avg_ring_size_all() {
        let (idx, vc) = triangle_strip_indices();
        let adj = build_vertex_adjacency(&idx, vc);
        let avg = avg_ring_size_all(&adj, 1);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_k_ring_to_json() {
        let r = KRingResult {
            center: 0,
            k: 1,
            vertices: vec![1, 2],
        };
        let j = k_ring_to_json(&r);
        assert!(j.contains("\"center\":0"));
    }

    #[test]
    fn test_two_ring_larger_than_one_ring() {
        let (idx, vc) = triangle_strip_indices();
        let adj = build_vertex_adjacency(&idx, vc);
        let r1 = k_ring(&adj, 0, 1);
        let r2 = k_ring(&adj, 0, 2);
        assert!(ring_size(&r2) >= ring_size(&r1));
    }

    #[test]
    fn test_empty_adj_avg_ring() {
        let avg = avg_ring_size_all(&[], 1);
        assert!((avg).abs() < 1e-9);
    }
}
