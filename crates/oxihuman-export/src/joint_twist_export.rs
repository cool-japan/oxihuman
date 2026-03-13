// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A joint twist entry.
#[allow(dead_code)]
#[derive(Clone)]
pub struct JointTwistEntry {
    pub joint_name: String,
    pub twist_angle_deg: f32,
    pub axis: [f32; 3],
}

/// Export bundle for joint twists.
#[allow(dead_code)]
#[derive(Default)]
pub struct JointTwistExport {
    pub entries: Vec<JointTwistEntry>,
}

/// Create a new joint twist export.
#[allow(dead_code)]
pub fn new_joint_twist_export() -> JointTwistExport {
    JointTwistExport::default()
}

/// Add a joint twist.
#[allow(dead_code)]
pub fn add_joint_twist(export: &mut JointTwistExport, name: &str, deg: f32, axis: [f32; 3]) {
    export.entries.push(JointTwistEntry {
        joint_name: name.to_string(),
        twist_angle_deg: deg,
        axis,
    });
}

/// Count twist entries.
#[allow(dead_code)]
pub fn joint_twist_count(export: &JointTwistExport) -> usize {
    export.entries.len()
}

/// Get twist in radians.
#[allow(dead_code)]
pub fn twist_rad(entry: &JointTwistEntry) -> f32 {
    entry.twist_angle_deg * PI / 180.0
}

/// Find twist entry by joint name.
#[allow(dead_code)]
pub fn find_joint_twist<'a>(
    export: &'a JointTwistExport,
    name: &str,
) -> Option<&'a JointTwistEntry> {
    export.entries.iter().find(|e| e.joint_name == name)
}

/// Maximum twist angle in degrees.
#[allow(dead_code)]
pub fn max_twist_deg(export: &JointTwistExport) -> f32 {
    export
        .entries
        .iter()
        .map(|e| e.twist_angle_deg.abs())
        .fold(0.0_f32, f32::max)
}

/// Average twist angle (absolute) in degrees.
#[allow(dead_code)]
pub fn avg_twist_deg(export: &JointTwistExport) -> f32 {
    if export.entries.is_empty() {
        return 0.0;
    }
    export
        .entries
        .iter()
        .map(|e| e.twist_angle_deg.abs())
        .sum::<f32>()
        / export.entries.len() as f32
}

/// Validate that all axes are approximately unit length.
#[allow(dead_code)]
pub fn validate_twist_axes(export: &JointTwistExport) -> bool {
    export.entries.iter().all(|e| {
        let a = e.axis;
        let len = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        (len - 1.0).abs() < 1e-3
    })
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn joint_twist_to_json(export: &JointTwistExport) -> String {
    format!(
        r#"{{"joint_twists":{},"max_deg":{:.2}}}"#,
        export.entries.len(),
        max_twist_deg(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "spine", 15.0, [0.0, 1.0, 0.0]);
        assert_eq!(joint_twist_count(&e), 1);
    }

    #[test]
    fn twist_rad_correct() {
        let entry = JointTwistEntry {
            joint_name: "x".to_string(),
            twist_angle_deg: 180.0,
            axis: [1.0, 0.0, 0.0],
        };
        assert!((twist_rad(&entry) - PI).abs() < 1e-5);
    }

    #[test]
    fn find_twist() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "arm", 30.0, [1.0, 0.0, 0.0]);
        assert!(find_joint_twist(&e, "arm").is_some());
    }

    #[test]
    fn find_missing() {
        let e = new_joint_twist_export();
        assert!(find_joint_twist(&e, "x").is_none());
    }

    #[test]
    fn max_twist() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "a", 10.0, [1.0, 0.0, 0.0]);
        add_joint_twist(&mut e, "b", 45.0, [1.0, 0.0, 0.0]);
        assert!((max_twist_deg(&e) - 45.0).abs() < 1e-5);
    }

    #[test]
    fn avg_twist() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "a", 20.0, [1.0, 0.0, 0.0]);
        add_joint_twist(&mut e, "b", 40.0, [1.0, 0.0, 0.0]);
        assert!((avg_twist_deg(&e) - 30.0).abs() < 1e-5);
    }

    #[test]
    fn validate_axes_valid() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "x", 0.0, [1.0, 0.0, 0.0]);
        assert!(validate_twist_axes(&e));
    }

    #[test]
    fn validate_axes_invalid() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "x", 0.0, [0.0, 0.0, 0.0]);
        assert!(!validate_twist_axes(&e));
    }

    #[test]
    fn json_has_count() {
        let e = new_joint_twist_export();
        let j = joint_twist_to_json(&e);
        assert!(j.contains("\"joint_twists\":0"));
    }

    #[test]
    fn empty_avg() {
        let e = new_joint_twist_export();
        assert!((avg_twist_deg(&e) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn negative_angle() {
        let mut e = new_joint_twist_export();
        add_joint_twist(&mut e, "x", -30.0, [0.0, 1.0, 0.0]);
        assert!((max_twist_deg(&e) - 30.0).abs() < 1e-5);
    }
}
