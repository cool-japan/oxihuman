#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lightweight type tag for runtime type identification.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A named type tag with a deterministic hash-based ID.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeTag {
    name: String,
    id: u64,
}

fn compute_id(name: &str) -> u64 {
    let mut h = DefaultHasher::new();
    name.hash(&mut h);
    h.finish()
}

#[allow(dead_code)]
pub fn new_type_tag(name: &str) -> TypeTag {
    TypeTag {
        name: name.to_string(),
        id: compute_id(name),
    }
}

#[allow(dead_code)]
pub fn type_tag_name(tag: &TypeTag) -> &str {
    &tag.name
}

#[allow(dead_code)]
pub fn type_tag_id(tag: &TypeTag) -> u64 {
    tag.id
}

#[allow(dead_code)]
pub fn type_tags_equal(a: &TypeTag, b: &TypeTag) -> bool {
    a.id == b.id && a.name == b.name
}

#[allow(dead_code)]
pub fn type_tag_hash(tag: &TypeTag) -> u64 {
    tag.id
}

#[allow(dead_code)]
pub fn type_tag_to_string(tag: &TypeTag) -> String {
    format!("{}:{}", tag.name, tag.id)
}

#[allow(dead_code)]
pub fn type_tag_from_str(s: &str) -> Option<TypeTag> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some(new_type_tag(parts[0]))
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn type_tag_is_empty(tag: &TypeTag) -> bool {
    tag.name.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let t = new_type_tag("Mesh");
        assert_eq!(type_tag_name(&t), "Mesh");
    }

    #[test]
    fn test_id_deterministic() {
        let a = new_type_tag("Foo");
        let b = new_type_tag("Foo");
        assert_eq!(type_tag_id(&a), type_tag_id(&b));
    }

    #[test]
    fn test_different_names_different_ids() {
        let a = new_type_tag("A");
        let b = new_type_tag("B");
        assert_ne!(type_tag_id(&a), type_tag_id(&b));
    }

    #[test]
    fn test_equal() {
        let a = new_type_tag("X");
        let b = new_type_tag("X");
        assert!(type_tags_equal(&a, &b));
    }

    #[test]
    fn test_not_equal() {
        let a = new_type_tag("X");
        let b = new_type_tag("Y");
        assert!(!type_tags_equal(&a, &b));
    }

    #[test]
    fn test_hash() {
        let t = new_type_tag("test");
        assert_ne!(type_tag_hash(&t), 0);
    }

    #[test]
    fn test_to_string() {
        let t = new_type_tag("Obj");
        let s = type_tag_to_string(&t);
        assert!(s.starts_with("Obj:"));
    }

    #[test]
    fn test_from_str() {
        let t = new_type_tag("Obj");
        let s = type_tag_to_string(&t);
        let parsed = type_tag_from_str(&s).unwrap();
        assert_eq!(type_tag_name(&parsed), "Obj");
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(type_tag_from_str("no_colon").is_none());
    }

    #[test]
    fn test_is_empty() {
        let t = new_type_tag("");
        assert!(type_tag_is_empty(&t));
        let t2 = new_type_tag("X");
        assert!(!type_tag_is_empty(&t2));
    }
}
