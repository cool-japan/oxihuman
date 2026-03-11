// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A string-interning table that deduplicates string storage and hands out
//! lightweight integer ids.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternId(u32);

#[allow(dead_code)]
impl InternId {
    pub fn as_u32(self) -> u32 { self.0 }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InternedStringTable {
    strings: Vec<String>,
    lookup: HashMap<String, u32>,
}

#[allow(dead_code)]
impl InternedStringTable {
    pub fn new() -> Self {
        Self { strings: Vec::new(), lookup: HashMap::new() }
    }

    pub fn intern(&mut self, s: &str) -> InternId {
        if let Some(&id) = self.lookup.get(s) {
            return InternId(id);
        }
        let id = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.lookup.insert(s.to_string(), id);
        InternId(id)
    }

    pub fn resolve(&self, id: InternId) -> Option<&str> {
        self.strings.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn contains(&self, s: &str) -> bool {
        self.lookup.contains_key(s)
    }

    pub fn id_of(&self, s: &str) -> Option<InternId> {
        self.lookup.get(s).map(|&id| InternId(id))
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn all_strings(&self) -> &[String] {
        &self.strings
    }

    pub fn clear(&mut self) {
        self.strings.clear();
        self.lookup.clear();
    }
}

impl Default for InternedStringTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_and_resolve() {
        let mut t = InternedStringTable::new();
        let id = t.intern("hello");
        assert_eq!(t.resolve(id), Some("hello"));
    }

    #[test]
    fn test_deduplication() {
        let mut t = InternedStringTable::new();
        let a = t.intern("test");
        let b = t.intern("test");
        assert_eq!(a, b);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn test_contains() {
        let mut t = InternedStringTable::new();
        t.intern("foo");
        assert!(t.contains("foo"));
        assert!(!t.contains("bar"));
    }

    #[test]
    fn test_id_of() {
        let mut t = InternedStringTable::new();
        let id = t.intern("abc");
        assert_eq!(t.id_of("abc"), Some(id));
        assert_eq!(t.id_of("xyz"), None);
    }

    #[test]
    fn test_multiple_strings() {
        let mut t = InternedStringTable::new();
        let a = t.intern("alpha");
        let b = t.intern("beta");
        assert_ne!(a, b);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut t = InternedStringTable::new();
        t.intern("x");
        t.clear();
        assert!(t.is_empty());
    }

    #[test]
    fn test_all_strings() {
        let mut t = InternedStringTable::new();
        t.intern("a");
        t.intern("b");
        assert_eq!(t.all_strings().len(), 2);
    }

    #[test]
    fn test_intern_id_as_u32() {
        let mut t = InternedStringTable::new();
        let id = t.intern("first");
        assert_eq!(id.as_u32(), 0);
    }

    #[test]
    fn test_resolve_invalid() {
        let t = InternedStringTable::new();
        assert_eq!(t.resolve(InternId(999)), None);
    }

    #[test]
    fn test_is_empty() {
        let t = InternedStringTable::new();
        assert!(t.is_empty());
    }
}
