// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A string interner that deduplicates strings by assigning each unique string a u32 ID.
#[allow(dead_code)]
pub struct StringInterner {
    pub strings: Vec<String>,
    pub map: HashMap<String, u32>,
}

/// Create a new empty `StringInterner`.
#[allow(dead_code)]
pub fn new_string_interner() -> StringInterner {
    StringInterner {
        strings: Vec::new(),
        map: HashMap::new(),
    }
}

/// Intern a string, returning its ID. If already interned, returns the existing ID.
#[allow(dead_code)]
pub fn intern(si: &mut StringInterner, s: &str) -> u32 {
    if let Some(&id) = si.map.get(s) {
        return id;
    }
    let id = si.strings.len() as u32;
    si.strings.push(s.to_string());
    si.map.insert(s.to_string(), id);
    id
}

/// Retrieve the string for a given ID.
#[allow(dead_code)]
pub fn get_str(si: &StringInterner, id: u32) -> Option<&str> {
    si.strings.get(id as usize).map(|s| s.as_str())
}

/// Return the number of interned strings.
#[allow(dead_code)]
pub fn intern_count(si: &StringInterner) -> usize {
    si.strings.len()
}

/// Check whether a string has already been interned.
#[allow(dead_code)]
pub fn contains_str(si: &StringInterner, s: &str) -> bool {
    si.map.contains_key(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_new_string_assigns_zero() {
        let mut si = new_string_interner();
        let id = intern(&mut si, "hello");
        assert_eq!(id, 0);
    }

    #[test]
    fn intern_same_string_returns_same_id() {
        let mut si = new_string_interner();
        let a = intern(&mut si, "foo");
        let b = intern(&mut si, "foo");
        assert_eq!(a, b);
    }

    #[test]
    fn intern_different_strings_different_ids() {
        let mut si = new_string_interner();
        let a = intern(&mut si, "alpha");
        let b = intern(&mut si, "beta");
        assert_ne!(a, b);
    }

    #[test]
    fn get_str_returns_correct_string() {
        let mut si = new_string_interner();
        let id = intern(&mut si, "world");
        assert_eq!(get_str(&si, id), Some("world"));
    }

    #[test]
    fn get_str_invalid_id_returns_none() {
        let si = new_string_interner();
        assert!(get_str(&si, 999).is_none());
    }

    #[test]
    fn intern_count_increments() {
        let mut si = new_string_interner();
        assert_eq!(intern_count(&si), 0);
        intern(&mut si, "a");
        assert_eq!(intern_count(&si), 1);
        intern(&mut si, "b");
        assert_eq!(intern_count(&si), 2);
    }

    #[test]
    fn intern_count_no_duplicate() {
        let mut si = new_string_interner();
        intern(&mut si, "dup");
        intern(&mut si, "dup");
        assert_eq!(intern_count(&si), 1);
    }

    #[test]
    fn contains_str_before_intern() {
        let si = new_string_interner();
        assert!(!contains_str(&si, "missing"));
    }

    #[test]
    fn contains_str_after_intern() {
        let mut si = new_string_interner();
        intern(&mut si, "present");
        assert!(contains_str(&si, "present"));
    }

    #[test]
    fn multiple_strings_preserved() {
        let mut si = new_string_interner();
        let ids: Vec<u32> = ["x", "y", "z"].iter().map(|s| intern(&mut si, s)).collect();
        assert_eq!(get_str(&si, ids[0]), Some("x"));
        assert_eq!(get_str(&si, ids[1]), Some("y"));
        assert_eq!(get_str(&si, ids[2]), Some("z"));
    }
}
