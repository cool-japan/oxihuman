// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A simple directed graph with integer node ids and string edge labels.

use std::collections::{HashMap, HashSet, VecDeque};

/// An edge in the graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: u32,
    pub to: u32,
    pub label: String,
    pub weight: f32,
}

/// Simple directed graph.
#[allow(dead_code)]
pub struct SimpleGraph {
    nodes: HashSet<u32>,
    edges: Vec<GraphEdge>,
    node_labels: HashMap<u32, String>,
}

#[allow(dead_code)]
impl SimpleGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: Vec::new(),
            node_labels: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: u32, label: &str) {
        self.nodes.insert(id);
        self.node_labels.insert(id, label.to_string());
    }

    pub fn remove_node(&mut self, id: u32) -> bool {
        if !self.nodes.remove(&id) {
            return false;
        }
        self.node_labels.remove(&id);
        self.edges.retain(|e| e.from != id && e.to != id);
        true
    }

    pub fn add_edge(&mut self, from: u32, to: u32, label: &str, weight: f32) {
        self.nodes.insert(from);
        self.nodes.insert(to);
        self.edges.push(GraphEdge {
            from,
            to,
            label: label.to_string(),
            weight,
        });
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn has_node(&self, id: u32) -> bool {
        self.nodes.contains(&id)
    }

    pub fn neighbors(&self, id: u32) -> Vec<u32> {
        self.edges
            .iter()
            .filter(|e| e.from == id)
            .map(|e| e.to)
            .collect()
    }

    pub fn in_neighbors(&self, id: u32) -> Vec<u32> {
        self.edges
            .iter()
            .filter(|e| e.to == id)
            .map(|e| e.from)
            .collect()
    }

    /// BFS reachability from `start`.
    pub fn bfs(&self, start: u32) -> Vec<u32> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut order = Vec::new();
        if !self.nodes.contains(&start) {
            return order;
        }
        queue.push_back(start);
        visited.insert(start);
        while let Some(node) = queue.pop_front() {
            order.push(node);
            for nb in self.neighbors(node) {
                if visited.insert(nb) {
                    queue.push_back(nb);
                }
            }
        }
        order
    }

    /// Check whether there is a path from `from` to `to`.
    pub fn has_path(&self, from: u32, to: u32) -> bool {
        self.bfs(from).contains(&to)
    }

    pub fn node_label(&self, id: u32) -> Option<&str> {
        self.node_labels.get(&id).map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.node_labels.clear();
    }
}

impl Default for SimpleGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_simple_graph() -> SimpleGraph {
    SimpleGraph::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_nodes_and_edges() {
        let mut g = new_simple_graph();
        g.add_node(1, "a");
        g.add_node(2, "b");
        g.add_edge(1, 2, "link", 1.0);
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn has_node() {
        let mut g = new_simple_graph();
        g.add_node(5, "x");
        assert!(g.has_node(5));
        assert!(!g.has_node(99));
    }

    #[test]
    fn neighbors() {
        let mut g = new_simple_graph();
        g.add_edge(1, 2, "", 1.0);
        g.add_edge(1, 3, "", 1.0);
        let mut nb = g.neighbors(1);
        nb.sort_unstable();
        assert_eq!(nb, vec![2, 3]);
    }

    #[test]
    fn bfs_order() {
        let mut g = new_simple_graph();
        g.add_edge(0, 1, "", 1.0);
        g.add_edge(0, 2, "", 1.0);
        g.add_edge(1, 3, "", 1.0);
        let visited = g.bfs(0);
        assert!(visited.contains(&0));
        assert!(visited.contains(&3));
    }

    #[test]
    fn has_path_true() {
        let mut g = new_simple_graph();
        g.add_edge(0, 1, "", 1.0);
        g.add_edge(1, 2, "", 1.0);
        assert!(g.has_path(0, 2));
    }

    #[test]
    fn has_path_false() {
        let mut g = new_simple_graph();
        g.add_edge(0, 1, "", 1.0);
        assert!(!g.has_path(1, 0));
    }

    #[test]
    fn remove_node_removes_edges() {
        let mut g = new_simple_graph();
        g.add_edge(1, 2, "", 1.0);
        g.remove_node(1);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn node_label() {
        let mut g = new_simple_graph();
        g.add_node(7, "seven");
        assert_eq!(g.node_label(7), Some("seven"));
    }

    #[test]
    fn clear() {
        let mut g = new_simple_graph();
        g.add_edge(1, 2, "", 1.0);
        g.clear();
        assert!(g.is_empty());
    }

    #[test]
    fn bfs_unreachable_node() {
        let mut g = new_simple_graph();
        g.add_node(10, "isolated");
        let visited = g.bfs(99);
        assert!(visited.is_empty());
    }
}
