// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Constraint graph tracking dependencies between constraints.

#![allow(dead_code)]

use std::collections::HashMap;

/// A node in the constraint graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintNode {
    pub id: u32,
    pub particle_a: u32,
    pub particle_b: u32,
}

/// Graph of constraint dependencies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintGraphNew {
    nodes: HashMap<u32, ConstraintNode>,
}

/// Create a new constraint graph.
#[allow(dead_code)]
pub fn new_constraint_graph_new() -> ConstraintGraphNew {
    ConstraintGraphNew { nodes: HashMap::new() }
}

/// Add a constraint to the graph.
#[allow(dead_code)]
pub fn graph_add_constraint(graph: &mut ConstraintGraphNew, node: ConstraintNode) {
    graph.nodes.insert(node.id, node);
}

/// Remove a constraint by id.
#[allow(dead_code)]
pub fn graph_remove_constraint(graph: &mut ConstraintGraphNew, id: u32) -> bool {
    graph.nodes.remove(&id).is_some()
}

/// Return the number of constraints.
#[allow(dead_code)]
pub fn graph_constraint_count(graph: &ConstraintGraphNew) -> usize {
    graph.nodes.len()
}

/// Return constraint ids that involve the given particle.
#[allow(dead_code)]
pub fn graph_particle_constraints(graph: &ConstraintGraphNew, particle: u32) -> Vec<u32> {
    graph.nodes.values()
        .filter(|n| n.particle_a == particle || n.particle_b == particle)
        .map(|n| n.id)
        .collect()
}

/// Check whether any constraint connects particle_a to particle_b.
#[allow(dead_code)]
pub fn graph_is_connected(graph: &ConstraintGraphNew, particle_a: u32, particle_b: u32) -> bool {
    graph.nodes.values().any(|n| {
        (n.particle_a == particle_a && n.particle_b == particle_b)
            || (n.particle_a == particle_b && n.particle_b == particle_a)
    })
}

/// Clear all constraints.
#[allow(dead_code)]
pub fn graph_clear(graph: &mut ConstraintGraphNew) {
    graph.nodes.clear();
}

/// Serialize the graph to a JSON string.
#[allow(dead_code)]
pub fn graph_to_json(graph: &ConstraintGraphNew) -> String {
    let parts: Vec<String> = graph.nodes.values()
        .map(|n| format!("{{\"id\":{},\"a\":{},\"b\":{}}}", n.id, n.particle_a, n.particle_b))
        .collect();
    format!("[{}]", parts.join(","))
}

/// Check whether a constraint with the given id exists.
#[allow(dead_code)]
pub fn graph_has_constraint(graph: &ConstraintGraphNew, id: u32) -> bool {
    graph.nodes.contains_key(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u32, a: u32, b: u32) -> ConstraintNode {
        ConstraintNode { id, particle_a: a, particle_b: b }
    }

    #[test]
    fn test_new_graph_empty() {
        let g = new_constraint_graph_new();
        assert_eq!(graph_constraint_count(&g), 0);
    }

    #[test]
    fn test_add_constraint() {
        let mut g = new_constraint_graph_new();
        graph_add_constraint(&mut g, make_node(1, 0, 1));
        assert_eq!(graph_constraint_count(&g), 1);
        assert!(graph_has_constraint(&g, 1));
    }

    #[test]
    fn test_remove_constraint() {
        let mut g = new_constraint_graph_new();
        graph_add_constraint(&mut g, make_node(5, 2, 3));
        assert!(graph_remove_constraint(&mut g, 5));
        assert!(!graph_has_constraint(&g, 5));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut g = new_constraint_graph_new();
        assert!(!graph_remove_constraint(&mut g, 99));
    }

    #[test]
    fn test_particle_constraints() {
        let mut g = new_constraint_graph_new();
        graph_add_constraint(&mut g, make_node(1, 0, 1));
        graph_add_constraint(&mut g, make_node(2, 1, 2));
        graph_add_constraint(&mut g, make_node(3, 3, 4));
        let cs = graph_particle_constraints(&g, 1);
        assert_eq!(cs.len(), 2);
        assert!(cs.contains(&1));
        assert!(cs.contains(&2));
    }

    #[test]
    fn test_is_connected() {
        let mut g = new_constraint_graph_new();
        graph_add_constraint(&mut g, make_node(1, 5, 6));
        assert!(graph_is_connected(&g, 5, 6));
        assert!(graph_is_connected(&g, 6, 5));
        assert!(!graph_is_connected(&g, 5, 7));
    }

    #[test]
    fn test_clear() {
        let mut g = new_constraint_graph_new();
        graph_add_constraint(&mut g, make_node(1, 0, 1));
        graph_clear(&mut g);
        assert_eq!(graph_constraint_count(&g), 0);
    }

    #[test]
    fn test_to_json_nonempty() {
        let mut g = new_constraint_graph_new();
        graph_add_constraint(&mut g, make_node(1, 2, 3));
        let json = graph_to_json(&g);
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_to_json_empty() {
        let g = new_constraint_graph_new();
        assert_eq!(graph_to_json(&g), "[]");
    }
}
