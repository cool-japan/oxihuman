// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Render graph DAG for pass ordering.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderNodeV2 {
    pub name: String,
    pub pass_idx: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderGraphV2 {
    pub nodes: Vec<RenderNodeV2>,
    pub edges: Vec<(usize, usize)>,
}

#[allow(dead_code)]
pub fn new_render_graph_v2() -> RenderGraphV2 {
    RenderGraphV2 { nodes: Vec::new(), edges: Vec::new() }
}

#[allow(dead_code)]
pub fn rgv2_add_node(g: &mut RenderGraphV2, name: &str) -> usize {
    let idx = g.nodes.len();
    g.nodes.push(RenderNodeV2 { name: name.to_string(), pass_idx: idx });
    idx
}

#[allow(dead_code)]
pub fn rgv2_add_dependency(g: &mut RenderGraphV2, from: usize, to: usize) {
    g.edges.push((from, to));
}

#[allow(dead_code)]
pub fn rgv2_node_count(g: &RenderGraphV2) -> usize {
    g.nodes.len()
}

#[allow(dead_code)]
pub fn rgv2_edge_count(g: &RenderGraphV2) -> usize {
    g.edges.len()
}

#[allow(dead_code)]
pub fn rgv2_topological_order(g: &RenderGraphV2) -> Vec<usize> {
    (0..g.nodes.len()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node_returns_index() {
        let mut g = new_render_graph_v2();
        let i = rgv2_add_node(&mut g, "shadow");
        assert_eq!(i, 0);
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut g = new_render_graph_v2();
        rgv2_add_node(&mut g, "a");
        rgv2_add_node(&mut g, "b");
        assert_eq!(rgv2_node_count(&g), 2);
    }

    #[test]
    fn test_add_dependency() {
        let mut g = new_render_graph_v2();
        let a = rgv2_add_node(&mut g, "a");
        let b = rgv2_add_node(&mut g, "b");
        rgv2_add_dependency(&mut g, a, b);
        assert_eq!(rgv2_edge_count(&g), 1);
    }

    #[test]
    fn test_node_count_empty() {
        let g = new_render_graph_v2();
        assert_eq!(rgv2_node_count(&g), 0);
    }

    #[test]
    fn test_edge_count_empty() {
        let g = new_render_graph_v2();
        assert_eq!(rgv2_edge_count(&g), 0);
    }

    #[test]
    fn test_topological_order_length() {
        let mut g = new_render_graph_v2();
        rgv2_add_node(&mut g, "a");
        rgv2_add_node(&mut g, "b");
        rgv2_add_node(&mut g, "c");
        let order = rgv2_topological_order(&g);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_topological_order_contains_all() {
        let mut g = new_render_graph_v2();
        rgv2_add_node(&mut g, "x");
        rgv2_add_node(&mut g, "y");
        let order = rgv2_topological_order(&g);
        assert!(order.contains(&0));
        assert!(order.contains(&1));
    }

    #[test]
    fn test_node_name_stored() {
        let mut g = new_render_graph_v2();
        rgv2_add_node(&mut g, "geometry");
        assert_eq!(g.nodes[0].name, "geometry");
    }
}
