// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A heterogeneous key-value bag for storing named state entries.

use std::collections::HashMap;

/// Value type stored in a [`StateBag`].
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BagValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    Text(String),
}

/// A named collection of typed state values.
#[allow(dead_code)]
pub struct StateBag {
    entries: HashMap<String, BagValue>,
    version: u64,
}

#[allow(dead_code)]
impl StateBag {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: 0,
        }
    }

    pub fn set(&mut self, key: &str, val: BagValue) {
        self.entries.insert(key.to_string(), val);
        self.version += 1;
    }

    pub fn get(&self, key: &str) -> Option<&BagValue> {
        self.entries.get(key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if self.entries.remove(key).is_some() {
            self.version += 1;
            true
        } else {
            false
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn keys(&self) -> Vec<String> {
        let mut k: Vec<String> = self.entries.keys().cloned().collect();
        k.sort();
        k
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.version += 1;
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.get(key)? {
            BagValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.entries.get(key)? {
            BagValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.entries.get(key)? {
            BagValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn get_text(&self, key: &str) -> Option<&str> {
        match self.entries.get(key)? {
            BagValue::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl Default for StateBag {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_state_bag() -> StateBag {
    StateBag::new()
}

pub fn sb_set(bag: &mut StateBag, key: &str, val: BagValue) {
    bag.set(key, val);
}

pub fn sb_get(bag: &StateBag, key: &str) -> Option<BagValue> {
    bag.get(key).cloned()
}

pub fn sb_remove(bag: &mut StateBag, key: &str) -> bool {
    bag.remove(key)
}

pub fn sb_len(bag: &StateBag) -> usize {
    bag.len()
}

pub fn sb_clear(bag: &mut StateBag) {
    bag.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let b = new_state_bag();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn set_and_get_bool() {
        let mut b = new_state_bag();
        sb_set(&mut b, "flag", BagValue::Bool(true));
        assert_eq!(b.get_bool("flag"), Some(true));
    }

    #[test]
    fn set_and_get_int() {
        let mut b = new_state_bag();
        sb_set(&mut b, "count", BagValue::Int(42));
        assert_eq!(b.get_int("count"), Some(42));
    }

    #[test]
    fn set_and_get_float() {
        let mut b = new_state_bag();
        sb_set(&mut b, "speed", BagValue::Float(1.5));
        assert!((b.get_float("speed").unwrap() - 1.5).abs() < 1e-6);
    }

    #[test]
    fn set_and_get_text() {
        let mut b = new_state_bag();
        sb_set(&mut b, "name", BagValue::Text("hero".to_string()));
        assert_eq!(b.get_text("name"), Some("hero"));
    }

    #[test]
    fn remove_decrements_len() {
        let mut b = new_state_bag();
        sb_set(&mut b, "x", BagValue::Bool(false));
        assert!(sb_remove(&mut b, "x"));
        assert_eq!(sb_len(&b), 0);
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut b = new_state_bag();
        assert!(!sb_remove(&mut b, "ghost"));
    }

    #[test]
    fn version_increments_on_mutation() {
        let mut b = new_state_bag();
        let v0 = b.version();
        sb_set(&mut b, "k", BagValue::Int(1));
        assert!(b.version() > v0);
    }

    #[test]
    fn contains_key() {
        let mut b = new_state_bag();
        sb_set(&mut b, "alive", BagValue::Bool(true));
        assert!(b.contains("alive"));
        assert!(!b.contains("dead"));
    }

    #[test]
    fn clear_empties_bag() {
        let mut b = new_state_bag();
        sb_set(&mut b, "a", BagValue::Int(1));
        sb_set(&mut b, "b", BagValue::Int(2));
        sb_clear(&mut b);
        assert!(b.is_empty());
    }
}
