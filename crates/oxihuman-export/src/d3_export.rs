// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! D3.js data export for force-directed and hierarchical graphs.

/// A node for D3 force-directed layout.
#[derive(Debug, Clone)]
pub struct D3Node {
    pub id: String,
    pub label: String,
    pub group: u32,
    pub value: f32,
}

impl D3Node {
    pub fn new(id: &str, label: &str) -> Self {
        Self { id: id.to_string(), label: label.to_string(), group: 0, value: 1.0 }
    }

    pub fn with_group(mut self, g: u32) -> Self {
        self.group = g;
        self
    }
}

/// A link for D3 force-directed layout.
#[derive(Debug, Clone)]
pub struct D3Link {
    pub source: String,
    pub target: String,
    pub value: f32,
}

impl D3Link {
    pub fn new(source: &str, target: &str) -> Self {
        Self { source: source.to_string(), target: target.to_string(), value: 1.0 }
    }

    pub fn with_value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }
}

/// D3 graph data export.
#[derive(Debug, Clone, Default)]
pub struct D3Export {
    pub nodes: Vec<D3Node>,
    pub links: Vec<D3Link>,
}

impl D3Export {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: D3Node) {
        self.nodes.push(node);
    }

    pub fn add_link(&mut self, link: D3Link) {
        self.links.push(link);
    }
}

/// Serialize to D3 JSON format.
pub fn to_d3_json(d: &D3Export) -> String {
    let mut nodes_json = String::new();
    for (i, n) in d.nodes.iter().enumerate() {
        if i > 0 {
            nodes_json.push(',');
        }
        nodes_json.push_str(&format!(
            "{{\"id\":\"{}\",\"label\":\"{}\",\"group\":{},\"value\":{}}}",
            n.id, n.label, n.group, n.value
        ));
    }
    let mut links_json = String::new();
    for (i, l) in d.links.iter().enumerate() {
        if i > 0 {
            links_json.push(',');
        }
        links_json.push_str(&format!(
            "{{\"source\":\"{}\",\"target\":\"{}\",\"value\":{}}}",
            l.source, l.target, l.value
        ));
    }
    format!("{{\"nodes\":[{}],\"links\":[{}]}}", nodes_json, links_json)
}

/// Count nodes.
pub fn d3_node_count(d: &D3Export) -> usize {
    d.nodes.len()
}

/// Count links.
pub fn d3_link_count(d: &D3Export) -> usize {
    d.links.len()
}

/// Create a new D3 export.
pub fn new_d3_export() -> D3Export {
    D3Export::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_d3_export_empty() {
        let d = new_d3_export();
        assert_eq!(d3_node_count(&d), 0);
    }

    #[test]
    fn test_add_node() {
        let mut d = D3Export::new();
        d.add_node(D3Node::new("a", "Alpha"));
        assert_eq!(d3_node_count(&d), 1);
    }

    #[test]
    fn test_add_link() {
        let mut d = D3Export::new();
        d.add_link(D3Link::new("a", "b"));
        assert_eq!(d3_link_count(&d), 1);
    }

    #[test]
    fn test_to_d3_json_structure() {
        let d = D3Export::new();
        let s = to_d3_json(&d);
        assert!(s.contains("nodes"));
        assert!(s.contains("links"));
    }

    #[test]
    fn test_to_d3_json_has_node() {
        let mut d = D3Export::new();
        d.add_node(D3Node::new("n1", "NodeOne"));
        let s = to_d3_json(&d);
        assert!(s.contains("n1"));
        assert!(s.contains("NodeOne"));
    }

    #[test]
    fn test_to_d3_json_has_link() {
        let mut d = D3Export::new();
        d.add_link(D3Link::new("a", "b").with_value(2.0));
        let s = to_d3_json(&d);
        assert!(s.contains("\"source\":\"a\""));
        assert!(s.contains("\"target\":\"b\""));
    }

    #[test]
    fn test_d3_node_with_group() {
        let n = D3Node::new("x", "X").with_group(3);
        assert_eq!(n.group, 3);
    }

    #[test]
    fn test_d3_link_with_value() {
        let l = D3Link::new("a", "b").with_value(5.0);
        assert!((l.value - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_d3_json_multiple_nodes() {
        let mut d = D3Export::new();
        d.add_node(D3Node::new("a", "A"));
        d.add_node(D3Node::new("b", "B"));
        let s = to_d3_json(&d);
        assert!(s.contains("\"id\":\"a\""));
        assert!(s.contains("\"id\":\"b\""));
    }

    #[test]
    fn test_to_d3_json_empty() {
        let d = D3Export::new();
        let s = to_d3_json(&d);
        assert!(s.contains("\"nodes\":[]"));
        assert!(s.contains("\"links\":[]"));
    }
}
