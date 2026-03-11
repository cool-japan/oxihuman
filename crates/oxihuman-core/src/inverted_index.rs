// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Inverted index for document retrieval.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InvertedIndex {
    pub index: HashMap<String, Vec<usize>>,
}

#[allow(dead_code)]
pub fn new_inverted_index() -> InvertedIndex {
    InvertedIndex {
        index: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn insert(idx: &mut InvertedIndex, doc_id: usize, tokens: &[&str]) {
    for &token in tokens {
        let entry = idx.index.entry(token.to_string()).or_default();
        if !entry.contains(&doc_id) {
            entry.push(doc_id);
        }
    }
}

#[allow(dead_code)]
pub fn search(idx: &InvertedIndex, token: &str) -> Vec<usize> {
    idx.index.get(token).cloned().unwrap_or_default()
}

#[allow(dead_code)]
pub fn search_all(idx: &InvertedIndex, tokens: &[&str]) -> Vec<usize> {
    if tokens.is_empty() {
        return Vec::new();
    }
    let mut result: Option<std::collections::HashSet<usize>> = None;
    for &token in tokens {
        let docs: std::collections::HashSet<usize> = search(idx, token).into_iter().collect();
        result = Some(match result {
            None => docs,
            Some(existing) => existing.intersection(&docs).copied().collect(),
        });
    }
    let mut out: Vec<usize> = result.unwrap_or_default().into_iter().collect();
    out.sort_unstable();
    out
}

#[allow(dead_code)]
pub fn term_count(idx: &InvertedIndex) -> usize {
    idx.index.len()
}

#[allow(dead_code)]
pub fn doc_freq(idx: &InvertedIndex, term: &str) -> usize {
    idx.index.get(term).map(|v| v.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_search() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["hello", "world"]);
        assert!(search(&idx, "hello").contains(&0));
    }

    #[test]
    fn test_intersection() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["rust", "fast"]);
        insert(&mut idx, 1, &["rust", "safe"]);
        let result = search_all(&idx, &["rust", "fast"]);
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_missing_term() {
        let idx = new_inverted_index();
        assert!(search(&idx, "missing").is_empty());
    }

    #[test]
    fn test_term_count() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["a", "b", "c"]);
        assert_eq!(term_count(&idx), 3);
    }

    #[test]
    fn test_doc_freq() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["rust"]);
        insert(&mut idx, 1, &["rust"]);
        assert_eq!(doc_freq(&idx, "rust"), 2);
    }

    #[test]
    fn test_search_all_empty_tokens() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["a"]);
        let r = search_all(&idx, &[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_no_duplicate_doc_ids() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["a"]);
        insert(&mut idx, 0, &["a"]);
        assert_eq!(doc_freq(&idx, "a"), 1);
    }

    #[test]
    fn test_multiple_docs() {
        let mut idx = new_inverted_index();
        insert(&mut idx, 0, &["x"]);
        insert(&mut idx, 1, &["x"]);
        insert(&mut idx, 2, &["x"]);
        let r = search(&idx, "x");
        assert_eq!(r.len(), 3);
    }
}
