#![allow(dead_code)]

use std::collections::HashMap;

/// A map where each key holds a bucket (Vec) of values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BucketMap<V> {
    buckets: HashMap<String, Vec<V>>,
}

/// Creates a new empty bucket map.
#[allow(dead_code)]
pub fn new_bucket_map<V>() -> BucketMap<V> {
    BucketMap {
        buckets: HashMap::new(),
    }
}

/// Inserts a value into the bucket for the given key.
#[allow(dead_code)]
pub fn bucket_insert<V>(map: &mut BucketMap<V>, key: &str, value: V) {
    map.buckets
        .entry(key.to_string())
        .or_default()
        .push(value);
}

/// Gets all values for a key.
#[allow(dead_code)]
pub fn bucket_get<'a, V>(map: &'a BucketMap<V>, key: &str) -> Option<&'a [V]> {
    map.buckets.get(key).map(|v| v.as_slice())
}

/// Returns the number of buckets (unique keys).
#[allow(dead_code)]
pub fn bucket_count<V>(map: &BucketMap<V>) -> usize {
    map.buckets.len()
}

/// Returns all keys sorted.
#[allow(dead_code)]
pub fn bucket_keys<V>(map: &BucketMap<V>) -> Vec<String> {
    let mut keys: Vec<String> = map.buckets.keys().cloned().collect();
    keys.sort();
    keys
}

/// Returns values at a specific bucket key.
#[allow(dead_code)]
pub fn bucket_values_at<V>(map: &BucketMap<V>, key: &str) -> usize {
    map.buckets.get(key).map_or(0, |v| v.len())
}

/// Returns total number of values across all buckets.
#[allow(dead_code)]
pub fn bucket_total_values<V>(map: &BucketMap<V>) -> usize {
    map.buckets.values().map(|v| v.len()).sum()
}

/// Clears all buckets.
#[allow(dead_code)]
pub fn bucket_clear<V>(map: &mut BucketMap<V>) {
    map.buckets.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bucket_map() {
        let map: BucketMap<i32> = new_bucket_map();
        assert_eq!(bucket_count(&map), 0);
    }

    #[test]
    fn test_insert_get() {
        let mut map = new_bucket_map();
        bucket_insert(&mut map, "fruits", "apple");
        assert_eq!(bucket_get(&map, "fruits"), Some(["apple"].as_slice()));
    }

    #[test]
    fn test_multiple_values() {
        let mut map = new_bucket_map();
        bucket_insert(&mut map, "k", 1);
        bucket_insert(&mut map, "k", 2);
        assert_eq!(bucket_values_at(&map, "k"), 2);
    }

    #[test]
    fn test_bucket_count() {
        let mut map: BucketMap<i32> = new_bucket_map();
        bucket_insert(&mut map, "a", 1);
        bucket_insert(&mut map, "b", 2);
        assert_eq!(bucket_count(&map), 2);
    }

    #[test]
    fn test_bucket_keys() {
        let mut map: BucketMap<i32> = new_bucket_map();
        bucket_insert(&mut map, "b", 1);
        bucket_insert(&mut map, "a", 2);
        assert_eq!(bucket_keys(&map), vec!["a", "b"]);
    }

    #[test]
    fn test_total_values() {
        let mut map: BucketMap<i32> = new_bucket_map();
        bucket_insert(&mut map, "a", 1);
        bucket_insert(&mut map, "a", 2);
        bucket_insert(&mut map, "b", 3);
        assert_eq!(bucket_total_values(&map), 3);
    }

    #[test]
    fn test_clear() {
        let mut map: BucketMap<i32> = new_bucket_map();
        bucket_insert(&mut map, "a", 1);
        bucket_clear(&mut map);
        assert_eq!(bucket_count(&map), 0);
    }

    #[test]
    fn test_get_nonexistent() {
        let map: BucketMap<i32> = new_bucket_map();
        assert_eq!(bucket_get(&map, "nope"), None);
    }

    #[test]
    fn test_values_at_nonexistent() {
        let map: BucketMap<i32> = new_bucket_map();
        assert_eq!(bucket_values_at(&map, "nope"), 0);
    }

    #[test]
    fn test_empty_total() {
        let map: BucketMap<i32> = new_bucket_map();
        assert_eq!(bucket_total_values(&map), 0);
    }
}
