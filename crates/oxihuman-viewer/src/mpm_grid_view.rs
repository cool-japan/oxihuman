// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! MPM grid visualization — shows material-point method grid nodes and mass.

/// MPM grid view configuration.
#[derive(Debug, Clone)]
pub struct MpmGridView {
    pub enabled: bool,
    pub show_mass: bool,
    pub show_momentum: bool,
    pub node_size: f32,
    pub mass_threshold: f32,
}

impl MpmGridView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_mass: true,
            show_momentum: false,
            node_size: 3.0,
            mass_threshold: 0.001,
        }
    }
}

impl Default for MpmGridView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new MPM grid view.
pub fn new_mpm_grid_view() -> MpmGridView {
    MpmGridView::new()
}

/// Enable or disable MPM grid display.
pub fn mpm_set_enabled(v: &mut MpmGridView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle mass visualization.
pub fn mpm_set_show_mass(v: &mut MpmGridView, show: bool) {
    v.show_mass = show;
}

/// Toggle momentum arrow visualization.
pub fn mpm_set_show_momentum(v: &mut MpmGridView, show: bool) {
    v.show_momentum = show;
}

/// Set grid node glyph size.
pub fn mpm_set_node_size(v: &mut MpmGridView, size: f32) {
    v.node_size = size.clamp(1.0, 20.0);
}

/// Set mass threshold below which nodes are hidden.
pub fn mpm_set_mass_threshold(v: &mut MpmGridView, thresh: f32) {
    v.mass_threshold = thresh.max(0.0);
}

/// Serialize to JSON-like string.
pub fn mpm_grid_view_to_json(v: &MpmGridView) -> String {
    format!(
        r#"{{"enabled":{},"show_mass":{},"show_momentum":{},"node_size":{:.4},"mass_threshold":{:.6}}}"#,
        v.enabled, v.show_mass, v.show_momentum, v.node_size, v.mass_threshold
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_mpm_grid_view();
        assert!(!v.enabled);
        assert!(v.show_mass);
        assert!(!v.show_momentum);
    }

    #[test]
    fn test_enable() {
        let mut v = new_mpm_grid_view();
        mpm_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_mass_toggle() {
        let mut v = new_mpm_grid_view();
        mpm_set_show_mass(&mut v, false);
        assert!(!v.show_mass);
    }

    #[test]
    fn test_momentum_toggle() {
        let mut v = new_mpm_grid_view();
        mpm_set_show_momentum(&mut v, true);
        assert!(v.show_momentum);
    }

    #[test]
    fn test_node_size_clamp_low() {
        let mut v = new_mpm_grid_view();
        mpm_set_node_size(&mut v, 0.0);
        assert_eq!(v.node_size, 1.0);
    }

    #[test]
    fn test_node_size_set() {
        let mut v = new_mpm_grid_view();
        mpm_set_node_size(&mut v, 8.0);
        assert!((v.node_size - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_mass_threshold_set() {
        let mut v = new_mpm_grid_view();
        mpm_set_mass_threshold(&mut v, 0.01);
        assert!((v.mass_threshold - 0.01).abs() < 1e-8);
    }

    #[test]
    fn test_json_keys() {
        let v = new_mpm_grid_view();
        let s = mpm_grid_view_to_json(&v);
        assert!(s.contains("mass_threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_mpm_grid_view();
        let v2 = v.clone();
        assert!((v2.node_size - v.node_size).abs() < 1e-6);
    }
}
