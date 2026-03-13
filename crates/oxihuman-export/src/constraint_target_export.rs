// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Constraint target export for bone/object constraints.

/// Constraint target data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintTargetExport {
    pub name: String,
    pub target_bone: String,
    pub influence: f32,
    pub offset: [f32; 3],
}

#[allow(dead_code)]
pub fn new_constraint_target(name: &str, target: &str) -> ConstraintTargetExport {
    ConstraintTargetExport {
        name: name.to_string(),
        target_bone: target.to_string(),
        influence: 1.0,
        offset: [0.0; 3],
    }
}

#[allow(dead_code)]
pub fn ct_set_influence(ct: &mut ConstraintTargetExport, v: f32) {
    ct.influence = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ct_set_offset(ct: &mut ConstraintTargetExport, o: [f32; 3]) {
    ct.offset = o;
}

#[allow(dead_code)]
pub fn ct_influence(ct: &ConstraintTargetExport) -> f32 {
    ct.influence
}

#[allow(dead_code)]
pub fn ct_target_name(ct: &ConstraintTargetExport) -> &str {
    &ct.target_bone
}

#[allow(dead_code)]
pub fn ct_offset_magnitude(ct: &ConstraintTargetExport) -> f32 {
    (ct.offset[0] * ct.offset[0] + ct.offset[1] * ct.offset[1] + ct.offset[2] * ct.offset[2]).sqrt()
}

#[allow(dead_code)]
pub fn ct_validate(ct: &ConstraintTargetExport) -> bool {
    (0.0..=1.0).contains(&ct.influence) && !ct.target_bone.is_empty()
}

#[allow(dead_code)]
pub fn constraint_target_to_json(ct: &ConstraintTargetExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"target\":\"{}\",\"influence\":{:.6}}}",
        ct.name, ct.target_bone, ct.influence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ct = new_constraint_target("ik_hand", "hand_bone");
        assert_eq!(ct_target_name(&ct), "hand_bone");
    }

    #[test]
    fn test_default_influence() {
        let ct = new_constraint_target("a", "b");
        assert!((ct_influence(&ct) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_influence() {
        let mut ct = new_constraint_target("a", "b");
        ct_set_influence(&mut ct, 0.5);
        assert!((ct_influence(&ct) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_influence() {
        let mut ct = new_constraint_target("a", "b");
        ct_set_influence(&mut ct, 2.0);
        assert!((ct_influence(&ct) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_offset() {
        let mut ct = new_constraint_target("a", "b");
        ct_set_offset(&mut ct, [1.0, 0.0, 0.0]);
        assert!((ct_offset_magnitude(&ct) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_offset() {
        let ct = new_constraint_target("a", "b");
        assert!((ct_offset_magnitude(&ct)).abs() < 1e-6);
    }

    #[test]
    fn test_validate_ok() {
        let ct = new_constraint_target("a", "b");
        assert!(ct_validate(&ct));
    }

    #[test]
    fn test_validate_empty_target() {
        let ct = new_constraint_target("a", "");
        assert!(!ct_validate(&ct));
    }

    #[test]
    fn test_to_json() {
        let ct = new_constraint_target("ik", "bone");
        assert!(constraint_target_to_json(&ct).contains("\"target\":\"bone\""));
    }

    #[test]
    fn test_clone() {
        let ct = new_constraint_target("a", "b");
        let ct2 = ct.clone();
        assert_eq!(ct2.name, "a");
    }
}
