// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Constraint arc/limit visualization view stub.

/// Constraint type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstraintType {
    RotationLimit,
    PositionLimit,
    DistanceLimit,
    LookAt,
}

/// Constraint arc view configuration.
#[derive(Debug, Clone)]
pub struct ConstraintArcView {
    pub constraint_type: ConstraintType,
    pub arc_color: [f32; 4],
    pub limit_color: [f32; 4],
    pub arc_segments: u32,
    pub show_labels: bool,
    pub enabled: bool,
}

impl ConstraintArcView {
    pub fn new() -> Self {
        ConstraintArcView {
            constraint_type: ConstraintType::RotationLimit,
            arc_color: [0.3, 0.8, 1.0, 1.0],
            limit_color: [1.0, 0.2, 0.2, 1.0],
            arc_segments: 32,
            show_labels: true,
            enabled: true,
        }
    }
}

impl Default for ConstraintArcView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new constraint arc view.
pub fn new_constraint_arc_view() -> ConstraintArcView {
    ConstraintArcView::new()
}

/// Set constraint type.
pub fn cav_set_type(view: &mut ConstraintArcView, constraint_type: ConstraintType) {
    view.constraint_type = constraint_type;
}

/// Set arc draw color.
pub fn cav_set_arc_color(view: &mut ConstraintArcView, color: [f32; 4]) {
    view.arc_color = color;
}

/// Set limit indicator color.
pub fn cav_set_limit_color(view: &mut ConstraintArcView, color: [f32; 4]) {
    view.limit_color = color;
}

/// Set arc segment count.
pub fn cav_set_arc_segments(view: &mut ConstraintArcView, segments: u32) {
    view.arc_segments = segments.max(4);
}

/// Toggle label display.
pub fn cav_show_labels(view: &mut ConstraintArcView, show: bool) {
    view.show_labels = show;
}

/// Enable or disable.
pub fn cav_set_enabled(view: &mut ConstraintArcView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn cav_to_json(view: &ConstraintArcView) -> String {
    let ct = match view.constraint_type {
        ConstraintType::RotationLimit => "rotation_limit",
        ConstraintType::PositionLimit => "position_limit",
        ConstraintType::DistanceLimit => "distance_limit",
        ConstraintType::LookAt => "look_at",
    };
    format!(
        r#"{{"constraint_type":"{}","arc_segments":{},"show_labels":{},"enabled":{}}}"#,
        ct, view.arc_segments, view.show_labels, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_type() {
        let v = new_constraint_arc_view();
        assert_eq!(
            v.constraint_type,
            ConstraintType::RotationLimit /* default must be RotationLimit */
        );
    }

    #[test]
    fn test_set_type() {
        let mut v = new_constraint_arc_view();
        cav_set_type(&mut v, ConstraintType::LookAt);
        assert_eq!(
            v.constraint_type,
            ConstraintType::LookAt /* type must be set */
        );
    }

    #[test]
    fn test_arc_segments_min() {
        let mut v = new_constraint_arc_view();
        cav_set_arc_segments(&mut v, 1);
        assert_eq!(v.arc_segments, 4 /* minimum arc segments must be 4 */);
    }

    #[test]
    fn test_set_arc_segments() {
        let mut v = new_constraint_arc_view();
        cav_set_arc_segments(&mut v, 64);
        assert_eq!(v.arc_segments, 64 /* arc segments must be set */);
    }

    #[test]
    fn test_show_labels() {
        let mut v = new_constraint_arc_view();
        cav_show_labels(&mut v, false);
        assert!(!v.show_labels /* labels must be hidden */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_constraint_arc_view();
        cav_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_constraint_type() {
        let v = new_constraint_arc_view();
        let j = cav_to_json(&v);
        assert!(j.contains("\"constraint_type\"") /* JSON must have constraint_type */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_constraint_arc_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_arc_segments() {
        let v = new_constraint_arc_view();
        assert_eq!(
            v.arc_segments,
            32 /* default arc segments must be 32 */
        );
    }
}
