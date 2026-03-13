// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Open-addressing hash map with linear probing.

const DEFAULT_CAPACITY: usize = 16;
const LOAD_FACTOR_MAX: f32 = 0.7;

#[derive(Debug, Clone)]
enum Slot<K, V> {
    Empty,
    Deleted,
    Occupied(K, V),
}

fn hash_key(key: i64, cap: usize) -> usize {
    let mut h = key as u64;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h as usize % cap
}

/// An open-addressing hash map with linear probing (i64 → i64).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OpenHashMap {
    slots: Vec<Slot<i64, i64>>,
    count: usize,
    deleted: usize,
    capacity: usize,
}

impl Default for OpenHashMap {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl OpenHashMap {
    /// Create a new hash map with default capacity.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a specific initial capacity.
    #[allow(dead_code)]
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(8);
        Self {
            slots: (0..cap).map(|_| Slot::Empty).collect(),
            count: 0,
            deleted: 0,
            capacity: cap,
        }
    }

    fn probe(&self, key: i64) -> Option<usize> {
        let start = hash_key(key, self.capacity);
        for i in 0..self.capacity {
            let idx = (start + i) % self.capacity;
            match &self.slots[idx] {
                Slot::Occupied(k, _) if *k == key => return Some(idx),
                Slot::Empty => return None,
                _ => {}
            }
        }
        None
    }

    fn probe_insert(&self, key: i64) -> usize {
        let start = hash_key(key, self.capacity);
        let mut first_deleted: Option<usize> = None;
        for i in 0..self.capacity {
            let idx = (start + i) % self.capacity;
            match &self.slots[idx] {
                Slot::Occupied(k, _) if *k == key => return idx,
                Slot::Empty => return first_deleted.unwrap_or(idx),
                Slot::Deleted => {
                    if first_deleted.is_none() {
                        first_deleted = Some(idx);
                    }
                }
                _ => {}
            }
        }
        first_deleted.unwrap_or(0)
    }

    fn maybe_resize(&mut self) {
        let load = (self.count + self.deleted) as f32 / self.capacity as f32;
        if load > LOAD_FACTOR_MAX {
            let new_cap = self.capacity * 2;
            let old_slots =
                std::mem::replace(&mut self.slots, (0..new_cap).map(|_| Slot::Empty).collect());
            self.capacity = new_cap;
            self.count = 0;
            self.deleted = 0;
            for slot in old_slots {
                if let Slot::Occupied(k, v) = slot {
                    self.insert(k, v);
                }
            }
        }
    }

    /// Insert or overwrite a key-value pair.
    #[allow(dead_code)]
    pub fn insert(&mut self, key: i64, value: i64) {
        self.maybe_resize();
        let idx = self.probe_insert(key);
        match &self.slots[idx] {
            Slot::Occupied(k, _) if *k == key => {
                self.slots[idx] = Slot::Occupied(key, value);
            }
            Slot::Deleted => {
                self.slots[idx] = Slot::Occupied(key, value);
                self.deleted = self.deleted.saturating_sub(1);
                self.count += 1;
            }
            _ => {
                self.slots[idx] = Slot::Occupied(key, value);
                self.count += 1;
            }
        }
    }

    /// Look up a key.
    #[allow(dead_code)]
    pub fn get(&self, key: i64) -> Option<i64> {
        let idx = self.probe(key)?;
        match &self.slots[idx] {
            Slot::Occupied(_, v) => Some(*v),
            _ => None,
        }
    }

    /// Check if a key exists.
    #[allow(dead_code)]
    pub fn contains(&self, key: i64) -> bool {
        self.probe(key).is_some()
    }

    /// Remove a key. Returns true if it existed.
    #[allow(dead_code)]
    pub fn remove(&mut self, key: i64) -> bool {
        if let Some(idx) = self.probe(key) {
            self.slots[idx] = Slot::Deleted;
            self.count -= 1;
            self.deleted += 1;
            true
        } else {
            false
        }
    }

    /// Number of live entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Load factor (live + deleted / capacity).
    #[allow(dead_code)]
    pub fn load_factor(&self) -> f32 {
        self.count as f32 / self.capacity as f32
    }

    /// Collect all (key, value) pairs.
    #[allow(dead_code)]
    pub fn pairs(&self) -> Vec<(i64, i64)> {
        self.slots
            .iter()
            .filter_map(|s| {
                if let Slot::Occupied(k, v) = s {
                    Some((*k, *v))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut m = OpenHashMap::new();
        m.insert(42, 420);
        assert_eq!(m.get(42), Some(420));
    }

    #[test]
    fn get_missing_is_none() {
        let m = OpenHashMap::new();
        assert!(m.get(1).is_none());
    }

    #[test]
    fn contains_after_insert() {
        let mut m = OpenHashMap::new();
        m.insert(7, 70);
        assert!(m.contains(7));
    }

    #[test]
    fn overwrite_existing() {
        let mut m = OpenHashMap::new();
        m.insert(1, 10);
        m.insert(1, 20);
        assert_eq!(m.get(1), Some(20));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn remove_existing() {
        let mut m = OpenHashMap::new();
        m.insert(3, 3);
        assert!(m.remove(3));
        assert!(!m.contains(3));
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut m = OpenHashMap::new();
        assert!(!m.remove(99));
    }

    #[test]
    fn len_tracks_insertions() {
        let mut m = OpenHashMap::new();
        m.insert(1, 1);
        m.insert(2, 2);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn is_empty_initially() {
        let m = OpenHashMap::new();
        assert!(m.is_empty());
    }

    #[test]
    fn many_inserts_all_findable() {
        let mut m = OpenHashMap::new();
        for i in 0..50i64 {
            m.insert(i, i * 2);
        }
        for i in 0..50i64 {
            assert_eq!(m.get(i), Some(i * 2));
        }
    }

    #[test]
    fn pairs_count_matches_len() {
        let mut m = OpenHashMap::new();
        m.insert(10, 10);
        m.insert(20, 20);
        assert_eq!(m.pairs().len(), m.len());
    }
}
