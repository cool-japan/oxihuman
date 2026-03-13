// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh traversal: walking along edges, face-to-face adjacency traversal.

use std::collections::{HashMap, HashSet, VecDeque};

/// Walk result: ordered list of face indices visited.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshWalkResult {
    pub visited_faces: Vec<usize>,
    pub visited_vertices: Vec<u32>,
}

/// Build face adjacency (faces sharing an edge).
#[allow(dead_code)]
pub fn face_adjacency(indices: &[u32]) -> Vec<Vec<usize>> {
    let tc = indices.len() / 3;
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(t);
        }
    }
    let mut adj = vec![Vec::new(); tc];
    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }
    adj
}

/// Breadth-first walk across faces starting from a seed face.
#[allow(dead_code)]
pub fn bfs_face_walk(indices: &[u32], start_face: usize) -> MeshWalkResult {
    let adj = face_adjacency(indices);
    let tc = indices.len() / 3;
    if start_face >= tc {
        return MeshWalkResult { visited_faces: Vec::new(), visited_vertices: Vec::new() };
    }
    let mut visited = vec![false; tc];
    let mut queue = VecDeque::new();
    let mut result_faces = Vec::new();
    let mut result_verts = HashSet::new();
    queue.push_back(start_face);
    visited[start_face] = true;
    while let Some(f) = queue.pop_front() {
        result_faces.push(f);
        for k in 0..3 {
            result_verts.insert(indices[f * 3 + k]);
        }
        for &neighbor in &adj[f] {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }
    let mut verts: Vec<u32> = result_verts.into_iter().collect();
    verts.sort();
    MeshWalkResult { visited_faces: result_faces, visited_vertices: verts }
}

/// Depth-first vertex walk.
#[allow(dead_code)]
pub fn dfs_vertex_walk(indices: &[u32], n_verts: usize, start: u32) -> Vec<u32> {
    let mut adj: Vec<HashSet<u32>> = vec![HashSet::new(); n_verts];
    let tc = indices.len() / 3;
    for t in 0..tc {
        for k in 0..3 {
            let a = indices[t * 3 + k];
            let b = indices[t * 3 + (k + 1) % 3];
            adj[a as usize].insert(b);
            adj[b as usize].insert(a);
        }
    }
    let mut visited = HashSet::new();
    let mut stack = vec![start];
    let mut order = Vec::new();
    while let Some(v) = stack.pop() {
        if visited.contains(&v) { continue; }
        visited.insert(v);
        order.push(v);
        for &nb in &adj[v as usize] {
            if !visited.contains(&nb) {
                stack.push(nb);
            }
        }
    }
    order
}

/// Count connected face components.
#[allow(dead_code)]
pub fn face_component_count(indices: &[u32]) -> usize {
    let adj = face_adjacency(indices);
    let tc = indices.len() / 3;
    let mut visited = vec![false; tc];
    let mut count = 0;
    for start in 0..tc {
        if visited[start] { continue; }
        count += 1;
        let mut stack = vec![start];
        while let Some(f) = stack.pop() {
            if visited[f] { continue; }
            visited[f] = true;
            for &nb in &adj[f] {
                if !visited[nb] { stack.push(nb); }
            }
        }
    }
    count
}

/// Walk face count.
#[allow(dead_code)]
pub fn walk_face_count(result: &MeshWalkResult) -> usize {
    result.visited_faces.len()
}

/// Walk vertex count.
#[allow(dead_code)]
pub fn walk_vertex_count(result: &MeshWalkResult) -> usize {
    result.visited_vertices.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_adjacency_quad() {
        let adj = face_adjacency(&[0, 1, 2, 0, 2, 3]);
        assert_eq!(adj.len(), 2);
        assert!(!adj[0].is_empty());
    }

    #[test]
    fn test_bfs_single_tri() {
        let result = bfs_face_walk(&[0, 1, 2], 0);
        assert_eq!(walk_face_count(&result), 1);
        assert_eq!(walk_vertex_count(&result), 3);
    }

    #[test]
    fn test_bfs_quad() {
        let result = bfs_face_walk(&[0, 1, 2, 0, 2, 3], 0);
        assert_eq!(walk_face_count(&result), 2);
    }

    #[test]
    fn test_dfs_vertex() {
        let order = dfs_vertex_walk(&[0, 1, 2], 3, 0);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_face_component_single() {
        assert_eq!(face_component_count(&[0, 1, 2, 0, 2, 3]), 1);
    }

    #[test]
    fn test_face_component_two() {
        // Two disconnected triangles
        assert_eq!(face_component_count(&[0, 1, 2, 3, 4, 5]), 2);
    }

    #[test]
    fn test_empty_walk() {
        let result = bfs_face_walk(&[], 0);
        assert_eq!(walk_face_count(&result), 0);
    }

    #[test]
    fn test_out_of_range_start() {
        let result = bfs_face_walk(&[0, 1, 2], 999);
        assert_eq!(walk_face_count(&result), 0);
    }

    #[test]
    fn test_dfs_disconnected() {
        let order = dfs_vertex_walk(&[0, 1, 2, 3, 4, 5], 6, 0);
        assert_eq!(order.len(), 3); // Only reaches first component
    }

    #[test]
    fn test_walk_vertex_sorted() {
        let result = bfs_face_walk(&[2, 1, 0], 0);
        let verts = &result.visited_vertices;
        for i in 1..verts.len() {
            assert!(verts[i] >= verts[i - 1]);
        }
    }

}
