// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! JSON-LD linked data export stub.

use std::collections::BTreeMap;

/// A JSON-LD node (simplified representation).
#[derive(Debug, Clone)]
pub struct JsonLdNode {
    /// Node ID (IRI or blank node).
    pub id: String,
    /// RDF type (e.g. `schema:Person`).
    pub types: Vec<String>,
    /// Properties as key → value strings.
    pub properties: BTreeMap<String, String>,
}

impl JsonLdNode {
    /// Create a new node.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            types: Vec::new(),
            properties: BTreeMap::new(),
        }
    }

    /// Add an RDF type.
    pub fn add_type(&mut self, t: impl Into<String>) {
        self.types.push(t.into());
    }

    /// Set a property.
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// Get a property.
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }
}

/// A JSON-LD document.
#[derive(Debug, Clone)]
pub struct JsonLdDocument {
    /// Context map.
    pub context: BTreeMap<String, String>,
    /// Nodes in the document.
    pub nodes: Vec<JsonLdNode>,
}

impl Default for JsonLdDocument {
    fn default() -> Self {
        let mut ctx = BTreeMap::new();
        ctx.insert("schema".into(), "https://schema.org/".into());
        Self {
            context: ctx,
            nodes: Vec::new(),
        }
    }
}

impl JsonLdDocument {
    /// Add a node.
    pub fn add_node(&mut self, node: JsonLdNode) {
        self.nodes.push(node);
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Find node by ID.
    pub fn find_node(&self, id: &str) -> Option<&JsonLdNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

/// Serialize a node to a JSON object string.
pub fn node_to_json(node: &JsonLdNode) -> String {
    let mut out = format!(r#"{{"@id":"{}""#, node.id);
    if !node.types.is_empty() {
        let types: Vec<String> = node.types.iter().map(|t| format!("\"{t}\"")).collect();
        out.push_str(&format!(r#","@type":[{}]"#, types.join(",")));
    }
    for (k, v) in &node.properties {
        out.push_str(&format!(r#","{k}":"{v}""#));
    }
    out.push('}');
    out
}

/// Serialize the entire JSON-LD document.
pub fn render_json_ld(doc: &JsonLdDocument) -> String {
    let ctx_pairs: Vec<String> = doc
        .context
        .iter()
        .map(|(k, v)| format!(r#""{k}":"{v}""#))
        .collect();
    let nodes: Vec<String> = doc.nodes.iter().map(node_to_json).collect();
    format!(
        r#"{{"@context":{{{}}},"@graph":[{}]}}"#,
        ctx_pairs.join(","),
        nodes.join(",")
    )
}

/// Validate that all nodes have non-empty IDs.
pub fn validate_document(doc: &JsonLdDocument) -> bool {
    doc.nodes.iter().all(|n| !n.id.is_empty())
}

/// Add a standard Schema.org context prefix.
pub fn add_schema_context(doc: &mut JsonLdDocument) {
    doc.context
        .insert("schema".into(), "https://schema.org/".into());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node() -> JsonLdNode {
        let mut n = JsonLdNode::new("https://example.com/person/1");
        n.add_type("schema:Person");
        n.set_property("schema:name", "Alice");
        n.set_property("schema:email", "alice@example.com");
        n
    }

    #[test]
    fn node_count() {
        let mut doc = JsonLdDocument::default();
        doc.add_node(sample_node());
        assert_eq!(doc.node_count(), 1);
    }

    #[test]
    fn find_node_found() {
        let mut doc = JsonLdDocument::default();
        doc.add_node(sample_node());
        assert!(doc.find_node("https://example.com/person/1").is_some());
    }

    #[test]
    fn find_node_missing() {
        assert!(JsonLdDocument::default().find_node("nope").is_none());
    }

    #[test]
    fn get_property() {
        assert_eq!(sample_node().get_property("schema:name"), Some("Alice"));
    }

    #[test]
    fn node_to_json_contains_id() {
        let s = node_to_json(&sample_node());
        assert!(s.contains("@id"));
    }

    #[test]
    fn node_to_json_contains_type() {
        let s = node_to_json(&sample_node());
        assert!(s.contains("@type"));
    }

    #[test]
    fn render_json_ld_context() {
        let doc = JsonLdDocument::default();
        assert!(render_json_ld(&doc).contains("@context"));
    }

    #[test]
    fn render_json_ld_graph() {
        let doc = JsonLdDocument::default();
        assert!(render_json_ld(&doc).contains("@graph"));
    }

    #[test]
    fn validate_ok() {
        let mut doc = JsonLdDocument::default();
        doc.add_node(sample_node());
        assert!(validate_document(&doc));
    }

    #[test]
    fn validate_empty_id() {
        let mut doc = JsonLdDocument::default();
        doc.add_node(JsonLdNode::new(""));
        assert!(!validate_document(&doc));
    }
}
