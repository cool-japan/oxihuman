// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! PlantUML diagram export for scene/dependency graphs.

/// A PlantUML component node.
#[derive(Debug, Clone)]
pub struct PlantUmlNode {
    pub name: String,
    pub stereotype: Option<String>,
    pub color: Option<String>,
}

impl PlantUmlNode {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), stereotype: None, color: None }
    }

    pub fn with_stereotype(mut self, s: &str) -> Self {
        self.stereotype = Some(s.to_string());
        self
    }

    pub fn with_color(mut self, c: &str) -> Self {
        self.color = Some(c.to_string());
        self
    }
}

/// A PlantUML relationship.
#[derive(Debug, Clone)]
pub struct PlantUmlRelation {
    pub from: String,
    pub to: String,
    pub arrow: String,
    pub label: Option<String>,
}

impl PlantUmlRelation {
    pub fn new(from: &str, to: &str) -> Self {
        Self { from: from.to_string(), to: to.to_string(), arrow: "-->".to_string(), label: None }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
}

/// PlantUML diagram export.
#[derive(Debug, Clone, Default)]
pub struct PlantUmlDiagramExport {
    pub title: Option<String>,
    pub nodes: Vec<PlantUmlNode>,
    pub relations: Vec<PlantUmlRelation>,
}

impl PlantUmlDiagramExport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn add_node(&mut self, node: PlantUmlNode) {
        self.nodes.push(node);
    }

    pub fn add_relation(&mut self, rel: PlantUmlRelation) {
        self.relations.push(rel);
    }
}

/// Serialize to PlantUML string.
pub fn to_plantuml_diagram_string(d: &PlantUmlDiagramExport) -> String {
    let mut out = "@startuml\n".to_string();
    if let Some(title) = &d.title {
        out.push_str(&format!("title {}\n", title));
    }
    for n in &d.nodes {
        let stereo = n
            .stereotype
            .as_deref()
            .map(|s| format!(" <<{}>>", s))
            .unwrap_or_default();
        let color = n.color.as_deref().map(|c| format!(" #{}", c)).unwrap_or_default();
        out.push_str(&format!("[{}]{}{}\n", n.name, stereo, color));
    }
    for r in &d.relations {
        let lbl = r.label.as_deref().map(|l| format!(" : {}", l)).unwrap_or_default();
        out.push_str(&format!("{} {} {}{}\n", r.from, r.arrow, r.to, lbl));
    }
    out.push_str("@enduml\n");
    out
}

/// Count nodes.
pub fn plantuml_diagram_node_count(d: &PlantUmlDiagramExport) -> usize {
    d.nodes.len()
}

/// Count relations.
pub fn plantuml_diagram_relation_count(d: &PlantUmlDiagramExport) -> usize {
    d.relations.len()
}

/// Create a new PlantUML export.
pub fn new_plantuml_diagram_export() -> PlantUmlDiagramExport {
    PlantUmlDiagramExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_plantuml_export() {
        let d = new_plantuml_diagram_export();
        assert!(d.nodes.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut d = PlantUmlDiagramExport::new();
        d.add_node(PlantUmlNode::new("MyComp"));
        assert_eq!(plantuml_diagram_node_count(&d), 1);
    }

    #[test]
    fn test_add_relation() {
        let mut d = PlantUmlDiagramExport::new();
        d.add_relation(PlantUmlRelation::new("A", "B"));
        assert_eq!(plantuml_diagram_relation_count(&d), 1);
    }

    #[test]
    fn test_to_plantuml_string_startuml() {
        let d = PlantUmlDiagramExport::new();
        let s = to_plantuml_diagram_string(&d);
        assert!(s.contains("@startuml"));
        assert!(s.contains("@enduml"));
    }

    #[test]
    fn test_to_plantuml_string_has_node() {
        let mut d = PlantUmlDiagramExport::new();
        d.add_node(PlantUmlNode::new("Alpha"));
        let s = to_plantuml_diagram_string(&d);
        assert!(s.contains("Alpha"));
    }

    #[test]
    fn test_to_plantuml_string_has_relation() {
        let mut d = PlantUmlDiagramExport::new();
        d.add_relation(PlantUmlRelation::new("A", "B").with_label("dep"));
        let s = to_plantuml_diagram_string(&d);
        assert!(s.contains("dep"));
    }

    #[test]
    fn test_node_with_stereotype() {
        let n = PlantUmlNode::new("X").with_stereotype("interface");
        let d = PlantUmlDiagramExport { nodes: vec![n], ..Default::default() };
        let s = to_plantuml_diagram_string(&d);
        assert!(s.contains("interface"));
    }

    #[test]
    fn test_with_title() {
        let d = PlantUmlDiagramExport::new().with_title("Scene Graph");
        let s = to_plantuml_diagram_string(&d);
        assert!(s.contains("Scene Graph"));
    }

    #[test]
    fn test_node_with_color() {
        let n = PlantUmlNode::new("Y").with_color("red");
        assert!(n.color.is_some());
    }

    #[test]
    fn test_relation_with_label() {
        let r = PlantUmlRelation::new("A", "B").with_label("owns");
        assert_eq!(r.label.as_deref(), Some("owns"));
    }
}
