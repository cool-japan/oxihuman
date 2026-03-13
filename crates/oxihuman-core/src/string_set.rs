// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! An ordered set of strings with fast lookup and sorted iteration.

use std::collections::BTreeSet;

/// An ordered set of owned strings.
#[allow(dead_code)]
pub struct StringSet {
    inner: BTreeSet<String>,
    add_count: u64,
    remove_count: u64,
}

#[allow(dead_code)]
impl StringSet {
    pub fn new() -> Self {
        Self {
            inner: BTreeSet::new(),
            add_count: 0,
            remove_count: 0,
        }
    }

    pub fn insert(&mut self, s: &str) -> bool {
        let inserted = self.inner.insert(s.to_string());
        if inserted {
            self.add_count += 1;
        }
        inserted
    }

    pub fn remove(&mut self, s: &str) -> bool {
        let removed = self.inner.remove(s);
        if removed {
            self.remove_count += 1;
        }
        removed
    }

    pub fn contains(&self, s: &str) -> bool {
        self.inner.contains(s)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.inner.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns elements present in both sets.
    pub fn intersection(&self, other: &StringSet) -> StringSet {
        let mut result = StringSet::new();
        for s in &self.inner {
            if other.contains(s) {
                result.insert(s);
            }
        }
        result
    }

    /// Returns elements present in either set.
    pub fn union(&self, other: &StringSet) -> StringSet {
        let mut result = StringSet::new();
        for s in &self.inner {
            result.insert(s);
        }
        for s in &other.inner {
            result.insert(s);
        }
        result
    }

    /// Returns elements in self but not in other.
    pub fn difference(&self, other: &StringSet) -> StringSet {
        let mut result = StringSet::new();
        for s in &self.inner {
            if !other.contains(s) {
                result.insert(s);
            }
        }
        result
    }

    pub fn add_count(&self) -> u64 {
        self.add_count
    }
    pub fn remove_count(&self) -> u64 {
        self.remove_count
    }

    pub fn starts_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.inner
            .iter()
            .filter(|s| s.starts_with(prefix))
            .cloned()
            .collect()
    }
}

impl Default for StringSet {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_string_set() -> StringSet {
    StringSet::new()
}

pub fn ss_insert(set: &mut StringSet, s: &str) -> bool {
    set.insert(s)
}

pub fn ss_contains(set: &StringSet, s: &str) -> bool {
    set.contains(s)
}

pub fn ss_remove(set: &mut StringSet, s: &str) -> bool {
    set.remove(s)
}

pub fn ss_len(set: &StringSet) -> usize {
    set.len()
}

pub fn ss_to_vec(set: &StringSet) -> Vec<String> {
    set.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let s = new_string_set();
        assert!(s.is_empty());
    }

    #[test]
    fn insert_and_contains() {
        let mut s = new_string_set();
        ss_insert(&mut s, "hello");
        assert!(ss_contains(&s, "hello"));
        assert!(!ss_contains(&s, "world"));
    }

    #[test]
    fn insert_duplicate_returns_false() {
        let mut s = new_string_set();
        assert!(ss_insert(&mut s, "x"));
        assert!(!ss_insert(&mut s, "x"));
        assert_eq!(ss_len(&s), 1);
    }

    #[test]
    fn remove_present() {
        let mut s = new_string_set();
        ss_insert(&mut s, "y");
        assert!(ss_remove(&mut s, "y"));
        assert!(!ss_contains(&s, "y"));
    }

    #[test]
    fn sorted_iteration() {
        let mut s = new_string_set();
        ss_insert(&mut s, "c");
        ss_insert(&mut s, "a");
        ss_insert(&mut s, "b");
        assert_eq!(
            ss_to_vec(&s),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn intersection() {
        let mut a = new_string_set();
        let mut b = new_string_set();
        ss_insert(&mut a, "x");
        ss_insert(&mut a, "y");
        ss_insert(&mut b, "y");
        ss_insert(&mut b, "z");
        let inter = a.intersection(&b);
        assert_eq!(inter.to_vec(), vec!["y".to_string()]);
    }

    #[test]
    fn union_combines() {
        let mut a = new_string_set();
        let mut b = new_string_set();
        ss_insert(&mut a, "x");
        ss_insert(&mut b, "y");
        assert_eq!(a.union(&b).len(), 2);
    }

    #[test]
    fn difference() {
        let mut a = new_string_set();
        let mut b = new_string_set();
        ss_insert(&mut a, "x");
        ss_insert(&mut a, "y");
        ss_insert(&mut b, "y");
        let diff = a.difference(&b);
        assert_eq!(diff.to_vec(), vec!["x".to_string()]);
    }

    #[test]
    fn starts_with_prefix() {
        let mut s = new_string_set();
        ss_insert(&mut s, "asset_mesh");
        ss_insert(&mut s, "asset_tex");
        ss_insert(&mut s, "config");
        let matches = s.starts_with_prefix("asset_");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn clear_empties_set() {
        let mut s = new_string_set();
        ss_insert(&mut s, "a");
        s.clear();
        assert!(s.is_empty());
    }
}
