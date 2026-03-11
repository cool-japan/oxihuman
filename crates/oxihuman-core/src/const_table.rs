// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A read-only lookup table built from a sorted list of key-value pairs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstTable<V: Clone> {
    entries: Vec<(String, V)>,
}

#[allow(dead_code)]
impl<V: Clone> ConstTable<V> {
    pub fn from_pairs(mut pairs: Vec<(String, V)>) -> Self {
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        Self { entries: pairs }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.entries
            .binary_search_by(|e| e.0.as_str().cmp(key))
            .ok()
            .map(|i| &self.entries[i].1)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries
            .binary_search_by(|e| e.0.as_str().cmp(key))
            .is_ok()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.0.as_str()).collect()
    }

    pub fn values(&self) -> Vec<&V> {
        self.entries.iter().map(|e| &e.1).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn get_index(&self, index: usize) -> Option<(&str, &V)> {
        self.entries.get(index).map(|(k, v)| (k.as_str(), v))
    }

    pub fn first(&self) -> Option<(&str, &V)> {
        self.entries.first().map(|(k, v)| (k.as_str(), v))
    }

    pub fn last(&self) -> Option<(&str, &V)> {
        self.entries.last().map(|(k, v)| (k.as_str(), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> ConstTable<i32> {
        ConstTable::from_pairs(vec![
            ("cherry".to_string(), 3),
            ("apple".to_string(), 1),
            ("banana".to_string(), 2),
        ])
    }

    #[test]
    fn test_get_existing() {
        let t = sample_table();
        assert_eq!(t.get("banana"), Some(&2));
    }

    #[test]
    fn test_get_missing() {
        let t = sample_table();
        assert!(t.get("date").is_none());
    }

    #[test]
    fn test_contains_key() {
        let t = sample_table();
        assert!(t.contains_key("apple"));
        assert!(!t.contains_key("mango"));
    }

    #[test]
    fn test_len() {
        let t = sample_table();
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn test_empty() {
        let t: ConstTable<i32> = ConstTable::from_pairs(vec![]);
        assert!(t.is_empty());
    }

    #[test]
    fn test_keys_sorted() {
        let t = sample_table();
        let keys = t.keys();
        assert_eq!(keys, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_values() {
        let t = sample_table();
        let vals = t.values();
        assert_eq!(vals, vec![&1, &2, &3]);
    }

    #[test]
    fn test_first_last() {
        let t = sample_table();
        assert_eq!(t.first().unwrap().0, "apple");
        assert_eq!(t.last().unwrap().0, "cherry");
    }

    #[test]
    fn test_get_index() {
        let t = sample_table();
        let (k, v) = t.get_index(1).unwrap();
        assert_eq!(k, "banana");
        assert_eq!(*v, 2);
    }

    #[test]
    fn test_iter() {
        let t = sample_table();
        let collected: Vec<_> = t.iter().collect();
        assert_eq!(collected.len(), 3);
    }
}
