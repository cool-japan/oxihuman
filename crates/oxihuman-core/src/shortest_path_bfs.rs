// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BFS shortest path on an unweighted directed/undirected graph.

use std::collections::VecDeque;

/// An unweighted adjacency-list graph.
pub struct BfsGraph {
    pub n: usize,
    pub adj: Vec<Vec<usize>>,
}

impl BfsGraph {
    pub fn new(n: usize) -> Self {
        BfsGraph {
            n,
            adj: vec![Vec::new(); n],
        }
    }
}

/// Create a new BFS graph with `n` nodes.
pub fn new_bfs_graph(n: usize) -> BfsGraph {
    BfsGraph::new(n)
}

/// Add a directed edge `u -> v`.
pub fn bfs_add_edge(g: &mut BfsGraph, u: usize, v: usize) {
    if u < g.n && v < g.n {
        g.adj[u].push(v);
    }
}

/// Add an undirected edge between `u` and `v`.
pub fn bfs_add_undirected(g: &mut BfsGraph, u: usize, v: usize) {
    bfs_add_edge(g, u, v);
    bfs_add_edge(g, v, u);
}

/// Return the shortest-path distance from `src` to every reachable node.
/// Unreachable nodes have distance `usize::MAX`.
pub fn bfs_distances(g: &BfsGraph, src: usize) -> Vec<usize> {
    let mut dist = vec![usize::MAX; g.n];
    if src >= g.n {
        return dist;
    }
    dist[src] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(src);
    while let Some(u) = queue.pop_front() {
        for &v in &g.adj[u] {
            if dist[v] == usize::MAX {
                dist[v] = dist[u] + 1;
                queue.push_back(v);
            }
        }
    }
    dist
}

/// Return the shortest path from `src` to `dst` as a vector of node indices,
/// or `None` if unreachable.
pub fn bfs_shortest_path(g: &BfsGraph, src: usize, dst: usize) -> Option<Vec<usize>> {
    if src >= g.n || dst >= g.n {
        return None;
    }
    let mut prev = vec![usize::MAX; g.n];
    let mut dist = vec![usize::MAX; g.n];
    dist[src] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(src);
    while let Some(u) = queue.pop_front() {
        if u == dst {
            break;
        }
        for &v in &g.adj[u] {
            if dist[v] == usize::MAX {
                dist[v] = dist[u] + 1;
                prev[v] = u;
                queue.push_back(v);
            }
        }
    }
    if dist[dst] == usize::MAX {
        return None;
    }
    /* reconstruct path by walking prev[] from dst back to src.
     * prev[src] == usize::MAX (never assigned), so the loop terminates naturally. */
    let mut path = Vec::new();
    let mut cur = dst;
    while cur != usize::MAX {
        path.push(cur);
        cur = prev[cur];
    }
    path.reverse();
    Some(path)
}

/// Return the hop count from `src` to `dst`, or `None` if unreachable.
pub fn bfs_distance(g: &BfsGraph, src: usize, dst: usize) -> Option<usize> {
    let dists = bfs_distances(g, src);
    if dst < g.n && dists[dst] != usize::MAX {
        Some(dists[dst])
    } else {
        None
    }
}

/// Return `true` if `dst` is reachable from `src`.
pub fn bfs_reachable(g: &BfsGraph, src: usize, dst: usize) -> bool {
    bfs_distance(g, src, dst).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        /* 0 -> 1 -> 2 -> 3 */
        let mut g = new_bfs_graph(4);
        bfs_add_edge(&mut g, 0, 1);
        bfs_add_edge(&mut g, 1, 2);
        bfs_add_edge(&mut g, 2, 3);
        assert_eq!(bfs_distance(&g, 0, 3), Some(3));
    }

    #[test]
    fn test_unreachable() {
        let mut g = new_bfs_graph(3);
        bfs_add_edge(&mut g, 0, 1);
        /* 2 is isolated */
        assert_eq!(bfs_distance(&g, 0, 2), None);
    }

    #[test]
    fn test_path_reconstruction() {
        let mut g = new_bfs_graph(4);
        bfs_add_edge(&mut g, 0, 1);
        bfs_add_edge(&mut g, 1, 3);
        bfs_add_edge(&mut g, 0, 2);
        bfs_add_edge(&mut g, 2, 3);
        let path = bfs_shortest_path(&g, 0, 3).expect("should succeed");
        assert_eq!(path[0], 0);
        assert_eq!(*path.last().expect("should succeed"), 3);
        assert_eq!(path.len(), 3); /* 0 -> 1 -> 3 or 0 -> 2 -> 3 */
    }

    #[test]
    fn test_same_src_dst() {
        let mut g = new_bfs_graph(3);
        bfs_add_edge(&mut g, 0, 1);
        assert_eq!(bfs_distance(&g, 1, 1), Some(0));
    }

    #[test]
    fn test_reachable_true() {
        let mut g = new_bfs_graph(5);
        for i in 0..4 {
            bfs_add_edge(&mut g, i, i + 1);
        }
        assert!(bfs_reachable(&g, 0, 4));
    }

    #[test]
    fn test_reachable_false() {
        let mut g = new_bfs_graph(3);
        bfs_add_edge(&mut g, 0, 1);
        assert!(!bfs_reachable(&g, 2, 0));
    }

    #[test]
    fn test_undirected_distances() {
        let mut g = new_bfs_graph(4);
        bfs_add_undirected(&mut g, 0, 1);
        bfs_add_undirected(&mut g, 1, 2);
        bfs_add_undirected(&mut g, 2, 3);
        let d = bfs_distances(&g, 3);
        assert_eq!(d[0], 3);
        assert_eq!(d[1], 2);
    }

    #[test]
    fn test_bfs_distances_length() {
        let g = new_bfs_graph(5);
        let d = bfs_distances(&g, 0);
        assert_eq!(d.len(), 5);
    }

    #[test]
    fn test_path_none_when_unreachable() {
        let g = new_bfs_graph(3);
        assert!(bfs_shortest_path(&g, 0, 2).is_none());
    }
}
