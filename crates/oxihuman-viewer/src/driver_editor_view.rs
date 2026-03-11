// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Driver expression editor view.

/// A driver definition.
#[derive(Debug, Clone)]
pub struct DriverDef {
    pub id: u32,
    pub target_prop: String,
    pub expression: String,
    pub enabled: bool,
}

/// State for the driver editor view.
#[derive(Debug, Clone)]
pub struct DriverEditorView {
    pub drivers: Vec<DriverDef>,
    pub selected_id: Option<u32>,
    pub show_variables: bool,
    pub enabled: bool,
}

/// Create a new driver editor view.
pub fn new_driver_editor_view() -> DriverEditorView {
    DriverEditorView {
        drivers: Vec::new(),
        selected_id: None,
        show_variables: true,
        enabled: true,
    }
}

/// Add a driver.
pub fn dev_add_driver(v: &mut DriverEditorView, id: u32, target: &str, expr: &str) {
    v.drivers.push(DriverDef {
        id,
        target_prop: target.to_string(),
        expression: expr.to_string(),
        enabled: true,
    });
}

/// Select a driver by ID.
pub fn dev_select(v: &mut DriverEditorView, id: u32) {
    v.selected_id = Some(id);
}

/// Update expression of the selected driver.
pub fn dev_set_expression(v: &mut DriverEditorView, id: u32, expr: &str) {
    if let Some(d) = v.drivers.iter_mut().find(|d| d.id == id) {
        d.expression = expr.to_string();
    }
}

/// Count active (enabled) drivers.
pub fn dev_active_count(v: &DriverEditorView) -> usize {
    v.drivers.iter().filter(|d| d.enabled).count()
}

/// Serialise to JSON.
pub fn dev_to_json(v: &DriverEditorView) -> String {
    format!(
        r#"{{"driver_count":{},"active":{},"enabled":{}}}"#,
        v.drivers.len(),
        dev_active_count(v),
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_driver_editor_view();
        assert!(v.drivers.is_empty() /* no drivers */);
    }

    #[test]
    fn add_driver() {
        let mut v = new_driver_editor_view();
        dev_add_driver(&mut v, 1, "scale.x", "var * 2");
        assert_eq!(v.drivers.len(), 1 /* one driver */);
    }

    #[test]
    fn select_driver() {
        let mut v = new_driver_editor_view();
        dev_select(&mut v, 5);
        assert_eq!(v.selected_id, Some(5) /* selected */);
    }

    #[test]
    fn set_expression() {
        let mut v = new_driver_editor_view();
        dev_add_driver(&mut v, 1, "prop", "0");
        dev_set_expression(&mut v, 1, "sin(t)");
        assert_eq!(
            v.drivers[0].expression,
            "sin(t)" /* expression updated */
        );
    }

    #[test]
    fn active_count() {
        let mut v = new_driver_editor_view();
        dev_add_driver(&mut v, 1, "a", "1");
        dev_add_driver(&mut v, 2, "b", "2");
        v.drivers[0].enabled = false;
        assert_eq!(dev_active_count(&v), 1 /* one active */);
    }

    #[test]
    fn json_has_driver_count() {
        let v = new_driver_editor_view();
        assert!(dev_to_json(&v).contains("driver_count") /* json field */);
    }

    #[test]
    fn show_variables_default() {
        let v = new_driver_editor_view();
        assert!(v.show_variables /* show variables by default */);
    }

    #[test]
    fn enabled_default() {
        let v = new_driver_editor_view();
        assert!(v.enabled /* enabled */);
    }
}
