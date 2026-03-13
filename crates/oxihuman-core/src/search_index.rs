// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Inverted-index based text search over string-keyed documents.

use std::collections::HashMap;

/// A document stored in the index.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SearchDoc {
    pub id: u64,
    pub title: String,
    pub body: String,
}

/// Simple inverted index that maps lowercase tokens to document ids.
#[allow(dead_code)]
pub struct SearchIndex {
    docs: HashMap<u64, SearchDoc>,
    index: HashMap<String, Vec<u64>>,
    next_id: u64,
    total_indexed: usize,
}

#[allow(dead_code)]
impl SearchIndex {
    pub fn new() -> Self {
        Self {
            docs: HashMap::new(),
            index: HashMap::new(),
            next_id: 0,
            total_indexed: 0,
        }
    }

    /// Tokenise text into lowercase words.
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// Add a document; returns its assigned id.
    pub fn insert(&mut self, title: &str, body: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let doc = SearchDoc {
            id,
            title: title.to_string(),
            body: body.to_string(),
        };
        let tokens = Self::tokenize(&format!("{} {}", title, body));
        self.total_indexed += tokens.len();
        for tok in tokens {
            self.index.entry(tok).or_default().push(id);
        }
        self.docs.insert(id, doc);
        id
    }

    /// Remove a document by id.
    pub fn remove(&mut self, id: u64) -> bool {
        if self.docs.remove(&id).is_none() {
            return false;
        }
        for ids in self.index.values_mut() {
            ids.retain(|&i| i != id);
        }
        true
    }

    /// Search for documents containing all query tokens. Returns doc ids.
    pub fn search(&self, query: &str) -> Vec<u64> {
        let tokens = Self::tokenize(query);
        if tokens.is_empty() {
            return Vec::new();
        }
        let mut result: Option<Vec<u64>> = None;
        for tok in &tokens {
            let ids: Vec<u64> = self.index.get(tok).cloned().unwrap_or_default();
            result = Some(match result {
                None => ids,
                Some(prev) => {
                    let mut set: Vec<u64> = prev;
                    set.retain(|id| ids.contains(id));
                    set
                }
            });
        }
        let mut out = result.unwrap_or_default();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Retrieve a document by id.
    pub fn get(&self, id: u64) -> Option<&SearchDoc> {
        self.docs.get(&id)
    }

    /// Number of documents in the index.
    pub fn doc_count(&self) -> usize {
        self.docs.len()
    }

    /// Number of unique tokens across all documents.
    pub fn token_count(&self) -> usize {
        self.index.len()
    }

    /// Total tokens indexed (with duplicates).
    pub fn total_indexed(&self) -> usize {
        self.total_indexed
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    pub fn clear(&mut self) {
        self.docs.clear();
        self.index.clear();
        self.total_indexed = 0;
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_search_index() -> SearchIndex {
    SearchIndex::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_search() {
        let mut idx = new_search_index();
        idx.insert("hello world", "foo bar");
        let hits = idx.search("hello");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn multi_token_and() {
        let mut idx = new_search_index();
        idx.insert("alpha beta", "body");
        idx.insert("alpha only", "stuff");
        let hits = idx.search("alpha beta");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn no_match_returns_empty() {
        let mut idx = new_search_index();
        idx.insert("rust programming", "systems");
        assert!(idx.search("java").is_empty());
    }

    #[test]
    fn case_insensitive() {
        let mut idx = new_search_index();
        idx.insert("Hello World", "body");
        let hits = idx.search("HELLO");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn remove_document() {
        let mut idx = new_search_index();
        let id = idx.insert("test doc", "");
        assert!(idx.remove(id));
        assert!(idx.search("test").is_empty());
    }

    #[test]
    fn doc_count() {
        let mut idx = new_search_index();
        idx.insert("a", "");
        idx.insert("b", "");
        assert_eq!(idx.doc_count(), 2);
    }

    #[test]
    fn get_doc() {
        let mut idx = new_search_index();
        let id = idx.insert("title", "content");
        let doc = idx.get(id).expect("should succeed");
        assert_eq!(doc.title, "title");
    }

    #[test]
    fn empty_query_returns_empty() {
        let mut idx = new_search_index();
        idx.insert("something", "");
        assert!(idx.search("").is_empty());
    }

    #[test]
    fn clear_index() {
        let mut idx = new_search_index();
        idx.insert("doc", "text");
        idx.clear();
        assert!(idx.is_empty());
    }

    #[test]
    fn token_count_nonzero_after_insert() {
        let mut idx = new_search_index();
        idx.insert("unique token here", "");
        assert!(idx.token_count() > 0);
    }
}
