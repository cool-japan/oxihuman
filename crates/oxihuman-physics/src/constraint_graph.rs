// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A graph of physics constraints connecting body pairs.

use std::collections::HashMap;

/// Type of a constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Distance,
    Hinge,
    Ball,
    Slider,
    Fixed,
}

/// A single constraint edge.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintEdge {
    pub id: u32,
    pub body_a: u32,
    pub body_b: u32,
    pub kind: ConstraintType,
    pub compliance: f32,
    pub enabled: bool,
}

/// Graph of all constraints between bodies.
#[allow(dead_code)]
pub struct ConstraintGraph {
    edges: Vec<ConstraintEdge>,
    body_constraints: HashMap<u32, Vec<u32>>, // body id -> edge ids
    next_id: u32,
}

#[allow(dead_code)]
impl ConstraintGraph {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            body_constraints: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add(&mut self, body_a: u32, body_b: u32, kind: ConstraintType, compliance: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.edges.push(ConstraintEdge {
            id,
            body_a,
            body_b,
            kind,
            compliance,
            enabled: true,
        });
        self.body_constraints.entry(body_a).or_default().push(id);
        self.body_constraints.entry(body_b).or_default().push(id);
        id
    }

    pub fn remove(&mut self, id: u32) -> bool {
        if let Some(pos) = self.edges.iter().position(|e| e.id == id) {
            let e = self.edges.remove(pos);
            for ids in self.body_constraints.values_mut() {
                ids.retain(|&x| x != id);
            }
            let _ = e;
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: u32) -> Option<&ConstraintEdge> {
        self.edges.iter().find(|e| e.id == id)
    }

    pub fn set_enabled(&mut self, id: u32, enabled: bool) -> bool {
        if let Some(e) = self.edges.iter_mut().find(|e| e.id == id) {
            e.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn constraints_for_body(&self, body_id: u32) -> Vec<&ConstraintEdge> {
        self.body_constraints.get(&body_id).map_or(vec![], |ids| {
            ids.iter().filter_map(|&id| self.get(id)).collect()
        })
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.edges.iter().filter(|e| e.enabled).count()
    }

    pub fn body_degree(&self, body_id: u32) -> usize {
        self.constraints_for_body(body_id).len()
    }

    pub fn all_edges(&self) -> &[ConstraintEdge] {
        &self.edges
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.body_constraints.clear();
    }
}

impl Default for ConstraintGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_constraint_graph() -> ConstraintGraph {
    ConstraintGraph::new()
}

pub fn cg_add(
    g: &mut ConstraintGraph,
    body_a: u32,
    body_b: u32,
    kind: ConstraintType,
    compliance: f32,
) -> u32 {
    g.add(body_a, body_b, kind, compliance)
}

pub fn cg_remove(g: &mut ConstraintGraph, id: u32) -> bool {
    g.remove(id)
}

pub fn cg_edge_count(g: &ConstraintGraph) -> usize {
    g.edge_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph_empty() {
        let g = new_constraint_graph();
        assert_eq!(cg_edge_count(&g), 0);
    }

    #[test]
    fn add_returns_id() {
        let mut g = new_constraint_graph();
        let id = cg_add(&mut g, 0, 1, ConstraintType::Distance, 0.0);
        assert_eq!(id, 0);
    }

    #[test]
    fn add_increments_count() {
        let mut g = new_constraint_graph();
        cg_add(&mut g, 0, 1, ConstraintType::Hinge, 0.01);
        cg_add(&mut g, 1, 2, ConstraintType::Ball, 0.01);
        assert_eq!(cg_edge_count(&g), 2);
    }

    #[test]
    fn remove_existing() {
        let mut g = new_constraint_graph();
        let id = cg_add(&mut g, 0, 1, ConstraintType::Fixed, 0.0);
        assert!(cg_remove(&mut g, id));
        assert_eq!(cg_edge_count(&g), 0);
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut g = new_constraint_graph();
        assert!(!cg_remove(&mut g, 999));
    }

    #[test]
    fn constraints_for_body() {
        let mut g = new_constraint_graph();
        cg_add(&mut g, 0, 1, ConstraintType::Distance, 0.0);
        cg_add(&mut g, 0, 2, ConstraintType::Distance, 0.0);
        assert_eq!(g.constraints_for_body(0).len(), 2);
        assert_eq!(g.constraints_for_body(3).len(), 0);
    }

    #[test]
    fn enable_disable() {
        let mut g = new_constraint_graph();
        let id = cg_add(&mut g, 0, 1, ConstraintType::Hinge, 0.0);
        assert!(g.set_enabled(id, false));
        assert_eq!(g.enabled_count(), 0);
        g.set_enabled(id, true);
        assert_eq!(g.enabled_count(), 1);
    }

    #[test]
    fn body_degree() {
        let mut g = new_constraint_graph();
        cg_add(&mut g, 5, 6, ConstraintType::Ball, 0.0);
        assert_eq!(g.body_degree(5), 1);
        assert_eq!(g.body_degree(99), 0);
    }

    #[test]
    fn clear_removes_all() {
        let mut g = new_constraint_graph();
        cg_add(&mut g, 0, 1, ConstraintType::Slider, 0.0);
        g.clear();
        assert_eq!(cg_edge_count(&g), 0);
    }

    #[test]
    fn get_existing_edge() {
        let mut g = new_constraint_graph();
        let id = cg_add(&mut g, 10, 11, ConstraintType::Fixed, 0.001);
        let e = g.get(id).expect("should succeed");
        assert_eq!(e.body_a, 10);
        assert_eq!(e.body_b, 11);
    }
}
