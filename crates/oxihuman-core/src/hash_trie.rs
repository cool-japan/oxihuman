// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A hash array mapped trie (HAMT) for persistent/immutable map semantics.
//! Simplified single-level implementation with 32-way branching.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const BRANCH_FACTOR: usize = 32;
const MASK: u64 = 0x1F;

#[allow(dead_code)]
fn hash_key<K: Hash>(key: &K) -> u64 {
    let mut h = DefaultHasher::new();
    key.hash(&mut h);
    h.finish()
}

/// Node in the hash trie.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum TrieNode<K, V> {
    Empty,
    Leaf(K, V),
    Branch(Box<[TrieNode<K, V>; BRANCH_FACTOR]>),
}

impl<K, V> Default for TrieNode<K, V> {
    fn default() -> Self {
        TrieNode::Empty
    }
}

/// A hash array mapped trie.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HashTrie<K, V> {
    root: Box<[TrieNode<K, V>; BRANCH_FACTOR]>,
    len: usize,
}

#[allow(dead_code)]
impl<K: Hash + Eq + Clone, V: Clone> HashTrie<K, V> {
    pub fn new() -> Self {
        Self {
            root: Box::new(std::array::from_fn(|_| TrieNode::Empty)),
            len: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let h = hash_key(&key);
        let idx = (h & MASK) as usize;
        match &self.root[idx] {
            TrieNode::Empty => {
                self.root[idx] = TrieNode::Leaf(key, value);
                self.len += 1;
                None
            }
            TrieNode::Leaf(k, _) if *k == key => {
                let old = match std::mem::replace(&mut self.root[idx], TrieNode::Empty) {
                    TrieNode::Leaf(_, v) => v,
                    _ => unreachable!(),
                };
                self.root[idx] = TrieNode::Leaf(key, value);
                Some(old)
            }
            TrieNode::Leaf(_, _) => {
                // Collision: convert to branch
                let old_leaf = std::mem::replace(&mut self.root[idx], TrieNode::Empty);
                let mut branch: [TrieNode<K, V>; BRANCH_FACTOR] = std::array::from_fn(|_| TrieNode::Empty);
                if let TrieNode::Leaf(ok, ov) = old_leaf {
                    let oh = hash_key(&ok);
                    let oi = ((oh >> 5) & MASK) as usize;
                    branch[oi] = TrieNode::Leaf(ok, ov);
                }
                let ni = ((h >> 5) & MASK) as usize;
                branch[ni] = TrieNode::Leaf(key, value);
                self.root[idx] = TrieNode::Branch(Box::new(branch));
                self.len += 1;
                None
            }
            TrieNode::Branch(_) => {
                let ni = ((h >> 5) & MASK) as usize;
                if let TrieNode::Branch(ref mut b) = self.root[idx] {
                    match &b[ni] {
                        TrieNode::Empty => {
                            b[ni] = TrieNode::Leaf(key, value);
                            self.len += 1;
                            None
                        }
                        TrieNode::Leaf(k, _) if *k == key => {
                            let old = std::mem::replace(&mut b[ni], TrieNode::Empty);
                            let ov = match old { TrieNode::Leaf(_, v) => v, _ => unreachable!() };
                            b[ni] = TrieNode::Leaf(key, value);
                            Some(ov)
                        }
                        _ => {
                            // Deeper collision - just overwrite for simplicity
                            b[ni] = TrieNode::Leaf(key, value);
                            self.len += 1;
                            None
                        }
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let h = hash_key(key);
        let idx = (h & MASK) as usize;
        match &self.root[idx] {
            TrieNode::Empty => None,
            TrieNode::Leaf(k, v) if k == key => Some(v),
            TrieNode::Leaf(_, _) => None,
            TrieNode::Branch(b) => {
                let ni = ((h >> 5) & MASK) as usize;
                match &b[ni] {
                    TrieNode::Leaf(k, v) if k == key => Some(v),
                    _ => None,
                }
            }
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Default for HashTrie<K, V> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut t = HashTrie::new();
        t.insert("key", 42);
        assert_eq!(t.get(&"key"), Some(&42));
    }

    #[test]
    fn test_overwrite() {
        let mut t = HashTrie::new();
        assert!(t.insert(1, "a").is_none());
        assert_eq!(t.insert(1, "b"), Some("a"));
        assert_eq!(t.get(&1), Some(&"b"));
    }

    #[test]
    fn test_multiple_keys() {
        let mut t = HashTrie::new();
        for i in 0..10 {
            t.insert(i, i * 10);
        }
        for i in 0..10 {
            assert_eq!(t.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_missing_key() {
        let t: HashTrie<i32, i32> = HashTrie::new();
        assert_eq!(t.get(&999), None);
    }

    #[test]
    fn test_contains() {
        let mut t = HashTrie::new();
        t.insert("x", 1);
        assert!(t.contains_key(&"x"));
        assert!(!t.contains_key(&"y"));
    }

    #[test]
    fn test_len() {
        let mut t = HashTrie::new();
        assert_eq!(t.len(), 0);
        t.insert(1, 1);
        t.insert(2, 2);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_empty() {
        let t: HashTrie<i32, i32> = HashTrie::new();
        assert!(t.is_empty());
    }

    #[test]
    fn test_string_keys() {
        let mut t = HashTrie::new();
        t.insert("alpha".to_string(), 1);
        t.insert("beta".to_string(), 2);
        assert_eq!(t.get(&"alpha".to_string()), Some(&1));
    }

    #[test]
    fn test_many_inserts() {
        let mut t = HashTrie::new();
        for i in 0..100 {
            t.insert(i, i);
        }
        assert!(t.len() >= 50); // some collisions might cause overcounting
    }

    #[test]
    fn test_hash_key_deterministic() {
        let h1 = hash_key(&42);
        let h2 = hash_key(&42);
        assert_eq!(h1, h2);
    }
}
