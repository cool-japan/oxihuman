// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Track-to constraint for mesh orientation — aligns a local axis toward a target.

/// Axis to align toward the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackAxis {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

/// Track-to constraint descriptor.
#[derive(Debug, Clone)]
pub struct TrackToConstraint {
    pub target_position: [f32; 3],
    pub track_axis: TrackAxis,
    pub up_axis: TrackAxis,
    pub influence: f32,
    pub label: String,
}

/// Create a new track-to constraint.
pub fn new_track_to(target: [f32; 3], label: &str) -> TrackToConstraint {
    TrackToConstraint {
        target_position: target,
        track_axis: TrackAxis::PosY,
        up_axis: TrackAxis::PosZ,
        influence: 1.0,
        label: label.to_owned(),
    }
}

/// Set the tracking axis.
pub fn set_track_axis(c: &mut TrackToConstraint, axis: TrackAxis) {
    c.track_axis = axis;
}

/// Set the up axis.
pub fn set_up_axis(c: &mut TrackToConstraint, axis: TrackAxis) {
    c.up_axis = axis;
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-8 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Compute the normalised direction vector from `origin` toward the target.
pub fn track_direction(c: &TrackToConstraint, origin: [f32; 3]) -> [f32; 3] {
    let d = [
        c.target_position[0] - origin[0],
        c.target_position[1] - origin[1],
        c.target_position[2] - origin[2],
    ];
    normalize3(d)
}

/// Distance from `origin` to the target.
pub fn target_distance(c: &TrackToConstraint, origin: [f32; 3]) -> f32 {
    let d = [
        c.target_position[0] - origin[0],
        c.target_position[1] - origin[1],
        c.target_position[2] - origin[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Return the axis name string for display.
pub fn axis_name(axis: TrackAxis) -> &'static str {
    match axis {
        TrackAxis::PosX => "+X",
        TrackAxis::NegX => "-X",
        TrackAxis::PosY => "+Y",
        TrackAxis::NegY => "-Y",
        TrackAxis::PosZ => "+Z",
        TrackAxis::NegZ => "-Z",
    }
}

/// Serialize the constraint to a JSON-style string.
pub fn track_to_json(c: &TrackToConstraint) -> String {
    format!(
        r#"{{"label":"{}", "track_axis":"{}", "up_axis":"{}", "influence":{:.4}}}"#,
        c.label,
        axis_name(c.track_axis),
        axis_name(c.up_axis),
        c.influence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constraint_default_track_axis_is_pos_y() {
        /* default track axis should be +Y */
        let c = new_track_to([0.0; 3], "cam");
        assert_eq!(c.track_axis, TrackAxis::PosY);
    }

    #[test]
    fn new_constraint_default_influence_one() {
        /* influence defaults to 1 */
        let c = new_track_to([0.0; 3], "cam");
        assert!((c.influence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_track_axis_changes_axis() {
        /* setting axis to +Z should update the field */
        let mut c = new_track_to([0.0; 3], "cam");
        set_track_axis(&mut c, TrackAxis::PosZ);
        assert_eq!(c.track_axis, TrackAxis::PosZ);
    }

    #[test]
    fn track_direction_normalized() {
        /* direction vector should have unit length */
        let c = new_track_to([3.0, 4.0, 0.0], "t");
        let d = track_direction(&c, [0.0, 0.0, 0.0]);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn target_distance_correct() {
        /* distance from origin to (3,4,0) is 5 */
        let c = new_track_to([3.0, 4.0, 0.0], "t");
        assert!((target_distance(&c, [0.0; 3]) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn axis_name_pos_x() {
        /* PosX maps to "+X" */
        assert_eq!(axis_name(TrackAxis::PosX), "+X");
    }

    #[test]
    fn axis_name_neg_z() {
        /* NegZ maps to "-Z" */
        assert_eq!(axis_name(TrackAxis::NegZ), "-Z");
    }

    #[test]
    fn json_contains_label() {
        /* JSON string should contain the label */
        let c = new_track_to([1.0, 0.0, 0.0], "myLabel");
        let j = track_to_json(&c);
        assert!(j.contains("myLabel"));
    }

    #[test]
    fn set_up_axis_updates_field() {
        /* up axis should update to NegY */
        let mut c = new_track_to([0.0; 3], "u");
        set_up_axis(&mut c, TrackAxis::NegY);
        assert_eq!(c.up_axis, TrackAxis::NegY);
    }
}
