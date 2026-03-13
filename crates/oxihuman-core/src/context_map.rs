// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A string-keyed context map for passing heterogeneous values through pipelines.

use std::collections::HashMap;

/// Supported value types in the context map.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CtxValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    Str(String),
    Bytes(Vec<u8>),
}

/// String-keyed map of typed context values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ContextMap {
    data: HashMap<String, CtxValue>,
}

#[allow(dead_code)]
impl ContextMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_bool(&mut self, key: &str, val: bool) {
        self.data.insert(key.to_string(), CtxValue::Bool(val));
    }

    pub fn set_int(&mut self, key: &str, val: i64) {
        self.data.insert(key.to_string(), CtxValue::Int(val));
    }

    pub fn set_float(&mut self, key: &str, val: f32) {
        self.data.insert(key.to_string(), CtxValue::Float(val));
    }

    pub fn set_str(&mut self, key: &str, val: &str) {
        self.data
            .insert(key.to_string(), CtxValue::Str(val.to_string()));
    }

    pub fn set_bytes(&mut self, key: &str, val: Vec<u8>) {
        self.data.insert(key.to_string(), CtxValue::Bytes(val));
    }

    pub fn get(&self, key: &str) -> Option<&CtxValue> {
        self.data.get(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.data.get(key) {
            Some(CtxValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.data.get(key) {
            Some(CtxValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.data.get(key) {
            Some(CtxValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.data.get(key) {
            Some(CtxValue::Str(v)) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|k| k.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Merge another map into self; other's values win on conflict.
    pub fn merge(&mut self, other: &ContextMap) {
        for (k, v) in &other.data {
            self.data.insert(k.clone(), v.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        assert!(ContextMap::new().is_empty());
    }

    #[test]
    fn set_get_bool() {
        let mut m = ContextMap::new();
        m.set_bool("flag", true);
        assert_eq!(m.get_bool("flag"), Some(true));
    }

    #[test]
    fn set_get_int() {
        let mut m = ContextMap::new();
        m.set_int("count", 42);
        assert_eq!(m.get_int("count"), Some(42));
    }

    #[test]
    fn set_get_float() {
        let mut m = ContextMap::new();
        m.set_float("ratio", std::f32::consts::PI);
        assert!(
            (m.get_float("ratio").expect("should succeed") - std::f32::consts::PI).abs() < 1e-6
        );
    }

    #[test]
    fn set_get_str() {
        let mut m = ContextMap::new();
        m.set_str("label", "hello");
        assert_eq!(m.get_str("label"), Some("hello"));
    }

    #[test]
    fn type_mismatch_returns_none() {
        let mut m = ContextMap::new();
        m.set_bool("x", false);
        assert!(m.get_int("x").is_none());
    }

    #[test]
    fn remove_entry() {
        let mut m = ContextMap::new();
        m.set_str("a", "v");
        assert!(m.remove("a"));
        assert!(!m.contains("a"));
        assert!(!m.remove("a"));
    }

    #[test]
    fn merge_overwrites() {
        let mut a = ContextMap::new();
        a.set_int("n", 1);
        let mut b = ContextMap::new();
        b.set_int("n", 2);
        b.set_str("extra", "hi");
        a.merge(&b);
        assert_eq!(a.get_int("n"), Some(2));
        assert_eq!(a.get_str("extra"), Some("hi"));
    }

    #[test]
    fn set_bytes_and_get() {
        let mut m = ContextMap::new();
        m.set_bytes("data", vec![1, 2, 3]);
        assert!(m
            .get("data")
            .is_some_and(|v| matches!(v, CtxValue::Bytes(_))));
    }

    #[test]
    fn clear_empties() {
        let mut m = ContextMap::new();
        m.set_int("a", 1);
        m.clear();
        assert!(m.is_empty());
    }
}
