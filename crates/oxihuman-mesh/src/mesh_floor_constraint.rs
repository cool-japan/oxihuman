// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Floor/ceiling constraint — prevents object from passing through a planar boundary.

/// Which side of the plane to clamp toward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorSide {
    Floor,
    Ceiling,
}

/// Floor/ceiling constraint descriptor.
#[derive(Debug, Clone)]
pub struct FloorConstraint {
    pub plane_height: f32,
    pub side: FloorSide,
    pub use_rotation: bool,
    pub influence: f32,
    pub label: String,
}

/// Create a floor constraint at the given Y height.
pub fn new_floor_constraint(plane_height: f32, label: &str) -> FloorConstraint {
    FloorConstraint {
        plane_height,
        side: FloorSide::Floor,
        use_rotation: false,
        influence: 1.0,
        label: label.to_owned(),
    }
}

/// Apply the floor/ceiling constraint to a 3-D position (Y axis).
pub fn apply_floor_constraint(c: &FloorConstraint, pos: [f32; 3]) -> [f32; 3] {
    let target_y = match c.side {
        FloorSide::Floor => pos[1].max(c.plane_height),
        FloorSide::Ceiling => pos[1].min(c.plane_height),
    };
    let blended_y = pos[1] + (target_y - pos[1]) * c.influence;
    [pos[0], blended_y, pos[2]]
}

/// Is the given position already on the correct side of the constraint?
pub fn is_satisfied(c: &FloorConstraint, pos: [f32; 3]) -> bool {
    match c.side {
        FloorSide::Floor => pos[1] >= c.plane_height,
        FloorSide::Ceiling => pos[1] <= c.plane_height,
    }
}

/// Distance from the position to the constraint plane (unsigned).
pub fn distance_to_plane(c: &FloorConstraint, pos: [f32; 3]) -> f32 {
    (pos[1] - c.plane_height).abs()
}

/// Serialize to JSON-style string.
pub fn floor_constraint_to_json(c: &FloorConstraint) -> String {
    let side_str = match c.side {
        FloorSide::Floor => "floor",
        FloorSide::Ceiling => "ceiling",
    };
    format!(
        r#"{{"label":"{}", "height":{:.4}, "side":"{}", "influence":{:.4}}}"#,
        c.label, c.plane_height, side_str, c.influence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_at_zero_clamps_below() {
        /* Y below 0 should be clamped to 0 */
        let c = new_floor_constraint(0.0, "ground");
        let out = apply_floor_constraint(&c, [0.0, -2.0, 0.0]);
        assert!(out[1] >= 0.0);
    }

    #[test]
    fn floor_above_plane_unchanged() {
        /* Y already above plane should remain unchanged */
        let c = new_floor_constraint(0.0, "ground");
        let out = apply_floor_constraint(&c, [0.0, 3.0, 0.0]);
        assert!((out[1] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn ceiling_clamps_above() {
        /* Y above ceiling should be clamped down */
        let mut c = new_floor_constraint(5.0, "ceiling");
        c.side = FloorSide::Ceiling;
        let out = apply_floor_constraint(&c, [0.0, 8.0, 0.0]);
        assert!(out[1] <= 5.0);
    }

    #[test]
    fn is_satisfied_floor_above_plane() {
        /* position above floor plane is satisfied */
        let c = new_floor_constraint(0.0, "g");
        assert!(is_satisfied(&c, [0.0, 1.0, 0.0]));
    }

    #[test]
    fn is_satisfied_floor_below_plane_false() {
        /* position below floor plane is not satisfied */
        let c = new_floor_constraint(0.0, "g");
        assert!(!is_satisfied(&c, [0.0, -1.0, 0.0]));
    }

    #[test]
    fn distance_to_plane_correct() {
        /* 3 units above plane at Y=2 gives distance 3 */
        let c = new_floor_constraint(2.0, "g");
        assert!((distance_to_plane(&c, [0.0, 5.0, 0.0]) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn zero_influence_does_not_move_vertex() {
        /* influence 0 should leave position unchanged */
        let mut c = new_floor_constraint(0.0, "g");
        c.influence = 0.0;
        let out = apply_floor_constraint(&c, [0.0, -5.0, 0.0]);
        assert!((out[1] - (-5.0)).abs() < 1e-5);
    }

    #[test]
    fn json_contains_label() {
        /* JSON should include the label */
        let c = new_floor_constraint(0.0, "floorLabel");
        assert!(floor_constraint_to_json(&c).contains("floorLabel"));
    }

    #[test]
    fn json_side_field_is_floor() {
        /* default side should appear as "floor" in JSON */
        let c = new_floor_constraint(0.0, "g");
        assert!(floor_constraint_to_json(&c).contains("floor"));
    }
}
