// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple word/token splitter with frequency analysis.

use std::collections::HashMap;

pub fn tokenize_words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

pub fn tokenize_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '.' || ch == '!' || ch == '?' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    sentences
}

pub fn token_count(text: &str) -> usize {
    tokenize_words(text).len()
}

pub fn unique_tokens(text: &str) -> Vec<String> {
    let mut tokens = tokenize_words(text);
    tokens.sort();
    tokens.dedup();
    tokens
}

pub fn token_frequency(text: &str) -> Vec<(String, usize)> {
    let tokens = tokenize_words(text);
    let mut freq: HashMap<String, usize> = HashMap::new();
    for t in tokens {
        *freq.entry(t).or_insert(0) += 1;
    }
    let mut result: Vec<(String, usize)> = freq.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_words_basic() {
        /* splits on whitespace and punctuation */
        let words = tokenize_words("hello, world!");
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world".to_string()));
    }

    #[test]
    fn test_tokenize_sentences() {
        /* splits on period, exclamation, question mark */
        let sents = tokenize_sentences("Hello. How are you? Fine!");
        assert_eq!(sents.len(), 3);
    }

    #[test]
    fn test_token_count() {
        /* counts the number of word tokens */
        let n = token_count("the quick brown fox");
        assert_eq!(n, 4);
    }

    #[test]
    fn test_unique_tokens_sorted() {
        /* unique tokens are sorted and deduplicated */
        let uniq = unique_tokens("a b a c b");
        assert_eq!(
            uniq,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_token_frequency_order() {
        /* most frequent tokens appear first */
        let freq = token_frequency("the the the cat cat dog");
        assert_eq!(freq[0].0, "the");
        assert_eq!(freq[0].1, 3);
    }

    #[test]
    fn test_empty_input() {
        /* empty string yields empty results */
        assert_eq!(token_count(""), 0);
        assert!(unique_tokens("").is_empty());
        assert!(token_frequency("").is_empty());
    }

    #[test]
    fn test_tokenize_sentences_no_terminator() {
        /* text without terminator is a single sentence */
        let sents = tokenize_sentences("No terminator here");
        assert_eq!(sents.len(), 1);
    }
}
