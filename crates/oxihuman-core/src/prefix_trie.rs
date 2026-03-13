// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compressed prefix trie (radix tree) for string keys.

use std::collections::HashMap;

/// A node in the prefix trie.
#[derive(Debug, Default, Clone)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_terminal: bool,
    count: usize, /* words passing through this node */
}

/// A compressed prefix trie.
#[derive(Debug, Default, Clone)]
pub struct PrefixTrie {
    root: TrieNode,
    word_count: usize,
}

impl PrefixTrie {
    /// Create a new empty trie.
    pub fn new() -> Self {
        PrefixTrie { root: TrieNode::default(), word_count: 0 }
    }

    /// Insert a word into the trie.
    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node.count += 1;
            node = node.children.entry(ch).or_default();
        }
        node.count += 1;
        if !node.is_terminal {
            node.is_terminal = true;
            self.word_count += 1;
        }
    }

    /// Returns true if `word` is present in the trie.
    pub fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return false,
            }
        }
        node.is_terminal
    }

    /// Returns true if any word in the trie starts with `prefix`.
    pub fn starts_with(&self, prefix: &str) -> bool {
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return false,
            }
        }
        true
    }

    /// Collect all words that start with `prefix`.
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return vec![],
            }
        }
        let mut result = Vec::new();
        collect_words(node, prefix.to_owned(), &mut result);
        result
    }

    /// Number of unique words stored.
    pub fn len(&self) -> usize {
        self.word_count
    }

    /// True if no words are stored.
    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }
}

fn collect_words(node: &TrieNode, prefix: String, out: &mut Vec<String>) {
    if node.is_terminal {
        out.push(prefix.clone());
    }
    for (ch, child) in &node.children {
        let mut p = prefix.clone();
        p.push(*ch);
        collect_words(child, p, out);
    }
}

/// Create a new prefix trie.
pub fn new_prefix_trie() -> PrefixTrie {
    PrefixTrie::new()
}

/// Insert a word.
pub fn pt_insert(trie: &mut PrefixTrie, word: &str) {
    trie.insert(word);
}

/// Check for exact match.
pub fn pt_contains(trie: &PrefixTrie, word: &str) -> bool {
    trie.contains(word)
}

/// Check for prefix existence.
pub fn pt_starts_with(trie: &PrefixTrie, prefix: &str) -> bool {
    trie.starts_with(prefix)
}

/// Words matching a prefix.
pub fn pt_words_with_prefix(trie: &PrefixTrie, prefix: &str) -> Vec<String> {
    trie.words_with_prefix(prefix)
}

/// Word count.
pub fn pt_len(trie: &PrefixTrie) -> usize {
    trie.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_contains() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "hello");
        assert!(pt_contains(&t, "hello") /* word present */);
    }

    #[test]
    fn test_not_contains() {
        let t = new_prefix_trie();
        assert!(!pt_contains(&t, "world") /* word absent */);
    }

    #[test]
    fn test_starts_with() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "hello");
        assert!(pt_starts_with(&t, "hel") /* prefix present */);
        assert!(!pt_starts_with(&t, "xyz"));
    }

    #[test]
    fn test_words_with_prefix() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "apple");
        pt_insert(&mut t, "app");
        pt_insert(&mut t, "banana");
        let words = pt_words_with_prefix(&t, "app");
        assert!(words.contains(&"app".to_string()) /* app found */);
        assert!(words.contains(&"apple".to_string()));
        assert!(!words.contains(&"banana".to_string()));
    }

    #[test]
    fn test_len() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "a");
        pt_insert(&mut t, "b");
        pt_insert(&mut t, "c");
        assert_eq!(pt_len(&t), 3 /* three words */);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "dup");
        pt_insert(&mut t, "dup");
        assert_eq!(pt_len(&t), 1 /* only one unique */);
    }

    #[test]
    fn test_is_empty() {
        let t = new_prefix_trie();
        assert!(t.is_empty() /* starts empty */);
    }

    #[test]
    fn test_prefix_not_word() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "apple");
        assert!(!pt_contains(&t, "app") /* prefix not a word */);
        assert!(pt_starts_with(&t, "app"));
    }

    #[test]
    fn test_empty_prefix() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "x");
        let all = pt_words_with_prefix(&t, "");
        assert!(all.contains(&"x".to_string()) /* all words returned */);
    }

    #[test]
    fn test_unicode() {
        let mut t = new_prefix_trie();
        pt_insert(&mut t, "café");
        assert!(pt_contains(&t, "café") /* unicode word */);
    }
}
