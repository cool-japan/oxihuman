// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Detect and manipulate border (boundary) loops on triangle meshes.

use std::collections::HashMap;

/// A boundary loop represented as an ordered list of vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BorderLoop {
    pub vertices: Vec<u32>,
    pub is_closed: bool,
}

/// Find all boundary edges (edges shared by exactly one triangle).
#[allow(dead_code)]
pub fn find_boundary_edges(indices: &[u32]) -> Vec<(u32, u32)> {
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    let tc = indices.len() / 3;
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count.into_iter().filter(|&(_, c)| c == 1).map(|(e, _)| e).collect()
}

/// Build adjacency from boundary edges.
#[allow(dead_code)]
pub fn build_boundary_adjacency(edges: &[(u32, u32)]) -> HashMap<u32, Vec<u32>> {
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    adj
}

/// Trace a single border loop starting from a vertex.
#[allow(dead_code)]
pub fn trace_border_loop(adj: &HashMap<u32, Vec<u32>>, start: u32) -> BorderLoop {
    let mut visited = std::collections::HashSet::new();
    let mut loop_verts = Vec::new();
    let mut current = start;
    loop {
        if visited.contains(&current) {
            break;
        }
        visited.insert(current);
        loop_verts.push(current);
        let neighbors = match adj.get(&current) {
            Some(n) => n,
            None => break,
        };
        let next = neighbors.iter().find(|&&v| !visited.contains(&v));
        match next {
            Some(&v) => current = v,
            None => break,
        }
    }
    let is_closed = loop_verts.len() > 2 && {
        let first = loop_verts[0];
        adj.get(&loop_verts[loop_verts.len() - 1])
            .map_or(false, |n| n.contains(&first))
    };
    BorderLoop { vertices: loop_verts, is_closed }
}

/// Detect all border loops in a mesh.
#[allow(dead_code)]
pub fn detect_border_loops(indices: &[u32]) -> Vec<BorderLoop> {
    let edges = find_boundary_edges(indices);
    if edges.is_empty() {
        return Vec::new();
    }
    let adj = build_boundary_adjacency(&edges);
    let mut visited = std::collections::HashSet::new();
    let mut loops = Vec::new();
    for &v in adj.keys() {
        if !visited.contains(&v) {
            let bl = trace_border_loop(&adj, v);
            for &u in &bl.vertices {
                visited.insert(u);
            }
            loops.push(bl);
        }
    }
    loops
}

/// Count border loops.
#[allow(dead_code)]
pub fn border_loop_count(indices: &[u32]) -> usize {
    detect_border_loops(indices).len()
}

/// Total boundary edge count.
#[allow(dead_code)]
pub fn boundary_edge_count(indices: &[u32]) -> usize {
    find_boundary_edges(indices).len()
}

/// Length of a border loop in 3D.
#[allow(dead_code)]
pub fn border_loop_length(positions: &[[f32; 3]], bl: &BorderLoop) -> f32 {
    if bl.vertices.len() < 2 {
        return 0.0;
    }
    let mut len = 0.0f32;
    for i in 0..bl.vertices.len() - 1 {
        let a = positions[bl.vertices[i] as usize];
        let b = positions[bl.vertices[i + 1] as usize];
        len += dist3(a, b);
    }
    if bl.is_closed {
        let a = positions[bl.vertices[bl.vertices.len() - 1] as usize];
        let b = positions[bl.vertices[0] as usize];
        len += dist3(a, b);
    }
    len
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_quad() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_find_boundary_edges_quad() {
        let idx = open_quad();
        let edges = find_boundary_edges(&idx);
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_detect_loops_quad() {
        let idx = open_quad();
        let loops = detect_border_loops(&idx);
        assert!(!loops.is_empty());
    }

    #[test]
    fn test_border_loop_count() {
        let idx = open_quad();
        assert!(border_loop_count(&idx) >= 1);
    }

    #[test]
    fn test_boundary_edge_count() {
        let idx = open_quad();
        assert!(boundary_edge_count(&idx) >= 3);
    }

    #[test]
    fn test_closed_mesh_no_border() {
        // tetrahedron: all edges shared by 2 triangles
        let idx = vec![0u32, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2];
        assert_eq!(border_loop_count(&idx), 0);
    }

    #[test]
    fn test_loop_length() {
        let pos = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        ];
        let idx = open_quad();
        let loops = detect_border_loops(&idx);
        if !loops.is_empty() {
            let len = border_loop_length(&pos, &loops[0]);
            assert!(len > 0.0);
        }
    }

    #[test]
    fn test_build_adjacency() {
        let edges = vec![(0u32, 1), (1, 2), (2, 0)];
        let adj = build_boundary_adjacency(&edges);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn test_trace_loop() {
        let edges = vec![(0u32, 1), (1, 2), (2, 0)];
        let adj = build_boundary_adjacency(&edges);
        let bl = trace_border_loop(&adj, 0);
        assert_eq!(bl.vertices.len(), 3);
        assert!(bl.is_closed);
    }

    #[test]
    fn test_empty_mesh() {
        let idx: Vec<u32> = Vec::new();
        assert_eq!(border_loop_count(&idx), 0);
    }

    #[test]
    fn test_single_triangle_boundary() {
        let idx = vec![0u32, 1, 2];
        assert_eq!(boundary_edge_count(&idx), 3);
    }

}
