// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Adjacency map for representing graph-like relationships between nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdjacencyMap {
    edges: HashMap<u32, Vec<u32>>,
    directed: bool,
}

#[allow(dead_code)]
impl AdjacencyMap {
    pub fn new(directed: bool) -> Self {
        Self {
            edges: HashMap::new(),
            directed,
        }
    }

    pub fn add_edge(&mut self, from: u32, to: u32) {
        self.edges.entry(from).or_default().push(to);
        if !self.directed {
            self.edges.entry(to).or_default().push(from);
        }
    }

    pub fn neighbors(&self, node: u32) -> &[u32] {
        self.edges.get(&node).map_or(&[], |v| v.as_slice())
    }

    pub fn has_edge(&self, from: u32, to: u32) -> bool {
        self.edges
            .get(&from)
            .is_some_and(|v| v.contains(&to))
    }

    pub fn node_count(&self) -> usize {
        self.edges.len()
    }

    pub fn edge_count(&self) -> usize {
        let total: usize = self.edges.values().map(|v| v.len()).sum();
        if self.directed {
            total
        } else {
            total / 2
        }
    }

    pub fn remove_edge(&mut self, from: u32, to: u32) {
        if let Some(v) = self.edges.get_mut(&from) {
            v.retain(|&x| x != to);
        }
        if !self.directed {
            if let Some(v) = self.edges.get_mut(&to) {
                v.retain(|&x| x != from);
            }
        }
    }

    pub fn degree(&self, node: u32) -> usize {
        self.edges.get(&node).map_or(0, |v| v.len())
    }

    pub fn clear(&mut self) {
        self.edges.clear();
    }

    pub fn is_directed(&self) -> bool {
        self.directed
    }

    pub fn nodes(&self) -> Vec<u32> {
        self.edges.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_directed() {
        let map = AdjacencyMap::new(true);
        assert!(map.is_directed());
        assert_eq!(map.node_count(), 0);
    }

    #[test]
    fn test_add_edge_directed() {
        let mut map = AdjacencyMap::new(true);
        map.add_edge(1, 2);
        assert!(map.has_edge(1, 2));
        assert!(!map.has_edge(2, 1));
    }

    #[test]
    fn test_add_edge_undirected() {
        let mut map = AdjacencyMap::new(false);
        map.add_edge(1, 2);
        assert!(map.has_edge(1, 2));
        assert!(map.has_edge(2, 1));
    }

    #[test]
    fn test_neighbors() {
        let mut map = AdjacencyMap::new(true);
        map.add_edge(1, 2);
        map.add_edge(1, 3);
        assert_eq!(map.neighbors(1).len(), 2);
        assert!(map.neighbors(5).is_empty());
    }

    #[test]
    fn test_edge_count_undirected() {
        let mut map = AdjacencyMap::new(false);
        map.add_edge(1, 2);
        map.add_edge(2, 3);
        assert_eq!(map.edge_count(), 2);
    }

    #[test]
    fn test_remove_edge() {
        let mut map = AdjacencyMap::new(false);
        map.add_edge(1, 2);
        map.remove_edge(1, 2);
        assert!(!map.has_edge(1, 2));
        assert!(!map.has_edge(2, 1));
    }

    #[test]
    fn test_degree() {
        let mut map = AdjacencyMap::new(true);
        map.add_edge(1, 2);
        map.add_edge(1, 3);
        map.add_edge(1, 4);
        assert_eq!(map.degree(1), 3);
        assert_eq!(map.degree(99), 0);
    }

    #[test]
    fn test_clear() {
        let mut map = AdjacencyMap::new(true);
        map.add_edge(1, 2);
        map.clear();
        assert_eq!(map.node_count(), 0);
    }

    #[test]
    fn test_nodes() {
        let mut map = AdjacencyMap::new(true);
        map.add_edge(10, 20);
        map.add_edge(30, 40);
        let nodes = map.nodes();
        assert!(nodes.contains(&10));
        assert!(nodes.contains(&30));
    }

    #[test]
    fn test_edge_count_directed() {
        let mut map = AdjacencyMap::new(true);
        map.add_edge(1, 2);
        map.add_edge(2, 3);
        map.add_edge(3, 1);
        assert_eq!(map.edge_count(), 3);
    }
}
