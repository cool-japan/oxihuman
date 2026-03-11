#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lightweight resource reference.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceRef {
    id: u64,
    path: String,
    res_type: String,
    loaded: bool,
    hash: u64,
}

#[allow(dead_code)]
pub fn new_resource_ref(id: u64, path: &str, res_type: &str) -> ResourceRef {
    let mut h: u64 = 0;
    for b in path.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u64::from(b));
    }
    ResourceRef {
        id,
        path: path.to_string(),
        res_type: res_type.to_string(),
        loaded: false,
        hash: h,
    }
}

#[allow(dead_code)]
pub fn ref_is_loaded(r: &ResourceRef) -> bool {
    r.loaded
}

#[allow(dead_code)]
pub fn ref_path(r: &ResourceRef) -> &str {
    &r.path
}

#[allow(dead_code)]
pub fn ref_type(r: &ResourceRef) -> &str {
    &r.res_type
}

#[allow(dead_code)]
pub fn ref_id(r: &ResourceRef) -> u64 {
    r.id
}

#[allow(dead_code)]
pub fn ref_to_json(r: &ResourceRef) -> String {
    format!(
        r#"{{"id":{},"path":"{}","type":"{}","loaded":{}}}"#,
        r.id, r.path, r.res_type, r.loaded
    )
}

#[allow(dead_code)]
pub fn ref_hash(r: &ResourceRef) -> u64 {
    r.hash
}

#[allow(dead_code)]
pub fn refs_equal(a: &ResourceRef, b: &ResourceRef) -> bool {
    a.id == b.id && a.path == b.path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ref() {
        let r = new_resource_ref(1, "/tex/a.png", "texture");
        assert_eq!(ref_id(&r), 1);
    }

    #[test]
    fn test_ref_path() {
        let r = new_resource_ref(1, "/tex/a.png", "texture");
        assert_eq!(ref_path(&r), "/tex/a.png");
    }

    #[test]
    fn test_ref_type() {
        let r = new_resource_ref(1, "/tex/a.png", "texture");
        assert_eq!(ref_type(&r), "texture");
    }

    #[test]
    fn test_ref_not_loaded() {
        let r = new_resource_ref(1, "/a", "t");
        assert!(!ref_is_loaded(&r));
    }

    #[test]
    fn test_ref_to_json() {
        let r = new_resource_ref(1, "/a", "t");
        let json = ref_to_json(&r);
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_ref_hash() {
        let r = new_resource_ref(1, "/a", "t");
        assert!(ref_hash(&r) > 0);
    }

    #[test]
    fn test_refs_equal() {
        let a = new_resource_ref(1, "/a", "t");
        let b = new_resource_ref(1, "/a", "t");
        assert!(refs_equal(&a, &b));
    }

    #[test]
    fn test_refs_not_equal() {
        let a = new_resource_ref(1, "/a", "t");
        let b = new_resource_ref(2, "/b", "t");
        assert!(!refs_equal(&a, &b));
    }

    #[test]
    fn test_hash_deterministic() {
        let a = new_resource_ref(1, "/same", "t");
        let b = new_resource_ref(2, "/same", "t");
        assert_eq!(ref_hash(&a), ref_hash(&b));
    }

    #[test]
    fn test_different_hash() {
        let a = new_resource_ref(1, "/aaa", "t");
        let b = new_resource_ref(1, "/bbb", "t");
        assert_ne!(ref_hash(&a), ref_hash(&b));
    }
}
