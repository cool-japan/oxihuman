// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Morph dependency graph for driven shapes.

#[allow(dead_code)]
pub struct MorphGraphNode {
    pub name: String,
    pub value: f32,
    pub drivers: Vec<usize>,
}

#[allow(dead_code)]
pub struct MorphGraph {
    pub nodes: Vec<MorphGraphNode>,
}

#[allow(dead_code)]
pub fn new_morph_graph() -> MorphGraph {
    MorphGraph { nodes: Vec::new() }
}

#[allow(dead_code)]
pub fn mg_add_node(g: &mut MorphGraph, name: &str, value: f32) -> usize {
    let idx = g.nodes.len();
    g.nodes.push(MorphGraphNode { name: name.to_string(), value, drivers: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn mg_add_driver(g: &mut MorphGraph, driven: usize, driver: usize) {
    if driven < g.nodes.len() {
        g.nodes[driven].drivers.push(driver);
    }
}

#[allow(dead_code)]
pub fn mg_node_count(g: &MorphGraph) -> usize {
    g.nodes.len()
}

#[allow(dead_code)]
pub fn mg_evaluate(g: &MorphGraph, idx: usize) -> f32 {
    if idx >= g.nodes.len() {
        return 0.0;
    }
    let node = &g.nodes[idx];
    if node.drivers.is_empty() {
        return node.value;
    }
    let sum: f32 = node.drivers.iter().map(|&d| {
        if d < g.nodes.len() { g.nodes[d].value } else { 0.0 }
    }).sum();
    sum / node.drivers.len() as f32
}

#[allow(dead_code)]
pub fn mg_get_node(g: &MorphGraph, name: &str) -> Option<usize> {
    g.nodes.iter().position(|n| n.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut g = new_morph_graph();
        let idx = mg_add_node(&mut g, "jaw", 0.5);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_node_count() {
        let mut g = new_morph_graph();
        mg_add_node(&mut g, "a", 0.0);
        mg_add_node(&mut g, "b", 0.0);
        assert_eq!(mg_node_count(&g), 2);
    }

    #[test]
    fn test_add_driver() {
        let mut g = new_morph_graph();
        let d = mg_add_node(&mut g, "driver", 0.8);
        let n = mg_add_node(&mut g, "driven", 0.0);
        mg_add_driver(&mut g, n, d);
        assert_eq!(g.nodes[n].drivers.len(), 1);
    }

    #[test]
    fn test_evaluate_no_drivers() {
        let mut g = new_morph_graph();
        let idx = mg_add_node(&mut g, "solo", 0.6);
        let v = mg_evaluate(&g, idx);
        assert!((v - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_with_drivers() {
        let mut g = new_morph_graph();
        let d1 = mg_add_node(&mut g, "d1", 0.4);
        let d2 = mg_add_node(&mut g, "d2", 0.8);
        let n = mg_add_node(&mut g, "driven", 0.0);
        mg_add_driver(&mut g, n, d1);
        mg_add_driver(&mut g, n, d2);
        let v = mg_evaluate(&g, n);
        assert!((v - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_get_node_found() {
        let mut g = new_morph_graph();
        mg_add_node(&mut g, "brow", 0.0);
        assert_eq!(mg_get_node(&g, "brow"), Some(0));
    }

    #[test]
    fn test_get_node_missing() {
        let g = new_morph_graph();
        assert_eq!(mg_get_node(&g, "none"), None);
    }

    #[test]
    fn test_evaluate_out_of_bounds() {
        let g = new_morph_graph();
        assert_eq!(mg_evaluate(&g, 99), 0.0);
    }
}
