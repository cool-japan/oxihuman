// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};

/// A lightweight dependency graph that tracks edges between string-named nodes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    edges: HashMap<String, HashSet<String>>,
}

#[allow(dead_code)]
impl DepGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, name: &str) {
        self.edges.entry(name.to_string()).or_default();
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());
        self.edges.entry(to.to_string()).or_default();
    }

    pub fn has_node(&self, name: &str) -> bool {
        self.edges.contains_key(name)
    }

    pub fn has_edge(&self, from: &str, to: &str) -> bool {
        self.edges
            .get(from)
            .is_some_and(|deps| deps.contains(to))
    }

    pub fn node_count(&self) -> usize {
        self.edges.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|s| s.len()).sum()
    }

    pub fn dependents(&self, name: &str) -> Vec<String> {
        self.edges
            .get(name)
            .map(|s| {
                let mut v: Vec<_> = s.iter().cloned().collect();
                v.sort();
                v
            })
            .unwrap_or_default()
    }

    /// BFS topological sort. Returns None if cycle detected.
    pub fn topological_order(&self) -> Option<Vec<String>> {
        let mut in_deg: HashMap<&str, usize> = HashMap::new();
        for (node, deps) in &self.edges {
            in_deg.entry(node.as_str()).or_insert(0);
            for d in deps {
                *in_deg.entry(d.as_str()).or_insert(0) += 1;
            }
        }
        let mut queue: VecDeque<String> = in_deg
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&n, _)| n.to_string())
            .collect();
        queue.make_contiguous().sort();
        let mut result = Vec::new();
        while let Some(n) = queue.pop_front() {
            if let Some(deps) = self.edges.get(&n) {
                let mut sorted_deps: Vec<_> = deps.iter().collect();
                sorted_deps.sort();
                for d in sorted_deps {
                    if let Some(deg) = in_deg.get_mut(d.as_str()) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(d.clone());
                        }
                    }
                }
            }
            result.push(n);
        }
        if result.len() == self.edges.len() {
            Some(result)
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, name: &str) {
        self.edges.remove(name);
        for deps in self.edges.values_mut() {
            deps.remove(name);
        }
    }

    pub fn nodes(&self) -> Vec<String> {
        let mut v: Vec<_> = self.edges.keys().cloned().collect();
        v.sort();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut g = DepGraph::new();
        g.add_node("a");
        assert!(g.has_node("a"));
    }

    #[test]
    fn test_add_edge() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        assert!(g.has_edge("a", "b"));
        assert!(!g.has_edge("b", "a"));
    }

    #[test]
    fn test_node_count() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        assert_eq!(g.node_count(), 2);
    }

    #[test]
    fn test_edge_count() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        g.add_edge("a", "c");
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn test_topological_order() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        g.add_edge("b", "c");
        let order = g.topological_order().unwrap();
        let ia = order.iter().position(|x| x == "a").unwrap();
        let ib = order.iter().position(|x| x == "b").unwrap();
        let ic = order.iter().position(|x| x == "c").unwrap();
        assert!(ia < ib);
        assert!(ib < ic);
    }

    #[test]
    fn test_cycle_detection() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        g.add_edge("b", "a");
        assert!(g.topological_order().is_none());
    }

    #[test]
    fn test_remove_node() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        g.remove_node("b");
        assert!(!g.has_node("b"));
        assert!(!g.has_edge("a", "b"));
    }

    #[test]
    fn test_dependents() {
        let mut g = DepGraph::new();
        g.add_edge("a", "b");
        g.add_edge("a", "c");
        let deps = g.dependents("a");
        assert_eq!(deps, vec!["b", "c"]);
    }

    #[test]
    fn test_nodes_sorted() {
        let mut g = DepGraph::new();
        g.add_node("z");
        g.add_node("a");
        let nodes = g.nodes();
        assert_eq!(nodes, vec!["a", "z"]);
    }

    #[test]
    fn test_empty_graph() {
        let g = DepGraph::new();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.topological_order(), Some(vec![]));
    }
}
