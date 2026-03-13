#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Data pack: a named key-value store of byte payloads.

use std::collections::HashMap;

/// A single entry in a data pack.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PackEntry {
    pub key: String,
    pub data: Vec<u8>,
}

/// A collection of named byte payloads.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DataPack {
    entries: HashMap<String, Vec<u8>>,
}

/// Create a new empty `DataPack`.
#[allow(dead_code)]
pub fn new_data_pack() -> DataPack {
    DataPack::default()
}

/// Insert or overwrite an entry.
#[allow(dead_code)]
pub fn pack_insert(pack: &mut DataPack, key: &str, data: Vec<u8>) {
    pack.entries.insert(key.to_string(), data);
}

/// Get a reference to an entry's data.
#[allow(dead_code)]
pub fn pack_get<'a>(pack: &'a DataPack, key: &str) -> Option<&'a [u8]> {
    pack.entries.get(key).map(|v| v.as_slice())
}

/// Check if the pack contains a key.
#[allow(dead_code)]
pub fn pack_contains(pack: &DataPack, key: &str) -> bool {
    pack.entries.contains_key(key)
}

/// Return the number of entries.
#[allow(dead_code)]
pub fn pack_len(pack: &DataPack) -> usize {
    pack.entries.len()
}

/// Remove an entry by key. Returns true if it existed.
#[allow(dead_code)]
pub fn pack_remove(pack: &mut DataPack, key: &str) -> bool {
    pack.entries.remove(key).is_some()
}

/// Return a sorted list of all keys.
#[allow(dead_code)]
pub fn pack_keys(pack: &DataPack) -> Vec<String> {
    let mut keys: Vec<String> = pack.entries.keys().cloned().collect();
    keys.sort();
    keys
}

/// Serialize the pack to a simple JSON string (byte arrays as decimal arrays).
#[allow(dead_code)]
pub fn pack_to_json(pack: &DataPack) -> String {
    let mut out = String::from("{");
    let mut keys = pack_keys(pack);
    keys.sort();
    for (i, key) in keys.iter().enumerate() {
        let data = &pack.entries[key];
        let arr: Vec<String> = data.iter().map(|b| b.to_string()).collect();
        out.push_str(&format!("\"{}\":[{}]", key, arr.join(",")));
        if i + 1 < pack.entries.len() {
            out.push(',');
        }
    }
    out.push('}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_data_pack() {
        let p = new_data_pack();
        assert_eq!(pack_len(&p), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "foo", vec![1, 2, 3]);
        assert_eq!(pack_get(&p, "foo"), Some([1u8, 2, 3].as_slice()));
    }

    #[test]
    fn test_pack_contains() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "bar", vec![]);
        assert!(pack_contains(&p, "bar"));
        assert!(!pack_contains(&p, "baz"));
    }

    #[test]
    fn test_pack_len() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "a", vec![]);
        pack_insert(&mut p, "b", vec![]);
        assert_eq!(pack_len(&p), 2);
    }

    #[test]
    fn test_pack_remove() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "x", vec![42]);
        assert!(pack_remove(&mut p, "x"));
        assert!(!pack_contains(&p, "x"));
        assert!(!pack_remove(&mut p, "x"));
    }

    #[test]
    fn test_pack_keys() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "c", vec![]);
        pack_insert(&mut p, "a", vec![]);
        pack_insert(&mut p, "b", vec![]);
        assert_eq!(pack_keys(&p), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_pack_to_json() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "k", vec![1, 2]);
        let json = pack_to_json(&p);
        assert!(json.contains("\"k\""));
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_overwrite_entry() {
        let mut p = new_data_pack();
        pack_insert(&mut p, "k", vec![1]);
        pack_insert(&mut p, "k", vec![2, 3]);
        assert_eq!(pack_get(&p, "k"), Some([2u8, 3].as_slice()));
        assert_eq!(pack_len(&p), 1);
    }
}
