// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct Graph {
    pub adjacency: Vec<Vec<usize>>,
    pub n_vertices: usize,
}

#[allow(dead_code)]
pub fn new_graph(n: usize) -> Graph {
    Graph { adjacency: vec![Vec::new(); n], n_vertices: n }
}

#[allow(dead_code)]
pub fn graph_add_edge(g: &mut Graph, u: usize, v: usize) {
    if u < g.n_vertices && v < g.n_vertices {
        if !g.adjacency[u].contains(&v) {
            g.adjacency[u].push(v);
        }
        if !g.adjacency[v].contains(&u) {
            g.adjacency[v].push(u);
        }
    }
}

#[allow(dead_code)]
pub fn graph_neighbors(g: &Graph, u: usize) -> &[usize] {
    if u < g.n_vertices { &g.adjacency[u] } else { &[] }
}

#[allow(dead_code)]
pub fn graph_vertex_count(g: &Graph) -> usize {
    g.n_vertices
}

#[allow(dead_code)]
pub fn graph_edge_count(g: &Graph) -> usize {
    let total: usize = g.adjacency.iter().map(|v| v.len()).sum();
    total / 2
}

#[allow(dead_code)]
pub fn graph_has_edge(g: &Graph, u: usize, v: usize) -> bool {
    if u < g.n_vertices {
        g.adjacency[u].contains(&v)
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn graph_bfs_reachable(g: &Graph, start: usize) -> Vec<usize> {
    if start >= g.n_vertices {
        return Vec::new();
    }
    let mut visited = vec![false; g.n_vertices];
    let mut queue = std::collections::VecDeque::new();
    let mut result = Vec::new();
    visited[start] = true;
    queue.push_back(start);
    while let Some(node) = queue.pop_front() {
        result.push(node);
        for &neighbor in &g.adjacency[node] {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_count() {
        let g = new_graph(5);
        assert_eq!(graph_vertex_count(&g), 5);
    }

    #[test]
    fn test_add_edge() {
        let mut g = new_graph(4);
        graph_add_edge(&mut g, 0, 1);
        assert!(graph_has_edge(&g, 0, 1));
        assert!(graph_has_edge(&g, 1, 0)); /* undirected */
    }

    #[test]
    fn test_edge_count() {
        let mut g = new_graph(4);
        graph_add_edge(&mut g, 0, 1);
        graph_add_edge(&mut g, 1, 2);
        assert_eq!(graph_edge_count(&g), 2);
    }

    #[test]
    fn test_has_edge_false() {
        let g = new_graph(3);
        assert!(!graph_has_edge(&g, 0, 2));
    }

    #[test]
    fn test_neighbors() {
        let mut g = new_graph(3);
        graph_add_edge(&mut g, 0, 1);
        graph_add_edge(&mut g, 0, 2);
        let neighbors = graph_neighbors(&g, 0);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_bfs_reachable_full() {
        let mut g = new_graph(4);
        graph_add_edge(&mut g, 0, 1);
        graph_add_edge(&mut g, 1, 2);
        graph_add_edge(&mut g, 2, 3);
        let reachable = graph_bfs_reachable(&g, 0);
        assert_eq!(reachable.len(), 4);
    }

    #[test]
    fn test_bfs_reachable_disconnected() {
        let mut g = new_graph(4);
        graph_add_edge(&mut g, 0, 1);
        let reachable = graph_bfs_reachable(&g, 0);
        assert_eq!(reachable.len(), 2);
    }

    #[test]
    fn test_no_duplicate_edges() {
        let mut g = new_graph(3);
        graph_add_edge(&mut g, 0, 1);
        graph_add_edge(&mut g, 0, 1);
        assert_eq!(graph_edge_count(&g), 1);
    }
}
