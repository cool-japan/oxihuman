// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Greedy graph coloring algorithm.

#![allow(dead_code)]

use std::collections::HashSet;

/// An undirected graph represented as adjacency lists.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ColorGraph {
    /// Adjacency list: adj[v] = set of neighbors of v.
    adj: Vec<HashSet<usize>>,
}

/// Create a new graph with `n` vertices.
#[allow(dead_code)]
pub fn new_color_graph(n: usize) -> ColorGraph {
    ColorGraph {
        adj: vec![HashSet::new(); n],
    }
}

/// Add an undirected edge between `u` and `v`.
#[allow(dead_code)]
pub fn cg_add_edge(graph: &mut ColorGraph, u: usize, v: usize) {
    if u < graph.adj.len() && v < graph.adj.len() && u != v {
        graph.adj[u].insert(v);
        graph.adj[v].insert(u);
    }
}

/// Return number of vertices.
#[allow(dead_code)]
pub fn cg_vertex_count(graph: &ColorGraph) -> usize {
    graph.adj.len()
}

/// Return number of edges (undirected).
#[allow(dead_code)]
pub fn cg_edge_count(graph: &ColorGraph) -> usize {
    graph.adj.iter().map(|s| s.len()).sum::<usize>() / 2
}

/// Return degree of vertex `v`.
#[allow(dead_code)]
pub fn cg_degree(graph: &ColorGraph, v: usize) -> usize {
    graph.adj.get(v).map(|s| s.len()).unwrap_or(0)
}

/// Greedy graph coloring. Returns a Vec where `colors[v]` is the color index
/// assigned to vertex `v`. Colors are assigned in vertex order 0..n using the
/// smallest available color not used by any neighbor.
#[allow(dead_code)]
pub fn greedy_color(graph: &ColorGraph) -> Vec<usize> {
    let n = graph.adj.len();
    let mut colors = vec![usize::MAX; n];
    for v in 0..n {
        let mut used = HashSet::new();
        for &nb in &graph.adj[v] {
            if colors[nb] != usize::MAX {
                used.insert(colors[nb]);
            }
        }
        let mut c = 0;
        while used.contains(&c) {
            c += 1;
        }
        colors[v] = c;
    }
    colors
}

/// Return the number of distinct colors used in a coloring.
#[allow(dead_code)]
pub fn coloring_num_colors(colors: &[usize]) -> usize {
    colors.iter().copied().max().map(|m| m + 1).unwrap_or(0)
}

/// Verify that a coloring is valid: no two adjacent vertices share a color.
#[allow(dead_code)]
pub fn coloring_is_valid(graph: &ColorGraph, colors: &[usize]) -> bool {
    for (v, nbrs) in graph.adj.iter().enumerate() {
        for &nb in nbrs {
            if v < nb && colors[v] == colors[nb] {
                return false;
            }
        }
    }
    true
}

/// Return vertices assigned a specific color.
#[allow(dead_code)]
pub fn vertices_with_color(colors: &[usize], color: usize) -> Vec<usize> {
    colors
        .iter()
        .enumerate()
        .filter_map(|(v, &c)| if c == color { Some(v) } else { None })
        .collect()
}

/// Return the maximum degree in the graph.
#[allow(dead_code)]
pub fn cg_max_degree(graph: &ColorGraph) -> usize {
    graph.adj.iter().map(|s| s.len()).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = new_color_graph(0);
        assert_eq!(cg_vertex_count(&g), 0);
        assert_eq!(cg_edge_count(&g), 0);
    }

    #[test]
    fn test_single_vertex() {
        let g = new_color_graph(1);
        let colors = greedy_color(&g);
        assert_eq!(colors[0], 0);
        assert_eq!(coloring_num_colors(&colors), 1);
    }

    #[test]
    fn test_two_connected_vertices() {
        let mut g = new_color_graph(2);
        cg_add_edge(&mut g, 0, 1);
        let colors = greedy_color(&g);
        assert_ne!(colors[0], colors[1]);
        assert!(coloring_is_valid(&g, &colors));
    }

    #[test]
    fn test_triangle_requires_3_colors() {
        let mut g = new_color_graph(3);
        cg_add_edge(&mut g, 0, 1);
        cg_add_edge(&mut g, 1, 2);
        cg_add_edge(&mut g, 0, 2);
        let colors = greedy_color(&g);
        assert!(coloring_is_valid(&g, &colors));
        assert_eq!(coloring_num_colors(&colors), 3);
    }

    #[test]
    fn test_bipartite_2_colors() {
        let mut g = new_color_graph(4);
        cg_add_edge(&mut g, 0, 2);
        cg_add_edge(&mut g, 0, 3);
        cg_add_edge(&mut g, 1, 2);
        cg_add_edge(&mut g, 1, 3);
        let colors = greedy_color(&g);
        assert!(coloring_is_valid(&g, &colors));
        assert_eq!(coloring_num_colors(&colors), 2);
    }

    #[test]
    fn test_degree() {
        let mut g = new_color_graph(4);
        cg_add_edge(&mut g, 0, 1);
        cg_add_edge(&mut g, 0, 2);
        cg_add_edge(&mut g, 0, 3);
        assert_eq!(cg_degree(&g, 0), 3);
        assert_eq!(cg_degree(&g, 1), 1);
    }

    #[test]
    fn test_max_degree() {
        let mut g = new_color_graph(3);
        cg_add_edge(&mut g, 0, 1);
        cg_add_edge(&mut g, 0, 2);
        assert_eq!(cg_max_degree(&g), 2);
    }

    #[test]
    fn test_vertices_with_color() {
        let mut g = new_color_graph(3);
        cg_add_edge(&mut g, 0, 1);
        let colors = greedy_color(&g);
        let c0 = colors[0];
        let verts = vertices_with_color(&colors, c0);
        assert!(verts.contains(&0));
    }

    #[test]
    fn test_edge_count() {
        let mut g = new_color_graph(4);
        cg_add_edge(&mut g, 0, 1);
        cg_add_edge(&mut g, 2, 3);
        assert_eq!(cg_edge_count(&g), 2);
    }

    #[test]
    fn test_no_self_loops() {
        let mut g = new_color_graph(2);
        cg_add_edge(&mut g, 0, 0);
        assert_eq!(cg_edge_count(&g), 0);
    }
}
