// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A map from string field names to typed field values, for lightweight struct-like storage.

use std::collections::HashMap;

/// A typed field value in a [`StructMap`].
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FieldVal {
    Bool(bool),
    Int(i64),
    Float(f32),
    Str(String),
    Bytes(Vec<u8>),
}

/// A map of named fields with typed values.
#[allow(dead_code)]
pub struct StructMap {
    fields: HashMap<String, FieldVal>,
    schema: Vec<String>,
}

#[allow(dead_code)]
impl StructMap {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            schema: Vec::new(),
        }
    }

    /// Declare a field name (optional schema enforcement).
    pub fn declare(&mut self, name: &str) {
        if !self.schema.contains(&name.to_string()) {
            self.schema.push(name.to_string());
        }
    }

    pub fn set(&mut self, name: &str, val: FieldVal) {
        self.fields.insert(name.to_string(), val);
    }

    pub fn get(&self, name: &str) -> Option<&FieldVal> {
        self.fields.get(name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.fields.remove(name).is_some()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn field_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.fields.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.fields.get(name)? {
            FieldVal::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.fields.get(name)? {
            FieldVal::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_float(&self, name: &str) -> Option<f32> {
        match self.fields.get(name)? {
            FieldVal::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn get_str(&self, name: &str) -> Option<&str> {
        match self.fields.get(name)? {
            FieldVal::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn missing_declared(&self) -> Vec<String> {
        self.schema
            .iter()
            .filter(|n| !self.fields.contains_key(*n))
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.fields.clear();
    }
}

impl Default for StructMap {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_struct_map() -> StructMap {
    StructMap::new()
}

pub fn stm_set(m: &mut StructMap, name: &str, val: FieldVal) {
    m.set(name, val);
}

pub fn stm_get(m: &StructMap, name: &str) -> Option<FieldVal> {
    m.get(name).cloned()
}

pub fn stm_contains(m: &StructMap, name: &str) -> bool {
    m.contains(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_on_creation() {
        let m = new_struct_map();
        assert!(m.is_empty());
    }

    #[test]
    fn set_and_get_bool() {
        let mut m = new_struct_map();
        stm_set(&mut m, "active", FieldVal::Bool(true));
        assert_eq!(m.get_bool("active"), Some(true));
    }

    #[test]
    fn set_and_get_int() {
        let mut m = new_struct_map();
        stm_set(&mut m, "age", FieldVal::Int(30));
        assert_eq!(m.get_int("age"), Some(30));
    }

    #[test]
    fn set_and_get_float() {
        let mut m = new_struct_map();
        stm_set(&mut m, "x", FieldVal::Float(std::f32::consts::PI));
        assert!((m.get_float("x").unwrap() - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn set_and_get_str() {
        let mut m = new_struct_map();
        stm_set(&mut m, "name", FieldVal::Str("Alice".to_string()));
        assert_eq!(m.get_str("name"), Some("Alice"));
    }

    #[test]
    fn remove_field() {
        let mut m = new_struct_map();
        stm_set(&mut m, "tmp", FieldVal::Bool(false));
        assert!(m.remove("tmp"));
        assert!(!stm_contains(&m, "tmp"));
    }

    #[test]
    fn field_names_sorted() {
        let mut m = new_struct_map();
        stm_set(&mut m, "z", FieldVal::Int(1));
        stm_set(&mut m, "a", FieldVal::Int(2));
        assert_eq!(m.field_names(), vec!["a".to_string(), "z".to_string()]);
    }

    #[test]
    fn missing_declared_fields() {
        let mut m = new_struct_map();
        m.declare("x");
        m.declare("y");
        stm_set(&mut m, "x", FieldVal::Int(0));
        assert_eq!(m.missing_declared(), vec!["y".to_string()]);
    }

    #[test]
    fn clear_resets() {
        let mut m = new_struct_map();
        stm_set(&mut m, "k", FieldVal::Bool(true));
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn wrong_type_returns_none() {
        let mut m = new_struct_map();
        stm_set(&mut m, "x", FieldVal::Int(5));
        assert_eq!(m.get_bool("x"), None);
    }
}
