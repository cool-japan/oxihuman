// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export skeleton/scene graph as a Mermaid diagram.

#![allow(dead_code)]

/// A Mermaid node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MermaidNode {
    pub id: String,
    pub label: String,
}

/// A Mermaid edge.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MermaidEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

/// A Mermaid diagram export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MermaidExport {
    pub diagram_type: String,
    pub direction: String,
    pub nodes: Vec<MermaidNode>,
    pub edges: Vec<MermaidEdge>,
}

/// Create a new Mermaid flowchart export.
#[allow(dead_code)]
pub fn new_mermaid_export(direction: &str) -> MermaidExport {
    MermaidExport {
        diagram_type: "flowchart".to_string(),
        direction: direction.to_string(),
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

/// Add a node with a label.
#[allow(dead_code)]
pub fn add_mermaid_node(doc: &mut MermaidExport, id: &str, label: &str) {
    doc.nodes.push(MermaidNode {
        id: id.to_string(),
        label: label.to_string(),
    });
}

/// Add an edge with optional label.
#[allow(dead_code)]
pub fn add_mermaid_edge(doc: &mut MermaidExport, from: &str, to: &str, label: &str) {
    doc.edges.push(MermaidEdge {
        from: from.to_string(),
        to: to.to_string(),
        label: label.to_string(),
    });
}

/// Return node count.
#[allow(dead_code)]
pub fn mermaid_node_count(doc: &MermaidExport) -> usize {
    doc.nodes.len()
}

/// Return edge count.
#[allow(dead_code)]
pub fn mermaid_edge_count(doc: &MermaidExport) -> usize {
    doc.edges.len()
}

/// Serialise as Mermaid text.
#[allow(dead_code)]
pub fn to_mermaid_string(doc: &MermaidExport) -> String {
    let mut out = format!("{} {}\n", doc.diagram_type, doc.direction);
    for node in &doc.nodes {
        out.push_str(&format!("  {}[\"{}\"]\n", node.id, node.label));
    }
    for edge in &doc.edges {
        if edge.label.is_empty() {
            out.push_str(&format!("  {} --> {}\n", edge.from, edge.to));
        } else {
            out.push_str(&format!(
                "  {} -->|{}| {}\n",
                edge.from, edge.label, edge.to
            ));
        }
    }
    out
}

/// Export a skeleton hierarchy as Mermaid.
#[allow(dead_code)]
pub fn export_skeleton_mermaid(bones: &[(&str, Option<&str>)]) -> String {
    let mut doc = new_mermaid_export("TD");
    for &(bone, parent) in bones {
        add_mermaid_node(&mut doc, bone, bone);
        if let Some(p) = parent {
            add_mermaid_edge(&mut doc, p, bone, "");
        }
    }
    to_mermaid_string(&doc)
}

/// Find a node by id.
#[allow(dead_code)]
pub fn find_mermaid_node<'a>(doc: &'a MermaidExport, id: &str) -> Option<&'a MermaidNode> {
    doc.nodes.iter().find(|n| n.id == id)
}

/// Change the diagram direction.
#[allow(dead_code)]
pub fn set_mermaid_direction(doc: &mut MermaidExport, direction: &str) {
    doc.direction = direction.to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mermaid_export() {
        let doc = new_mermaid_export("TD");
        assert_eq!(doc.direction, "TD");
        assert_eq!(mermaid_node_count(&doc), 0);
    }

    #[test]
    fn test_add_node() {
        let mut doc = new_mermaid_export("TD");
        add_mermaid_node(&mut doc, "hip", "Hip");
        assert_eq!(mermaid_node_count(&doc), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut doc = new_mermaid_export("TD");
        add_mermaid_edge(&mut doc, "a", "b", "");
        assert_eq!(mermaid_edge_count(&doc), 1);
    }

    #[test]
    fn test_to_mermaid_contains_flowchart() {
        let doc = new_mermaid_export("LR");
        let s = to_mermaid_string(&doc);
        assert!(s.contains("flowchart"));
    }

    #[test]
    fn test_to_mermaid_contains_node() {
        let mut doc = new_mermaid_export("TD");
        add_mermaid_node(&mut doc, "spine", "Spine");
        let s = to_mermaid_string(&doc);
        assert!(s.contains("spine"));
    }

    #[test]
    fn test_to_mermaid_contains_edge() {
        let mut doc = new_mermaid_export("TD");
        add_mermaid_edge(&mut doc, "a", "b", "");
        let s = to_mermaid_string(&doc);
        assert!(s.contains("-->"));
    }

    #[test]
    fn test_edge_with_label() {
        let mut doc = new_mermaid_export("TD");
        add_mermaid_edge(&mut doc, "a", "b", "child");
        let s = to_mermaid_string(&doc);
        assert!(s.contains("|child|"));
    }

    #[test]
    fn test_export_skeleton_mermaid() {
        let bones = vec![("root", None), ("spine", Some("root"))];
        let s = export_skeleton_mermaid(&bones);
        assert!(s.contains("root"));
    }

    #[test]
    fn test_find_mermaid_node() {
        let mut doc = new_mermaid_export("TD");
        add_mermaid_node(&mut doc, "knee", "Knee");
        assert!(find_mermaid_node(&doc, "knee").is_some());
    }

    #[test]
    fn test_set_direction() {
        let mut doc = new_mermaid_export("TD");
        set_mermaid_direction(&mut doc, "LR");
        assert_eq!(doc.direction, "LR");
    }
}
