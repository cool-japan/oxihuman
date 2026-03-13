// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pose constraint — restricts a morph target's activation based on body pose conditions.
//!
//! Each `PoseConstraint` holds a morph target name and a list of `PoseCondition`
//! entries that define the valid angular range for specific joints.  When all
//! conditions are satisfied the morph weight returned is 1.0; when none are
//! satisfied it is 0.0.  Partial satisfaction yields a proportional blend.

#[allow(dead_code)]
/// A single joint-angle condition.
pub struct PoseCondition {
    pub joint_name: String,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[allow(dead_code)]
/// Configuration for a pose constraint.
pub struct PoseConstraintConfig {
    /// Speed at which the constraint weight transitions (normalised per frame).
    pub blend_speed: f32,
    /// Whether all conditions must be satisfied (AND logic) or just one (OR logic).
    pub require_all: bool,
}

#[allow(dead_code)]
/// A pose constraint binding a morph target to a set of pose conditions.
pub struct PoseConstraint {
    pub morph_name: String,
    pub conditions: Vec<PoseCondition>,
    pub blend_speed: f32,
    pub require_all: bool,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[allow(dead_code)]
/// Returns a sensible default [`PoseConstraintConfig`].
pub fn default_pose_constraint_config() -> PoseConstraintConfig {
    PoseConstraintConfig {
        blend_speed: 1.0,
        require_all: true,
    }
}

#[allow(dead_code)]
/// Creates a new [`PoseConstraint`] for the given morph name using `cfg`.
pub fn new_pose_constraint(morph_name: &str, cfg: &PoseConstraintConfig) -> PoseConstraint {
    PoseConstraint {
        morph_name: morph_name.to_string(),
        conditions: Vec::new(),
        blend_speed: cfg.blend_speed,
        require_all: cfg.require_all,
    }
}

#[allow(dead_code)]
/// Appends a joint-angle condition to `constraint`.
pub fn add_pose_condition(
    constraint: &mut PoseConstraint,
    joint: &str,
    min_angle: f32,
    max_angle: f32,
) {
    constraint.conditions.push(PoseCondition {
        joint_name: joint.to_string(),
        min_angle,
        max_angle,
    });
}

/// Returns `true` if every (or any, depending on `require_all`) condition in
/// `constraint` is satisfied by the supplied `joint_angles` slice.
#[allow(dead_code)]
pub fn pose_constraint_satisfied(
    constraint: &PoseConstraint,
    joint_angles: &[(&str, f32)],
) -> bool {
    if constraint.conditions.is_empty() {
        return true;
    }
    let cond_ok = |cond: &PoseCondition| {
        joint_angles
            .iter()
            .find(|(name, _)| *name == cond.joint_name)
            .is_some_and(|(_, angle)| *angle >= cond.min_angle && *angle <= cond.max_angle)
    };

    if constraint.require_all {
        constraint.conditions.iter().all(cond_ok)
    } else {
        constraint.conditions.iter().any(cond_ok)
    }
}

/// Returns a weight in `[0.0, 1.0]` representing how well the pose conditions
/// are satisfied.  Each condition contributes equally; the aggregate is the
/// fraction of conditions that are satisfied.
#[allow(dead_code)]
pub fn pose_constraint_weight(
    constraint: &PoseConstraint,
    joint_angles: &[(&str, f32)],
) -> f32 {
    if constraint.conditions.is_empty() {
        return 1.0;
    }
    let satisfied = constraint.conditions.iter().filter(|cond| {
        joint_angles
            .iter()
            .find(|(name, _)| *name == cond.joint_name)
            .is_some_and(|(_, angle)| *angle >= cond.min_angle && *angle <= cond.max_angle)
    }).count();
    satisfied as f32 / constraint.conditions.len() as f32
}

#[allow(dead_code)]
/// Returns the morph target name this constraint controls.
pub fn pose_constraint_morph_name(constraint: &PoseConstraint) -> &str {
    &constraint.morph_name
}

#[allow(dead_code)]
/// Returns the number of pose conditions attached to `constraint`.
pub fn pose_condition_count(constraint: &PoseConstraint) -> usize {
    constraint.conditions.len()
}

#[allow(dead_code)]
/// Removes all conditions from `constraint`.
pub fn reset_pose_constraint(constraint: &mut PoseConstraint) {
    constraint.conditions.clear();
}

#[allow(dead_code)]
/// Sets the blend speed for `constraint`.
pub fn set_constraint_blend_speed(constraint: &mut PoseConstraint, speed: f32) {
    constraint.blend_speed = speed.clamp(0.0, 1.0);
}

#[allow(dead_code)]
/// Returns `true` if the constraint is considered active (weight > 0.5).
pub fn pose_constraint_is_active(
    constraint: &PoseConstraint,
    joint_angles: &[(&str, f32)],
) -> bool {
    pose_constraint_weight(constraint, joint_angles) > 0.5
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_constraint() -> PoseConstraint {
        let cfg = default_pose_constraint_config();
        let mut c = new_pose_constraint("corrective_elbow", &cfg);
        add_pose_condition(&mut c, "elbow_l", 45.0, 135.0);
        c
    }

    #[test]
    fn test_default_config() {
        let cfg = default_pose_constraint_config();
        assert_eq!(cfg.blend_speed, 1.0);
        assert!(cfg.require_all);
    }

    #[test]
    fn test_new_constraint_empty() {
        let cfg = default_pose_constraint_config();
        let c = new_pose_constraint("test_morph", &cfg);
        assert_eq!(c.morph_name, "test_morph");
        assert!(c.conditions.is_empty());
    }

    #[test]
    fn test_add_condition() {
        let mut c = make_constraint();
        assert_eq!(pose_condition_count(&c), 1);
        add_pose_condition(&mut c, "shoulder_l", 0.0, 90.0);
        assert_eq!(pose_condition_count(&c), 2);
    }

    #[test]
    fn test_satisfied_in_range() {
        let c = make_constraint();
        let angles = [("elbow_l", 90.0f32)];
        assert!(pose_constraint_satisfied(&c, &angles));
    }

    #[test]
    fn test_not_satisfied_out_of_range() {
        let c = make_constraint();
        let angles = [("elbow_l", 10.0f32)];
        assert!(!pose_constraint_satisfied(&c, &angles));
    }

    #[test]
    fn test_weight_partial() {
        let cfg = default_pose_constraint_config();
        let mut c = new_pose_constraint("m", &cfg);
        add_pose_condition(&mut c, "j1", 0.0, 90.0);
        add_pose_condition(&mut c, "j2", 0.0, 90.0);
        // Only j1 in range
        let angles = [("j1", 45.0f32), ("j2", 120.0f32)];
        let w = pose_constraint_weight(&c, &angles);
        assert!((w - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_morph_name() {
        let c = make_constraint();
        assert_eq!(pose_constraint_morph_name(&c), "corrective_elbow");
    }

    #[test]
    fn test_reset_constraint() {
        let mut c = make_constraint();
        assert!(!c.conditions.is_empty());
        reset_pose_constraint(&mut c);
        assert!(c.conditions.is_empty());
    }

    #[test]
    fn test_blend_speed_clamp() {
        let cfg = default_pose_constraint_config();
        let mut c = new_pose_constraint("m", &cfg);
        set_constraint_blend_speed(&mut c, 2.0);
        assert_eq!(c.blend_speed, 1.0);
        set_constraint_blend_speed(&mut c, -1.0);
        assert_eq!(c.blend_speed, 0.0);
    }

    #[test]
    fn test_is_active() {
        let c = make_constraint();
        let in_range = [("elbow_l", 90.0f32)];
        let out_range = [("elbow_l", 5.0f32)];
        assert!(pose_constraint_is_active(&c, &in_range));
        assert!(!pose_constraint_is_active(&c, &out_range));
    }

    #[test]
    fn test_empty_conditions_always_satisfied() {
        let cfg = default_pose_constraint_config();
        let c = new_pose_constraint("m", &cfg);
        let angles: [(&str, f32); 0] = [];
        assert!(pose_constraint_satisfied(&c, &angles));
        assert_eq!(pose_constraint_weight(&c, &angles), 1.0);
    }
}
