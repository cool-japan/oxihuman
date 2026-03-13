// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SPARQL query stub export.

/// SPARQL query type.
#[derive(Debug, Clone, PartialEq)]
pub enum SparqlQueryType {
    Select,
    Construct,
    Ask,
    Describe,
}

impl SparqlQueryType {
    /// Keyword for this query type.
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Select => "SELECT",
            Self::Construct => "CONSTRUCT",
            Self::Ask => "ASK",
            Self::Describe => "DESCRIBE",
        }
    }
}

/// A SPARQL prefix declaration.
#[derive(Debug, Clone)]
pub struct SparqlPrefix {
    pub prefix: String,
    pub iri: String,
}

/// A SPARQL query builder.
#[derive(Debug, Clone)]
pub struct SparqlQuery {
    pub query_type: SparqlQueryType,
    pub prefixes: Vec<SparqlPrefix>,
    pub variables: Vec<String>,
    pub where_patterns: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl SparqlQuery {
    /// Create a new SELECT query.
    pub fn select(variables: Vec<String>) -> Self {
        Self {
            query_type: SparqlQueryType::Select,
            prefixes: Vec::new(),
            variables,
            where_patterns: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Add a prefix.
    pub fn add_prefix(&mut self, prefix: impl Into<String>, iri: impl Into<String>) {
        self.prefixes.push(SparqlPrefix {
            prefix: prefix.into(),
            iri: iri.into(),
        });
    }

    /// Add a WHERE clause triple pattern.
    pub fn add_pattern(&mut self, pattern: impl Into<String>) {
        self.where_patterns.push(pattern.into());
    }

    /// Number of WHERE patterns.
    pub fn pattern_count(&self) -> usize {
        self.where_patterns.len()
    }
}

/// Render a SPARQL query to a string.
pub fn render_sparql(query: &SparqlQuery) -> String {
    let mut out = String::new();
    for p in &query.prefixes {
        out.push_str(&format!("PREFIX {}: <{}>\n", p.prefix, p.iri));
    }
    let vars: Vec<String> = query.variables.iter().map(|v| format!("?{v}")).collect();
    out.push_str(&format!(
        "\n{} {}\n",
        query.query_type.keyword(),
        vars.join(" ")
    ));
    out.push_str("WHERE {\n");
    for pat in &query.where_patterns {
        out.push_str(&format!("  {pat}\n"));
    }
    out.push('}');
    if let Some(limit) = query.limit {
        out.push_str(&format!("\nLIMIT {limit}"));
    }
    if let Some(offset) = query.offset {
        out.push_str(&format!("\nOFFSET {offset}"));
    }
    out.push('\n');
    out
}

/// Validate that the query has at least one variable (for SELECT) and one pattern.
pub fn validate_query(query: &SparqlQuery) -> bool {
    (query.query_type != SparqlQueryType::Select || !query.variables.is_empty())
        && !query.where_patterns.is_empty()
}

/// Add a standard Schema.org prefix.
pub fn add_schema_prefix(query: &mut SparqlQuery) {
    query.add_prefix("schema", "https://schema.org/");
}

/// Add an RDF prefix.
pub fn add_rdf_prefix(query: &mut SparqlQuery) {
    query.add_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_query() -> SparqlQuery {
        let mut q = SparqlQuery::select(vec!["s".into(), "p".into(), "o".into()]);
        q.add_prefix("schema", "https://schema.org/");
        q.add_pattern("?s ?p ?o .");
        q.limit = Some(10);
        q
    }

    #[test]
    fn pattern_count() {
        assert_eq!(sample_query().pattern_count(), 1);
    }

    #[test]
    fn render_contains_select() {
        assert!(render_sparql(&sample_query()).contains("SELECT"));
    }

    #[test]
    fn render_contains_where() {
        assert!(render_sparql(&sample_query()).contains("WHERE"));
    }

    #[test]
    fn render_contains_limit() {
        assert!(render_sparql(&sample_query()).contains("LIMIT 10"));
    }

    #[test]
    fn render_contains_prefix() {
        assert!(render_sparql(&sample_query()).contains("PREFIX"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_query(&sample_query()));
    }

    #[test]
    fn validate_no_pattern() {
        let q = SparqlQuery::select(vec!["x".into()]);
        assert!(!validate_query(&q));
    }

    #[test]
    fn query_type_keyword() {
        assert_eq!(SparqlQueryType::Ask.keyword(), "ASK");
    }

    #[test]
    fn add_schema_prefix_adds_one() {
        let mut q = SparqlQuery::select(vec![]);
        add_schema_prefix(&mut q);
        assert_eq!(q.prefixes.len(), 1);
    }

    #[test]
    fn add_rdf_prefix_works() {
        let mut q = SparqlQuery::select(vec![]);
        add_rdf_prefix(&mut q);
        assert!(q.prefixes[0].prefix == "rdf");
    }
}
