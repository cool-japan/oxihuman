// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Property panel view.

/// A property entry in the panel.
#[derive(Debug, Clone)]
pub struct PropertyEntry {
    pub name: String,
    pub value: String,
    pub read_only: bool,
}

/// State for the property panel view.
#[derive(Debug, Clone)]
pub struct PropertyPanelView {
    pub properties: Vec<PropertyEntry>,
    pub search_text: String,
    pub pinned: bool,
    pub enabled: bool,
}

/// Create a new property panel view.
pub fn new_property_panel_view() -> PropertyPanelView {
    PropertyPanelView {
        properties: Vec::new(),
        search_text: String::new(),
        pinned: false,
        enabled: true,
    }
}

/// Add a property.
pub fn ppv_add_property(v: &mut PropertyPanelView, name: &str, value: &str, read_only: bool) {
    v.properties.push(PropertyEntry {
        name: name.to_string(),
        value: value.to_string(),
        read_only,
    });
}

/// Set a property value by name (if not read-only).
pub fn ppv_set_value(v: &mut PropertyPanelView, name: &str, value: &str) -> bool {
    if let Some(p) = v.properties.iter_mut().find(|p| p.name == name) {
        if !p.read_only {
            p.value = value.to_string();
            return true;
        }
    }
    false
}

/// Set search filter.
pub fn ppv_set_search(v: &mut PropertyPanelView, text: &str) {
    v.search_text = text.to_string();
}

/// Count visible (matching search) properties.
pub fn ppv_visible_count(v: &PropertyPanelView) -> usize {
    if v.search_text.is_empty() {
        return v.properties.len();
    }
    v.properties
        .iter()
        .filter(|p| p.name.contains(v.search_text.as_str()))
        .count()
}

/// Serialise to JSON.
pub fn ppv_to_json(v: &PropertyPanelView) -> String {
    format!(
        r#"{{"prop_count":{},"pinned":{},"enabled":{}}}"#,
        v.properties.len(),
        v.pinned,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_property_panel_view();
        assert!(v.properties.is_empty() /* no properties */);
    }

    #[test]
    fn add_property() {
        let mut v = new_property_panel_view();
        ppv_add_property(&mut v, "scale", "1.0", false);
        assert_eq!(v.properties.len(), 1 /* one property */);
    }

    #[test]
    fn set_value_writable() {
        let mut v = new_property_panel_view();
        ppv_add_property(&mut v, "pos", "0", false);
        assert!(ppv_set_value(&mut v, "pos", "5") /* value set */);
        assert_eq!(v.properties[0].value, "5" /* updated */);
    }

    #[test]
    fn set_value_readonly_fails() {
        let mut v = new_property_panel_view();
        ppv_add_property(&mut v, "name", "Obj", true);
        assert!(!ppv_set_value(&mut v, "name", "New") /* readonly rejected */);
    }

    #[test]
    fn visible_count_unfiltered() {
        let mut v = new_property_panel_view();
        ppv_add_property(&mut v, "a", "1", false);
        ppv_add_property(&mut v, "b", "2", false);
        assert_eq!(ppv_visible_count(&v), 2 /* all visible */);
    }

    #[test]
    fn visible_count_filtered() {
        let mut v = new_property_panel_view();
        ppv_add_property(&mut v, "scale_x", "1", false);
        ppv_add_property(&mut v, "pos", "0", false);
        ppv_set_search(&mut v, "scale");
        assert_eq!(ppv_visible_count(&v), 1 /* one match */);
    }

    #[test]
    fn json_has_prop_count() {
        let v = new_property_panel_view();
        assert!(ppv_to_json(&v).contains("prop_count") /* json field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_property_panel_view();
        assert!(v.enabled /* enabled */);
    }
}
