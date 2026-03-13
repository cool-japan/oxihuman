// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Material instance debug view — shows material instance hierarchy and parameter overrides.

/// Material instance view configuration.
#[derive(Debug, Clone)]
pub struct MaterialInstanceView {
    pub enabled: bool,
    pub total_instances: u32,
    pub unique_base_materials: u32,
    pub show_overrides: bool,
    pub highlight_overdraw_materials: bool,
}

impl MaterialInstanceView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_instances: 0,
            unique_base_materials: 0,
            show_overrides: true,
            highlight_overdraw_materials: false,
        }
    }
}

impl Default for MaterialInstanceView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new material instance view.
pub fn new_material_instance_view() -> MaterialInstanceView {
    MaterialInstanceView::new()
}

/// Enable or disable material instance debug overlay.
pub fn miv_set_enabled(v: &mut MaterialInstanceView, enabled: bool) {
    v.enabled = enabled;
}

/// Update instance and base material counts.
pub fn miv_update_counts(v: &mut MaterialInstanceView, instances: u32, base_materials: u32) {
    v.total_instances = instances;
    v.unique_base_materials = base_materials.min(instances.max(1));
}

/// Toggle override parameter display.
pub fn miv_set_show_overrides(v: &mut MaterialInstanceView, show: bool) {
    v.show_overrides = show;
}

/// Toggle overdraw-heavy material highlighting.
pub fn miv_set_highlight_overdraw(v: &mut MaterialInstanceView, highlight: bool) {
    v.highlight_overdraw_materials = highlight;
}

/// Compute average instances per base material.
pub fn miv_avg_instances_per_base(v: &MaterialInstanceView) -> f32 {
    if v.unique_base_materials == 0 {
        return 0.0;
    }
    v.total_instances as f32 / v.unique_base_materials as f32
}

/// Serialize to JSON-like string.
pub fn material_instance_view_to_json(v: &MaterialInstanceView) -> String {
    format!(
        r#"{{"enabled":{},"total_instances":{},"unique_base_materials":{},"show_overrides":{},"highlight_overdraw_materials":{}}}"#,
        v.enabled,
        v.total_instances,
        v.unique_base_materials,
        v.show_overrides,
        v.highlight_overdraw_materials
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_material_instance_view();
        assert!(!v.enabled);
        assert_eq!(v.total_instances, 0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_material_instance_view();
        miv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_counts() {
        let mut v = new_material_instance_view();
        miv_update_counts(&mut v, 100, 10);
        assert_eq!(v.total_instances, 100);
        assert_eq!(v.unique_base_materials, 10);
    }

    #[test]
    fn test_show_overrides_toggle() {
        let mut v = new_material_instance_view();
        miv_set_show_overrides(&mut v, false);
        assert!(!v.show_overrides);
    }

    #[test]
    fn test_highlight_toggle() {
        let mut v = new_material_instance_view();
        miv_set_highlight_overdraw(&mut v, true);
        assert!(v.highlight_overdraw_materials);
    }

    #[test]
    fn test_avg_instances_zero_base() {
        let v = new_material_instance_view();
        assert_eq!(miv_avg_instances_per_base(&v), 0.0);
    }

    #[test]
    fn test_avg_instances_computed() {
        let mut v = new_material_instance_view();
        miv_update_counts(&mut v, 50, 5);
        assert!((miv_avg_instances_per_base(&v) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_base_materials_capped() {
        let mut v = new_material_instance_view();
        miv_update_counts(&mut v, 5, 100);
        assert!(v.unique_base_materials <= v.total_instances.max(1));
    }

    #[test]
    fn test_json_keys() {
        let v = new_material_instance_view();
        let s = material_instance_view_to_json(&v);
        assert!(s.contains("unique_base_materials"));
    }

    #[test]
    fn test_clone() {
        let v = new_material_instance_view();
        let v2 = v.clone();
        assert_eq!(v2.total_instances, v.total_instances);
    }
}
