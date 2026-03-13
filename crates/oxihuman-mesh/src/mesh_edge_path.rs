// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// A path of edges through a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgePath {
    pub vertices: Vec<u32>,
    pub is_closed: bool,
}

#[allow(dead_code)]
impl EdgePath {
    /// Create a new open edge path.
    pub fn new(vertices: Vec<u32>) -> Self {
        Self { vertices, is_closed: false }
    }

    /// Create a closed edge path.
    pub fn closed(vertices: Vec<u32>) -> Self {
        Self { vertices, is_closed: true }
    }

    /// Number of edges in the path.
    pub fn edge_count(&self) -> usize {
        if self.vertices.len() < 2 { return 0; }
        if self.is_closed { self.vertices.len() } else { self.vertices.len() - 1 }
    }

    /// Total path length.
    pub fn length(&self, positions: &[[f32; 3]]) -> f32 {
        let mut total = 0.0f32;
        let n = self.vertices.len();
        if n < 2 { return 0.0; }
        let edge_n = if self.is_closed { n } else { n - 1 };
        for i in 0..edge_n {
            let a = self.vertices[i] as usize;
            let b = self.vertices[(i + 1) % n] as usize;
            let dx = positions[a][0] - positions[b][0];
            let dy = positions[a][1] - positions[b][1];
            let dz = positions[a][2] - positions[b][2];
            total += (dx * dx + dy * dy + dz * dz).sqrt();
        }
        total
    }

    /// Check if a vertex is on the path.
    pub fn contains(&self, vertex: u32) -> bool {
        self.vertices.contains(&vertex)
    }

    /// Reverse the path.
    pub fn reversed(&self) -> Self {
        let mut v = self.vertices.clone();
        v.reverse();
        Self { vertices: v, is_closed: self.is_closed }
    }
}

/// Find shortest path between two vertices using BFS on adjacency.
#[allow(dead_code)]
pub fn shortest_edge_path(indices: &[u32], vertex_count: usize, start: u32, end: u32) -> Option<EdgePath> {
    if start == end {
        return Some(EdgePath::new(vec![start]));
    }
    let mut adj: Vec<Vec<u32>> = vec![vec![]; vertex_count];
    for tri in indices.chunks_exact(3) {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            if !adj[a as usize].contains(&b) { adj[a as usize].push(b); }
            if !adj[b as usize].contains(&a) { adj[b as usize].push(a); }
        }
    }
    let mut visited = vec![false; vertex_count];
    let mut parent = vec![u32::MAX; vertex_count];
    let mut queue = std::collections::VecDeque::new();
    visited[start as usize] = true;
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        if current == end {
            let mut path = vec![end];
            let mut c = end;
            while c != start {
                c = parent[c as usize];
                path.push(c);
            }
            path.reverse();
            return Some(EdgePath::new(path));
        }
        for &neighbor in &adj[current as usize] {
            if !visited[neighbor as usize] {
                visited[neighbor as usize] = true;
                parent[neighbor as usize] = current;
                queue.push_back(neighbor);
            }
        }
    }
    None
}

/// Compute path midpoint.
#[allow(dead_code)]
pub fn path_midpoint(positions: &[[f32; 3]], path: &EdgePath) -> [f32; 3] {
    if path.vertices.is_empty() { return [0.0, 0.0, 0.0]; }
    let mut c = [0.0f32; 3];
    for &v in &path.vertices {
        let p = &positions[v as usize];
        c[0] += p[0]; c[1] += p[1]; c[2] += p[2];
    }
    let n = path.vertices.len() as f32;
    [c[0] / n, c[1] / n, c[2] / n]
}

/// Serialize edge path to JSON.
#[allow(dead_code)]
pub fn edge_path_to_json(path: &EdgePath) -> String {
    format!("{{\"vertices\":{},\"edges\":{},\"closed\":{}}}", path.vertices.len(), path.edge_count(), path.is_closed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_mesh() -> Vec<u32> { vec![0,1,2, 1,3,2] }

    #[test]
    fn test_path_new() {
        let p = EdgePath::new(vec![0, 1, 2]);
        assert_eq!(p.edge_count(), 2);
        assert!(!p.is_closed);
    }

    #[test]
    fn test_path_closed() {
        let p = EdgePath::closed(vec![0, 1, 2]);
        assert_eq!(p.edge_count(), 3);
    }

    #[test]
    fn test_path_length() {
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[2.0,0.0,0.0]];
        let p = EdgePath::new(vec![0, 1, 2]);
        assert!((p.length(&pos) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_contains() {
        let p = EdgePath::new(vec![0, 1, 2]);
        assert!(p.contains(1));
        assert!(!p.contains(5));
    }

    #[test]
    fn test_reversed() {
        let p = EdgePath::new(vec![0, 1, 2]);
        let r = p.reversed();
        assert_eq!(r.vertices, vec![2, 1, 0]);
    }

    #[test]
    fn test_shortest_path() {
        let path = shortest_edge_path(&line_mesh(), 4, 0, 3);
        assert!(path.is_some());
    }

    #[test]
    fn test_shortest_path_same() {
        let path = shortest_edge_path(&line_mesh(), 4, 0, 0);
        assert!(path.is_some());
        assert_eq!(path.expect("should succeed").vertices.len(), 1);
    }

    #[test]
    fn test_no_path() {
        let path = shortest_edge_path(&[0, 1, 2], 5, 0, 4);
        assert!(path.is_none());
    }

    #[test]
    fn test_midpoint() {
        let pos = vec![[0.0,0.0,0.0],[2.0,0.0,0.0]];
        let p = EdgePath::new(vec![0, 1]);
        let m = path_midpoint(&pos, &p);
        assert!((m[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let p = EdgePath::new(vec![0, 1, 2]);
        let json = edge_path_to_json(&p);
        assert!(json.contains("edges"));
    }
}
