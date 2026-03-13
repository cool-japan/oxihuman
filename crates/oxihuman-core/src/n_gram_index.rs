// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! N-gram index for approximate string search.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NGramIndex {
    pub n: usize,
    pub index: HashMap<String, Vec<usize>>,
}

#[allow(dead_code)]
pub fn new_ngram_index(n: usize) -> NGramIndex {
    NGramIndex {
        n: n.max(1),
        index: HashMap::new(),
    }
}

fn extract_ngrams(text: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < n {
        if chars.is_empty() {
            return Vec::new();
        }
        return vec![chars.iter().collect()];
    }
    chars.windows(n).map(|w| w.iter().collect()).collect()
}

#[allow(dead_code)]
pub fn insert(idx: &mut NGramIndex, doc_id: usize, text: &str) {
    for gram in extract_ngrams(text, idx.n) {
        idx.index.entry(gram).or_default().push(doc_id);
    }
}

#[allow(dead_code)]
pub fn query(idx: &NGramIndex, text: &str) -> Vec<usize> {
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for gram in extract_ngrams(text, idx.n) {
        if let Some(docs) = idx.index.get(&gram) {
            for &doc_id in docs {
                *counts.entry(doc_id).or_insert(0) += 1;
            }
        }
    }
    let mut result: Vec<usize> = counts.keys().copied().collect();
    result.sort_unstable();
    result
}

#[allow(dead_code)]
pub fn ngram_count(idx: &NGramIndex) -> usize {
    idx.index.len()
}

#[allow(dead_code)]
pub fn doc_count(idx: &NGramIndex) -> usize {
    let mut docs: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for ids in idx.index.values() {
        for &id in ids {
            docs.insert(id);
        }
    }
    docs.len()
}

#[allow(dead_code)]
pub fn clear(idx: &mut NGramIndex) {
    idx.index.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "hello");
        let results = query(&idx, "he");
        assert!(results.contains(&0));
    }

    #[test]
    fn test_empty_query() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "hello");
        let results = query(&idx, "");
        assert!(results.is_empty());
    }

    #[test]
    fn test_multiple_docs() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "hello");
        insert(&mut idx, 1, "world");
        let r0 = query(&idx, "he");
        let r1 = query(&idx, "wo");
        assert!(r0.contains(&0));
        assert!(r1.contains(&1));
    }

    #[test]
    fn test_ngram_count() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "abc");
        assert!(ngram_count(&idx) > 0);
    }

    #[test]
    fn test_doc_count() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "hello");
        insert(&mut idx, 1, "world");
        assert_eq!(doc_count(&idx), 2);
    }

    #[test]
    fn test_clear() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "hello");
        clear(&mut idx);
        assert_eq!(ngram_count(&idx), 0);
    }

    #[test]
    fn test_unigram() {
        let mut idx = new_ngram_index(1);
        insert(&mut idx, 0, "abc");
        let r = query(&idx, "a");
        assert!(r.contains(&0));
    }

    #[test]
    fn test_no_match() {
        let mut idx = new_ngram_index(2);
        insert(&mut idx, 0, "hello");
        let r = query(&idx, "zz");
        assert!(r.is_empty());
    }
}
