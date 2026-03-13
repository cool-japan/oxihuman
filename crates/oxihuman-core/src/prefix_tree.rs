// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A trie (prefix tree) for efficient string prefix lookups and auto-completion.

use std::collections::HashMap;

/// A node in the prefix tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PrefixNode {
    children: HashMap<char, PrefixNode>,
    is_terminal: bool,
    count: usize,
}

impl PrefixNode {
    fn new() -> Self {
        Self { children: HashMap::new(), is_terminal: false, count: 0 }
    }
}

/// A trie for string prefix operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PrefixTree {
    root: PrefixNode,
    size: usize,
}

#[allow(dead_code)]
impl PrefixTree {
    pub fn new() -> Self {
        Self { root: PrefixNode::new(), size: 0 }
    }

    pub fn insert(&mut self, word: &str) -> bool {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node.count += 1;
            node = node.children.entry(ch).or_insert_with(PrefixNode::new);
        }
        node.count += 1;
        if node.is_terminal {
            false
        } else {
            node.is_terminal = true;
            self.size += 1;
            true
        }
    }

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

    /// Returns all words with the given prefix.
    pub fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return Vec::new(),
            }
        }
        let mut results = Vec::new();
        Self::collect(node, &mut prefix.to_string(), &mut results);
        results
    }

    fn collect(node: &PrefixNode, current: &mut String, results: &mut Vec<String>) {
        if node.is_terminal {
            results.push(current.clone());
        }
        let mut chars: Vec<char> = node.children.keys().copied().collect();
        chars.sort();
        for ch in chars {
            current.push(ch);
            Self::collect(&node.children[&ch], current, results);
            current.pop();
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Count of words starting with prefix.
    pub fn count_prefix(&self, prefix: &str) -> usize {
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(n) => node = n,
                None => return 0,
            }
        }
        Self::count_terminals(node)
    }

    fn count_terminals(node: &PrefixNode) -> usize {
        let mut c = if node.is_terminal { 1 } else { 0 };
        for child in node.children.values() {
            c += Self::count_terminals(child);
        }
        c
    }

    /// Longest common prefix among all stored words.
    pub fn longest_common_prefix(&self) -> String {
        let mut result = String::new();
        let mut node = &self.root;
        loop {
            if node.children.len() != 1 || node.is_terminal {
                break;
            }
            let Some((&ch, next)) = node.children.iter().next() else { break };
            result.push(ch);
            node = next;
        }
        result
    }
}

impl Default for PrefixTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_contains() {
        let mut t = PrefixTree::new();
        assert!(t.insert("hello"));
        assert!(t.contains("hello"));
        assert!(!t.contains("hell"));
    }

    #[test]
    fn test_duplicate() {
        let mut t = PrefixTree::new();
        assert!(t.insert("abc"));
        assert!(!t.insert("abc"));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn test_starts_with() {
        let mut t = PrefixTree::new();
        t.insert("apple");
        assert!(t.starts_with("app"));
        assert!(!t.starts_with("ban"));
    }

    #[test]
    fn test_autocomplete() {
        let mut t = PrefixTree::new();
        t.insert("car");
        t.insert("card");
        t.insert("care");
        t.insert("dog");
        let results = t.autocomplete("car");
        assert_eq!(results.len(), 3);
        assert!(results.contains(&"car".to_string()));
        assert!(results.contains(&"card".to_string()));
    }

    #[test]
    fn test_empty_autocomplete() {
        let t = PrefixTree::new();
        assert!(t.autocomplete("xyz").is_empty());
    }

    #[test]
    fn test_count_prefix() {
        let mut t = PrefixTree::new();
        t.insert("abc");
        t.insert("abd");
        t.insert("xyz");
        assert_eq!(t.count_prefix("ab"), 2);
        assert_eq!(t.count_prefix("x"), 1);
    }

    #[test]
    fn test_longest_common_prefix() {
        let mut t = PrefixTree::new();
        t.insert("interleave");
        t.insert("internet");
        t.insert("internal");
        assert_eq!(t.longest_common_prefix(), "inter");
    }

    #[test]
    fn test_len() {
        let mut t = PrefixTree::new();
        t.insert("a");
        t.insert("b");
        t.insert("c");
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn test_empty() {
        let t = PrefixTree::new();
        assert!(t.is_empty());
    }

    #[test]
    fn test_single_char() {
        let mut t = PrefixTree::new();
        t.insert("a");
        assert!(t.contains("a"));
        assert!(!t.contains("b"));
    }
}
