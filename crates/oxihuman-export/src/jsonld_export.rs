// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! JSON-LD (Linked Data) document export.

/// A JSON-LD context entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LdContextEntry {
    pub prefix: String,
    pub iri: String,
}

/// A JSON-LD node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LdNode {
    pub id: Option<String>,
    pub ld_type: Option<String>,
    pub properties: Vec<(String, String)>,
}

impl LdNode {
    #[allow(dead_code)]
    pub fn new() -> Self {
        LdNode {
            id: None,
            ld_type: None,
            properties: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_type(mut self, t: &str) -> Self {
        self.ld_type = Some(t.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn add_property(&mut self, key: &str, val: &str) {
        self.properties.push((key.to_string(), val.to_string()));
    }
}

impl Default for LdNode {
    fn default() -> Self {
        LdNode::new()
    }
}

/// A JSON-LD document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LdDocument {
    pub context: Vec<LdContextEntry>,
    pub nodes: Vec<LdNode>,
}

impl LdDocument {
    #[allow(dead_code)]
    pub fn new() -> Self {
        LdDocument {
            context: Vec::new(),
            nodes: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_context(&mut self, prefix: &str, iri: &str) {
        self.context.push(LdContextEntry {
            prefix: prefix.to_string(),
            iri: iri.to_string(),
        });
    }

    #[allow(dead_code)]
    pub fn add_node(&mut self, node: LdNode) {
        self.nodes.push(node);
    }
}

impl Default for LdDocument {
    fn default() -> Self {
        LdDocument::new()
    }
}

/// Serialize a JSON-LD node to JSON.
#[allow(dead_code)]
pub fn serialize_node(node: &LdNode) -> String {
    let mut parts = Vec::new();
    if let Some(ref id) = node.id {
        parts.push(format!(r#""@id":"{}""#, id));
    }
    if let Some(ref t) = node.ld_type {
        parts.push(format!(r#""@type":"{}""#, t));
    }
    for (k, v) in &node.properties {
        parts.push(format!(r#""{}":"{}""#, k, v));
    }
    format!("{{{}}}", parts.join(","))
}

/// Export a JSON-LD document to JSON text.
#[allow(dead_code)]
pub fn export_jsonld(doc: &LdDocument) -> String {
    let ctx_parts: Vec<String> = doc
        .context
        .iter()
        .map(|e| format!(r#""{}":{{"@id":"{}"}}"#, e.prefix, e.iri))
        .collect();
    let ctx = format!("{{{}}}", ctx_parts.join(","));

    let nodes: Vec<String> = doc.nodes.iter().map(serialize_node).collect();
    let graph = format!("[{}]", nodes.join(","));

    format!(r#"{{"@context":{},"@graph":{}}}"#, ctx, graph)
}

/// Node count.
#[allow(dead_code)]
pub fn node_count(doc: &LdDocument) -> usize {
    doc.nodes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_empty() {
        let doc = LdDocument::new();
        assert_eq!(node_count(&doc), 0);
    }

    #[test]
    fn add_node_count() {
        let mut doc = LdDocument::new();
        doc.add_node(LdNode::new().with_id("http://example.com/1"));
        assert_eq!(node_count(&doc), 1);
    }

    #[test]
    fn serialize_node_with_id() {
        let node = LdNode::new().with_id("http://example.com/1");
        let s = serialize_node(&node);
        assert!(s.contains("@id"));
    }

    #[test]
    fn serialize_node_with_type() {
        let node = LdNode::new().with_type("Person");
        let s = serialize_node(&node);
        assert!(s.contains("Person"));
    }

    #[test]
    fn serialize_node_with_property() {
        let mut node = LdNode::new();
        node.add_property("schema:name", "Alice");
        let s = serialize_node(&node);
        assert!(s.contains("Alice"));
    }

    #[test]
    fn export_contains_context() {
        let mut doc = LdDocument::new();
        doc.add_context("schema", "http://schema.org/");
        let s = export_jsonld(&doc);
        assert!(s.contains("@context"));
    }

    #[test]
    fn export_contains_graph() {
        let doc = LdDocument::new();
        let s = export_jsonld(&doc);
        assert!(s.contains("@graph"));
    }

    #[test]
    fn context_prefix_in_export() {
        let mut doc = LdDocument::new();
        doc.add_context("schema", "http://schema.org/");
        let s = export_jsonld(&doc);
        assert!(s.contains("schema"));
    }

    #[test]
    fn node_id_in_export() {
        let mut doc = LdDocument::new();
        doc.add_node(LdNode::new().with_id("http://example.com/1"));
        let s = export_jsonld(&doc);
        assert!(s.contains("http://example.com/1"));
    }

    #[test]
    fn empty_node_braces() {
        let node = LdNode::new();
        assert_eq!(serialize_node(&node), "{}");
    }
}
