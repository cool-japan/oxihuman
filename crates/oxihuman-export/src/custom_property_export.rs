// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Custom property/metadata export.

#![allow(dead_code)]

/// Typed value for a custom property.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PropType {
    /// Integer value.
    Int(i64),
    /// Floating-point value.
    Float(f64),
    /// String value.
    String(String),
    /// Boolean value.
    Bool(bool),
}

/// A named custom property.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CustomProperty {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: PropType,
}

/// Container for all custom properties.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CustomPropertyExport {
    /// All properties.
    pub props: Vec<CustomProperty>,
}

/// Creates a new empty [`CustomPropertyExport`].
#[allow(dead_code)]
pub fn new_custom_property_export() -> CustomPropertyExport {
    CustomPropertyExport { props: Vec::new() }
}

fn cp_upsert(export: &mut CustomPropertyExport, name: &str, value: PropType) {
    if let Some(p) = export.props.iter_mut().find(|p| p.name == name) {
        p.value = value;
    } else {
        export.props.push(CustomProperty { name: name.to_string(), value });
    }
}

/// Adds (or updates) an integer property.
#[allow(dead_code)]
pub fn cp_add_int(export: &mut CustomPropertyExport, name: &str, value: i64) {
    cp_upsert(export, name, PropType::Int(value));
}

/// Adds (or updates) a float property.
#[allow(dead_code)]
pub fn cp_add_float(export: &mut CustomPropertyExport, name: &str, value: f64) {
    cp_upsert(export, name, PropType::Float(value));
}

/// Adds (or updates) a string property.
#[allow(dead_code)]
pub fn cp_add_string(export: &mut CustomPropertyExport, name: &str, value: &str) {
    cp_upsert(export, name, PropType::String(value.to_string()));
}

/// Adds (or updates) a boolean property.
#[allow(dead_code)]
pub fn cp_add_bool(export: &mut CustomPropertyExport, name: &str, value: bool) {
    cp_upsert(export, name, PropType::Bool(value));
}

/// Returns a reference to the property value, if it exists.
#[allow(dead_code)]
pub fn cp_get<'a>(export: &'a CustomPropertyExport, name: &str) -> Option<&'a PropType> {
    export.props.iter().find(|p| p.name == name).map(|p| &p.value)
}

/// Removes a property by name. Returns true if found and removed.
#[allow(dead_code)]
pub fn cp_remove(export: &mut CustomPropertyExport, name: &str) -> bool {
    let before = export.props.len();
    export.props.retain(|p| p.name != name);
    export.props.len() < before
}

/// Returns the number of properties.
#[allow(dead_code)]
pub fn cp_count(export: &CustomPropertyExport) -> usize {
    export.props.len()
}

/// Serialises to a minimal JSON string.
#[allow(dead_code)]
pub fn cp_to_json(export: &CustomPropertyExport) -> String {
    let items: Vec<String> = export.props.iter().map(|p| {
        let val = match &p.value {
            PropType::Int(v) => v.to_string(),
            PropType::Float(v) => format!("{v}"),
            PropType::String(s) => format!("\"{s}\""),
            PropType::Bool(b) => b.to_string(),
        };
        format!("\"{}\":{}", p.name, val)
    }).collect();
    format!("{{{}}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_export() {
        let export = new_custom_property_export();
        assert_eq!(cp_count(&export), 0);
    }

    #[test]
    fn test_add_int() {
        let mut export = new_custom_property_export();
        cp_add_int(&mut export, "version", 1);
        assert_eq!(cp_count(&export), 1);
        assert_eq!(cp_get(&export, "version"), Some(&PropType::Int(1)));
    }

    #[test]
    fn test_add_float() {
        let mut export = new_custom_property_export();
        cp_add_float(&mut export, "scale", 1.5);
        assert_eq!(cp_get(&export, "scale"), Some(&PropType::Float(1.5)));
    }

    #[test]
    fn test_add_string() {
        let mut export = new_custom_property_export();
        cp_add_string(&mut export, "author", "test");
        assert_eq!(cp_get(&export, "author"), Some(&PropType::String("test".to_string())));
    }

    #[test]
    fn test_add_bool() {
        let mut export = new_custom_property_export();
        cp_add_bool(&mut export, "visible", true);
        assert_eq!(cp_get(&export, "visible"), Some(&PropType::Bool(true)));
    }

    #[test]
    fn test_remove() {
        let mut export = new_custom_property_export();
        cp_add_int(&mut export, "x", 0);
        assert!(cp_remove(&mut export, "x"));
        assert_eq!(cp_count(&export), 0);
    }

    #[test]
    fn test_get_missing() {
        let export = new_custom_property_export();
        assert!(cp_get(&export, "missing").is_none());
    }

    #[test]
    fn test_upsert() {
        let mut export = new_custom_property_export();
        cp_add_int(&mut export, "v", 1);
        cp_add_int(&mut export, "v", 2);
        assert_eq!(cp_count(&export), 1);
        assert_eq!(cp_get(&export, "v"), Some(&PropType::Int(2)));
    }

    #[test]
    fn test_to_json() {
        let mut export = new_custom_property_export();
        cp_add_int(&mut export, "k", 42);
        let json = cp_to_json(&export);
        assert!(json.contains("\"k\""));
    }

    #[test]
    fn test_to_json_string_value() {
        let mut export = new_custom_property_export();
        cp_add_string(&mut export, "name", "abc");
        let json = cp_to_json(&export);
        assert!(json.contains("\"abc\""));
    }
}
