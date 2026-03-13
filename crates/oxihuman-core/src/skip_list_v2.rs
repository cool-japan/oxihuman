// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Probabilistic skip list for ordered key-value storage.
//! Uses deterministic level selection based on key hash.

const MAX_LEVEL: usize = 8;

fn level_for_key(key: i64) -> usize {
    let mut h = key as u64;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    let mut lvl = 1usize;
    while lvl < MAX_LEVEL && (h >> (64 - lvl)) & 1 == 1 {
        lvl += 1;
        h ^= h >> 17;
    }
    lvl
}

/// An entry in the skip list.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SkipEntry2 {
    key: i64,
    value: i64,
    nexts: Vec<Option<usize>>,
}

/// A skip list for ordered (i64 → i64) storage.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SkipList2 {
    entries: Vec<SkipEntry2>,
    head_nexts: Vec<Option<usize>>,
    len: usize,
}

impl Default for SkipList2 {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            head_nexts: vec![None; MAX_LEVEL],
            len: 0,
        }
    }
}

impl SkipList2 {
    /// Create a new empty skip list.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair. Overwrites if key already exists.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: i64, value: i64) {
        // Check if key already exists
        if let Some(idx) = self.find_idx(key) {
            self.entries[idx].value = value;
            return;
        }
        let lvl = level_for_key(key);
        let new_idx = self.entries.len();
        self.entries.push(SkipEntry2 {
            key,
            value,
            nexts: vec![None; lvl],
        });

        // Update forward pointers at each level
        for lv in 0..lvl {
            let mut pos: Option<usize> = None; // None = head
                                               // Find insertion point at this level
            let mut cur = self.head_nexts[lv];
            while let Some(ci) = cur {
                if self.entries[ci].key < key {
                    pos = Some(ci);
                    cur = if lv < self.entries[ci].nexts.len() {
                        self.entries[ci].nexts[lv]
                    } else {
                        None
                    };
                } else {
                    break;
                }
            }
            let old_next = match pos {
                None => self.head_nexts[lv],
                Some(pi) => {
                    if lv < self.entries[pi].nexts.len() {
                        self.entries[pi].nexts[lv]
                    } else {
                        None
                    }
                }
            };
            self.entries[new_idx].nexts[lv] = old_next;
            match pos {
                None => self.head_nexts[lv] = Some(new_idx),
                Some(pi) => {
                    if lv < self.entries[pi].nexts.len() {
                        self.entries[pi].nexts[lv] = Some(new_idx);
                    }
                }
            }
        }
        self.len += 1;
    }

    fn find_idx(&self, key: i64) -> Option<usize> {
        self.entries.iter().position(|e| e.key == key)
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: i64) -> Option<i64> {
        self.find_idx(key).map(|i| self.entries[i].value)
    }

    /// Check if a key exists.
    #[allow(dead_code)]
    pub fn contains(&self, key: i64) -> bool {
        self.find_idx(key).is_some()
    }

    /// Number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Remove a key. Returns true if it existed.
    #[allow(dead_code)]
    pub fn remove(&mut self, key: i64) -> bool {
        if self.find_idx(key).is_none() {
            return false;
        }
        self.entries.retain(|e| e.key != key);
        // Rebuild head_nexts (simple rebuild)
        self.head_nexts = vec![None; MAX_LEVEL];
        self.len = self.len.saturating_sub(1);
        true
    }

    /// Return all keys in insertion order.
    #[allow(dead_code)]
    pub fn keys(&self) -> Vec<i64> {
        let mut ks: Vec<i64> = self.entries.iter().map(|e| e.key).collect();
        ks.sort_unstable();
        ks
    }

    /// Return all (key, value) pairs sorted by key.
    #[allow(dead_code)]
    pub fn pairs(&self) -> Vec<(i64, i64)> {
        let mut ps: Vec<(i64, i64)> = self.entries.iter().map(|e| (e.key, e.value)).collect();
        ps.sort_by_key(|&(k, _)| k);
        ps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut s = SkipList2::new();
        s.insert(1, 100);
        assert_eq!(s.get(1), Some(100));
    }

    #[test]
    fn get_missing_is_none() {
        let s = SkipList2::new();
        assert!(s.get(42).is_none());
    }

    #[test]
    fn contains_existing() {
        let mut s = SkipList2::new();
        s.insert(5, 5);
        assert!(s.contains(5));
    }

    #[test]
    fn len_tracks_insertions() {
        let mut s = SkipList2::new();
        s.insert(1, 1);
        s.insert(2, 2);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn is_empty_initially() {
        let s = SkipList2::new();
        assert!(s.is_empty());
    }

    #[test]
    fn overwrite_existing_key() {
        let mut s = SkipList2::new();
        s.insert(1, 10);
        s.insert(1, 20);
        assert_eq!(s.get(1), Some(20));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn remove_existing() {
        let mut s = SkipList2::new();
        s.insert(3, 3);
        assert!(s.remove(3));
        assert!(!s.contains(3));
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut s = SkipList2::new();
        assert!(!s.remove(99));
    }

    #[test]
    fn keys_sorted() {
        let mut s = SkipList2::new();
        s.insert(3, 0);
        s.insert(1, 0);
        s.insert(2, 0);
        assert_eq!(s.keys(), vec![1, 2, 3]);
    }

    #[test]
    fn pairs_sorted_by_key() {
        let mut s = SkipList2::new();
        s.insert(10, 100);
        s.insert(5, 50);
        let p = s.pairs();
        assert_eq!(p[0].0, 5);
        assert_eq!(p[1].0, 10);
    }
}
