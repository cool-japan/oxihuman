// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Polymorphic value map: string key -> typed value variant.

use std::collections::HashMap;

/// Supported value types.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MapVal {
    Float(f32),
    Int(i64),
    Bool(bool),
    Text(String),
    Bytes(Vec<u8>),
}

impl MapVal {
    /// Return value type name.
    #[allow(dead_code)]
    pub fn type_name(&self) -> &'static str {
        match self {
            MapVal::Float(_) => "float",
            MapVal::Int(_) => "int",
            MapVal::Bool(_) => "bool",
            MapVal::Text(_) => "text",
            MapVal::Bytes(_) => "bytes",
        }
    }
}

/// Polymorphic value map.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ValueMap {
    inner: HashMap<String, MapVal>,
}

/// Create a new `ValueMap`.
#[allow(dead_code)]
pub fn new_value_map() -> ValueMap {
    ValueMap::default()
}

/// Set a float value.
#[allow(dead_code)]
pub fn vm_set_float(vm: &mut ValueMap, key: &str, v: f32) {
    vm.inner.insert(key.to_string(), MapVal::Float(v));
}

/// Set an int value.
#[allow(dead_code)]
pub fn vm_set_int(vm: &mut ValueMap, key: &str, v: i64) {
    vm.inner.insert(key.to_string(), MapVal::Int(v));
}

/// Set a bool value.
#[allow(dead_code)]
pub fn vm_set_bool(vm: &mut ValueMap, key: &str, v: bool) {
    vm.inner.insert(key.to_string(), MapVal::Bool(v));
}

/// Set a text value.
#[allow(dead_code)]
pub fn vm_set_text(vm: &mut ValueMap, key: &str, v: &str) {
    vm.inner
        .insert(key.to_string(), MapVal::Text(v.to_string()));
}

/// Get value by key.
#[allow(dead_code)]
pub fn vm_get<'a>(vm: &'a ValueMap, key: &str) -> Option<&'a MapVal> {
    vm.inner.get(key)
}

/// Get float value.
#[allow(dead_code)]
pub fn vm_get_float(vm: &ValueMap, key: &str) -> Option<f32> {
    vm.inner.get(key).and_then(|v| {
        if let MapVal::Float(f) = v {
            Some(*f)
        } else {
            None
        }
    })
}

/// Get int value.
#[allow(dead_code)]
pub fn vm_get_int(vm: &ValueMap, key: &str) -> Option<i64> {
    vm.inner.get(key).and_then(|v| {
        if let MapVal::Int(i) = v {
            Some(*i)
        } else {
            None
        }
    })
}

/// Get bool value.
#[allow(dead_code)]
pub fn vm_get_bool(vm: &ValueMap, key: &str) -> Option<bool> {
    vm.inner.get(key).and_then(|v| {
        if let MapVal::Bool(b) = v {
            Some(*b)
        } else {
            None
        }
    })
}

/// Remove a key.
#[allow(dead_code)]
pub fn vm_remove(vm: &mut ValueMap, key: &str) -> bool {
    vm.inner.remove(key).is_some()
}

/// Whether a key exists.
#[allow(dead_code)]
pub fn vm_contains(vm: &ValueMap, key: &str) -> bool {
    vm.inner.contains_key(key)
}

/// Number of entries.
#[allow(dead_code)]
pub fn vm_len(vm: &ValueMap) -> usize {
    vm.inner.len()
}

/// Clear all entries.
#[allow(dead_code)]
pub fn vm_clear(vm: &mut ValueMap) {
    vm.inner.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_float() {
        let mut vm = new_value_map();
        vm_set_float(&mut vm, "speed", 3.0);
        assert!((vm_get_float(&vm, "speed").expect("should succeed") - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_and_get_int() {
        let mut vm = new_value_map();
        vm_set_int(&mut vm, "count", 42);
        assert_eq!(vm_get_int(&vm, "count"), Some(42));
    }

    #[test]
    fn test_set_and_get_bool() {
        let mut vm = new_value_map();
        vm_set_bool(&mut vm, "active", true);
        assert_eq!(vm_get_bool(&vm, "active"), Some(true));
    }

    #[test]
    fn test_type_mismatch_returns_none() {
        let mut vm = new_value_map();
        vm_set_float(&mut vm, "x", 1.0);
        assert!(vm_get_int(&vm, "x").is_none());
    }

    #[test]
    fn test_remove() {
        let mut vm = new_value_map();
        vm_set_bool(&mut vm, "flag", false);
        assert!(vm_remove(&mut vm, "flag"));
        assert!(!vm_contains(&vm, "flag"));
    }

    #[test]
    fn test_len() {
        let mut vm = new_value_map();
        vm_set_float(&mut vm, "a", 1.0);
        vm_set_int(&mut vm, "b", 2);
        assert_eq!(vm_len(&vm), 2);
    }

    #[test]
    fn test_clear() {
        let mut vm = new_value_map();
        vm_set_bool(&mut vm, "x", true);
        vm_clear(&mut vm);
        assert_eq!(vm_len(&vm), 0);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(MapVal::Float(1.0).type_name(), "float");
        assert_eq!(MapVal::Bool(true).type_name(), "bool");
    }

    #[test]
    fn test_text_value() {
        let mut vm = new_value_map();
        vm_set_text(&mut vm, "name", "Alice");
        let val = vm_get(&vm, "name").expect("should succeed");
        assert_eq!(*val, MapVal::Text("Alice".to_string()));
    }
}
