// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export a graph as GraphML XML.

#![allow(dead_code)]

/// A GraphML node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphMlNode {
    pub id: String,
    pub label: String,
}

/// A GraphML edge.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GraphMlEdge {
    pub source: String,
    pub target: String,
    pub id: String,
}

/// A GraphML export document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GraphMlExport {
    pub graph_id: String,
    pub directed: bool,
    pub nodes: Vec<GraphMlNode>,
    pub edges: Vec<GraphMlEdge>,
}

/// Create a new GraphML export.
#[allow(dead_code)]
pub fn new_graphml_export(graph_id: &str, directed: bool) -> GraphMlExport {
    GraphMlExport {
        graph_id: graph_id.to_string(),
        directed,
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

/// Add a node.
#[allow(dead_code)]
pub fn add_graphml_node(doc: &mut GraphMlExport, id: &str, label: &str) {
    doc.nodes.push(GraphMlNode {
        id: id.to_string(),
        label: label.to_string(),
    });
}

/// Add an edge.
#[allow(dead_code)]
pub fn add_graphml_edge(doc: &mut GraphMlExport, source: &str, target: &str) {
    let edge_id = format!("e{}", doc.edges.len());
    doc.edges.push(GraphMlEdge {
        source: source.to_string(),
        target: target.to_string(),
        id: edge_id,
    });
}

/// Return node count.
#[allow(dead_code)]
pub fn graphml_node_count(doc: &GraphMlExport) -> usize {
    doc.nodes.len()
}

/// Return edge count.
#[allow(dead_code)]
pub fn graphml_edge_count(doc: &GraphMlExport) -> usize {
    doc.edges.len()
}

/// Serialise as GraphML XML.
#[allow(dead_code)]
pub fn to_graphml_string(doc: &GraphMlExport) -> String {
    let edge_default = if doc.directed {
        "directed"
    } else {
        "undirected"
    };
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/graphml\">\n");
    out.push_str(&format!(
        "  <graph id=\"{}\" edgedefault=\"{}\">\n",
        doc.graph_id, edge_default
    ));
    for node in &doc.nodes {
        out.push_str(&format!(
            "    <node id=\"{}\"><data key=\"label\">{}</data></node>\n",
            node.id, node.label
        ));
    }
    for edge in &doc.edges {
        out.push_str(&format!(
            "    <edge id=\"{}\" source=\"{}\" target=\"{}\"/>\n",
            edge.id, edge.source, edge.target
        ));
    }
    out.push_str("  </graph>\n</graphml>");
    out
}

/// Find a node by id.
#[allow(dead_code)]
pub fn find_graphml_node<'a>(doc: &'a GraphMlExport, id: &str) -> Option<&'a GraphMlNode> {
    doc.nodes.iter().find(|n| n.id == id)
}

/// Export a list of bones as GraphML.
#[allow(dead_code)]
pub fn export_bones_graphml(bones: &[(&str, Option<&str>)]) -> String {
    let mut doc = new_graphml_export("skeleton", true);
    for &(bone, parent) in bones {
        add_graphml_node(&mut doc, bone, bone);
        if let Some(p) = parent {
            add_graphml_edge(&mut doc, p, bone);
        }
    }
    to_graphml_string(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graphml_empty() {
        let doc = new_graphml_export("g", true);
        assert_eq!(graphml_node_count(&doc), 0);
        assert_eq!(graphml_edge_count(&doc), 0);
    }

    #[test]
    fn test_add_node() {
        let mut doc = new_graphml_export("g", true);
        add_graphml_node(&mut doc, "n1", "Node 1");
        assert_eq!(graphml_node_count(&doc), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut doc = new_graphml_export("g", true);
        add_graphml_edge(&mut doc, "a", "b");
        assert_eq!(graphml_edge_count(&doc), 1);
    }

    #[test]
    fn test_to_graphml_contains_xml_header() {
        let doc = new_graphml_export("g", true);
        let s = to_graphml_string(&doc);
        assert!(s.contains("<?xml"));
    }

    #[test]
    fn test_to_graphml_contains_graph_id() {
        let doc = new_graphml_export("mygraph", true);
        let s = to_graphml_string(&doc);
        assert!(s.contains("mygraph"));
    }

    #[test]
    fn test_to_graphml_contains_node() {
        let mut doc = new_graphml_export("g", true);
        add_graphml_node(&mut doc, "hip", "Hip");
        let s = to_graphml_string(&doc);
        assert!(s.contains("hip"));
    }

    #[test]
    fn test_to_graphml_contains_edge() {
        let mut doc = new_graphml_export("g", true);
        add_graphml_edge(&mut doc, "a", "b");
        let s = to_graphml_string(&doc);
        assert!(s.contains("<edge"));
    }

    #[test]
    fn test_find_graphml_node() {
        let mut doc = new_graphml_export("g", true);
        add_graphml_node(&mut doc, "knee", "Knee");
        assert!(find_graphml_node(&doc, "knee").is_some());
    }

    #[test]
    fn test_export_bones_graphml() {
        let bones = vec![("root", None), ("spine", Some("root"))];
        let s = export_bones_graphml(&bones);
        assert!(s.contains("skeleton"));
    }

    #[test]
    fn test_undirected_edge_default() {
        let doc = new_graphml_export("g", false);
        let s = to_graphml_string(&doc);
        assert!(s.contains("undirected"));
    }
}
