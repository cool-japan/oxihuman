// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Render-graph (frame-graph) node and dependency management.

use std::collections::HashMap;

/// Pass kind.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassKind {
    Render,
    Compute,
    Transfer,
}

/// A render-graph node.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RgNode {
    pub id: u32,
    pub name: String,
    pub kind: PassKind,
    pub enabled: bool,
}

/// The render graph.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct RenderGraph {
    pub nodes: Vec<RgNode>,
    /// dependency edges: node_id -> list of dependency node_ids.
    pub deps: HashMap<u32, Vec<u32>>,
    pub next_id: u32,
}

#[allow(dead_code)]
pub fn new_render_graph() -> RenderGraph {
    RenderGraph::default()
}

#[allow(dead_code)]
pub fn rg_add_node(graph: &mut RenderGraph, name: &str, kind: PassKind) -> u32 {
    let id = graph.next_id;
    graph.next_id += 1;
    graph.nodes.push(RgNode {
        id,
        name: name.to_string(),
        kind,
        enabled: true,
    });
    id
}

#[allow(dead_code)]
pub fn rg_add_dep(graph: &mut RenderGraph, node: u32, dep: u32) {
    graph.deps.entry(node).or_default().push(dep);
}

#[allow(dead_code)]
pub fn rg_set_enabled(graph: &mut RenderGraph, id: u32, v: bool) {
    if let Some(n) = graph.nodes.iter_mut().find(|n| n.id == id) {
        n.enabled = v;
    }
}

#[allow(dead_code)]
pub fn rg_node_count(graph: &RenderGraph) -> usize {
    graph.nodes.len()
}

#[allow(dead_code)]
pub fn rg_enabled_count(graph: &RenderGraph) -> usize {
    graph.nodes.iter().filter(|n| n.enabled).count()
}

#[allow(dead_code)]
pub fn rg_dep_count(graph: &RenderGraph, node: u32) -> usize {
    graph.deps.get(&node).map(|v| v.len()).unwrap_or(0)
}

#[allow(dead_code)]
pub fn rg_pass_name(kind: PassKind) -> &'static str {
    match kind {
        PassKind::Render => "render",
        PassKind::Compute => "compute",
        PassKind::Transfer => "transfer",
    }
}

#[allow(dead_code)]
pub fn rg_clear(graph: &mut RenderGraph) {
    graph.nodes.clear();
    graph.deps.clear();
}

#[allow(dead_code)]
pub fn rg_to_json(graph: &RenderGraph) -> String {
    let nodes: Vec<String> = graph
        .nodes
        .iter()
        .map(|n| {
            format!(
                "{{\"id\":{},\"name\":\"{}\",\"kind\":\"{}\",\"enabled\":{}}}",
                n.id,
                n.name,
                rg_pass_name(n.kind),
                n.enabled
            )
        })
        .collect();
    format!("{{\"nodes\":[{}]}}", nodes.join(","))
}

/// Simple topological sort (no cycle detection — educational stub).
#[allow(dead_code)]
pub fn rg_topo_sort(graph: &RenderGraph) -> Vec<u32> {
    let mut order: Vec<u32> = graph.nodes.iter().map(|n| n.id).collect();
    order.sort();
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        assert_eq!(rg_node_count(&new_render_graph()), 0);
    }

    #[test]
    fn add_node() {
        let mut g = new_render_graph();
        rg_add_node(&mut g, "shadows", PassKind::Render);
        assert_eq!(rg_node_count(&g), 1);
    }

    #[test]
    fn ids_increment() {
        let mut g = new_render_graph();
        let a = rg_add_node(&mut g, "a", PassKind::Render);
        let b = rg_add_node(&mut g, "b", PassKind::Compute);
        assert_ne!(a, b);
    }

    #[test]
    fn add_dep() {
        let mut g = new_render_graph();
        let a = rg_add_node(&mut g, "a", PassKind::Render);
        let b = rg_add_node(&mut g, "b", PassKind::Render);
        rg_add_dep(&mut g, b, a);
        assert_eq!(rg_dep_count(&g, b), 1);
    }

    #[test]
    fn set_disabled() {
        let mut g = new_render_graph();
        let id = rg_add_node(&mut g, "x", PassKind::Transfer);
        rg_set_enabled(&mut g, id, false);
        assert_eq!(rg_enabled_count(&g), 0);
    }

    #[test]
    fn all_enabled_by_default() {
        let mut g = new_render_graph();
        rg_add_node(&mut g, "a", PassKind::Render);
        rg_add_node(&mut g, "b", PassKind::Render);
        assert_eq!(rg_enabled_count(&g), 2);
    }

    #[test]
    fn pass_name() {
        assert_eq!(rg_pass_name(PassKind::Compute), "compute");
    }

    #[test]
    fn clear() {
        let mut g = new_render_graph();
        rg_add_node(&mut g, "x", PassKind::Render);
        rg_clear(&mut g);
        assert_eq!(rg_node_count(&g), 0);
    }

    #[test]
    fn topo_sort_len() {
        let mut g = new_render_graph();
        rg_add_node(&mut g, "a", PassKind::Render);
        rg_add_node(&mut g, "b", PassKind::Render);
        assert_eq!(rg_topo_sort(&g).len(), 2);
    }

    #[test]
    fn json_has_nodes() {
        assert!(rg_to_json(&new_render_graph()).contains("nodes"));
    }
}
