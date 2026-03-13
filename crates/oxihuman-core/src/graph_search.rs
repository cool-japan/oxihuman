// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BFS/DFS graph search on adjacency list graphs.

use std::collections::{HashSet, VecDeque};

/// An adjacency-list directed graph.
pub struct AdjGraph {
    adj: Vec<Vec<usize>>,
}

/// Construct a new graph with `n` vertices.
pub fn new_adj_graph(n: usize) -> AdjGraph {
    AdjGraph {
        adj: vec![Vec::new(); n],
    }
}

impl AdjGraph {
    /// Add a directed edge from `u` to `v`.
    pub fn add_edge(&mut self, u: usize, v: usize) {
        if u < self.adj.len() {
            self.adj[u].push(v);
        }
    }

    /// Add an undirected edge.
    pub fn add_undirected(&mut self, u: usize, v: usize) {
        self.add_edge(u, v);
        self.add_edge(v, u);
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.adj.len()
    }

    /// Number of (directed) edges.
    pub fn edge_count(&self) -> usize {
        self.adj.iter().map(|v| v.len()).sum()
    }

    /// BFS from `start`; returns visited vertices in BFS order.
    pub fn bfs(&self, start: usize) -> Vec<usize> {
        let n = self.adj.len();
        if start >= n {
            return Vec::new();
        }
        let mut visited = vec![false; n];
        let mut order = Vec::new();
        let mut queue = VecDeque::new();
        visited[start] = true;
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &v in &self.adj[u] {
                if v < n && !visited[v] {
                    visited[v] = true;
                    queue.push_back(v);
                }
            }
        }
        order
    }

    /// DFS from `start`; returns visited vertices in DFS order.
    pub fn dfs(&self, start: usize) -> Vec<usize> {
        let n = self.adj.len();
        if start >= n {
            return Vec::new();
        }
        let mut visited = vec![false; n];
        let mut order = Vec::new();
        self.dfs_rec(start, &mut visited, &mut order);
        order
    }

    fn dfs_rec(&self, u: usize, visited: &mut Vec<bool>, order: &mut Vec<usize>) {
        visited[u] = true;
        order.push(u);
        for &v in &self.adj[u] {
            if v < self.adj.len() && !visited[v] {
                self.dfs_rec(v, visited, order);
            }
        }
    }

    /// BFS shortest path from `src` to `dst` (unweighted). Returns `None` if unreachable.
    pub fn shortest_path(&self, src: usize, dst: usize) -> Option<Vec<usize>> {
        let n = self.adj.len();
        if src >= n || dst >= n {
            return None;
        }
        let mut prev = vec![usize::MAX; n];
        let mut visited = vec![false; n];
        let mut queue = VecDeque::new();
        visited[src] = true;
        queue.push_back(src);
        while let Some(u) = queue.pop_front() {
            if u == dst {
                let mut path = Vec::new();
                let mut cur = dst;
                while cur != src {
                    path.push(cur);
                    cur = prev[cur];
                }
                path.push(src);
                path.reverse();
                return Some(path);
            }
            for &v in &self.adj[u] {
                if v < n && !visited[v] {
                    visited[v] = true;
                    prev[v] = u;
                    queue.push_back(v);
                }
            }
        }
        None
    }

    /// Check if `dst` is reachable from `src` via BFS.
    pub fn is_reachable(&self, src: usize, dst: usize) -> bool {
        let visited: HashSet<usize> = self.bfs(src).into_iter().collect();
        visited.contains(&dst)
    }

    /// Return the BFS level (distance) of each vertex from `start`.
    pub fn bfs_levels(&self, start: usize) -> Vec<Option<usize>> {
        let n = self.adj.len();
        let mut levels = vec![None; n];
        if start >= n {
            return levels;
        }
        levels[start] = Some(0);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            let lvl = levels[u].unwrap_or(0);
            for &v in &self.adj[u] {
                if v < n && levels[v].is_none() {
                    levels[v] = Some(lvl + 1);
                    queue.push_back(v);
                }
            }
        }
        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        /* new_adj_graph creates graph with correct vertex count */
        let g = new_adj_graph(5);
        assert_eq!(g.vertex_count(), 5);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_add_edge() {
        /* add_edge increases edge count */
        let mut g = new_adj_graph(3);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn test_bfs_order() {
        /* BFS visits vertices level-by-level */
        let mut g = new_adj_graph(4);
        g.add_undirected(0, 1);
        g.add_undirected(0, 2);
        g.add_undirected(1, 3);
        let order = g.bfs(0);
        assert_eq!(order[0], 0);
        assert!(order.contains(&1));
        assert!(order.contains(&3));
    }

    #[test]
    fn test_dfs_visits_all() {
        /* DFS from 0 visits all connected vertices */
        let mut g = new_adj_graph(5);
        for i in 0..4 {
            g.add_undirected(i, i + 1);
        }
        let order = g.dfs(0);
        assert_eq!(order.len(), 5);
    }

    #[test]
    fn test_shortest_path_found() {
        /* shortest_path finds a path between connected vertices */
        let mut g = new_adj_graph(5);
        g.add_undirected(0, 1);
        g.add_undirected(1, 2);
        g.add_undirected(2, 3);
        let path = g.shortest_path(0, 3).expect("should succeed");
        assert_eq!(path[0], 0);
        assert_eq!(*path.last().expect("should succeed"), 3);
    }

    #[test]
    fn test_shortest_path_none() {
        /* shortest_path returns None for disconnected vertices */
        let mut g = new_adj_graph(4);
        g.add_edge(0, 1);
        assert!(g.shortest_path(0, 3).is_none());
    }

    #[test]
    fn test_is_reachable() {
        /* is_reachable returns true for connected, false for disconnected */
        let mut g = new_adj_graph(4);
        g.add_undirected(0, 1);
        g.add_undirected(1, 2);
        assert!(g.is_reachable(0, 2));
        assert!(!g.is_reachable(0, 3));
    }

    #[test]
    fn test_bfs_levels() {
        /* bfs_levels correctly computes distances from source */
        let mut g = new_adj_graph(5);
        g.add_undirected(0, 1);
        g.add_undirected(1, 2);
        g.add_undirected(2, 3);
        let levels = g.bfs_levels(0);
        assert_eq!(levels[0], Some(0));
        assert_eq!(levels[1], Some(1));
        assert_eq!(levels[3], Some(3));
        assert_eq!(levels[4], None);
    }

    #[test]
    fn test_add_undirected() {
        /* add_undirected creates two directed edges */
        let mut g = new_adj_graph(2);
        g.add_undirected(0, 1);
        assert_eq!(g.edge_count(), 2);
    }
}
