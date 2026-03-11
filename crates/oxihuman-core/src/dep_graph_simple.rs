// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Simple directed graph for dependency resolution via Kahn's topological sort.
pub struct DepGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(usize, usize)>,
}

impl DepGraph {
    pub fn new() -> Self {
        DepGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl Default for DepGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_dep_graph() -> DepGraph {
    DepGraph::new()
}

pub fn dep_add_node(g: &mut DepGraph, name: &str) -> usize {
    let idx = g.nodes.len();
    g.nodes.push(name.to_string());
    idx
}

pub fn dep_add_edge(g: &mut DepGraph, from: usize, to: usize) {
    g.edges.push((from, to));
}

pub fn dep_node_count(g: &DepGraph) -> usize {
    g.nodes.len()
}

/// Kahn's algorithm. Returns `None` if a cycle is detected.
pub fn dep_topo_sort(g: &DepGraph) -> Option<Vec<usize>> {
    let n = g.nodes.len();
    let mut in_degree = vec![0usize; n];
    for &(_, to) in &g.edges {
        in_degree[to] += 1;
    }

    let mut queue: std::collections::VecDeque<usize> =
        (0..n).filter(|&i| in_degree[i] == 0).collect();

    let mut order = Vec::with_capacity(n);
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &(from, to) in &g.edges {
            if from == node {
                in_degree[to] -= 1;
                if in_degree[to] == 0 {
                    queue.push_back(to);
                }
            }
        }
    }

    if order.len() == n {
        Some(order)
    } else {
        None
    }
}

pub fn dep_has_cycle(g: &DepGraph) -> bool {
    dep_topo_sort(g).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new graph is empty */
        let g = new_dep_graph();
        assert_eq!(dep_node_count(&g), 0);
    }

    #[test]
    fn test_add_nodes() {
        /* adding nodes increments count */
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "a");
        let b = dep_add_node(&mut g, "b");
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(dep_node_count(&g), 2);
    }

    #[test]
    fn test_topo_sort_linear() {
        /* linear chain: a -> b -> c */
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "a");
        let b = dep_add_node(&mut g, "b");
        let c = dep_add_node(&mut g, "c");
        dep_add_edge(&mut g, a, b);
        dep_add_edge(&mut g, b, c);
        let order = dep_topo_sort(&g).unwrap();
        assert_eq!(order, vec![a, b, c]);
    }

    #[test]
    fn test_no_cycle() {
        /* acyclic graph has no cycle */
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "a");
        let b = dep_add_node(&mut g, "b");
        dep_add_edge(&mut g, a, b);
        assert!(!dep_has_cycle(&g));
    }

    #[test]
    fn test_cycle_detected() {
        /* cycle: a -> b -> a */
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "a");
        let b = dep_add_node(&mut g, "b");
        dep_add_edge(&mut g, a, b);
        dep_add_edge(&mut g, b, a);
        assert!(dep_has_cycle(&g));
        assert!(dep_topo_sort(&g).is_none());
    }

    #[test]
    fn test_no_edges() {
        /* nodes with no edges sort in insertion order */
        let mut g = new_dep_graph();
        dep_add_node(&mut g, "x");
        dep_add_node(&mut g, "y");
        let order = dep_topo_sort(&g).unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let g = DepGraph::default();
        assert_eq!(g.nodes.len(), 0);
    }

    #[test]
    fn test_diamond() {
        /* diamond graph: a -> b, a -> c, b -> d, c -> d */
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "a");
        let b = dep_add_node(&mut g, "b");
        let c = dep_add_node(&mut g, "c");
        let d = dep_add_node(&mut g, "d");
        dep_add_edge(&mut g, a, b);
        dep_add_edge(&mut g, a, c);
        dep_add_edge(&mut g, b, d);
        dep_add_edge(&mut g, c, d);
        let order = dep_topo_sort(&g).unwrap();
        assert_eq!(order.len(), 4);
        /* a must come before b, c, d */
        let pos_a = order.iter().position(|&x| x == a).unwrap();
        let pos_d = order.iter().position(|&x| x == d).unwrap();
        assert!(pos_a < pos_d);
    }
}
