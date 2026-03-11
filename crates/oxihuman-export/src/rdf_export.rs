// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RDF/Turtle export stub.

/// An RDF triple (subject, predicate, object).
#[derive(Debug, Clone)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    /// Whether the object is a literal (true) or IRI (false).
    pub object_is_literal: bool,
}

impl RdfTriple {
    /// Create an IRI triple.
    pub fn iri(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            object_is_literal: false,
        }
    }

    /// Create a literal triple.
    pub fn literal(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: value.into(),
            object_is_literal: true,
        }
    }
}

/// An RDF graph (collection of triples).
#[derive(Debug, Clone, Default)]
pub struct RdfGraph {
    pub triples: Vec<RdfTriple>,
    pub prefixes: Vec<(String, String)>,
}

impl RdfGraph {
    /// Add a prefix declaration.
    pub fn add_prefix(&mut self, prefix: impl Into<String>, iri: impl Into<String>) {
        self.prefixes.push((prefix.into(), iri.into()));
    }

    /// Add a triple.
    pub fn add_triple(&mut self, triple: RdfTriple) {
        self.triples.push(triple);
    }

    /// Number of triples.
    pub fn triple_count(&self) -> usize {
        self.triples.len()
    }
}

/// Render a triple in Turtle notation.
pub fn render_triple_turtle(t: &RdfTriple) -> String {
    let obj = if t.object_is_literal {
        format!("\"{}\"", t.object)
    } else {
        format!("<{}>", t.object)
    };
    format!("<{}> <{}> {} .\n", t.subject, t.predicate, obj)
}

/// Render the full RDF graph as Turtle.
pub fn render_turtle(graph: &RdfGraph) -> String {
    let mut out = String::new();
    for (prefix, iri) in &graph.prefixes {
        out.push_str(&format!("@prefix {prefix}: <{iri}> .\n"));
    }
    if !graph.prefixes.is_empty() {
        out.push('\n');
    }
    for triple in &graph.triples {
        out.push_str(&render_triple_turtle(triple));
    }
    out
}

/// Validate that all triples have non-empty subject, predicate, object.
pub fn validate_graph(graph: &RdfGraph) -> bool {
    graph
        .triples
        .iter()
        .all(|t| !t.subject.is_empty() && !t.predicate.is_empty() && !t.object.is_empty())
}

/// Count triples with a given predicate.
pub fn count_by_predicate(graph: &RdfGraph, predicate: &str) -> usize {
    graph
        .triples
        .iter()
        .filter(|t| t.predicate == predicate)
        .count()
}

/// Find all subjects linked to a given object IRI.
pub fn subjects_with_object<'a>(graph: &'a RdfGraph, object: &str) -> Vec<&'a str> {
    graph
        .triples
        .iter()
        .filter(|t| t.object == object)
        .map(|t| t.subject.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> RdfGraph {
        let mut g = RdfGraph::default();
        g.add_prefix("schema", "https://schema.org/");
        g.add_triple(RdfTriple::iri(
            "https://example.com/s",
            "https://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://schema.org/Person",
        ));
        g.add_triple(RdfTriple::literal(
            "https://example.com/s",
            "https://schema.org/name",
            "Alice",
        ));
        g
    }

    #[test]
    fn triple_count() {
        assert_eq!(sample_graph().triple_count(), 2);
    }

    #[test]
    fn render_contains_prefix() {
        assert!(render_turtle(&sample_graph()).contains("@prefix"));
    }

    #[test]
    fn render_literal_quoted() {
        let t = RdfTriple::literal("s", "p", "value");
        assert!(render_triple_turtle(&t).contains("\"value\""));
    }

    #[test]
    fn render_iri_angle_brackets() {
        let t = RdfTriple::iri("s", "p", "http://x.org/");
        assert!(render_triple_turtle(&t).contains("<http://x.org/>"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_graph(&sample_graph()));
    }

    #[test]
    fn validate_empty_subject() {
        let mut g = RdfGraph::default();
        g.add_triple(RdfTriple::iri("", "p", "o"));
        assert!(!validate_graph(&g));
    }

    #[test]
    fn count_by_predicate_works() {
        let pred = "https://schema.org/name";
        let g = sample_graph();
        assert_eq!(count_by_predicate(&g, pred), 1);
    }

    #[test]
    fn subjects_with_object_found() {
        let g = sample_graph();
        let subjects = subjects_with_object(&g, "https://schema.org/Person");
        assert_eq!(subjects.len(), 1);
    }

    #[test]
    fn subjects_with_object_empty() {
        let g = sample_graph();
        assert!(subjects_with_object(&g, "nope").is_empty());
    }
}
