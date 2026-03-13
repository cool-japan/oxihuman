// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! draw.io XML export.

/// A draw.io cell (node).
#[derive(Debug, Clone)]
pub struct DrawioCell {
    pub id: String,
    pub value: String,
    pub style: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DrawioCell {
    pub fn new(id: &str, value: &str, x: f32, y: f32) -> Self {
        Self {
            id: id.to_string(),
            value: value.to_string(),
            style: "rounded=1;".to_string(),
            x,
            y,
            width: 120.0,
            height: 60.0,
        }
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }
}

/// A draw.io edge (connection).
#[derive(Debug, Clone)]
pub struct DrawioEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub value: Option<String>,
}

impl DrawioEdge {
    pub fn new(id: &str, source: &str, target: &str) -> Self {
        Self { id: id.to_string(), source: source.to_string(), target: target.to_string(), value: None }
    }
}

/// draw.io diagram export.
#[derive(Debug, Clone, Default)]
pub struct DrawioExport {
    pub cells: Vec<DrawioCell>,
    pub edges: Vec<DrawioEdge>,
}

impl DrawioExport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cell(&mut self, cell: DrawioCell) {
        self.cells.push(cell);
    }

    pub fn add_edge(&mut self, edge: DrawioEdge) {
        self.edges.push(edge);
    }
}

/// Serialize to draw.io XML format.
pub fn to_drawio_xml(d: &DrawioExport) -> String {
    let mut out = String::from(
        "<mxGraphModel><root><mxCell id=\"0\"/><mxCell id=\"1\" parent=\"0\"/>\n",
    );
    for c in &d.cells {
        out.push_str(&format!(
            "<mxCell id=\"{}\" value=\"{}\" style=\"{}\" vertex=\"1\" parent=\"1\">\
             <mxGeometry x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" as=\"geometry\"/>\
             </mxCell>\n",
            c.id, c.value, c.style, c.x, c.y, c.width, c.height
        ));
    }
    for e in &d.edges {
        let val = e.value.as_deref().unwrap_or("");
        out.push_str(&format!(
            "<mxCell id=\"{}\" value=\"{}\" edge=\"1\" source=\"{}\" target=\"{}\" parent=\"1\">\
             <mxGeometry relative=\"1\" as=\"geometry\"/></mxCell>\n",
            e.id, val, e.source, e.target
        ));
    }
    out.push_str("</root></mxGraphModel>\n");
    out
}

/// Count cells.
pub fn drawio_cell_count(d: &DrawioExport) -> usize {
    d.cells.len()
}

/// Count edges.
pub fn drawio_edge_count(d: &DrawioExport) -> usize {
    d.edges.len()
}

/// Create a new draw.io export.
pub fn new_drawio_export() -> DrawioExport {
    DrawioExport::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_drawio_export_empty() {
        let d = new_drawio_export();
        assert_eq!(drawio_cell_count(&d), 0);
    }

    #[test]
    fn test_add_cell() {
        let mut d = DrawioExport::new();
        d.add_cell(DrawioCell::new("c1", "Hello", 10.0, 20.0));
        assert_eq!(drawio_cell_count(&d), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut d = DrawioExport::new();
        d.add_edge(DrawioEdge::new("e1", "c1", "c2"));
        assert_eq!(drawio_edge_count(&d), 1);
    }

    #[test]
    fn test_to_drawio_xml_root_structure() {
        let d = DrawioExport::new();
        let s = to_drawio_xml(&d);
        assert!(s.contains("mxGraphModel"));
        assert!(s.contains("mxCell"));
    }

    #[test]
    fn test_to_drawio_xml_has_cell() {
        let mut d = DrawioExport::new();
        d.add_cell(DrawioCell::new("n1", "My Node", 0.0, 0.0));
        let s = to_drawio_xml(&d);
        assert!(s.contains("My Node"));
    }

    #[test]
    fn test_to_drawio_xml_has_edge() {
        let mut d = DrawioExport::new();
        d.add_edge(DrawioEdge::new("e1", "src", "tgt"));
        let s = to_drawio_xml(&d);
        assert!(s.contains("src"));
        assert!(s.contains("tgt"));
    }

    #[test]
    fn test_cell_with_style() {
        let c = DrawioCell::new("x", "X", 0.0, 0.0).with_style("ellipse;");
        assert_eq!(c.style, "ellipse;");
    }

    #[test]
    fn test_drawio_cell_default_dimensions() {
        let c = DrawioCell::new("c", "C", 10.0, 20.0);
        assert!((c.width - 120.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_drawio_xml_multiple_cells() {
        let mut d = DrawioExport::new();
        d.add_cell(DrawioCell::new("c1", "A", 0.0, 0.0));
        d.add_cell(DrawioCell::new("c2", "B", 200.0, 0.0));
        let s = to_drawio_xml(&d);
        assert!(s.contains("c1"));
        assert!(s.contains("c2"));
    }

    #[test]
    fn test_to_drawio_xml_ends_with_graphmodel() {
        let d = DrawioExport::new();
        let s = to_drawio_xml(&d);
        assert!(s.contains("</mxGraphModel>"));
    }
}
