// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bipartite matching stub — Hopcroft-Karp inspired BFS+DFS approach.

/// A bipartite graph with `left_n` left nodes and `right_n` right nodes.
pub struct BipartiteGraph {
    pub left_n: usize,
    pub right_n: usize,
    /// `adj[u]` = list of right nodes adjacent to left node u.
    pub adj: Vec<Vec<usize>>,
}

impl BipartiteGraph {
    pub fn new(left_n: usize, right_n: usize) -> Self {
        BipartiteGraph {
            left_n,
            right_n,
            adj: vec![Vec::new(); left_n],
        }
    }
}

/// Create a new bipartite graph.
pub fn new_bipartite(left_n: usize, right_n: usize) -> BipartiteGraph {
    BipartiteGraph::new(left_n, right_n)
}

/// Add an edge from left node `u` to right node `v`.
pub fn bip_add_edge(g: &mut BipartiteGraph, u: usize, v: usize) {
    if u < g.left_n && v < g.right_n {
        g.adj[u].push(v);
    }
}

/// Run augmenting-path bipartite matching. Returns the matching as
/// a `Vec` of (left, right) pairs.
pub fn bipartite_matching(g: &BipartiteGraph) -> Vec<(usize, usize)> {
    /* match_right[v] = left node matched to right v, or MAX */
    let mut match_right = vec![usize::MAX; g.right_n];
    /* match_left[u] = right node matched to left u, or MAX */
    let mut match_left = vec![usize::MAX; g.left_n];
    let mut result_count = 0;

    for u in 0..g.left_n {
        /* DFS for augmenting path */
        let mut visited = vec![false; g.right_n];
        if augment(g, u, &mut visited, &mut match_right, &mut match_left) {
            result_count += 1;
        }
    }

    let _ = result_count;
    /* collect matched pairs */
    (0..g.left_n)
        .filter(|&u| match_left[u] != usize::MAX)
        .map(|u| (u, match_left[u]))
        .collect()
}

fn augment(
    g: &BipartiteGraph,
    u: usize,
    visited: &mut Vec<bool>,
    match_right: &mut Vec<usize>,
    match_left: &mut Vec<usize>,
) -> bool {
    for &v in &g.adj[u] {
        if !visited[v] {
            visited[v] = true;
            let mr = match_right[v];
            if mr == usize::MAX || augment(g, mr, visited, match_right, match_left) {
                match_right[v] = u;
                match_left[u] = v;
                return true;
            }
        }
    }
    false
}

/// Return the maximum matching size.
pub fn max_matching_size(g: &BipartiteGraph) -> usize {
    bipartite_matching(g).len()
}

/// Return `true` if a perfect matching exists (all left nodes matched).
pub fn has_perfect_matching(g: &BipartiteGraph) -> bool {
    max_matching_size(g) == g.left_n
}

/// Return the number of edges in the bipartite graph.
pub fn bip_edge_count(g: &BipartiteGraph) -> usize {
    g.adj.iter().map(|v| v.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_matching_3x3() {
        let mut g = new_bipartite(3, 3);
        bip_add_edge(&mut g, 0, 0);
        bip_add_edge(&mut g, 1, 1);
        bip_add_edge(&mut g, 2, 2);
        assert!(has_perfect_matching(&g));
        assert_eq!(max_matching_size(&g), 3);
    }

    #[test]
    fn test_no_edges_no_matching() {
        let g = new_bipartite(3, 3);
        assert_eq!(max_matching_size(&g), 0);
    }

    #[test]
    fn test_partial_matching() {
        let mut g = new_bipartite(3, 2);
        bip_add_edge(&mut g, 0, 0);
        bip_add_edge(&mut g, 1, 0);
        bip_add_edge(&mut g, 2, 1);
        /* only 2 right nodes, so max matching <= 2 */
        assert_eq!(max_matching_size(&g), 2);
    }

    #[test]
    fn test_augmenting_path_used() {
        /* 0-0; 1-0,1; 2-1 — should find size 3 via augmenting */
        let mut g = new_bipartite(3, 3);
        bip_add_edge(&mut g, 0, 0);
        bip_add_edge(&mut g, 1, 0);
        bip_add_edge(&mut g, 1, 1);
        bip_add_edge(&mut g, 2, 1);
        bip_add_edge(&mut g, 2, 2);
        assert_eq!(max_matching_size(&g), 3);
    }

    #[test]
    fn test_edge_count() {
        let mut g = new_bipartite(2, 2);
        bip_add_edge(&mut g, 0, 0);
        bip_add_edge(&mut g, 0, 1);
        assert_eq!(bip_edge_count(&g), 2);
    }

    #[test]
    fn test_single_edge() {
        let mut g = new_bipartite(1, 1);
        bip_add_edge(&mut g, 0, 0);
        assert_eq!(max_matching_size(&g), 1);
    }

    #[test]
    fn test_empty_graph() {
        let g = new_bipartite(0, 0);
        assert_eq!(max_matching_size(&g), 0);
    }

    #[test]
    fn test_matching_pairs() {
        let mut g = new_bipartite(2, 2);
        bip_add_edge(&mut g, 0, 0);
        bip_add_edge(&mut g, 1, 1);
        let pairs = bipartite_matching(&g);
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_no_perfect_matching() {
        let mut g = new_bipartite(3, 1);
        bip_add_edge(&mut g, 0, 0);
        bip_add_edge(&mut g, 1, 0);
        bip_add_edge(&mut g, 2, 0);
        /* only 1 right node, no perfect matching for 3 left */
        assert!(!has_perfect_matching(&g));
    }
}
