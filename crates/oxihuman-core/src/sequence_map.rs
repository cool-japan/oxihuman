// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! An ordered map that preserves insertion order and supports sequence access.

/// Entry in the sequence map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SeqEntry<V> {
    pub key: String,
    pub value: V,
    pub seq: u64,
}

/// Map that preserves insertion order and provides sequence-numbered access.
#[allow(dead_code)]
pub struct SequenceMap<V> {
    entries: Vec<SeqEntry<V>>,
    next_seq: u64,
}

#[allow(dead_code)]
impl<V: Clone> SequenceMap<V> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_seq: 0,
        }
    }

    pub fn insert(&mut self, key: &str, value: V) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        if let Some(e) = self.entries.iter_mut().find(|e| e.key == key) {
            e.value = value;
            e.seq = seq;
        } else {
            self.entries.push(SeqEntry {
                key: key.to_string(),
                value,
                seq,
            });
        }
        seq
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.key != key);
        self.entries.len() < before
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.entries.iter().find(|e| e.key == key).map(|e| &e.value)
    }

    pub fn get_by_index(&self, index: usize) -> Option<&SeqEntry<V>> {
        self.entries.get(index)
    }

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.entries.iter().position(|e| e.key == key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|e| e.key == key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.key.as_str()).collect()
    }

    pub fn values(&self) -> Vec<&V> {
        self.entries.iter().map(|e| &e.value).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    pub fn seq_of(&self, key: &str) -> Option<u64> {
        self.entries.iter().find(|e| e.key == key).map(|e| e.seq)
    }
}

impl<V: Clone> Default for SequenceMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_sequence_map<V: Clone>() -> SequenceMap<V> {
    SequenceMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("a", 1);
        assert_eq!(m.get("a"), Some(&1));
    }

    #[test]
    fn preserves_order() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("first", 1);
        m.insert("second", 2);
        assert_eq!(m.keys(), vec!["first", "second"]);
    }

    #[test]
    fn update_in_place() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("k", 1);
        m.insert("k", 99);
        assert_eq!(m.get("k"), Some(&99));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn remove_entry() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("x", 5);
        assert!(m.remove("x"));
        assert!(!m.contains("x"));
    }

    #[test]
    fn index_of() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("a", 1);
        m.insert("b", 2);
        assert_eq!(m.index_of("b"), Some(1));
    }

    #[test]
    fn get_by_index() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("z", 7);
        let e = m.get_by_index(0).unwrap();
        assert_eq!(e.key, "z");
    }

    #[test]
    fn seq_increments() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        let s1 = m.insert("a", 1);
        let s2 = m.insert("b", 2);
        assert!(s2 > s1);
    }

    #[test]
    fn is_empty() {
        let m: SequenceMap<i32> = new_sequence_map();
        assert!(m.is_empty());
    }

    #[test]
    fn clear() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        m.insert("a", 1);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn seq_of_known_key() {
        let mut m: SequenceMap<i32> = new_sequence_map();
        let seq = m.insert("key", 1);
        assert_eq!(m.seq_of("key"), Some(seq));
    }
}
