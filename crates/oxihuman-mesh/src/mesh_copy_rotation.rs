// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Copy rotation constraint — replicates euler-angle rotation from a target object.

/// Euler rotation in radians (XYZ order).
#[derive(Debug, Clone, Copy, Default)]
pub struct EulerRot {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Copy rotation constraint descriptor.
#[derive(Debug, Clone)]
pub struct CopyRotationConstraint {
    pub target_rotation: EulerRot,
    pub copy_x: bool,
    pub copy_y: bool,
    pub copy_z: bool,
    pub influence: f32,
    pub label: String,
}

/// Create a new copy-rotation constraint.
pub fn new_copy_rotation(target: EulerRot, label: &str) -> CopyRotationConstraint {
    CopyRotationConstraint {
        target_rotation: target,
        copy_x: true,
        copy_y: true,
        copy_z: true,
        influence: 1.0,
        label: label.to_owned(),
    }
}

/// Apply constraint: blend source rotation toward target.
pub fn apply_copy_rotation(c: &CopyRotationConstraint, source: EulerRot) -> EulerRot {
    let blend = |src: f32, tgt: f32| src + (tgt - src) * c.influence;
    EulerRot {
        x: if c.copy_x {
            blend(source.x, c.target_rotation.x)
        } else {
            source.x
        },
        y: if c.copy_y {
            blend(source.y, c.target_rotation.y)
        } else {
            source.y
        },
        z: if c.copy_z {
            blend(source.z, c.target_rotation.z)
        } else {
            source.z
        },
    }
}

/// Number of active copy axes.
pub fn active_rot_axis_count(c: &CopyRotationConstraint) -> usize {
    [c.copy_x, c.copy_y, c.copy_z]
        .iter()
        .filter(|&&b| b)
        .count()
}

/// Convert degrees to radians helper.
pub fn deg_to_rad_cr(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

/// Serialize the constraint to a JSON-style string.
pub fn copy_rotation_to_json(c: &CopyRotationConstraint) -> String {
    format!(
        r#"{{"label":"{}", "influence":{:.4}, "copy_x":{}, "copy_y":{}, "copy_z":{}}}"#,
        c.label, c.influence, c.copy_x, c.copy_y, c.copy_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constraint_all_axes_active() {
        /* all three copy flags default to true */
        let c = new_copy_rotation(EulerRot::default(), "rot");
        assert_eq!(active_rot_axis_count(&c), 3);
    }

    #[test]
    fn apply_full_influence_copies_target() {
        /* influence 1 should copy target X exactly */
        let tgt = EulerRot {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let c = new_copy_rotation(tgt, "r");
        let out = apply_copy_rotation(&c, EulerRot::default());
        assert!((out.x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_zero_influence_leaves_source() {
        /* influence 0 should return source unchanged */
        let tgt = EulerRot {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let mut c = new_copy_rotation(tgt, "r");
        c.influence = 0.0;
        let src = EulerRot {
            x: 0.5,
            y: 0.0,
            z: 0.0,
        };
        let out = apply_copy_rotation(&c, src);
        assert!((out.x - 0.5).abs() < 1e-5);
    }

    #[test]
    fn disabled_z_leaves_z_unchanged() {
        /* copy_z=false should not change Z */
        let tgt = EulerRot {
            x: 0.0,
            y: 0.0,
            z: 2.0,
        };
        let mut c = new_copy_rotation(tgt, "r");
        c.copy_z = false;
        let src = EulerRot {
            x: 0.0,
            y: 0.0,
            z: 0.3,
        };
        let out = apply_copy_rotation(&c, src);
        assert!((out.z - 0.3).abs() < 1e-5);
    }

    #[test]
    fn active_axis_count_when_y_disabled() {
        /* disabling Y yields count 2 */
        let mut c = new_copy_rotation(EulerRot::default(), "r");
        c.copy_y = false;
        assert_eq!(active_rot_axis_count(&c), 2);
    }

    #[test]
    fn deg_to_rad_90_is_pi_over_2() {
        /* 90 degrees should equal PI/2 */
        let r = deg_to_rad_cr(90.0);
        assert!((r - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn json_contains_label() {
        /* JSON should include label field */
        let c = new_copy_rotation(EulerRot::default(), "myRot");
        assert!(copy_rotation_to_json(&c).contains("myRot"));
    }

    #[test]
    fn half_influence_interpolates_x() {
        /* influence 0.5 between 0 and 2 gives 1 */
        let tgt = EulerRot {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        };
        let mut c = new_copy_rotation(tgt, "r");
        c.influence = 0.5;
        let out = apply_copy_rotation(&c, EulerRot::default());
        assert!((out.x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn default_influence_is_one() {
        /* default influence should be exactly 1 */
        let c = new_copy_rotation(EulerRot::default(), "r");
        assert!((c.influence - 1.0).abs() < 1e-6);
    }
}
