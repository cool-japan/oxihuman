// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple in-memory key-value store with string keys and typed values.

use std::collections::HashMap;

/// Value variant stored in the KV store.
#[derive(Debug, Clone, PartialEq)]
pub enum KvValue {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
    Bytes(Vec<u8>),
}

/// A simple in-memory key-value store.
#[derive(Debug, Default, Clone)]
pub struct KeyValueStore {
    data: HashMap<String, KvValue>,
}

impl KeyValueStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        KeyValueStore { data: HashMap::new() }
    }

    /// Insert or replace a key-value pair.
    pub fn set(&mut self, key: impl Into<String>, value: KvValue) {
        self.data.insert(key.into(), value);
    }

    /// Retrieve a reference to the value for `key`.
    pub fn get(&self, key: &str) -> Option<&KvValue> {
        self.data.get(key)
    }

    /// Delete a key.  Returns true if the key existed.
    pub fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// True if the store contains `key`.
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// List all keys.
    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }
}

/// Create a new key-value store.
pub fn new_kv_store() -> KeyValueStore {
    KeyValueStore::new()
}

/// Set a key.
pub fn kv_set(store: &mut KeyValueStore, key: &str, value: KvValue) {
    store.set(key, value);
}

/// Get a value.
pub fn kv_get<'a>(store: &'a KeyValueStore, key: &str) -> Option<&'a KvValue> {
    store.get(key)
}

/// Delete a key.
pub fn kv_delete(store: &mut KeyValueStore, key: &str) -> bool {
    store.delete(key)
}

/// Number of entries.
pub fn kv_len(store: &KeyValueStore) -> usize {
    store.len()
}

/// Check existence.
pub fn kv_contains(store: &KeyValueStore, key: &str) -> bool {
    store.contains(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_int() {
        let mut s = new_kv_store();
        kv_set(&mut s, "count", KvValue::Int(42));
        assert_eq!(kv_get(&s, "count"), Some(&KvValue::Int(42)) /* int stored */);
    }

    #[test]
    fn test_set_and_get_text() {
        let mut s = new_kv_store();
        kv_set(&mut s, "name", KvValue::Text("alice".into()));
        assert_eq!(kv_get(&s, "name"), Some(&KvValue::Text("alice".into())) /* text */);
    }

    #[test]
    fn test_delete() {
        let mut s = new_kv_store();
        kv_set(&mut s, "k", KvValue::Bool(true));
        assert!(kv_delete(&mut s, "k") /* key existed */);
        assert!(!kv_contains(&s, "k"));
    }

    #[test]
    fn test_delete_missing() {
        let mut s = new_kv_store();
        assert!(!kv_delete(&mut s, "ghost") /* not found */);
    }

    #[test]
    fn test_len() {
        let mut s = new_kv_store();
        kv_set(&mut s, "a", KvValue::Int(1));
        kv_set(&mut s, "b", KvValue::Int(2));
        assert_eq!(kv_len(&s), 2 /* two entries */);
    }

    #[test]
    fn test_contains() {
        let mut s = new_kv_store();
        kv_set(&mut s, "x", KvValue::Float(1.5));
        assert!(kv_contains(&s, "x") /* present */);
        assert!(!kv_contains(&s, "y") /* absent */);
    }

    #[test]
    fn test_clear() {
        let mut s = new_kv_store();
        kv_set(&mut s, "a", KvValue::Int(1));
        s.clear();
        assert!(s.is_empty() /* cleared */);
    }

    #[test]
    fn test_bytes_value() {
        let mut s = new_kv_store();
        kv_set(&mut s, "data", KvValue::Bytes(vec![0, 1, 2]));
        assert_eq!(kv_get(&s, "data"), Some(&KvValue::Bytes(vec![0, 1, 2])) /* bytes */);
    }

    #[test]
    fn test_overwrite() {
        let mut s = new_kv_store();
        kv_set(&mut s, "v", KvValue::Int(1));
        kv_set(&mut s, "v", KvValue::Int(2));
        assert_eq!(kv_get(&s, "v"), Some(&KvValue::Int(2)) /* overwritten */);
        assert_eq!(kv_len(&s), 1);
    }

    #[test]
    fn test_keys() {
        let mut s = new_kv_store();
        kv_set(&mut s, "p", KvValue::Bool(false));
        let keys = s.keys();
        assert!(keys.contains(&"p") /* key present */);
    }
}
