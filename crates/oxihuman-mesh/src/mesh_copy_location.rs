// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Copy location constraint — replicates the world-space position of a target object.

/// Axes that may be copied.
#[derive(Debug, Clone)]
pub struct CopyAxes {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl Default for CopyAxes {
    fn default() -> Self {
        CopyAxes {
            x: true,
            y: true,
            z: true,
        }
    }
}

/// Copy location constraint descriptor.
#[derive(Debug, Clone)]
pub struct CopyLocationConstraint {
    pub target_position: [f32; 3],
    pub offset: [f32; 3],
    pub axes: CopyAxes,
    pub influence: f32,
    pub label: String,
}

/// Create a new copy-location constraint.
pub fn new_copy_location(target: [f32; 3], label: &str) -> CopyLocationConstraint {
    CopyLocationConstraint {
        target_position: target,
        offset: [0.0; 3],
        axes: CopyAxes::default(),
        influence: 1.0,
        label: label.to_owned(),
    }
}

/// Apply the constraint to `source_pos`, returning the constrained position.
pub fn apply_copy_location(c: &CopyLocationConstraint, source_pos: [f32; 3]) -> [f32; 3] {
    let mut out = source_pos;
    if c.axes.x {
        out[0] = (c.target_position[0] + c.offset[0]) * c.influence
            + source_pos[0] * (1.0 - c.influence);
    }
    if c.axes.y {
        out[1] = (c.target_position[1] + c.offset[1]) * c.influence
            + source_pos[1] * (1.0 - c.influence);
    }
    if c.axes.z {
        out[2] = (c.target_position[2] + c.offset[2]) * c.influence
            + source_pos[2] * (1.0 - c.influence);
    }
    out
}

/// Set a per-axis offset.
pub fn set_offset(c: &mut CopyLocationConstraint, offset: [f32; 3]) {
    c.offset = offset;
}

/// Number of axes that are active.
pub fn active_axis_count(c: &CopyLocationConstraint) -> usize {
    [c.axes.x, c.axes.y, c.axes.z]
        .iter()
        .filter(|&&b| b)
        .count()
}

/// Serialize to JSON-style string.
pub fn copy_location_to_json(c: &CopyLocationConstraint) -> String {
    format!(
        r#"{{"label":"{}", "influence":{:.4}, "axes":{{"x":{}, "y":{}, "z":{}}}}}"#,
        c.label, c.influence, c.axes.x, c.axes.y, c.axes.z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constraint_all_axes_enabled() {
        /* all three axes should be enabled by default */
        let c = new_copy_location([0.0; 3], "loc");
        assert_eq!(active_axis_count(&c), 3);
    }

    #[test]
    fn apply_full_influence_returns_target() {
        /* with influence 1 and no offset the result equals the target */
        let c = new_copy_location([5.0, 6.0, 7.0], "loc");
        let out = apply_copy_location(&c, [0.0; 3]);
        assert!((out[0] - 5.0).abs() < 1e-5);
        assert!((out[1] - 6.0).abs() < 1e-5);
        assert!((out[2] - 7.0).abs() < 1e-5);
    }

    #[test]
    fn apply_zero_influence_returns_source() {
        /* influence 0 should leave source unchanged */
        let mut c = new_copy_location([5.0; 3], "loc");
        c.influence = 0.0;
        let out = apply_copy_location(&c, [1.0, 2.0, 3.0]);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn offset_shifts_result() {
        /* offset (1,0,0) should shift X by 1 */
        let mut c = new_copy_location([0.0; 3], "loc");
        set_offset(&mut c, [1.0, 0.0, 0.0]);
        let out = apply_copy_location(&c, [0.0; 3]);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn disabled_x_axis_leaves_x_unchanged() {
        /* disabling X should keep source X */
        let mut c = new_copy_location([9.0; 3], "loc");
        c.axes.x = false;
        let out = apply_copy_location(&c, [3.0, 0.0, 0.0]);
        assert!((out[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn active_axis_count_when_one_disabled() {
        /* with Y disabled, count should be 2 */
        let mut c = new_copy_location([0.0; 3], "loc");
        c.axes.y = false;
        assert_eq!(active_axis_count(&c), 2);
    }

    #[test]
    fn json_contains_label() {
        /* JSON should include the label string */
        let c = new_copy_location([0.0; 3], "myLoc");
        assert!(copy_location_to_json(&c).contains("myLoc"));
    }

    #[test]
    fn default_influence_is_one() {
        /* default influence should be 1 */
        let c = new_copy_location([0.0; 3], "loc");
        assert!((c.influence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn half_influence_interpolates() {
        /* influence 0.5 halfway between 0 and 10 gives 5 */
        let mut c = new_copy_location([10.0, 0.0, 0.0], "loc");
        c.influence = 0.5;
        let out = apply_copy_location(&c, [0.0; 3]);
        assert!((out[0] - 5.0).abs() < 1e-4);
    }
}
