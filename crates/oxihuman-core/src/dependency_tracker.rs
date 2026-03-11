// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tracks bidirectional dependencies between named nodes and detects dirty propagation.

use std::collections::{HashMap, HashSet, VecDeque};

/// Tracks dependencies and reverse-dependencies between named nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DependencyTracker {
    deps: HashMap<String, HashSet<String>>,
    rev_deps: HashMap<String, HashSet<String>>,
    dirty: HashSet<String>,
}

#[allow(dead_code)]
impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            deps: HashMap::new(),
            rev_deps: HashMap::new(),
            dirty: HashSet::new(),
        }
    }

    pub fn add_dependency(&mut self, node: &str, depends_on: &str) {
        self.deps.entry(node.to_string()).or_default().insert(depends_on.to_string());
        self.rev_deps.entry(depends_on.to_string()).or_default().insert(node.to_string());
    }

    pub fn remove_dependency(&mut self, node: &str, depends_on: &str) -> bool {
        let removed = self.deps.get_mut(node).map(|s| s.remove(depends_on)).unwrap_or(false);
        if removed {
            if let Some(s) = self.rev_deps.get_mut(depends_on) {
                s.remove(node);
            }
        }
        removed
    }

    pub fn dependencies_of(&self, node: &str) -> Vec<String> {
        self.deps.get(node).map(|s| s.iter().cloned().collect()).unwrap_or_default()
    }

    pub fn dependents_of(&self, node: &str) -> Vec<String> {
        self.rev_deps.get(node).map(|s| s.iter().cloned().collect()).unwrap_or_default()
    }

    /// Mark a node dirty and propagate to all transitive dependents.
    pub fn mark_dirty(&mut self, node: &str) {
        let mut queue = VecDeque::new();
        queue.push_back(node.to_string());
        while let Some(n) = queue.pop_front() {
            if self.dirty.insert(n.clone()) {
                if let Some(dependents) = self.rev_deps.get(&n) {
                    for d in dependents {
                        queue.push_back(d.clone());
                    }
                }
            }
        }
    }

    pub fn is_dirty(&self, node: &str) -> bool {
        self.dirty.contains(node)
    }

    pub fn dirty_nodes(&self) -> Vec<String> {
        self.dirty.iter().cloned().collect()
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    pub fn node_count(&self) -> usize {
        let mut nodes: HashSet<&String> = HashSet::new();
        for (k, vs) in &self.deps {
            nodes.insert(k);
            for v in vs { nodes.insert(v); }
        }
        nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.deps.values().map(|s| s.len()).sum()
    }

    /// Check for cycles using DFS.
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        for node in self.deps.keys() {
            if self.dfs_cycle(node, &mut visited, &mut stack) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle(&self, node: &str, visited: &mut HashSet<String>, stack: &mut HashSet<String>) -> bool {
        if stack.contains(node) { return true; }
        if visited.contains(node) { return false; }
        visited.insert(node.to_string());
        stack.insert(node.to_string());
        if let Some(deps) = self.deps.get(node) {
            for d in deps {
                if self.dfs_cycle(d, visited, stack) { return true; }
            }
        }
        stack.remove(node);
        false
    }
}

impl Default for DependencyTracker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_dependency() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        assert_eq!(dt.dependencies_of("b"), vec!["a".to_string()]);
    }

    #[test]
    fn test_reverse_deps() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        assert_eq!(dt.dependents_of("a"), vec!["b".to_string()]);
    }

    #[test]
    fn test_dirty_propagation() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        dt.add_dependency("c", "b");
        dt.mark_dirty("a");
        assert!(dt.is_dirty("a"));
        assert!(dt.is_dirty("b"));
        assert!(dt.is_dirty("c"));
    }

    #[test]
    fn test_clear_dirty() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        dt.mark_dirty("a");
        dt.clear_dirty();
        assert!(!dt.is_dirty("a"));
    }

    #[test]
    fn test_remove_dep() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        assert!(dt.remove_dependency("b", "a"));
        assert!(dt.dependencies_of("b").is_empty());
    }

    #[test]
    fn test_no_cycle() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        dt.add_dependency("c", "b");
        assert!(!dt.has_cycle());
    }

    #[test]
    fn test_cycle() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("a", "b");
        dt.add_dependency("b", "a");
        assert!(dt.has_cycle());
    }

    #[test]
    fn test_edge_count() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        dt.add_dependency("c", "a");
        assert_eq!(dt.edge_count(), 2);
    }

    #[test]
    fn test_node_count() {
        let mut dt = DependencyTracker::new();
        dt.add_dependency("b", "a");
        assert_eq!(dt.node_count(), 2);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut dt = DependencyTracker::new();
        assert!(!dt.remove_dependency("x", "y"));
    }
}
