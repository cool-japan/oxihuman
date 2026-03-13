// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export a dependency/scene graph as Graphviz DOT format.

#![allow(dead_code)]

/// A node in the DOT graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DotNode {
    pub id: String,
    pub label: String,
    pub shape: String,
}

/// An edge in the DOT graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DotEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

/// A DOT graph export document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DotExport {
    pub name: String,
    pub directed: bool,
    pub nodes: Vec<DotNode>,
    pub edges: Vec<DotEdge>,
}

/// Create a new DOT export.
#[allow(dead_code)]
pub fn new_dot_export(name: &str, directed: bool) -> DotExport {
    DotExport {
        name: name.to_string(),
        directed,
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

/// Add a node.
#[allow(dead_code)]
pub fn add_dot_node(graph: &mut DotExport, id: &str, label: &str, shape: &str) {
    graph.nodes.push(DotNode {
        id: id.to_string(),
        label: label.to_string(),
        shape: shape.to_string(),
    });
}

/// Add an edge.
#[allow(dead_code)]
pub fn add_dot_edge(graph: &mut DotExport, from: &str, to: &str, label: &str) {
    graph.edges.push(DotEdge {
        from: from.to_string(),
        to: to.to_string(),
        label: label.to_string(),
    });
}

/// Return number of nodes.
#[allow(dead_code)]
pub fn dot_node_count(graph: &DotExport) -> usize {
    graph.nodes.len()
}

/// Return number of edges.
#[allow(dead_code)]
pub fn dot_edge_count(graph: &DotExport) -> usize {
    graph.edges.len()
}

/// Serialise as DOT string.
#[allow(dead_code)]
pub fn to_dot_string(graph: &DotExport) -> String {
    let kw = if graph.directed { "digraph" } else { "graph" };
    let arrow = if graph.directed { " -> " } else { " -- " };
    let mut out = format!("{} {} {{\n", kw, graph.name);
    for node in &graph.nodes {
        out.push_str(&format!(
            "  {} [label=\"{}\", shape={}];\n",
            node.id, node.label, node.shape
        ));
    }
    for edge in &graph.edges {
        let lbl = if edge.label.is_empty() {
            String::new()
        } else {
            format!(" [label=\"{}\"]", edge.label)
        };
        out.push_str(&format!("  {}{}{}{};\n", edge.from, arrow, edge.to, lbl));
    }
    out.push('}');
    out
}

/// Find a node by id.
#[allow(dead_code)]
pub fn find_dot_node<'a>(graph: &'a DotExport, id: &str) -> Option<&'a DotNode> {
    graph.nodes.iter().find(|n| n.id == id)
}

/// Export a simple skeleton as a DOT graph.
#[allow(dead_code)]
pub fn export_skeleton_dot(bones: &[(&str, Option<&str>)]) -> String {
    let mut graph = new_dot_export("skeleton", true);
    for &(bone, parent) in bones {
        add_dot_node(&mut graph, bone, bone, "ellipse");
        if let Some(p) = parent {
            add_dot_edge(&mut graph, p, bone, "");
        }
    }
    to_dot_string(&graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dot_export_empty() {
        let g = new_dot_export("g", true);
        assert_eq!(dot_node_count(&g), 0);
    }

    #[test]
    fn test_add_node() {
        let mut g = new_dot_export("g", true);
        add_dot_node(&mut g, "n1", "Node 1", "box");
        assert_eq!(dot_node_count(&g), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut g = new_dot_export("g", true);
        add_dot_edge(&mut g, "a", "b", "dep");
        assert_eq!(dot_edge_count(&g), 1);
    }

    #[test]
    fn test_to_dot_directed() {
        let g = new_dot_export("mygraph", true);
        let s = to_dot_string(&g);
        assert!(s.contains("digraph"));
    }

    #[test]
    fn test_to_dot_undirected() {
        let g = new_dot_export("g", false);
        let s = to_dot_string(&g);
        assert!(s.contains("graph"));
    }

    #[test]
    fn test_to_dot_contains_node() {
        let mut g = new_dot_export("g", true);
        add_dot_node(&mut g, "n1", "MyNode", "box");
        let s = to_dot_string(&g);
        assert!(s.contains("n1"));
    }

    #[test]
    fn test_to_dot_contains_edge() {
        let mut g = new_dot_export("g", true);
        add_dot_edge(&mut g, "a", "b", "");
        let s = to_dot_string(&g);
        assert!(s.contains("->"));
    }

    #[test]
    fn test_find_dot_node() {
        let mut g = new_dot_export("g", true);
        add_dot_node(&mut g, "hip", "Hip", "ellipse");
        let n = find_dot_node(&g, "hip");
        assert!(n.is_some());
    }

    #[test]
    fn test_export_skeleton_dot() {
        let bones = vec![("root", None), ("hip", Some("root")), ("knee", Some("hip"))];
        let s = export_skeleton_dot(&bones);
        assert!(s.contains("digraph"));
        assert!(s.contains("root"));
    }

    #[test]
    fn test_find_missing_node() {
        let g = new_dot_export("g", true);
        assert!(find_dot_node(&g, "missing").is_none());
    }
}
