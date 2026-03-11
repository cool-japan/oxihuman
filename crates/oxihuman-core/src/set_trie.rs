// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A trie over sorted string-sets supporting subset / superset queries.

use std::collections::HashMap;

/// A node in the set-trie.
#[allow(dead_code)]
#[derive(Debug, Default)]
struct TrieNode {
    children: HashMap<String, TrieNode>,
    is_terminal: bool,
    stored_payload: Option<String>,
}

/// Set-Trie: inserts sets of strings and answers subset/superset queries.
#[allow(dead_code)]
pub struct SetTrie {
    root: TrieNode,
    set_count: usize,
}

#[allow(dead_code)]
impl SetTrie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
            set_count: 0,
        }
    }

    fn sorted(elements: &[&str]) -> Vec<String> {
        let mut v: Vec<String> = elements.iter().map(|s| s.to_string()).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Insert a set with an optional payload string.
    pub fn insert(&mut self, elements: &[&str], payload: &str) {
        let sorted = Self::sorted(elements);
        let mut node = &mut self.root;
        for elem in &sorted {
            node = node.children.entry(elem.clone()).or_default();
        }
        if !node.is_terminal {
            self.set_count += 1;
        }
        node.is_terminal = true;
        node.stored_payload = Some(payload.to_string());
    }

    /// Check exact membership.
    pub fn contains(&self, elements: &[&str]) -> bool {
        let sorted = Self::sorted(elements);
        let mut node = &self.root;
        for elem in &sorted {
            match node.children.get(elem) {
                Some(n) => node = n,
                None => return false,
            }
        }
        node.is_terminal
    }

    /// Return payload for an exact set, if present.
    pub fn get_payload(&self, elements: &[&str]) -> Option<&str> {
        let sorted = Self::sorted(elements);
        let mut node = &self.root;
        for elem in &sorted {
            node = node.children.get(elem)?;
        }
        if node.is_terminal {
            node.stored_payload.as_deref()
        } else {
            None
        }
    }

    /// Count how many stored sets are subsets of `superset`.
    pub fn count_subsets(&self, superset: &[&str]) -> usize {
        let sorted = Self::sorted(superset);
        Self::count_subsets_rec(&self.root, &sorted, 0)
    }

    fn count_subsets_rec(node: &TrieNode, superset: &[String], start: usize) -> usize {
        let mut count = if node.is_terminal { 1 } else { 0 };
        for (key, child) in &node.children {
            for i in start..superset.len() {
                if &superset[i] == key {
                    count += Self::count_subsets_rec(child, superset, i + 1);
                    break;
                }
            }
        }
        count
    }

    pub fn set_count(&self) -> usize {
        self.set_count
    }

    pub fn is_empty(&self) -> bool {
        self.set_count == 0
    }

    pub fn clear(&mut self) {
        self.root = TrieNode::default();
        self.set_count = 0;
    }
}

impl Default for SetTrie {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_set_trie() -> SetTrie {
    SetTrie::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_contains() {
        let mut t = new_set_trie();
        t.insert(&["a", "b", "c"], "abc");
        assert!(t.contains(&["a", "b", "c"]));
    }

    #[test]
    fn order_independent() {
        let mut t = new_set_trie();
        t.insert(&["c", "a", "b"], "x");
        assert!(t.contains(&["a", "b", "c"]));
    }

    #[test]
    fn not_contains_superset() {
        let mut t = new_set_trie();
        t.insert(&["a", "b"], "x");
        assert!(!t.contains(&["a", "b", "c"]));
    }

    #[test]
    fn not_contains_subset() {
        let mut t = new_set_trie();
        t.insert(&["a", "b", "c"], "x");
        assert!(!t.contains(&["a", "b"]));
    }

    #[test]
    fn payload_retrieved() {
        let mut t = new_set_trie();
        t.insert(&["x", "y"], "payload");
        assert_eq!(t.get_payload(&["x", "y"]), Some("payload"));
    }

    #[test]
    fn count_subsets() {
        let mut t = new_set_trie();
        t.insert(&["a"], "1");
        t.insert(&["a", "b"], "2");
        t.insert(&["a", "b", "c"], "3");
        assert_eq!(t.count_subsets(&["a", "b", "c"]), 3);
    }

    #[test]
    fn set_count() {
        let mut t = new_set_trie();
        t.insert(&["a"], "");
        t.insert(&["b"], "");
        assert_eq!(t.set_count(), 2);
    }

    #[test]
    fn is_empty_initial() {
        let t = new_set_trie();
        assert!(t.is_empty());
    }

    #[test]
    fn clear_resets() {
        let mut t = new_set_trie();
        t.insert(&["a"], "");
        t.clear();
        assert!(t.is_empty());
        assert!(!t.contains(&["a"]));
    }

    #[test]
    fn duplicate_elements_deduped() {
        let mut t = new_set_trie();
        t.insert(&["a", "a", "b"], "x");
        assert!(t.contains(&["a", "b"]));
    }
}
