// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hash-bucket map: a simple open-bucket hash table for u64 keys.

/// Single bucket entry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BucketEntry {
    pub key: u64,
    pub value: u64,
}

/// Hash-bucket map with configurable bucket count.
#[derive(Debug)]
#[allow(dead_code)]
pub struct HashBucket {
    buckets: Vec<Vec<BucketEntry>>,
    count: usize,
}

/// Create a HashBucket with `n_buckets` buckets.
#[allow(dead_code)]
pub fn new_hash_bucket(n_buckets: usize) -> HashBucket {
    let n = if n_buckets == 0 { 16 } else { n_buckets };
    HashBucket {
        buckets: vec![Vec::new(); n],
        count: 0,
    }
}

fn bucket_idx(hb: &HashBucket, key: u64) -> usize {
    (key as usize) % hb.buckets.len()
}

/// Insert or update a key-value pair.
#[allow(dead_code)]
pub fn hb_insert(hb: &mut HashBucket, key: u64, value: u64) {
    let idx = bucket_idx(hb, key);
    for entry in &mut hb.buckets[idx] {
        if entry.key == key {
            entry.value = value;
            return;
        }
    }
    hb.buckets[idx].push(BucketEntry { key, value });
    hb.count += 1;
}

/// Look up a key.
#[allow(dead_code)]
pub fn hb_get(hb: &HashBucket, key: u64) -> Option<u64> {
    let idx = bucket_idx(hb, key);
    hb.buckets[idx]
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.value)
}

/// Remove a key; returns old value if present.
#[allow(dead_code)]
pub fn hb_remove(hb: &mut HashBucket, key: u64) -> Option<u64> {
    let idx = bucket_idx(hb, key);
    let bucket = &mut hb.buckets[idx];
    if let Some(pos) = bucket.iter().position(|e| e.key == key) {
        let val = bucket.remove(pos).value;
        hb.count -= 1;
        Some(val)
    } else {
        None
    }
}

/// Whether a key exists.
#[allow(dead_code)]
pub fn hb_contains(hb: &HashBucket, key: u64) -> bool {
    hb_get(hb, key).is_some()
}

/// Total number of stored entries.
#[allow(dead_code)]
pub fn hb_count(hb: &HashBucket) -> usize {
    hb.count
}

/// Number of buckets.
#[allow(dead_code)]
pub fn hb_bucket_count(hb: &HashBucket) -> usize {
    hb.buckets.len()
}

/// Clear all entries.
#[allow(dead_code)]
pub fn hb_clear(hb: &mut HashBucket) {
    for bucket in &mut hb.buckets {
        bucket.clear();
    }
    hb.count = 0;
}

/// Collect all keys.
#[allow(dead_code)]
pub fn hb_keys(hb: &HashBucket) -> Vec<u64> {
    hb.buckets
        .iter()
        .flat_map(|b| b.iter().map(|e| e.key))
        .collect()
}

/// Load factor (entries / buckets).
#[allow(dead_code)]
pub fn hb_load_factor(hb: &HashBucket) -> f32 {
    hb.count as f32 / hb.buckets.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut hb = new_hash_bucket(8);
        hb_insert(&mut hb, 1, 100);
        assert_eq!(hb_get(&hb, 1), Some(100));
    }

    #[test]
    fn test_update() {
        let mut hb = new_hash_bucket(8);
        hb_insert(&mut hb, 5, 10);
        hb_insert(&mut hb, 5, 20);
        assert_eq!(hb_get(&hb, 5), Some(20));
        assert_eq!(hb_count(&hb), 1);
    }

    #[test]
    fn test_remove() {
        let mut hb = new_hash_bucket(8);
        hb_insert(&mut hb, 3, 300);
        assert_eq!(hb_remove(&mut hb, 3), Some(300));
        assert!(!hb_contains(&hb, 3));
    }

    #[test]
    fn test_contains() {
        let mut hb = new_hash_bucket(8);
        hb_insert(&mut hb, 7, 77);
        assert!(hb_contains(&hb, 7));
        assert!(!hb_contains(&hb, 8));
    }

    #[test]
    fn test_count() {
        let mut hb = new_hash_bucket(4);
        hb_insert(&mut hb, 1, 1);
        hb_insert(&mut hb, 2, 2);
        assert_eq!(hb_count(&hb), 2);
    }

    #[test]
    fn test_clear() {
        let mut hb = new_hash_bucket(8);
        hb_insert(&mut hb, 1, 1);
        hb_clear(&mut hb);
        assert_eq!(hb_count(&hb), 0);
    }

    #[test]
    fn test_keys() {
        let mut hb = new_hash_bucket(8);
        hb_insert(&mut hb, 10, 1);
        hb_insert(&mut hb, 20, 2);
        let mut keys = hb_keys(&hb);
        keys.sort();
        assert_eq!(keys, vec![10, 20]);
    }

    #[test]
    fn test_collision_same_bucket() {
        let mut hb = new_hash_bucket(1);
        hb_insert(&mut hb, 1, 111);
        hb_insert(&mut hb, 2, 222);
        assert_eq!(hb_get(&hb, 1), Some(111));
        assert_eq!(hb_get(&hb, 2), Some(222));
    }

    #[test]
    fn test_load_factor() {
        let mut hb = new_hash_bucket(10);
        hb_insert(&mut hb, 1, 1);
        hb_insert(&mut hb, 2, 2);
        let lf = hb_load_factor(&hb);
        assert!((lf - 0.2_f32).abs() < 1e-5);
    }
}
