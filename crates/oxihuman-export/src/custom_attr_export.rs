// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Custom attribute export: arbitrary per-object key-value metadata.

/// Attribute value types.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AttrValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Text(String),
    FloatArray(Vec<f32>),
}

/// A single custom attribute.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CustomAttr {
    pub key: String,
    pub value: AttrValue,
}

/// Custom attribute export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CustomAttrExport {
    pub object_name: String,
    pub attrs: Vec<CustomAttr>,
}

/// Create a new custom attribute export.
#[allow(dead_code)]
pub fn new_custom_attr_export(object_name: &str) -> CustomAttrExport {
    CustomAttrExport {
        object_name: object_name.to_string(),
        attrs: Vec::new(),
    }
}

/// Add an attribute.
#[allow(dead_code)]
pub fn add_attr(exp: &mut CustomAttrExport, key: &str, value: AttrValue) {
    exp.attrs.push(CustomAttr {
        key: key.to_string(),
        value,
    });
}

/// Attribute count.
#[allow(dead_code)]
pub fn attr_count(exp: &CustomAttrExport) -> usize {
    exp.attrs.len()
}

/// Find attribute by key.
#[allow(dead_code)]
pub fn find_attr<'a>(exp: &'a CustomAttrExport, key: &str) -> Option<&'a AttrValue> {
    exp.attrs.iter().find(|a| a.key == key).map(|a| &a.value)
}

/// Get float value by key.
#[allow(dead_code)]
pub fn get_float(exp: &CustomAttrExport, key: &str) -> Option<f32> {
    if let Some(AttrValue::Float(v)) = find_attr(exp, key) {
        Some(*v)
    } else {
        None
    }
}

/// Get int value by key.
#[allow(dead_code)]
pub fn get_int(exp: &CustomAttrExport, key: &str) -> Option<i32> {
    if let Some(AttrValue::Int(v)) = find_attr(exp, key) {
        Some(*v)
    } else {
        None
    }
}

/// Get bool value by key.
#[allow(dead_code)]
pub fn get_bool(exp: &CustomAttrExport, key: &str) -> Option<bool> {
    if let Some(AttrValue::Bool(v)) = find_attr(exp, key) {
        Some(*v)
    } else {
        None
    }
}

/// Remove an attribute by key.
#[allow(dead_code)]
pub fn remove_attr(exp: &mut CustomAttrExport, key: &str) {
    exp.attrs.retain(|a| a.key != key);
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn custom_attr_to_json(exp: &CustomAttrExport) -> String {
    format!(
        "{{\"object\":\"{}\",\"attr_count\":{}}}",
        exp.object_name,
        attr_count(exp)
    )
}

/// Validate: no empty keys.
#[allow(dead_code)]
pub fn validate_custom_attrs(exp: &CustomAttrExport) -> bool {
    exp.attrs.iter().all(|a| !a.key.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_custom_attr_export("mesh");
        assert_eq!(attr_count(&exp), 0);
    }

    #[test]
    fn add_float_attr() {
        let mut exp = new_custom_attr_export("obj");
        add_attr(&mut exp, "weight", AttrValue::Float(0.5));
        assert_eq!(attr_count(&exp), 1);
    }

    #[test]
    fn get_float_correct() {
        let mut exp = new_custom_attr_export("obj");
        add_attr(&mut exp, "mass", AttrValue::Float(1.5));
        assert!((get_float(&exp, "mass").expect("should succeed") - 1.5).abs() < 1e-5);
    }

    #[test]
    fn get_int_correct() {
        let mut exp = new_custom_attr_export("obj");
        add_attr(&mut exp, "lod", AttrValue::Int(2));
        assert_eq!(get_int(&exp, "lod"), Some(2));
    }

    #[test]
    fn get_bool_correct() {
        let mut exp = new_custom_attr_export("obj");
        add_attr(&mut exp, "visible", AttrValue::Bool(true));
        assert_eq!(get_bool(&exp, "visible"), Some(true));
    }

    #[test]
    fn find_missing_none() {
        let exp = new_custom_attr_export("obj");
        assert!(find_attr(&exp, "missing").is_none());
    }

    #[test]
    fn remove_attr_works() {
        let mut exp = new_custom_attr_export("obj");
        add_attr(&mut exp, "a", AttrValue::Int(1));
        remove_attr(&mut exp, "a");
        assert_eq!(attr_count(&exp), 0);
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_custom_attr_export("obj");
        add_attr(&mut exp, "k", AttrValue::Bool(false));
        assert!(validate_custom_attrs(&exp));
    }

    #[test]
    fn json_contains_object() {
        let exp = new_custom_attr_export("hero");
        let j = custom_attr_to_json(&exp);
        assert!(j.contains("hero"));
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
