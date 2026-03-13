// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Custom property panel view.

/// A custom property entry.
#[derive(Debug, Clone)]
pub struct CustomProperty {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub ui_visible: bool,
}

/// Custom property panel state.
#[derive(Debug, Clone)]
pub struct CustomPropertyView {
    pub properties: Vec<CustomProperty>,
    pub visible: bool,
}

impl Default for CustomPropertyView {
    fn default() -> Self {
        Self {
            properties: Vec::new(),
            visible: true,
        }
    }
}

/// Create a new CustomPropertyView.
pub fn new_custom_property_view() -> CustomPropertyView {
    CustomPropertyView::default()
}

/// Add a custom property.
pub fn custom_prop_add(view: &mut CustomPropertyView, name: &str, value: f32, min: f32, max: f32) {
    view.properties.push(CustomProperty {
        name: name.to_string(),
        value: value.clamp(min, max),
        min,
        max,
        ui_visible: true,
    });
}

/// Set value of a named property.
pub fn custom_prop_set(view: &mut CustomPropertyView, name: &str, value: f32) -> bool {
    for p in &mut view.properties {
        if p.name == name {
            p.value = value.clamp(p.min, p.max);
            return true;
        }
    }
    false
}

/// Get value of a named property.
pub fn custom_prop_get(view: &CustomPropertyView, name: &str) -> Option<f32> {
    view.properties
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.value)
}

/// Remove a named property.
pub fn custom_prop_remove(view: &mut CustomPropertyView, name: &str) {
    view.properties.retain(|p| p.name != name);
}

/// Serialize to JSON.
pub fn custom_property_to_json(view: &CustomPropertyView) -> String {
    format!(
        r#"{{"visible":{},"count":{}}}"#,
        view.visible,
        view.properties.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_custom_property_view();
        assert!(v.properties.is_empty() /* empty */);
    }

    #[test]
    fn test_add() {
        let mut v = new_custom_property_view();
        custom_prop_add(&mut v, "foo", 0.5, 0.0, 1.0);
        assert_eq!(v.properties.len(), 1 /* one prop */);
    }

    #[test]
    fn test_add_clamps_value() {
        let mut v = new_custom_property_view();
        custom_prop_add(&mut v, "bar", 5.0, 0.0, 1.0);
        assert!((v.properties[0].value - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_set_value() {
        let mut v = new_custom_property_view();
        custom_prop_add(&mut v, "x", 0.0, 0.0, 10.0);
        let ok = custom_prop_set(&mut v, "x", 5.0);
        assert!(ok /* set ok */);
        assert!((v.properties[0].value - 5.0).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_set_missing() {
        let mut v = new_custom_property_view();
        let ok = custom_prop_set(&mut v, "missing", 1.0);
        assert!(!ok /* not found */);
    }

    #[test]
    fn test_get_value() {
        let mut v = new_custom_property_view();
        custom_prop_add(&mut v, "y", 3.0, 0.0, 10.0);
        assert!((custom_prop_get(&v, "y").expect("should succeed") - 3.0).abs() < 1e-6 /* retrieved */);
    }

    #[test]
    fn test_remove() {
        let mut v = new_custom_property_view();
        custom_prop_add(&mut v, "z", 1.0, 0.0, 10.0);
        custom_prop_remove(&mut v, "z");
        assert!(v.properties.is_empty() /* removed */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_custom_property_view();
        let j = custom_property_to_json(&v);
        assert!(j.contains("count") /* key */);
    }

    #[test]
    fn test_default_visible() {
        let v = CustomPropertyView::default();
        assert!(v.visible /* visible */);
    }
}
