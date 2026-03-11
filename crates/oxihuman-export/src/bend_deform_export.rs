// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bend deformer export.

use std::f32::consts::PI;

/// Bend deform export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BendDeformExport {
    pub axis: [f32; 3],
    pub angle_rad: f32,
    pub origin: [f32; 3],
    pub upper_limit: f32,
    pub lower_limit: f32,
}

/// Create a default bend deform export.
#[allow(dead_code)]
pub fn default_bend_deform() -> BendDeformExport {
    BendDeformExport {
        axis: [0.0, 1.0, 0.0],
        angle_rad: 0.0,
        origin: [0.0; 3],
        upper_limit: 1.0,
        lower_limit: 0.0,
    }
}

/// Set the bend angle in degrees.
#[allow(dead_code)]
pub fn set_bend_angle_deg(e: &mut BendDeformExport, degrees: f32) {
    e.angle_rad = degrees * PI / 180.0;
}

/// Get bend angle in degrees.
#[allow(dead_code)]
pub fn bend_angle_deg(e: &BendDeformExport) -> f32 {
    e.angle_rad * 180.0 / PI
}

/// Set bend axis.
#[allow(dead_code)]
pub fn set_bend_axis(e: &mut BendDeformExport, axis: [f32; 3]) {
    e.axis = axis;
}

/// Set bend limits.
#[allow(dead_code)]
pub fn set_bend_limits(e: &mut BendDeformExport, lower: f32, upper: f32) {
    e.lower_limit = lower;
    e.upper_limit = upper;
}

/// Validate limits.
#[allow(dead_code)]
pub fn bend_validate(e: &BendDeformExport) -> bool {
    e.lower_limit <= e.upper_limit
}

/// Export to JSON.
#[allow(dead_code)]
pub fn bend_deform_to_json(e: &BendDeformExport) -> String {
    format!(
        "{{\"axis\":[{:.4},{:.4},{:.4}],\"angle_rad\":{:.6},\"origin\":[{:.4},{:.4},{:.4}]}}",
        e.axis[0], e.axis[1], e.axis[2], e.angle_rad, e.origin[0], e.origin[1], e.origin[2],
    )
}

/// Axis length (should be ~1 for normalized axis).
#[allow(dead_code)]
pub fn bend_axis_length(e: &BendDeformExport) -> f32 {
    (e.axis[0] * e.axis[0] + e.axis[1] * e.axis[1] + e.axis[2] * e.axis[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let e = default_bend_deform();
        assert!((e.angle_rad).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_deg() {
        let mut e = default_bend_deform();
        set_bend_angle_deg(&mut e, 90.0);
        assert!((e.angle_rad - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_get_angle_deg() {
        let mut e = default_bend_deform();
        e.angle_rad = PI;
        assert!((bend_angle_deg(&e) - 180.0).abs() < 1e-3);
    }

    #[test]
    fn test_set_axis() {
        let mut e = default_bend_deform();
        set_bend_axis(&mut e, [1.0, 0.0, 0.0]);
        assert!((e.axis[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_limits() {
        let mut e = default_bend_deform();
        set_bend_limits(&mut e, 0.2, 0.8);
        assert!((e.lower_limit - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_validate_ok() {
        let e = default_bend_deform();
        assert!(bend_validate(&e));
    }

    #[test]
    fn test_validate_bad() {
        let e = BendDeformExport {
            axis: [0.0; 3],
            angle_rad: 0.0,
            origin: [0.0; 3],
            upper_limit: 0.0,
            lower_limit: 1.0,
        };
        assert!(!bend_validate(&e));
    }

    #[test]
    fn test_to_json() {
        let e = default_bend_deform();
        assert!(bend_deform_to_json(&e).contains("\"angle_rad\""));
    }

    #[test]
    fn test_axis_length() {
        let e = default_bend_deform();
        assert!((bend_axis_length(&e) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        let e = default_bend_deform();
        let e2 = e.clone();
        assert!((e2.angle_rad - e.angle_rad).abs() < 1e-6);
    }
}
