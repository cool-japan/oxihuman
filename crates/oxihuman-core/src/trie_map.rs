// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! String trie map: maps string keys to values via a character trie.

use std::collections::HashMap;

/// An internal trie node.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
struct TrieNode<V> {
    children: HashMap<char, TrieNode<V>>,
    value: Option<V>,
}

/// A trie map from `String` keys to values `V`.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TrieMap<V> {
    root: TrieNode<V>,
    count: usize,
}

/// Create a new empty `TrieMap`.
#[allow(dead_code)]
pub fn new_trie_map<V>() -> TrieMap<V> {
    TrieMap {
        root: TrieNode {
            children: std::collections::HashMap::new(),
            value: None,
        },
        count: 0,
    }
}

/// Insert a key-value pair. Returns old value if key existed.
#[allow(dead_code)]
pub fn trm_insert<V: Clone>(tm: &mut TrieMap<V>, key: &str, val: V) -> Option<V> {
    let mut node = &mut tm.root;
    for ch in key.chars() {
        node = node.children.entry(ch).or_insert_with(|| TrieNode {
            children: std::collections::HashMap::new(),
            value: None,
        });
    }
    let old = node.value.take();
    node.value = Some(val);
    if old.is_none() {
        tm.count += 1;
    }
    old
}

/// Get a reference to the value for `key`.
#[allow(dead_code)]
pub fn trm_get<'a, V>(tm: &'a TrieMap<V>, key: &str) -> Option<&'a V> {
    let mut node = &tm.root;
    for ch in key.chars() {
        node = node.children.get(&ch)?;
    }
    node.value.as_ref()
}

/// Check if key exists.
#[allow(dead_code)]
pub fn trm_contains<V>(tm: &TrieMap<V>, key: &str) -> bool {
    trm_get(tm, key).is_some()
}

/// Remove a key and return its value.
#[allow(dead_code)]
pub fn trm_remove<V: Clone>(tm: &mut TrieMap<V>, key: &str) -> Option<V> {
    fn remove_rec<V: Clone>(node: &mut TrieNode<V>, chars: &[char]) -> Option<V> {
        if chars.is_empty() {
            return node.value.take();
        }
        let ch = chars[0];
        let result = node
            .children
            .get_mut(&ch)
            .and_then(|child| remove_rec(child, &chars[1..]));
        result
    }
    let chars: Vec<char> = key.chars().collect();
    let result = remove_rec(&mut tm.root, &chars);
    if result.is_some() {
        tm.count -= 1;
    }
    result
}

/// Number of entries in the trie.
#[allow(dead_code)]
pub fn trm_len<V>(tm: &TrieMap<V>) -> usize {
    tm.count
}

/// Whether the trie is empty.
#[allow(dead_code)]
pub fn trm_is_empty<V>(tm: &TrieMap<V>) -> bool {
    tm.count == 0
}

/// Collect all keys with a given prefix.
#[allow(dead_code)]
pub fn trm_keys_with_prefix<V>(tm: &TrieMap<V>, prefix: &str) -> Vec<String> {
    let mut node = &tm.root;
    for ch in prefix.chars() {
        match node.children.get(&ch) {
            Some(n) => node = n,
            None => return Vec::new(),
        }
    }
    let mut results = Vec::new();
    collect_keys(node, &mut prefix.to_string(), &mut results);
    results
}

fn collect_keys<V>(node: &TrieNode<V>, prefix: &mut String, results: &mut Vec<String>) {
    if node.value.is_some() {
        results.push(prefix.clone());
    }
    let mut sorted_chars: Vec<char> = node.children.keys().copied().collect();
    sorted_chars.sort_unstable();
    for ch in sorted_chars {
        prefix.push(ch);
        collect_keys(&node.children[&ch], prefix, results);
        prefix.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut tm = new_trie_map::<i32>();
        trm_insert(&mut tm, "hello", 42);
        assert_eq!(trm_get(&tm, "hello"), Some(&42));
    }

    #[test]
    fn test_missing_key() {
        let tm = new_trie_map::<i32>();
        assert!(trm_get(&tm, "missing").is_none());
    }

    #[test]
    fn test_contains() {
        let mut tm = new_trie_map::<i32>();
        trm_insert(&mut tm, "abc", 1);
        assert!(trm_contains(&tm, "abc"));
        assert!(!trm_contains(&tm, "ab"));
    }

    #[test]
    fn test_remove() {
        let mut tm = new_trie_map::<i32>();
        trm_insert(&mut tm, "key", 99);
        let old = trm_remove(&mut tm, "key");
        assert_eq!(old, Some(99));
        assert!(!trm_contains(&tm, "key"));
    }

    #[test]
    fn test_len() {
        let mut tm = new_trie_map::<i32>();
        assert_eq!(trm_len(&tm), 0);
        trm_insert(&mut tm, "a", 1);
        trm_insert(&mut tm, "b", 2);
        assert_eq!(trm_len(&tm), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut tm = new_trie_map::<i32>();
        assert!(trm_is_empty(&tm));
        trm_insert(&mut tm, "x", 0);
        assert!(!trm_is_empty(&tm));
    }

    #[test]
    fn test_keys_with_prefix() {
        let mut tm = new_trie_map::<i32>();
        trm_insert(&mut tm, "apple", 1);
        trm_insert(&mut tm, "application", 2);
        trm_insert(&mut tm, "banana", 3);
        let keys = trm_keys_with_prefix(&tm, "app");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"apple".to_string()));
    }

    #[test]
    fn test_overwrite_returns_old() {
        let mut tm = new_trie_map::<i32>();
        trm_insert(&mut tm, "k", 1);
        let old = trm_insert(&mut tm, "k", 2);
        assert_eq!(old, Some(1));
        assert_eq!(trm_len(&tm), 1);
    }

    #[test]
    fn test_prefix_no_match() {
        let mut tm = new_trie_map::<i32>();
        trm_insert(&mut tm, "hello", 1);
        let keys = trm_keys_with_prefix(&tm, "xyz");
        assert!(keys.is_empty());
    }
}
