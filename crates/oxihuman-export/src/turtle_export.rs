// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Turtle (Terse RDF Triple Language) export.

/// An RDF triple.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

/// A Turtle document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TurtleDoc {
    pub prefixes: Vec<(String, String)>,
    pub triples: Vec<RdfTriple>,
}

impl TurtleDoc {
    #[allow(dead_code)]
    pub fn new() -> Self {
        TurtleDoc::default()
    }

    #[allow(dead_code)]
    pub fn add_prefix(&mut self, prefix: &str, iri: &str) {
        self.prefixes.push((prefix.to_string(), iri.to_string()));
    }

    #[allow(dead_code)]
    pub fn add_triple(&mut self, s: &str, p: &str, o: &str) {
        self.triples.push(RdfTriple {
            subject: s.to_string(),
            predicate: p.to_string(),
            object: o.to_string(),
        });
    }
}

/// Serialize a Turtle document to text.
#[allow(dead_code)]
pub fn export_turtle(doc: &TurtleDoc) -> String {
    let mut out = String::new();
    for (prefix, iri) in &doc.prefixes {
        out.push_str(&format!("@prefix {}: <{}> .\n", prefix, iri));
    }
    if !doc.prefixes.is_empty() && !doc.triples.is_empty() {
        out.push('\n');
    }
    for t in &doc.triples {
        out.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object));
    }
    out
}

/// Triple count.
#[allow(dead_code)]
pub fn triple_count(doc: &TurtleDoc) -> usize {
    doc.triples.len()
}

/// Prefix count.
#[allow(dead_code)]
pub fn prefix_count(doc: &TurtleDoc) -> usize {
    doc.prefixes.len()
}

/// Check if output contains a specific triple.
#[allow(dead_code)]
pub fn contains_triple(doc: &TurtleDoc, s: &str, p: &str) -> bool {
    doc.triples
        .iter()
        .any(|t| t.subject == s && t.predicate == p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc_empty() {
        let doc = TurtleDoc::new();
        assert_eq!(triple_count(&doc), 0);
    }

    #[test]
    fn add_triple_count() {
        let mut doc = TurtleDoc::new();
        doc.add_triple("<http://a>", "<http://b>", "<http://c>");
        assert_eq!(triple_count(&doc), 1);
    }

    #[test]
    fn add_prefix_count() {
        let mut doc = TurtleDoc::new();
        doc.add_prefix("schema", "http://schema.org/");
        assert_eq!(prefix_count(&doc), 1);
    }

    #[test]
    fn export_contains_prefix() {
        let mut doc = TurtleDoc::new();
        doc.add_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
        let s = export_turtle(&doc);
        assert!(s.contains("@prefix rdf:"));
    }

    #[test]
    fn export_contains_triple() {
        let mut doc = TurtleDoc::new();
        doc.add_triple("<http://a>", "<http://b>", "<http://c>");
        let s = export_turtle(&doc);
        assert!(s.contains("<http://a>"));
    }

    #[test]
    fn triple_ends_with_dot() {
        let mut doc = TurtleDoc::new();
        doc.add_triple("<http://a>", "<http://b>", "\"value\"");
        let s = export_turtle(&doc);
        assert!(s.contains(" ."));
    }

    #[test]
    fn contains_triple_helper() {
        let mut doc = TurtleDoc::new();
        doc.add_triple("<http://a>", "rdf:type", "<http://Person>");
        assert!(contains_triple(&doc, "<http://a>", "rdf:type"));
    }

    #[test]
    fn not_contains_nonexistent_triple() {
        let doc = TurtleDoc::new();
        assert!(!contains_triple(&doc, "<http://x>", "rdf:type"));
    }

    #[test]
    fn empty_doc_empty_string() {
        let doc = TurtleDoc::new();
        assert!(export_turtle(&doc).is_empty());
    }

    #[test]
    fn multiple_triples() {
        let mut doc = TurtleDoc::new();
        doc.add_triple("a", "b", "c");
        doc.add_triple("d", "e", "f");
        let s = export_turtle(&doc);
        assert!(s.contains("a b c ."));
        assert!(s.contains("d e f ."));
    }
}
