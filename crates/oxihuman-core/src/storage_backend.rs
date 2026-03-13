// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Abstract in-memory storage backend with namespaced key-value buckets.

use std::collections::HashMap;

/// A single named storage bucket.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Bucket {
    pub name: String,
    pub entries: HashMap<String, Vec<u8>>,
    pub write_count: u64,
}

#[allow(dead_code)]
impl Bucket {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: HashMap::new(),
            write_count: 0,
        }
    }

    pub fn put(&mut self, key: &str, data: Vec<u8>) {
        self.entries.insert(key.to_string(), data);
        self.write_count += 1;
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).map(|v| v.as_slice())
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_bytes(&self) -> usize {
        self.entries.values().map(|v| v.len()).sum()
    }
}

/// An in-memory storage backend with multiple named buckets.
#[allow(dead_code)]
pub struct StorageBackend {
    buckets: HashMap<String, Bucket>,
    read_count: u64,
}

#[allow(dead_code)]
impl StorageBackend {
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
            read_count: 0,
        }
    }

    pub fn ensure_bucket(&mut self, name: &str) -> &mut Bucket {
        self.buckets
            .entry(name.to_string())
            .or_insert_with(|| Bucket::new(name))
    }

    pub fn put(&mut self, bucket: &str, key: &str, data: Vec<u8>) {
        self.ensure_bucket(bucket).put(key, data);
    }

    pub fn get(&mut self, bucket: &str, key: &str) -> Option<&[u8]> {
        self.read_count += 1;
        self.buckets
            .get(bucket)?
            .entries
            .get(key)
            .map(|v| v.as_slice())
    }

    pub fn remove(&mut self, bucket: &str, key: &str) -> bool {
        self.buckets.get_mut(bucket).is_some_and(|b| b.remove(key))
    }

    pub fn contains(&self, bucket: &str, key: &str) -> bool {
        self.buckets.get(bucket).is_some_and(|b| b.contains(key))
    }

    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    pub fn total_entries(&self) -> usize {
        self.buckets.values().map(|b| b.entry_count()).sum()
    }

    pub fn total_bytes(&self) -> usize {
        self.buckets.values().map(|b| b.total_bytes()).sum()
    }

    pub fn read_count(&self) -> u64 {
        self.read_count
    }

    pub fn drop_bucket(&mut self, name: &str) -> bool {
        self.buckets.remove(name).is_some()
    }
}

impl Default for StorageBackend {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_storage_backend() -> StorageBackend {
    StorageBackend::new()
}

pub fn sb_put(sb: &mut StorageBackend, bucket: &str, key: &str, data: Vec<u8>) {
    sb.put(bucket, key, data);
}

pub fn sb_get<'a>(sb: &'a mut StorageBackend, bucket: &str, key: &str) -> Option<&'a [u8]> {
    sb.get(bucket, key)
}

pub fn sb_remove(sb: &mut StorageBackend, bucket: &str, key: &str) -> bool {
    sb.remove(bucket, key)
}

pub fn sb_contains(sb: &StorageBackend, bucket: &str, key: &str) -> bool {
    sb.contains(bucket, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_on_creation() {
        let sb = new_storage_backend();
        assert_eq!(sb.bucket_count(), 0);
        assert_eq!(sb.total_entries(), 0);
    }

    #[test]
    fn put_and_get() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "assets", "mesh.bin", vec![1, 2, 3]);
        let data = sb_get(&mut sb, "assets", "mesh.bin").expect("should succeed");
        assert_eq!(data, &[1, 2, 3]);
    }

    #[test]
    fn missing_key_returns_none() {
        let mut sb = new_storage_backend();
        assert!(sb_get(&mut sb, "bucket", "missing").is_none());
    }

    #[test]
    fn remove_returns_true_once() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "b", "k", vec![0]);
        assert!(sb_remove(&mut sb, "b", "k"));
        assert!(!sb_remove(&mut sb, "b", "k"));
    }

    #[test]
    fn contains_check() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "b", "key", vec![]);
        assert!(sb_contains(&sb, "b", "key"));
        assert!(!sb_contains(&sb, "b", "other"));
    }

    #[test]
    fn total_bytes_sums_across_buckets() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "a", "k1", vec![1, 2]);
        sb_put(&mut sb, "b", "k2", vec![3, 4, 5]);
        assert_eq!(sb.total_bytes(), 5);
    }

    #[test]
    fn drop_bucket() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "tmp", "k", vec![0]);
        assert!(sb.drop_bucket("tmp"));
        assert_eq!(sb.bucket_count(), 0);
    }

    #[test]
    fn read_count_increments() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "b", "k", vec![0]);
        sb_get(&mut sb, "b", "k");
        sb_get(&mut sb, "b", "k");
        assert_eq!(sb.read_count(), 2);
    }

    #[test]
    fn bucket_auto_created() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "new_bucket", "key", vec![7]);
        assert_eq!(sb.bucket_count(), 1);
    }

    #[test]
    fn total_entries_across_buckets() {
        let mut sb = new_storage_backend();
        sb_put(&mut sb, "a", "k1", vec![]);
        sb_put(&mut sb, "a", "k2", vec![]);
        sb_put(&mut sb, "b", "k3", vec![]);
        assert_eq!(sb.total_entries(), 3);
    }
}
