// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mermaid diagram export for scene/dependency graphs.

/// Mermaid diagram type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MermaidDiagramType {
    Flowchart,
    Sequence,
    ClassDiagram,
    StateDiagram,
}

impl MermaidDiagramType {
    pub fn header(self, direction: &str) -> String {
        match self {
            Self::Flowchart => format!("flowchart {}", direction),
            Self::Sequence => "sequenceDiagram".to_string(),
            Self::ClassDiagram => "classDiagram".to_string(),
            Self::StateDiagram => "stateDiagram-v2".to_string(),
        }
    }
}

/// A node in the Mermaid diagram.
#[derive(Debug, Clone)]
pub struct MermaidDiagramNode {
    pub id: String,
    pub label: String,
}

/// An edge in the Mermaid diagram.
#[derive(Debug, Clone)]
pub struct MermaidDiagramEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// Mermaid diagram export builder.
#[derive(Debug, Clone)]
pub struct MermaidDiagramExport {
    pub diagram_type: MermaidDiagramType,
    pub direction: String,
    pub nodes: Vec<MermaidDiagramNode>,
    pub edges: Vec<MermaidDiagramEdge>,
}

impl Default for MermaidDiagramExport {
    fn default() -> Self {
        Self {
            diagram_type: MermaidDiagramType::Flowchart,
            direction: "TD".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl MermaidDiagramExport {
    pub fn new(diagram_type: MermaidDiagramType) -> Self {
        Self { diagram_type, ..Self::default() }
    }

    pub fn with_direction(mut self, dir: &str) -> Self {
        self.direction = dir.to_string();
        self
    }

    pub fn add_node(&mut self, id: &str, label: &str) {
        self.nodes.push(MermaidDiagramNode { id: id.to_string(), label: label.to_string() });
    }

    pub fn add_edge(&mut self, from: &str, to: &str, label: Option<&str>) {
        self.edges.push(MermaidDiagramEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(|s| s.to_string()),
        });
    }
}

/// Serialize Mermaid diagram to string.
pub fn to_mermaid_diagram_string(d: &MermaidDiagramExport) -> String {
    let mut out = format!("{}\n", d.diagram_type.header(&d.direction));
    for n in &d.nodes {
        out.push_str(&format!("  {}[\"{}\"]\n", n.id, n.label));
    }
    for e in &d.edges {
        if let Some(lbl) = &e.label {
            out.push_str(&format!("  {} -->|{}| {}\n", e.from, lbl, e.to));
        } else {
            out.push_str(&format!("  {} --> {}\n", e.from, e.to));
        }
    }
    out
}

/// Count nodes.
pub fn mermaid_diagram_node_count(d: &MermaidDiagramExport) -> usize {
    d.nodes.len()
}

/// Count edges.
pub fn mermaid_diagram_edge_count(d: &MermaidDiagramExport) -> usize {
    d.edges.len()
}

/// Create a new default Mermaid diagram export.
pub fn new_mermaid_diagram_export() -> MermaidDiagramExport {
    MermaidDiagramExport::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mermaid_default() {
        let d = new_mermaid_diagram_export();
        assert_eq!(d.diagram_type, MermaidDiagramType::Flowchart);
    }

    #[test]
    fn test_add_node_increments() {
        let mut d = MermaidDiagramExport::default();
        d.add_node("A", "Alpha");
        assert_eq!(mermaid_diagram_node_count(&d), 1);
    }

    #[test]
    fn test_add_edge_increments() {
        let mut d = MermaidDiagramExport::default();
        d.add_edge("A", "B", None);
        assert_eq!(mermaid_diagram_edge_count(&d), 1);
    }

    #[test]
    fn test_to_mermaid_string_contains_flowchart() {
        let d = MermaidDiagramExport::new(MermaidDiagramType::Flowchart);
        let s = to_mermaid_diagram_string(&d);
        assert!(s.contains("flowchart"));
    }

    #[test]
    fn test_to_mermaid_string_has_node() {
        let mut d = MermaidDiagramExport::default();
        d.add_node("N1", "My Node");
        let s = to_mermaid_diagram_string(&d);
        assert!(s.contains("N1"));
    }

    #[test]
    fn test_to_mermaid_string_has_edge() {
        let mut d = MermaidDiagramExport::default();
        d.add_edge("A", "B", None);
        let s = to_mermaid_diagram_string(&d);
        assert!(s.contains("A --> B"));
    }

    #[test]
    fn test_to_mermaid_string_edge_with_label() {
        let mut d = MermaidDiagramExport::default();
        d.add_edge("A", "B", Some("uses"));
        let s = to_mermaid_diagram_string(&d);
        assert!(s.contains("uses"));
    }

    #[test]
    fn test_diagram_type_sequence_header() {
        let h = MermaidDiagramType::Sequence.header("LR");
        assert_eq!(h, "sequenceDiagram");
    }

    #[test]
    fn test_with_direction() {
        let d = MermaidDiagramExport::new(MermaidDiagramType::Flowchart).with_direction("LR");
        assert_eq!(d.direction, "LR");
    }

    #[test]
    fn test_class_diagram_header() {
        let h = MermaidDiagramType::ClassDiagram.header("TD");
        assert_eq!(h, "classDiagram");
    }
}
