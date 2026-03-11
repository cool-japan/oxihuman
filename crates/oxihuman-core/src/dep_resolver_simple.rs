// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// A dependency graph using Kahn's topological sort algorithm.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    pub nodes: Vec<String>,
    /// (from, to): `from` depends on `to` (must come after `to`)
    pub edges: Vec<(usize, usize)>,
}

/// Create a new empty `DepGraph`.
#[allow(dead_code)]
pub fn new_dep_graph() -> DepGraph {
    DepGraph::default()
}

/// Add a node by name. Returns its index.
#[allow(dead_code)]
pub fn dep_add_node(g: &mut DepGraph, name: &str) -> usize {
    if let Some(pos) = g.nodes.iter().position(|n| n == name) {
        return pos;
    }
    g.nodes.push(name.to_string());
    g.nodes.len() - 1
}

/// Add a directed edge: `from` must come after `to`.
#[allow(dead_code)]
pub fn dep_add_edge(g: &mut DepGraph, from: usize, to: usize) {
    if from < g.nodes.len() && to < g.nodes.len() && from != to {
        g.edges.push((from, to));
    }
}

/// Topological sort using Kahn's algorithm.
/// Returns `Ok(sorted_indices)` or `Err(...)` if a cycle exists.
#[allow(dead_code)]
pub fn topo_sort(g: &DepGraph) -> Result<Vec<usize>, String> {
    let n = g.nodes.len();
    let mut in_degree = vec![0usize; n];
    // Build adjacency: edge (from, to) means to → from in Kahn (to must appear before from)
    // So in_degree[from] += 1 for each edge (from, to)
    for &(from, _to) in &g.edges {
        in_degree[from] += 1;
    }

    let mut queue: std::collections::VecDeque<usize> = (0..n)
        .filter(|&i| in_degree[i] == 0)
        .collect();

    let mut result = Vec::with_capacity(n);
    while let Some(node) = queue.pop_front() {
        result.push(node);
        // Find all nodes that depend on `node` and reduce their in-degree
        for &(from, to) in &g.edges {
            if to == node {
                in_degree[from] -= 1;
                if in_degree[from] == 0 {
                    queue.push_back(from);
                }
            }
        }
    }

    if result.len() == n {
        Ok(result)
    } else {
        Err("cycle detected in dependency graph".to_string())
    }
}

/// Get the number of nodes in the graph.
#[allow(dead_code)]
pub fn dep_node_count(g: &DepGraph) -> usize {
    g.nodes.len()
}

/// Check whether the graph has a cycle.
#[allow(dead_code)]
pub fn has_cycle(g: &DepGraph) -> bool {
    topo_sort(g).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_nodes() {
        let mut g = new_dep_graph();
        dep_add_node(&mut g, "A");
        dep_add_node(&mut g, "B");
        assert_eq!(dep_node_count(&g), 2);
    }

    #[test]
    fn topo_sort_simple() {
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "A");
        let b = dep_add_node(&mut g, "B");
        let c = dep_add_node(&mut g, "C");
        dep_add_edge(&mut g, b, a); // B depends on A
        dep_add_edge(&mut g, c, b); // C depends on B
        let sorted = topo_sort(&g).unwrap();
        // A must appear before B, B before C
        let pa = sorted.iter().position(|&x| x == a).unwrap();
        let pb = sorted.iter().position(|&x| x == b).unwrap();
        let pc = sorted.iter().position(|&x| x == c).unwrap();
        assert!(pa < pb);
        assert!(pb < pc);
    }

    #[test]
    fn topo_sort_cycle_detected() {
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "A");
        let b = dep_add_node(&mut g, "B");
        dep_add_edge(&mut g, a, b);
        dep_add_edge(&mut g, b, a);
        assert!(topo_sort(&g).is_err());
    }

    #[test]
    fn has_cycle_true() {
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "A");
        let b = dep_add_node(&mut g, "B");
        dep_add_edge(&mut g, a, b);
        dep_add_edge(&mut g, b, a);
        assert!(has_cycle(&g));
    }

    #[test]
    fn has_cycle_false() {
        let mut g = new_dep_graph();
        let a = dep_add_node(&mut g, "A");
        let b = dep_add_node(&mut g, "B");
        dep_add_edge(&mut g, b, a);
        assert!(!has_cycle(&g));
    }

    #[test]
    fn duplicate_node_returns_same_index() {
        let mut g = new_dep_graph();
        let a1 = dep_add_node(&mut g, "A");
        let a2 = dep_add_node(&mut g, "A");
        assert_eq!(a1, a2);
        assert_eq!(dep_node_count(&g), 1);
    }

    #[test]
    fn empty_graph_topo_sort() {
        let g = new_dep_graph();
        let sorted = topo_sort(&g).unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn single_node_topo_sort() {
        let mut g = new_dep_graph();
        dep_add_node(&mut g, "Solo");
        let sorted = topo_sort(&g).unwrap();
        assert_eq!(sorted.len(), 1);
    }

    #[test]
    fn topo_sort_result_has_all_nodes() {
        let mut g = new_dep_graph();
        dep_add_node(&mut g, "X");
        dep_add_node(&mut g, "Y");
        dep_add_node(&mut g, "Z");
        let sorted = topo_sort(&g).unwrap();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn dep_node_count_zero_initial() {
        let g = new_dep_graph();
        assert_eq!(dep_node_count(&g), 0);
    }
}
