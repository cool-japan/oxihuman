// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Corrective pose driver — triggers corrective shape keys based on joint pose.

/// Configuration for a corrective pose driver.
#[derive(Debug, Clone)]
pub struct CorrectivePoseDriverConfig {
    /// Activation angle threshold in radians.
    pub threshold_rad: f32,
    /// Maximum weight when fully activated.
    pub max_weight: f32,
}

impl Default for CorrectivePoseDriverConfig {
    fn default() -> Self {
        CorrectivePoseDriverConfig {
            threshold_rad: 0.3,
            max_weight: 1.0,
        }
    }
}

/// A single corrective pose driver binding.
#[derive(Debug, Clone)]
pub struct CorrectivePoseDriver {
    pub joint_name: String,
    pub target_shape: String,
    pub config: CorrectivePoseDriverConfig,
    pub current_weight: f32,
}

impl CorrectivePoseDriver {
    pub fn new(joint_name: &str, target_shape: &str) -> Self {
        CorrectivePoseDriver {
            joint_name: joint_name.to_string(),
            target_shape: target_shape.to_string(),
            config: CorrectivePoseDriverConfig::default(),
            current_weight: 0.0,
        }
    }
}

/// Create a new corrective pose driver.
pub fn new_corrective_pose_driver(joint_name: &str, target_shape: &str) -> CorrectivePoseDriver {
    CorrectivePoseDriver::new(joint_name, target_shape)
}

/// Evaluate the driver weight for the given joint angle.
pub fn evaluate_pose_driver(driver: &mut CorrectivePoseDriver, joint_angle_rad: f32) -> f32 {
    let t = driver.config.threshold_rad;
    let w = if joint_angle_rad >= t {
        (joint_angle_rad - t) / (std::f32::consts::PI - t)
    } else {
        0.0
    };
    driver.current_weight = w.clamp(0.0, driver.config.max_weight);
    driver.current_weight
}

/// Reset the driver weight to zero.
pub fn reset_pose_driver(driver: &mut CorrectivePoseDriver) {
    driver.current_weight = 0.0;
}

/// Return the driver weight as a JSON-like string.
pub fn pose_driver_to_json(driver: &CorrectivePoseDriver) -> String {
    format!(
        r#"{{"joint":"{}","shape":"{}","weight":{:.4}}}"#,
        driver.joint_name, driver.target_shape, driver.current_weight
    )
}

/// Set the threshold angle.
pub fn set_pose_driver_threshold(driver: &mut CorrectivePoseDriver, threshold_rad: f32) {
    driver.config.threshold_rad = threshold_rad.clamp(0.0, std::f32::consts::PI);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_driver_zero_weight() {
        let d = new_corrective_pose_driver("shoulder", "shoulder_corrective");
        assert!((d.current_weight).abs() < 1e-6, /* weight should be 0 */);
    }

    #[test]
    fn test_evaluate_below_threshold_is_zero() {
        let mut d = new_corrective_pose_driver("elbow", "elbow_bulge");
        let w = evaluate_pose_driver(&mut d, 0.1);
        assert!((w).abs() < 1e-6, /* below threshold should give 0 weight */);
    }

    #[test]
    fn test_evaluate_above_threshold_positive() {
        let mut d = new_corrective_pose_driver("elbow", "elbow_bulge");
        let w = evaluate_pose_driver(&mut d, 1.5);
        assert!(w > 0.0, /* above threshold should give positive weight */);
    }

    #[test]
    fn test_evaluate_clamps_to_max_weight() {
        let mut d = new_corrective_pose_driver("knee", "knee_corrective");
        let w = evaluate_pose_driver(&mut d, std::f32::consts::PI);
        assert!(w <= d.config.max_weight, /* weight must not exceed max */);
    }

    #[test]
    fn test_reset_clears_weight() {
        let mut d = new_corrective_pose_driver("hip", "hip_corrective");
        evaluate_pose_driver(&mut d, 2.0);
        reset_pose_driver(&mut d);
        assert!((d.current_weight).abs() < 1e-6, /* reset should zero the weight */);
    }

    #[test]
    fn test_set_threshold() {
        let mut d = new_corrective_pose_driver("wrist", "wrist_corrective");
        set_pose_driver_threshold(&mut d, 0.8);
        assert!((d.config.threshold_rad - 0.8).abs() < 1e-6, /* threshold should be updated */);
    }

    #[test]
    fn test_to_json_contains_joint_name() {
        let d = new_corrective_pose_driver("ankle", "ankle_corrective");
        let s = pose_driver_to_json(&d);
        assert!(s.contains("ankle"), /* JSON should contain joint name */);
    }

    #[test]
    fn test_threshold_clamps_to_pi() {
        let mut d = new_corrective_pose_driver("test", "test_shape");
        set_pose_driver_threshold(&mut d, 99.0);
        assert!(d.config.threshold_rad <= std::f32::consts::PI, /* threshold clamped to PI */);
    }

    #[test]
    fn test_joint_name_stored() {
        let d = new_corrective_pose_driver("shoulder_l", "shoulder_bulge");
        assert_eq!(d.joint_name, "shoulder_l" /* joint name must match */,);
    }

    #[test]
    fn test_shape_name_stored() {
        let d = new_corrective_pose_driver("hip", "hip_crease");
        assert_eq!(
            d.target_shape,
            "hip_crease", /* shape name must match */
        );
    }
}
