// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Graphviz DOT format scene graph export.

#[derive(Debug, Clone)]
pub struct DotNode {
    pub id: String,
    pub label: String,
    pub shape: String,
}

#[derive(Debug, Clone)]
pub struct DotEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct DotGraph {
    pub name: String,
    pub directed: bool,
    pub nodes: Vec<DotNode>,
    pub edges: Vec<DotEdge>,
}

pub fn new_dot_graph(name: &str, directed: bool) -> DotGraph {
    DotGraph {
        name: name.to_string(),
        directed,
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

pub fn add_dot_node(graph: &mut DotGraph, id: &str, label: &str, shape: &str) {
    graph.nodes.push(DotNode {
        id: id.to_string(),
        label: label.to_string(),
        shape: shape.to_string(),
    });
}

pub fn add_dot_edge(graph: &mut DotGraph, from: &str, to: &str, label: &str) {
    graph.edges.push(DotEdge {
        from: from.to_string(),
        to: to.to_string(),
        label: label.to_string(),
    });
}

pub fn render_dot(graph: &DotGraph) -> String {
    let kind = if graph.directed { "digraph" } else { "graph" };
    let arrow = if graph.directed { "->" } else { "--" };
    let mut s = format!("{} {} {{\n  rankdir=TB;\n", kind, graph.name);
    for n in &graph.nodes {
        s.push_str(&format!(
            "  {} [label=\"{}\", shape={}];\n",
            n.id, n.label, n.shape
        ));
    }
    for e in &graph.edges {
        let lbl = if e.label.is_empty() {
            String::new()
        } else {
            format!(" [label=\"{}\"]", e.label)
        };
        s.push_str(&format!("  {} {} {}{};\n", e.from, arrow, e.to, lbl));
    }
    s.push_str("}\n");
    s
}

pub fn export_dot(graph: &DotGraph) -> Vec<u8> {
    render_dot(graph).into_bytes()
}
pub fn dot_node_count(graph: &DotGraph) -> usize {
    graph.nodes.len()
}
pub fn dot_edge_count(graph: &DotGraph) -> usize {
    graph.edges.len()
}
pub fn validate_dot_graph(graph: &DotGraph) -> bool {
    !graph.name.is_empty()
}
pub fn dot_size_bytes(graph: &DotGraph) -> usize {
    render_dot(graph).len()
}

pub fn scene_to_dot(nodes: &[(&str, Option<&str>)]) -> DotGraph {
    let mut g = new_dot_graph("scene", true);
    for (name, parent) in nodes {
        add_dot_node(&mut g, name, name, "box");
        if let Some(p) = parent {
            add_dot_edge(&mut g, p, name, "");
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dot_graph() {
        let g = new_dot_graph("test", true);
        assert_eq!(g.name, "test");
        assert!(g.directed);
    }

    #[test]
    fn test_add_node_edge() {
        let mut g = new_dot_graph("g", true);
        add_dot_node(&mut g, "a", "A", "box");
        add_dot_edge(&mut g, "a", "b", "");
        assert_eq!(dot_node_count(&g), 1);
        assert_eq!(dot_edge_count(&g), 1);
    }

    #[test]
    fn test_render_dot_contains_digraph() {
        let g = new_dot_graph("mygraph", true);
        let s = render_dot(&g);
        assert!(s.contains("digraph"));
    }

    #[test]
    fn test_render_dot_undirected() {
        let g = new_dot_graph("g", false);
        let s = render_dot(&g);
        assert!(s.contains("graph"));
        assert!(!s.contains("digraph"));
    }

    #[test]
    fn test_export_dot_nonempty() {
        let g = new_dot_graph("g", true);
        assert!(!export_dot(&g).is_empty());
    }

    #[test]
    fn test_validate_dot_graph() {
        let g = new_dot_graph("v", true);
        assert!(validate_dot_graph(&g));
    }

    #[test]
    fn test_scene_to_dot() {
        let nodes = vec![("root", None), ("child", Some("root"))];
        let g = scene_to_dot(&nodes);
        assert_eq!(dot_node_count(&g), 2);
        assert_eq!(dot_edge_count(&g), 1);
    }

    #[test]
    fn test_dot_size_bytes() {
        let g = new_dot_graph("x", true);
        assert!(dot_size_bytes(&g) > 0);
    }
}
