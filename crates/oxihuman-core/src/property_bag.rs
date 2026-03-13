// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dynamic property bag: key → typed value.

#![allow(dead_code)]

use std::collections::HashMap;

/// A typed property value.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

/// A bag of named properties.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PropertyBag {
    pub props: HashMap<String, PropValue>,
}

/// Create a new empty property bag.
#[allow(dead_code)]
pub fn new_property_bag() -> PropertyBag {
    PropertyBag::default()
}

/// Set an integer property.
#[allow(dead_code)]
pub fn set_int(bag: &mut PropertyBag, key: &str, val: i64) {
    bag.props.insert(key.to_string(), PropValue::Int(val));
}

/// Set a float property.
#[allow(dead_code)]
pub fn set_float(bag: &mut PropertyBag, key: &str, val: f64) {
    bag.props.insert(key.to_string(), PropValue::Float(val));
}

/// Set a boolean property.
#[allow(dead_code)]
pub fn set_bool(bag: &mut PropertyBag, key: &str, val: bool) {
    bag.props.insert(key.to_string(), PropValue::Bool(val));
}

/// Set a string property.
#[allow(dead_code)]
pub fn set_str(bag: &mut PropertyBag, key: &str, val: &str) {
    bag.props
        .insert(key.to_string(), PropValue::Str(val.to_string()));
}

/// Get an integer property value.
#[allow(dead_code)]
pub fn get_int(bag: &PropertyBag, key: &str) -> Option<i64> {
    match bag.props.get(key) {
        Some(PropValue::Int(v)) => Some(*v),
        _ => None,
    }
}

/// Get a float property value.
#[allow(dead_code)]
pub fn get_float(bag: &PropertyBag, key: &str) -> Option<f64> {
    match bag.props.get(key) {
        Some(PropValue::Float(v)) => Some(*v),
        _ => None,
    }
}

/// Return the total number of stored properties.
#[allow(dead_code)]
pub fn prop_count(bag: &PropertyBag) -> usize {
    bag.props.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bag_empty() {
        let bag = new_property_bag();
        assert_eq!(prop_count(&bag), 0);
    }

    #[test]
    fn test_set_get_int() {
        let mut bag = new_property_bag();
        set_int(&mut bag, "count", 42);
        assert_eq!(get_int(&bag, "count"), Some(42));
    }

    #[test]
    fn test_set_get_float() {
        let mut bag = new_property_bag();
        let expected = std::f64::consts::PI;
        set_float(&mut bag, "ratio", expected);
        let v = get_float(&bag, "ratio").expect("should succeed");
        assert!((v - expected).abs() < 1e-9);
    }

    #[test]
    fn test_set_bool() {
        let mut bag = new_property_bag();
        set_bool(&mut bag, "active", true);
        assert_eq!(bag.props.get("active"), Some(&PropValue::Bool(true)));
    }

    #[test]
    fn test_set_str() {
        let mut bag = new_property_bag();
        set_str(&mut bag, "name", "oxihuman");
        assert_eq!(
            bag.props.get("name"),
            Some(&PropValue::Str("oxihuman".to_string()))
        );
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let bag = new_property_bag();
        assert_eq!(get_int(&bag, "missing"), None);
        assert_eq!(get_float(&bag, "missing"), None);
    }

    #[test]
    fn test_overwrite_value() {
        let mut bag = new_property_bag();
        set_int(&mut bag, "x", 1);
        set_int(&mut bag, "x", 99);
        assert_eq!(get_int(&bag, "x"), Some(99));
    }

    #[test]
    fn test_prop_count_increments() {
        let mut bag = new_property_bag();
        set_int(&mut bag, "a", 1);
        set_float(&mut bag, "b", 2.0);
        assert_eq!(prop_count(&bag), 2);
    }

    #[test]
    fn test_get_int_wrong_type_returns_none() {
        let mut bag = new_property_bag();
        set_str(&mut bag, "key", "text");
        assert_eq!(get_int(&bag, "key"), None);
    }
}
