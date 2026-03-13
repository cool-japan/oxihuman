// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compressed trie (radix tree) for string lookup.

use std::collections::HashMap;

/// A node in the compressed trie.
#[derive(Debug, Clone, Default)]
struct TrieV2Node {
    children: HashMap<String, TrieV2Node>,
    value: Option<usize>,
    is_terminal: bool,
}

/// A compressed trie (radix tree).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct TrieV2 {
    root: TrieV2Node,
    count: usize,
}

impl TrieV2 {
    /// Create a new empty trie.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key with a value.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: &str, value: usize) {
        insert_node(&mut self.root, key, value);
        self.count += 1;
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<usize> {
        get_node(&self.root, key)
    }

    /// Check if a key exists.
    #[allow(dead_code)]
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Number of inserted keys.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Collect all keys with a given prefix.
    #[allow(dead_code)]
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut result = Vec::new();
        collect_prefix(&self.root, prefix, "", &mut result);
        result
    }

    /// Remove a key (returns true if it existed).
    #[allow(dead_code)]
    pub fn remove(&mut self, key: &str) -> bool {
        if remove_node(&mut self.root, key) {
            self.count = self.count.saturating_sub(1);
            true
        } else {
            false
        }
    }
}

fn insert_node(node: &mut TrieV2Node, key: &str, value: usize) {
    if key.is_empty() {
        node.value = Some(value);
        node.is_terminal = true;
        return;
    }
    // Find a matching child prefix
    let matching_key = node
        .children
        .keys()
        .find(|k| {
            let shared = common_prefix(k, key);
            !shared.is_empty()
        })
        .cloned();

    if let Some(k) = matching_key {
        let shared = common_prefix(&k, key);
        if shared == k {
            // Descend
            let rest = &key[shared.len()..];
            let Some(child) = node.children.get_mut(&k) else {
                return;
            };
            insert_node(child, rest, value);
        } else {
            // Split the edge
            let Some(child) = node.children.remove(&k) else {
                return;
            };
            let old_rest = k[shared.len()..].to_string();
            let new_rest = key[shared.len()..].to_string();

            let mut branch = TrieV2Node::default();
            branch.children.insert(old_rest, child);

            if new_rest.is_empty() {
                branch.value = Some(value);
                branch.is_terminal = true;
            } else {
                let new_leaf = TrieV2Node {
                    value: Some(value),
                    is_terminal: true,
                    children: HashMap::new(),
                };
                branch.children.insert(new_rest, new_leaf);
            }
            node.children.insert(shared.to_string(), branch);
        }
    } else {
        let leaf = TrieV2Node {
            value: Some(value),
            is_terminal: true,
            children: HashMap::new(),
        };
        node.children.insert(key.to_string(), leaf);
    }
}

fn common_prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let len = a
        .chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count();
    &a[..len]
}

fn get_node(node: &TrieV2Node, key: &str) -> Option<usize> {
    if key.is_empty() {
        return node.value;
    }
    for (k, child) in &node.children {
        if key.starts_with(k.as_str()) {
            return get_node(child, &key[k.len()..]);
        }
    }
    None
}

fn collect_prefix(node: &TrieV2Node, prefix: &str, current: &str, result: &mut Vec<String>) {
    let full = current.to_string();
    if (prefix.is_empty() || full.starts_with(prefix) || prefix.starts_with(full.as_str()))
        && node.is_terminal
        && full.starts_with(prefix)
    {
        result.push(full.clone());
    }
    for (k, child) in &node.children {
        let next = format!("{}{}", current, k);
        if next.starts_with(prefix) || prefix.starts_with(next.as_str()) {
            collect_prefix_inner(child, prefix, &next, result);
        }
    }
}

fn collect_prefix_inner(node: &TrieV2Node, prefix: &str, current: &str, result: &mut Vec<String>) {
    if node.is_terminal && current.starts_with(prefix) {
        result.push(current.to_string());
    }
    for (k, child) in &node.children {
        let next = format!("{}{}", current, k);
        if next.starts_with(prefix) || prefix.starts_with(next.as_str()) {
            collect_prefix_inner(child, prefix, &next, result);
        }
    }
}

fn remove_node(node: &mut TrieV2Node, key: &str) -> bool {
    if key.is_empty() {
        if node.is_terminal {
            node.is_terminal = false;
            node.value = None;
            return true;
        }
        return false;
    }
    for k in node.children.keys().cloned().collect::<Vec<_>>() {
        if key.starts_with(k.as_str()) {
            let rest = key[k.len()..].to_string();
            if let Some(child) = node.children.get_mut(&k) {
                let found = remove_node(child, &rest);
                return found;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut t = TrieV2::new();
        t.insert("hello", 1);
        assert_eq!(t.get("hello"), Some(1));
    }

    #[test]
    fn get_missing_returns_none() {
        let t = TrieV2::new();
        assert_eq!(t.get("foo"), None);
    }

    #[test]
    fn contains_existing() {
        let mut t = TrieV2::new();
        t.insert("abc", 42);
        assert!(t.contains("abc"));
    }

    #[test]
    fn contains_missing() {
        let t = TrieV2::new();
        assert!(!t.contains("abc"));
    }

    #[test]
    fn len_tracks_inserts() {
        let mut t = TrieV2::new();
        t.insert("a", 1);
        t.insert("b", 2);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn empty_initially() {
        let t = TrieV2::new();
        assert!(t.is_empty());
    }

    #[test]
    fn prefix_lookup() {
        let mut t = TrieV2::new();
        t.insert("apple", 1);
        t.insert("application", 2);
        t.insert("banana", 3);
        let keys = t.keys_with_prefix("app");
        assert!(!keys.is_empty());
    }

    #[test]
    fn remove_existing_key() {
        let mut t = TrieV2::new();
        t.insert("hello", 1);
        assert!(t.remove("hello"));
        assert!(!t.contains("hello"));
    }

    #[test]
    fn remove_missing_key_returns_false() {
        let mut t = TrieV2::new();
        assert!(!t.remove("missing"));
    }

    #[test]
    fn multiple_keys_shared_prefix() {
        let mut t = TrieV2::new();
        t.insert("test", 1);
        t.insert("testing", 2);
        assert_eq!(t.get("test"), Some(1));
        assert_eq!(t.get("testing"), Some(2));
    }
}
