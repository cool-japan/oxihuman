// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Adjacency list representation for mesh vertices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdjacencyList {
    /// For each vertex, the list of neighbor vertex indices.
    neighbors: Vec<Vec<usize>>,
}

#[allow(dead_code)]
impl AdjacencyList {
    /// Build an adjacency list from triangle indices.
    pub fn from_triangles(vertex_count: usize, indices: &[u32]) -> Self {
        let mut neighbors: Vec<Vec<usize>> = vec![vec![]; vertex_count];
        for tri in indices.chunks_exact(3) {
            let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[a].contains(&c) {
                neighbors[a].push(c);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
            if !neighbors[b].contains(&c) {
                neighbors[b].push(c);
            }
            if !neighbors[c].contains(&a) {
                neighbors[c].push(a);
            }
            if !neighbors[c].contains(&b) {
                neighbors[c].push(b);
            }
        }
        Self { neighbors }
    }

    /// Number of vertices in the adjacency list.
    pub fn vertex_count(&self) -> usize {
        self.neighbors.len()
    }

    /// Get neighbors for a given vertex.
    pub fn neighbors_of(&self, vertex: usize) -> &[usize] {
        &self.neighbors[vertex]
    }

    /// Get the valence (degree) of a vertex.
    pub fn valence(&self, vertex: usize) -> usize {
        self.neighbors[vertex].len()
    }

    /// Check if two vertices are neighbors.
    pub fn are_neighbors(&self, a: usize, b: usize) -> bool {
        self.neighbors[a].contains(&b)
    }

    /// Total number of edges (each counted once).
    pub fn edge_count(&self) -> usize {
        let sum: usize = self.neighbors.iter().map(|n| n.len()).sum();
        sum / 2
    }

    /// Average valence across all vertices.
    pub fn average_valence(&self) -> f32 {
        if self.neighbors.is_empty() {
            return 0.0;
        }
        let sum: usize = self.neighbors.iter().map(|n| n.len()).sum();
        sum as f32 / self.neighbors.len() as f32
    }

    /// Collect all vertices with a specific valence.
    pub fn vertices_with_valence(&self, val: usize) -> Vec<usize> {
        self.neighbors
            .iter()
            .enumerate()
            .filter(|(_, n)| n.len() == val)
            .map(|(i, _)| i)
            .collect()
    }

    /// Check if the adjacency list is empty.
    pub fn is_empty(&self) -> bool {
        self.neighbors.is_empty()
    }
}

/// Build adjacency from triangle indices (convenience function).
#[allow(dead_code)]
pub fn build_adjacency_list(vertex_count: usize, indices: &[u32]) -> AdjacencyList {
    AdjacencyList::from_triangles(vertex_count, indices)
}

/// Count unique edges from adjacency.
#[allow(dead_code)]
pub fn count_edges(adj: &AdjacencyList) -> usize {
    adj.edge_count()
}

/// Compute maximum valence.
#[allow(dead_code)]
pub fn max_valence(adj: &AdjacencyList) -> usize {
    adj.neighbors.iter().map(|n| n.len()).max().unwrap_or(0)
}

/// Compute minimum valence among vertices with at least one neighbor.
#[allow(dead_code)]
pub fn min_valence(adj: &AdjacencyList) -> usize {
    adj.neighbors
        .iter()
        .filter(|n| !n.is_empty())
        .map(|n| n.len())
        .min()
        .unwrap_or(0)
}

/// Serialize adjacency list to JSON-like string.
#[allow(dead_code)]
pub fn adjacency_to_json(adj: &AdjacencyList) -> String {
    let mut s = String::from("{");
    for (i, nbrs) in adj.neighbors.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("\"{}\":[", i));
        for (j, n) in nbrs.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&n.to_string());
        }
        s.push(']');
    }
    s.push('}');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_single_triangle() {
        let adj = build_adjacency_list(3, &triangle_indices());
        assert_eq!(adj.vertex_count(), 3);
        assert_eq!(adj.valence(0), 2);
        assert_eq!(adj.valence(1), 2);
        assert_eq!(adj.valence(2), 2);
    }

    #[test]
    fn test_edge_count_triangle() {
        let adj = build_adjacency_list(3, &triangle_indices());
        assert_eq!(count_edges(&adj), 3);
    }

    #[test]
    fn test_neighbors() {
        let adj = build_adjacency_list(3, &triangle_indices());
        assert!(adj.are_neighbors(0, 1));
        assert!(adj.are_neighbors(1, 2));
        assert!(adj.are_neighbors(0, 2));
    }

    #[test]
    fn test_quad_valence() {
        let adj = build_adjacency_list(4, &quad_indices());
        assert_eq!(adj.valence(0), 3);
        assert_eq!(adj.valence(2), 3);
    }

    #[test]
    fn test_empty() {
        let adj = build_adjacency_list(0, &[]);
        assert!(adj.is_empty());
        assert_eq!(adj.edge_count(), 0);
    }

    #[test]
    fn test_average_valence() {
        let adj = build_adjacency_list(3, &triangle_indices());
        assert!((adj.average_valence() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_min_valence() {
        let adj = build_adjacency_list(4, &quad_indices());
        assert_eq!(max_valence(&adj), 3);
        assert_eq!(min_valence(&adj), 2);
    }

    #[test]
    fn test_vertices_with_valence() {
        let adj = build_adjacency_list(3, &triangle_indices());
        let v2 = adj.vertices_with_valence(2);
        assert_eq!(v2.len(), 3);
    }

    #[test]
    fn test_adjacency_to_json() {
        let adj = build_adjacency_list(3, &triangle_indices());
        let json = adjacency_to_json(&adj);
        assert!(json.contains("\"0\":["));
    }

    #[test]
    fn test_no_duplicate_neighbors() {
        let indices = vec![0, 1, 2, 0, 1, 2];
        let adj = build_adjacency_list(3, &indices);
        assert_eq!(adj.valence(0), 2);
    }
}
