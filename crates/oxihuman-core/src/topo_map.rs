// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Topological map: directed graph with topological sort utilities.

use std::collections::HashMap;

/// A directed graph node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TopoNode {
    pub id: u32,
    pub label: String,
    pub edges: Vec<u32>,
}

/// A topological map (DAG).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TopoMap {
    nodes: HashMap<u32, TopoNode>,
    next_id: u32,
}

/// Create a new `TopoMap`.
#[allow(dead_code)]
pub fn new_topo_map() -> TopoMap {
    TopoMap::default()
}

/// Add a node and return its id.
#[allow(dead_code)]
pub fn tm_add_node(tm: &mut TopoMap, label: &str) -> u32 {
    let id = tm.next_id;
    tm.next_id += 1;
    tm.nodes.insert(
        id,
        TopoNode {
            id,
            label: label.to_string(),
            edges: Vec::new(),
        },
    );
    id
}

/// Add a directed edge from `from` to `to`.
#[allow(dead_code)]
pub fn tm_add_edge(tm: &mut TopoMap, from: u32, to: u32) {
    if let Some(node) = tm.nodes.get_mut(&from) {
        if !node.edges.contains(&to) {
            node.edges.push(to);
        }
    }
}

/// Number of nodes.
#[allow(dead_code)]
pub fn tm_node_count(tm: &TopoMap) -> usize {
    tm.nodes.len()
}

/// Whether a node exists.
#[allow(dead_code)]
pub fn tm_has_node(tm: &TopoMap, id: u32) -> bool {
    tm.nodes.contains_key(&id)
}

/// Get label for a node.
#[allow(dead_code)]
pub fn tm_label(tm: &TopoMap, id: u32) -> Option<&str> {
    tm.nodes.get(&id).map(|n| n.label.as_str())
}

/// Topological sort. Returns `None` if cycle detected.
#[allow(dead_code)]
pub fn tm_topo_sort(tm: &TopoMap) -> Option<Vec<u32>> {
    let mut in_degree: HashMap<u32, usize> = tm.nodes.keys().map(|&k| (k, 0)).collect();
    for node in tm.nodes.values() {
        for &to in &node.edges {
            *in_degree.entry(to).or_insert(0) += 1;
        }
    }
    let mut queue: Vec<u32> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&k, _)| k)
        .collect();
    queue.sort_unstable();
    let mut result = Vec::new();
    while !queue.is_empty() {
        let u = queue.remove(0);
        result.push(u);
        if let Some(node) = tm.nodes.get(&u) {
            let mut nexts: Vec<u32> = node.edges.clone();
            nexts.sort_unstable();
            for v in nexts {
                let deg = in_degree.entry(v).or_insert(0);
                *deg -= 1;
                if *deg == 0 {
                    queue.push(v);
                }
            }
        }
    }
    if result.len() == tm.nodes.len() {
        Some(result)
    } else {
        None
    }
}

/// Remove a node and all edges pointing to it.
#[allow(dead_code)]
pub fn tm_remove_node(tm: &mut TopoMap, id: u32) {
    tm.nodes.remove(&id);
    for node in tm.nodes.values_mut() {
        node.edges.retain(|&e| e != id);
    }
}

/// Clear all nodes and edges.
#[allow(dead_code)]
pub fn tm_clear(tm: &mut TopoMap) {
    tm.nodes.clear();
    tm.next_id = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut tm = new_topo_map();
        let id = tm_add_node(&mut tm, "root");
        assert!(tm_has_node(&tm, id));
        assert_eq!(tm_node_count(&tm), 1);
    }

    #[test]
    fn test_label() {
        let mut tm = new_topo_map();
        let id = tm_add_node(&mut tm, "alpha");
        assert_eq!(tm_label(&tm, id), Some("alpha"));
    }

    #[test]
    fn test_topo_sort_linear() {
        let mut tm = new_topo_map();
        let a = tm_add_node(&mut tm, "a");
        let b = tm_add_node(&mut tm, "b");
        let c = tm_add_node(&mut tm, "c");
        tm_add_edge(&mut tm, a, b);
        tm_add_edge(&mut tm, b, c);
        let sorted = tm_topo_sort(&tm).unwrap();
        let pos_a = sorted.iter().position(|&x| x == a).unwrap();
        let pos_b = sorted.iter().position(|&x| x == b).unwrap();
        let pos_c = sorted.iter().position(|&x| x == c).unwrap();
        assert!(pos_a < pos_b && pos_b < pos_c);
    }

    #[test]
    fn test_topo_sort_empty() {
        let tm = new_topo_map();
        let sorted = tm_topo_sort(&tm).unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_remove_node() {
        let mut tm = new_topo_map();
        let a = tm_add_node(&mut tm, "a");
        let b = tm_add_node(&mut tm, "b");
        tm_add_edge(&mut tm, a, b);
        tm_remove_node(&mut tm, b);
        assert!(!tm_has_node(&tm, b));
        assert_eq!(tm_node_count(&tm), 1);
    }

    #[test]
    fn test_clear() {
        let mut tm = new_topo_map();
        tm_add_node(&mut tm, "x");
        tm_clear(&mut tm);
        assert_eq!(tm_node_count(&tm), 0);
    }

    #[test]
    fn test_duplicate_edge_ignored() {
        let mut tm = new_topo_map();
        let a = tm_add_node(&mut tm, "a");
        let b = tm_add_node(&mut tm, "b");
        tm_add_edge(&mut tm, a, b);
        tm_add_edge(&mut tm, a, b);
        assert_eq!(tm.nodes[&a].edges.len(), 1);
    }

    #[test]
    fn test_topo_sort_diamond() {
        let mut tm = new_topo_map();
        let a = tm_add_node(&mut tm, "a");
        let b = tm_add_node(&mut tm, "b");
        let c = tm_add_node(&mut tm, "c");
        let d = tm_add_node(&mut tm, "d");
        tm_add_edge(&mut tm, a, b);
        tm_add_edge(&mut tm, a, c);
        tm_add_edge(&mut tm, b, d);
        tm_add_edge(&mut tm, c, d);
        let sorted = tm_topo_sort(&tm).unwrap();
        let pos_a = sorted.iter().position(|&x| x == a).unwrap();
        let pos_d = sorted.iter().position(|&x| x == d).unwrap();
        assert!(pos_a < pos_d);
    }

    #[test]
    fn test_missing_node_label() {
        let tm = new_topo_map();
        assert!(tm_label(&tm, 999).is_none());
    }
}
