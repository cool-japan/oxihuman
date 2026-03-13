// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Type-erased value store: heterogeneous map keyed by string.

use std::collections::HashMap;

/// A type-erased slot holding raw bytes plus a type tag.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErasedSlot {
    pub type_tag: String,
    pub bytes: Vec<u8>,
}

/// Type-erased value store.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TypeErased {
    slots: HashMap<String, ErasedSlot>,
}

/// Create a new `TypeErased` store.
#[allow(dead_code)]
pub fn new_type_erased() -> TypeErased {
    TypeErased::default()
}

/// Insert a value as raw bytes with a type tag.
#[allow(dead_code)]
pub fn te_insert(te: &mut TypeErased, key: &str, type_tag: &str, bytes: Vec<u8>) {
    te.slots.insert(
        key.to_string(),
        ErasedSlot {
            type_tag: type_tag.to_string(),
            bytes,
        },
    );
}

/// Get raw bytes for a key.
#[allow(dead_code)]
pub fn te_get<'a>(te: &'a TypeErased, key: &str) -> Option<&'a [u8]> {
    te.slots.get(key).map(|s| s.bytes.as_slice())
}

/// Get the type tag for a key.
#[allow(dead_code)]
pub fn te_type_tag<'a>(te: &'a TypeErased, key: &str) -> Option<&'a str> {
    te.slots.get(key).map(|s| s.type_tag.as_str())
}

/// Whether a key exists.
#[allow(dead_code)]
pub fn te_contains(te: &TypeErased, key: &str) -> bool {
    te.slots.contains_key(key)
}

/// Remove a key.
#[allow(dead_code)]
pub fn te_remove(te: &mut TypeErased, key: &str) -> bool {
    te.slots.remove(key).is_some()
}

/// Number of stored values.
#[allow(dead_code)]
pub fn te_len(te: &TypeErased) -> usize {
    te.slots.len()
}

/// Whether the store is empty.
#[allow(dead_code)]
pub fn te_is_empty(te: &TypeErased) -> bool {
    te.slots.is_empty()
}

/// Clear all values.
#[allow(dead_code)]
pub fn te_clear(te: &mut TypeErased) {
    te.slots.clear();
}

/// All keys as a sorted Vec.
#[allow(dead_code)]
pub fn te_keys(te: &TypeErased) -> Vec<String> {
    let mut keys: Vec<String> = te.slots.keys().cloned().collect();
    keys.sort_unstable();
    keys
}

/// All keys with a given type tag.
#[allow(dead_code)]
pub fn te_keys_by_type(te: &TypeErased, type_tag: &str) -> Vec<String> {
    let mut keys: Vec<String> = te
        .slots
        .iter()
        .filter(|(_, s)| s.type_tag == type_tag)
        .map(|(k, _)| k.clone())
        .collect();
    keys.sort_unstable();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let te = new_type_erased();
        assert!(te_is_empty(&te));
    }

    #[test]
    fn test_insert_and_get() {
        let mut te = new_type_erased();
        te_insert(&mut te, "pos", "Vec3", vec![0, 0, 0]);
        assert_eq!(te_get(&te, "pos"), Some([0u8, 0, 0].as_slice()));
    }

    #[test]
    fn test_type_tag() {
        let mut te = new_type_erased();
        te_insert(&mut te, "val", "f32", vec![0, 0, 0, 0]);
        assert_eq!(te_type_tag(&te, "val"), Some("f32"));
    }

    #[test]
    fn test_contains() {
        let mut te = new_type_erased();
        te_insert(&mut te, "x", "i32", vec![1, 0, 0, 0]);
        assert!(te_contains(&te, "x"));
        assert!(!te_contains(&te, "y"));
    }

    #[test]
    fn test_remove() {
        let mut te = new_type_erased();
        te_insert(&mut te, "k", "u8", vec![5]);
        assert!(te_remove(&mut te, "k"));
        assert!(!te_contains(&te, "k"));
    }

    #[test]
    fn test_len() {
        let mut te = new_type_erased();
        te_insert(&mut te, "a", "u8", vec![]);
        te_insert(&mut te, "b", "u8", vec![]);
        assert_eq!(te_len(&te), 2);
    }

    #[test]
    fn test_clear() {
        let mut te = new_type_erased();
        te_insert(&mut te, "a", "u8", vec![1]);
        te_clear(&mut te);
        assert!(te_is_empty(&te));
    }

    #[test]
    fn test_keys_sorted() {
        let mut te = new_type_erased();
        te_insert(&mut te, "c", "u8", vec![]);
        te_insert(&mut te, "a", "u8", vec![]);
        te_insert(&mut te, "b", "u8", vec![]);
        let keys = te_keys(&te);
        assert_eq!(
            keys,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_keys_by_type() {
        let mut te = new_type_erased();
        te_insert(&mut te, "x", "Vec3", vec![]);
        te_insert(&mut te, "y", "f32", vec![]);
        te_insert(&mut te, "z", "Vec3", vec![]);
        let vec3_keys = te_keys_by_type(&te, "Vec3");
        assert_eq!(vec3_keys.len(), 2);
    }
}
