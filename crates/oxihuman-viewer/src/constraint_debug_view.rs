// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Constraint debug visualization — renders joint limits and constraint forces.

/// Constraint debug view configuration.
#[derive(Debug, Clone)]
pub struct ConstraintDebugView {
    pub enabled: bool,
    pub show_limits: bool,
    pub show_forces: bool,
    pub color_satisfied: [f32; 4],
    pub color_violated: [f32; 4],
    pub force_scale: f32,
}

impl ConstraintDebugView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_limits: true,
            show_forces: true,
            color_satisfied: [0.0, 1.0, 0.0, 0.7],
            color_violated: [1.0, 0.0, 0.0, 0.9],
            force_scale: 0.01,
        }
    }
}

impl Default for ConstraintDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new constraint debug view.
pub fn new_constraint_debug_view() -> ConstraintDebugView {
    ConstraintDebugView::new()
}

/// Enable or disable constraint debug display.
pub fn cdv_set_enabled(v: &mut ConstraintDebugView, enabled: bool) {
    v.enabled = enabled;
}

/// Toggle limit arc display.
pub fn cdv_set_show_limits(v: &mut ConstraintDebugView, show: bool) {
    v.show_limits = show;
}

/// Toggle constraint force arrows.
pub fn cdv_set_show_forces(v: &mut ConstraintDebugView, show: bool) {
    v.show_forces = show;
}

/// Set force arrow scale.
pub fn cdv_set_force_scale(v: &mut ConstraintDebugView, scale: f32) {
    v.force_scale = scale.clamp(0.0001, 1.0);
}

/// Choose colour based on constraint satisfaction.
pub fn cdv_color_for_state(v: &ConstraintDebugView, satisfied: bool) -> [f32; 4] {
    if satisfied {
        v.color_satisfied
    } else {
        v.color_violated
    }
}

/// Serialize to JSON-like string.
pub fn constraint_debug_view_to_json(v: &ConstraintDebugView) -> String {
    format!(
        r#"{{"enabled":{},"show_limits":{},"show_forces":{},"force_scale":{:.6}}}"#,
        v.enabled, v.show_limits, v.show_forces, v.force_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_constraint_debug_view();
        assert!(!v.enabled);
        assert!(v.show_limits);
    }

    #[test]
    fn test_enable() {
        let mut v = new_constraint_debug_view();
        cdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_limits_toggle() {
        let mut v = new_constraint_debug_view();
        cdv_set_show_limits(&mut v, false);
        assert!(!v.show_limits);
    }

    #[test]
    fn test_forces_toggle() {
        let mut v = new_constraint_debug_view();
        cdv_set_show_forces(&mut v, false);
        assert!(!v.show_forces);
    }

    #[test]
    fn test_force_scale_clamp() {
        let mut v = new_constraint_debug_view();
        cdv_set_force_scale(&mut v, 0.0);
        assert_eq!(v.force_scale, 0.0001);
    }

    #[test]
    fn test_color_satisfied() {
        let v = new_constraint_debug_view();
        let c = cdv_color_for_state(&v, true);
        assert!((c[1] - 1.0).abs() < 1e-6); /* green channel */
    }

    #[test]
    fn test_color_violated() {
        let v = new_constraint_debug_view();
        let c = cdv_color_for_state(&v, false);
        assert!((c[0] - 1.0).abs() < 1e-6); /* red channel */
    }

    #[test]
    fn test_json_keys() {
        let v = new_constraint_debug_view();
        let s = constraint_debug_view_to_json(&v);
        assert!(s.contains("force_scale"));
    }

    #[test]
    fn test_clone() {
        let v = new_constraint_debug_view();
        let v2 = v.clone();
        assert!((v2.force_scale - v.force_scale).abs() < 1e-6);
    }
}
