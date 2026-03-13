// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Joint rotation limit export for skeleton rigs.

use std::f32::consts::PI;

/// Joint limit definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointLimitExport {
    pub joint_name: String,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
}

/// Create new joint limit (all angles in radians).
#[allow(dead_code)]
pub fn new_joint_limit(name: &str) -> JointLimitExport {
    JointLimitExport {
        joint_name: name.to_string(),
        min_x: -PI,
        max_x: PI,
        min_y: -PI,
        max_y: PI,
        min_z: -PI,
        max_z: PI,
    }
}

/// Set X axis limits.
#[allow(dead_code)]
pub fn set_x_limits(j: &mut JointLimitExport, min: f32, max: f32) {
    j.min_x = min;
    j.max_x = max;
}

/// Set Y axis limits.
#[allow(dead_code)]
pub fn set_y_limits(j: &mut JointLimitExport, min: f32, max: f32) {
    j.min_y = min;
    j.max_y = max;
}

/// Set Z axis limits.
#[allow(dead_code)]
pub fn set_z_limits(j: &mut JointLimitExport, min: f32, max: f32) {
    j.min_z = min;
    j.max_z = max;
}

/// Total range of motion (sum of all axis ranges).
#[allow(dead_code)]
pub fn total_range(j: &JointLimitExport) -> f32 {
    (j.max_x - j.min_x) + (j.max_y - j.min_y) + (j.max_z - j.min_z)
}

/// Clamp angle to joint limits on X axis.
#[allow(dead_code)]
pub fn clamp_x(j: &JointLimitExport, angle: f32) -> f32 {
    angle.clamp(j.min_x, j.max_x)
}

/// Check if limits are symmetric.
#[allow(dead_code)]
pub fn is_symmetric(j: &JointLimitExport) -> bool {
    (j.min_x + j.max_x).abs() < 1e-6
        && (j.min_y + j.max_y).abs() < 1e-6
        && (j.min_z + j.max_z).abs() < 1e-6
}

/// Validate (min <= max).
#[allow(dead_code)]
pub fn jl_validate(j: &JointLimitExport) -> bool {
    j.min_x <= j.max_x && j.min_y <= j.max_y && j.min_z <= j.max_z
}

/// Export to JSON.
#[allow(dead_code)]
pub fn joint_limit_to_json(j: &JointLimitExport) -> String {
    format!(
        "{{\"joint\":\"{}\",\"total_range\":{:.6}}}",
        j.joint_name,
        total_range(j)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let j = new_joint_limit("elbow");
        assert_eq!(j.joint_name, "elbow");
    }
    #[test]
    fn test_set_x() {
        let mut j = new_joint_limit("a");
        set_x_limits(&mut j, -0.5, 0.5);
        assert!((j.min_x - (-0.5)).abs() < 1e-6);
    }
    #[test]
    fn test_set_y() {
        let mut j = new_joint_limit("a");
        set_y_limits(&mut j, -1.0, 1.0);
        assert!((j.max_y - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_set_z() {
        let mut j = new_joint_limit("a");
        set_z_limits(&mut j, 0.0, PI);
        assert!((j.max_z - PI).abs() < 1e-6);
    }
    #[test]
    fn test_total_range() {
        let mut j = new_joint_limit("a");
        set_x_limits(&mut j, 0.0, 1.0);
        set_y_limits(&mut j, 0.0, 1.0);
        set_z_limits(&mut j, 0.0, 1.0);
        assert!((total_range(&j) - 3.0).abs() < 1e-6);
    }
    #[test]
    fn test_clamp() {
        let j = new_joint_limit("a");
        assert!((clamp_x(&j, 100.0) - PI).abs() < 1e-5);
    }
    #[test]
    fn test_symmetric() {
        let j = new_joint_limit("a");
        assert!(is_symmetric(&j));
    }
    #[test]
    fn test_not_symmetric() {
        let mut j = new_joint_limit("a");
        set_x_limits(&mut j, 0.0, 1.0);
        assert!(!is_symmetric(&j));
    }
    #[test]
    fn test_validate() {
        let j = new_joint_limit("a");
        assert!(jl_validate(&j));
    }
    #[test]
    fn test_validate_bad() {
        let mut j = new_joint_limit("a");
        set_x_limits(&mut j, 1.0, -1.0);
        assert!(!jl_validate(&j));
    }
    #[test]
    fn test_to_json() {
        let j = new_joint_limit("knee");
        assert!(joint_limit_to_json(&j).contains("\"joint\":\"knee\""));
    }
}
