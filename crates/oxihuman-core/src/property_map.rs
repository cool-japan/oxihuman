#![allow(dead_code)]

use std::collections::HashMap;

/// A value that can be stored in a property map.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

/// A string-keyed property map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PropertyMap {
    entries: HashMap<String, PropertyValue>,
}

/// Creates a new empty property map.
#[allow(dead_code)]
pub fn new_property_map() -> PropertyMap {
    PropertyMap {
        entries: HashMap::new(),
    }
}

/// Sets a property.
#[allow(dead_code)]
pub fn set_property(map: &mut PropertyMap, key: &str, value: PropertyValue) {
    map.entries.insert(key.to_string(), value);
}

/// Gets a property by key.
#[allow(dead_code)]
pub fn get_property<'a>(map: &'a PropertyMap, key: &str) -> Option<&'a PropertyValue> {
    map.entries.get(key)
}

/// Checks if a property exists.
#[allow(dead_code)]
pub fn has_property(map: &PropertyMap, key: &str) -> bool {
    map.entries.contains_key(key)
}

/// Returns the number of properties.
#[allow(dead_code)]
pub fn property_count(map: &PropertyMap) -> usize {
    map.entries.len()
}

/// Removes a property by key.
#[allow(dead_code)]
pub fn remove_property(map: &mut PropertyMap, key: &str) -> Option<PropertyValue> {
    map.entries.remove(key)
}

/// Returns all keys.
#[allow(dead_code)]
pub fn property_keys(map: &PropertyMap) -> Vec<String> {
    let mut keys: Vec<String> = map.entries.keys().cloned().collect();
    keys.sort();
    keys
}

/// Serializes properties to a JSON-like string.
#[allow(dead_code)]
pub fn properties_to_json(map: &PropertyMap) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut keys: Vec<&String> = map.entries.keys().collect();
    keys.sort();
    for key in keys {
        let val = &map.entries[key];
        let v = match val {
            PropertyValue::Int(i) => format!("{i}"),
            PropertyValue::Float(f) => format!("{f}"),
            PropertyValue::Text(s) => format!("\"{s}\""),
            PropertyValue::Bool(b) => format!("{b}"),
        };
        parts.push(format!("\"{key}\":{v}"));
    }
    format!("{{{}}}", parts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_property_map() {
        let map = new_property_map();
        assert_eq!(property_count(&map), 0);
    }

    #[test]
    fn test_set_get() {
        let mut map = new_property_map();
        set_property(&mut map, "name", PropertyValue::Text("test".into()));
        assert_eq!(
            get_property(&map, "name"),
            Some(&PropertyValue::Text("test".into()))
        );
    }

    #[test]
    fn test_has_property() {
        let mut map = new_property_map();
        set_property(&mut map, "x", PropertyValue::Int(42));
        assert!(has_property(&map, "x"));
        assert!(!has_property(&map, "y"));
    }

    #[test]
    fn test_property_count() {
        let mut map = new_property_map();
        set_property(&mut map, "a", PropertyValue::Int(1));
        set_property(&mut map, "b", PropertyValue::Float(2.0));
        assert_eq!(property_count(&map), 2);
    }

    #[test]
    fn test_remove() {
        let mut map = new_property_map();
        set_property(&mut map, "k", PropertyValue::Bool(true));
        let removed = remove_property(&mut map, "k");
        assert_eq!(removed, Some(PropertyValue::Bool(true)));
        assert!(!has_property(&map, "k"));
    }

    #[test]
    fn test_property_keys() {
        let mut map = new_property_map();
        set_property(&mut map, "b", PropertyValue::Int(2));
        set_property(&mut map, "a", PropertyValue::Int(1));
        assert_eq!(property_keys(&map), vec!["a", "b"]);
    }

    #[test]
    fn test_properties_to_json() {
        let mut map = new_property_map();
        set_property(&mut map, "x", PropertyValue::Int(10));
        let json = properties_to_json(&map);
        assert!(json.contains("\"x\":10"));
    }

    #[test]
    fn test_overwrite() {
        let mut map = new_property_map();
        set_property(&mut map, "k", PropertyValue::Int(1));
        set_property(&mut map, "k", PropertyValue::Int(2));
        assert_eq!(get_property(&map, "k"), Some(&PropertyValue::Int(2)));
        assert_eq!(property_count(&map), 1);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut map = new_property_map();
        assert!(remove_property(&mut map, "nope").is_none());
    }

    #[test]
    fn test_multiple_types() {
        let mut map = new_property_map();
        set_property(&mut map, "i", PropertyValue::Int(1));
        set_property(&mut map, "f", PropertyValue::Float(1.5));
        set_property(&mut map, "s", PropertyValue::Text("hi".into()));
        set_property(&mut map, "b", PropertyValue::Bool(false));
        assert_eq!(property_count(&map), 4);
    }
}
