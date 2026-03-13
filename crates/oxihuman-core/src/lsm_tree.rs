// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Log-Structured Merge tree (MemTable + SSTables stub).

#![allow(dead_code)]

use std::collections::BTreeMap;

/// An entry in the LSM tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LsmEntry {
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub seq: u64,
}

/// In-memory table (sorted by key).
#[allow(dead_code)]
pub struct MemTable {
    data: BTreeMap<String, LsmEntry>,
    size_bytes: usize,
    seq_counter: u64,
}

/// A simulated SSTable (sorted string table).
#[allow(dead_code)]
#[derive(Clone)]
pub struct SsTable {
    pub level: usize,
    pub entries: Vec<LsmEntry>,
    pub min_key: String,
    pub max_key: String,
}

/// LSM tree combining MemTable + multiple SSTable levels.
#[allow(dead_code)]
pub struct LsmTree {
    pub memtable: MemTable,
    pub levels: Vec<Vec<SsTable>>,
    pub flush_threshold: usize,
}

impl MemTable {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            size_bytes: 0,
            seq_counter: 0,
        }
    }

    #[allow(dead_code)]
    pub fn put(&mut self, key: &str, value: Vec<u8>) {
        self.seq_counter += 1;
        let entry_size = key.len() + value.len();
        self.size_bytes += entry_size;
        self.data.insert(
            key.to_string(),
            LsmEntry {
                key: key.to_string(),
                value: Some(value),
                seq: self.seq_counter,
            },
        );
    }

    #[allow(dead_code)]
    pub fn delete(&mut self, key: &str) {
        self.seq_counter += 1;
        self.data.insert(
            key.to_string(),
            LsmEntry {
                key: key.to_string(),
                value: None,
                seq: self.seq_counter,
            },
        );
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&LsmEntry> {
        self.data.get(key)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[allow(dead_code)]
    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    #[allow(dead_code)]
    pub fn flush_to_sstable(&mut self, level: usize) -> Option<SsTable> {
        if self.data.is_empty() {
            return None;
        }
        let entries: Vec<LsmEntry> = self.data.values().cloned().collect();
        let min_key = entries.first()?.key.clone();
        let max_key = entries.last()?.key.clone();
        self.data.clear();
        self.size_bytes = 0;
        Some(SsTable {
            level,
            entries,
            min_key,
            max_key,
        })
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SsTable {
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&LsmEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    #[allow(dead_code)]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    #[allow(dead_code)]
    pub fn overlaps(&self, other: &SsTable) -> bool {
        self.min_key <= other.max_key && other.min_key <= self.max_key
    }
}

impl LsmTree {
    #[allow(dead_code)]
    pub fn new(flush_threshold: usize) -> Self {
        Self {
            memtable: MemTable::new(),
            levels: (0..4).map(|_| Vec::new()).collect(),
            flush_threshold,
        }
    }

    #[allow(dead_code)]
    pub fn put(&mut self, key: &str, value: Vec<u8>) {
        self.memtable.put(key, value);
        if self.memtable.size_bytes() >= self.flush_threshold {
            self.flush();
        }
    }

    #[allow(dead_code)]
    pub fn delete(&mut self, key: &str) {
        self.memtable.delete(key);
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.memtable.get(key) {
            return entry.value.clone();
        }
        for level in &self.levels {
            for sst in level.iter().rev() {
                if let Some(entry) = sst.get(key) {
                    return entry.value.clone();
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn flush(&mut self) {
        if let Some(sst) = self.memtable.flush_to_sstable(0) {
            if let Some(level0) = self.levels.first_mut() {
                level0.push(sst);
            }
        }
    }

    #[allow(dead_code)]
    pub fn level_count(&self, level: usize) -> usize {
        self.levels.get(level).map_or(0, |l| l.len())
    }

    #[allow(dead_code)]
    pub fn compact_level(&mut self, level: usize) {
        if level + 1 >= self.levels.len() {
            return;
        }
        let moved: Vec<SsTable> = self.levels[level].drain(..).collect();
        self.levels[level + 1].extend(moved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memtable_put_get() {
        let mut mt = MemTable::new();
        mt.put("foo", b"bar".to_vec());
        let e = mt.get("foo").expect("should succeed");
        assert_eq!(e.value.as_deref(), Some(b"bar".as_ref()));
    }

    #[test]
    fn test_memtable_delete_tombstone() {
        let mut mt = MemTable::new();
        mt.put("x", b"hello".to_vec());
        mt.delete("x");
        let e = mt.get("x").expect("should succeed");
        assert!(e.value.is_none());
    }

    #[test]
    fn test_memtable_len() {
        let mut mt = MemTable::new();
        assert!(mt.is_empty());
        mt.put("a", vec![1]);
        mt.put("b", vec![2]);
        assert_eq!(mt.len(), 2);
    }

    #[test]
    fn test_memtable_flush() {
        let mut mt = MemTable::new();
        mt.put("alpha", b"v1".to_vec());
        mt.put("beta", b"v2".to_vec());
        let sst = mt.flush_to_sstable(0).expect("should succeed");
        assert_eq!(sst.level, 0);
        assert_eq!(sst.entry_count(), 2);
        assert!(mt.is_empty());
    }

    #[test]
    fn test_sstable_get() {
        let sst = SsTable {
            level: 0,
            entries: vec![LsmEntry { key: "k".into(), value: Some(vec![9]), seq: 1 }],
            min_key: "k".into(),
            max_key: "k".into(),
        };
        assert!(sst.get("k").is_some());
        assert!(sst.get("missing").is_none());
    }

    #[test]
    fn test_sstable_overlaps() {
        let a = SsTable { level: 0, entries: vec![], min_key: "a".into(), max_key: "m".into() };
        let b = SsTable { level: 0, entries: vec![], min_key: "g".into(), max_key: "z".into() };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_lsmtree_put_get() {
        let mut tree = LsmTree::new(1024 * 1024);
        tree.put("hello", b"world".to_vec());
        assert_eq!(tree.get("hello"), Some(b"world".to_vec()));
    }

    #[test]
    fn test_lsmtree_flush_and_get() {
        let mut tree = LsmTree::new(1);
        tree.put("key1", b"val1".to_vec());
        tree.flush();
        assert_eq!(tree.level_count(0), 1);
        assert_eq!(tree.get("key1"), Some(b"val1".to_vec()));
    }

    #[test]
    fn test_lsmtree_compact() {
        let mut tree = LsmTree::new(1);
        tree.put("z", b"zv".to_vec());
        tree.flush();
        tree.compact_level(0);
        assert_eq!(tree.level_count(0), 0);
        assert_eq!(tree.level_count(1), 1);
    }

    #[test]
    fn test_lsmtree_missing_key() {
        let tree = LsmTree::new(1024);
        assert!(tree.get("nope").is_none());
    }
}
