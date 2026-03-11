// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Copy scale constraint — copies object scale from a target on selected axes.

/// Copy scale constraint descriptor.
#[derive(Debug, Clone)]
pub struct CopyScaleConstraint {
    pub target_scale: [f32; 3],
    pub copy_x: bool,
    pub copy_y: bool,
    pub copy_z: bool,
    pub power: f32,
    pub influence: f32,
    pub label: String,
}

/// Create a new copy-scale constraint.
pub fn new_copy_scale(target_scale: [f32; 3], label: &str) -> CopyScaleConstraint {
    CopyScaleConstraint {
        target_scale,
        copy_x: true,
        copy_y: true,
        copy_z: true,
        power: 1.0,
        influence: 1.0,
        label: label.to_owned(),
    }
}

/// Apply the constraint to a source scale, returning the constrained scale.
pub fn apply_copy_scale(c: &CopyScaleConstraint, source: [f32; 3]) -> [f32; 3] {
    let blend_s = |src: f32, tgt: f32| {
        let powered = tgt.powf(c.power);
        src + (powered - src) * c.influence
    };
    [
        if c.copy_x {
            blend_s(source[0], c.target_scale[0])
        } else {
            source[0]
        },
        if c.copy_y {
            blend_s(source[1], c.target_scale[1])
        } else {
            source[1]
        },
        if c.copy_z {
            blend_s(source[2], c.target_scale[2])
        } else {
            source[2]
        },
    ]
}

/// Number of active copy axes.
pub fn active_scale_axis_count(c: &CopyScaleConstraint) -> usize {
    [c.copy_x, c.copy_y, c.copy_z]
        .iter()
        .filter(|&&b| b)
        .count()
}

/// Uniform scale helper: all three components of `target_scale` are equal.
pub fn is_uniform_target(c: &CopyScaleConstraint) -> bool {
    let [x, y, z] = c.target_scale;
    (x - y).abs() < 1e-6 && (y - z).abs() < 1e-6
}

/// Serialize to JSON-style string.
pub fn copy_scale_to_json(c: &CopyScaleConstraint) -> String {
    format!(
        r#"{{"label":"{}", "influence":{:.4}, "power":{:.4}, "copy_x":{}, "copy_y":{}, "copy_z":{}}}"#,
        c.label, c.influence, c.power, c.copy_x, c.copy_y, c.copy_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_all_axes_active() {
        /* all axes enabled by default */
        let c = new_copy_scale([1.0; 3], "sc");
        assert_eq!(active_scale_axis_count(&c), 3);
    }

    #[test]
    fn apply_full_influence_copies_target() {
        /* influence 1 copies target exactly */
        let c = new_copy_scale([2.0, 3.0, 4.0], "sc");
        let out = apply_copy_scale(&c, [1.0; 3]);
        assert!((out[0] - 2.0).abs() < 1e-5);
        assert!((out[1] - 3.0).abs() < 1e-5);
        assert!((out[2] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn apply_zero_influence_leaves_source() {
        /* influence 0 leaves source unchanged */
        let mut c = new_copy_scale([5.0; 3], "sc");
        c.influence = 0.0;
        let out = apply_copy_scale(&c, [2.0, 3.0, 4.0]);
        assert!((out[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn disabled_z_leaves_z() {
        /* copy_z false should keep source Z */
        let mut c = new_copy_scale([9.0; 3], "sc");
        c.copy_z = false;
        let out = apply_copy_scale(&c, [1.0, 1.0, 0.5]);
        assert!((out[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn active_axis_count_one_disabled() {
        /* disabling Y yields count 2 */
        let mut c = new_copy_scale([1.0; 3], "sc");
        c.copy_y = false;
        assert_eq!(active_scale_axis_count(&c), 2);
    }

    #[test]
    fn is_uniform_target_for_uniform_scale() {
        /* all-same target is uniform */
        let c = new_copy_scale([2.0, 2.0, 2.0], "sc");
        assert!(is_uniform_target(&c));
    }

    #[test]
    fn is_uniform_target_false_for_non_uniform() {
        /* non-uniform target is not uniform */
        let c = new_copy_scale([1.0, 2.0, 3.0], "sc");
        assert!(!is_uniform_target(&c));
    }

    #[test]
    fn json_contains_label() {
        /* JSON output includes label */
        let c = new_copy_scale([1.0; 3], "myScale");
        assert!(copy_scale_to_json(&c).contains("myScale"));
    }

    #[test]
    fn default_influence_is_one() {
        /* default influence is 1 */
        let c = new_copy_scale([1.0; 3], "sc");
        assert!((c.influence - 1.0).abs() < 1e-6);
    }
}
