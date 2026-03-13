// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bone constraint export for skeletal animation rigs.

use std::f32::consts::PI;

/// Type of bone constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstraintType {
    CopyLocation,
    CopyRotation,
    CopyScale,
    LookAt,
    LimitDistance,
}

/// A bone constraint definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneConstraint {
    pub name: String,
    pub bone_name: String,
    pub target_bone: String,
    pub constraint_type: ConstraintType,
    pub influence: f32,
}

/// Create a new bone constraint.
#[allow(dead_code)]
pub fn new_bone_constraint(
    name: &str,
    bone: &str,
    target: &str,
    ct: ConstraintType,
) -> BoneConstraint {
    BoneConstraint {
        name: name.to_string(),
        bone_name: bone.to_string(),
        target_bone: target.to_string(),
        constraint_type: ct,
        influence: 1.0,
    }
}

/// Set influence.
#[allow(dead_code)]
pub fn set_influence(c: &mut BoneConstraint, influence: f32) {
    c.influence = influence.clamp(0.0, 1.0);
}

/// Get constraint type name.
#[allow(dead_code)]
pub fn constraint_type_name(ct: ConstraintType) -> &'static str {
    match ct {
        ConstraintType::CopyLocation => "COPY_LOCATION",
        ConstraintType::CopyRotation => "COPY_ROTATION",
        ConstraintType::CopyScale => "COPY_SCALE",
        ConstraintType::LookAt => "LOOK_AT",
        ConstraintType::LimitDistance => "LIMIT_DISTANCE",
    }
}

/// Validate constraint.
#[allow(dead_code)]
pub fn bc_validate(c: &BoneConstraint) -> bool {
    !c.name.is_empty() && !c.bone_name.is_empty() && (0.0..=1.0).contains(&c.influence)
}

/// Export to JSON.
#[allow(dead_code)]
pub fn bone_constraint_to_json(c: &BoneConstraint) -> String {
    format!(
        "{{\"name\":\"{}\",\"bone\":\"{}\",\"target\":\"{}\",\"type\":\"{}\",\"influence\":{:.6}}}",
        c.name,
        c.bone_name,
        c.target_bone,
        constraint_type_name(c.constraint_type),
        c.influence
    )
}

/// Angle to radians helper.
#[allow(dead_code)]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

/// Radians to degrees helper.
#[allow(dead_code)]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let c = new_bone_constraint("c1", "arm", "hand", ConstraintType::CopyLocation);
        assert_eq!(c.name, "c1");
    }
    #[test]
    fn test_set_influence() {
        let mut c = new_bone_constraint("c", "a", "b", ConstraintType::CopyRotation);
        set_influence(&mut c, 0.5);
        assert!((c.influence - 0.5).abs() < 1e-6);
    }
    #[test]
    fn test_influence_clamp() {
        let mut c = new_bone_constraint("c", "a", "b", ConstraintType::CopyScale);
        set_influence(&mut c, 2.0);
        assert!((c.influence - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_type_name() {
        assert_eq!(constraint_type_name(ConstraintType::LookAt), "LOOK_AT");
    }
    #[test]
    fn test_validate() {
        let c = new_bone_constraint("c", "a", "b", ConstraintType::LimitDistance);
        assert!(bc_validate(&c));
    }
    #[test]
    fn test_validate_empty() {
        let c = BoneConstraint {
            name: String::new(),
            bone_name: "a".to_string(),
            target_bone: "b".to_string(),
            constraint_type: ConstraintType::CopyLocation,
            influence: 1.0,
        };
        assert!(!bc_validate(&c));
    }
    #[test]
    fn test_to_json() {
        let c = new_bone_constraint("c1", "arm", "hand", ConstraintType::CopyLocation);
        let j = bone_constraint_to_json(&c);
        assert!(j.contains("\"name\":\"c1\""));
    }
    #[test]
    fn test_deg_to_rad() {
        assert!((deg_to_rad(180.0) - PI).abs() < 1e-5);
    }
    #[test]
    fn test_rad_to_deg() {
        assert!((rad_to_deg(PI) - 180.0).abs() < 1e-5);
    }
    #[test]
    fn test_copy_scale_type() {
        assert_eq!(
            constraint_type_name(ConstraintType::CopyScale),
            "COPY_SCALE"
        );
    }
}
