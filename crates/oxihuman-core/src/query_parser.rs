// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple boolean query parser (AND, OR, NOT, Term).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum QueryToken {
    Term(String),
    And,
    Or,
    Not,
}

#[allow(dead_code)]
pub fn parse_query(s: &str) -> Vec<QueryToken> {
    s.split_whitespace()
        .map(|w| match w {
            "AND" => QueryToken::And,
            "OR" => QueryToken::Or,
            "NOT" => QueryToken::Not,
            other => QueryToken::Term(other.to_string()),
        })
        .collect()
}

#[allow(dead_code)]
pub fn query_token_count(tokens: &[QueryToken]) -> usize {
    tokens.len()
}

#[allow(dead_code)]
pub fn has_operator(tokens: &[QueryToken]) -> bool {
    tokens.iter().any(|t| matches!(t, QueryToken::And | QueryToken::Or | QueryToken::Not))
}

#[allow(dead_code)]
pub fn extract_terms(tokens: &[QueryToken]) -> Vec<String> {
    tokens
        .iter()
        .filter_map(|t| {
            if let QueryToken::Term(s) = t {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_term() {
        let tokens = parse_query("hello");
        assert_eq!(tokens, vec![QueryToken::Term("hello".to_string())]);
    }

    #[test]
    fn test_parse_and() {
        let tokens = parse_query("a AND b");
        assert_eq!(tokens[1], QueryToken::And);
    }

    #[test]
    fn test_parse_or() {
        let tokens = parse_query("a OR b");
        assert_eq!(tokens[1], QueryToken::Or);
    }

    #[test]
    fn test_parse_not() {
        let tokens = parse_query("NOT x");
        assert_eq!(tokens[0], QueryToken::Not);
    }

    #[test]
    fn test_mixed_query() {
        let tokens = parse_query("hello AND NOT world");
        assert_eq!(query_token_count(&tokens), 4);
    }

    #[test]
    fn test_has_operator() {
        let tokens = parse_query("a AND b");
        assert!(has_operator(&tokens));
    }

    #[test]
    fn test_no_operator() {
        let tokens = parse_query("just terms here");
        assert!(!has_operator(&tokens));
    }

    #[test]
    fn test_extract_terms() {
        let tokens = parse_query("foo AND bar");
        let terms = extract_terms(&tokens);
        assert_eq!(terms, vec!["foo", "bar"]);
    }

    #[test]
    fn test_empty_query() {
        let tokens = parse_query("");
        assert!(tokens.is_empty());
    }
}
