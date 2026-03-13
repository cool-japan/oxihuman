// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export IK constraint data for a skeleton.

/// Type of IK constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IkConstraintType {
    Position,
    Rotation,
    LookAt,
    TwoJoint,
}

/// A single IK constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkConstraintEntry {
    pub bone_name: String,
    pub target_name: String,
    pub constraint_type: IkConstraintType,
    pub weight: f32,
    pub chain_length: u32,
}

/// IK constraint export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkConstraintExport {
    pub constraints: Vec<IkConstraintEntry>,
}

/// Create a new IK constraint export.
#[allow(dead_code)]
pub fn new_ik_constraint_export() -> IkConstraintExport {
    IkConstraintExport {
        constraints: Vec::new(),
    }
}

/// Add a constraint.
#[allow(dead_code)]
pub fn add_ik_constraint(export: &mut IkConstraintExport, entry: IkConstraintEntry) {
    export.constraints.push(entry);
}

/// Count constraints.
#[allow(dead_code)]
pub fn ik_constraint_count(export: &IkConstraintExport) -> usize {
    export.constraints.len()
}

/// Count constraints of a given type.
#[allow(dead_code)]
pub fn count_constraint_type(export: &IkConstraintExport, ctype: IkConstraintType) -> usize {
    export
        .constraints
        .iter()
        .filter(|c| c.constraint_type == ctype)
        .count()
}

/// Find constraint by bone name.
#[allow(dead_code)]
pub fn find_constraint_by_bone<'a>(
    export: &'a IkConstraintExport,
    bone: &str,
) -> Option<&'a IkConstraintEntry> {
    export.constraints.iter().find(|c| c.bone_name == bone)
}

/// Validate weights are in [0.0, 1.0].
#[allow(dead_code)]
pub fn validate_ik_weights(export: &IkConstraintExport) -> bool {
    export
        .constraints
        .iter()
        .all(|c| (0.0..=1.0).contains(&c.weight))
}

/// Clamp all weights to [0.0, 1.0].
#[allow(dead_code)]
pub fn clamp_ik_weights(export: &mut IkConstraintExport) {
    for c in &mut export.constraints {
        c.weight = c.weight.clamp(0.0, 1.0);
    }
}

/// Average chain length.
#[allow(dead_code)]
pub fn avg_chain_length_ik(export: &IkConstraintExport) -> f32 {
    let n = export.constraints.len();
    if n == 0 {
        return 0.0;
    }
    export
        .constraints
        .iter()
        .map(|c| c.chain_length as f32)
        .sum::<f32>()
        / n as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn ik_constraint_to_json(export: &IkConstraintExport) -> String {
    format!("{{\"constraint_count\":{}}}", export.constraints.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_constraint(bone: &str) -> IkConstraintEntry {
        IkConstraintEntry {
            bone_name: bone.to_string(),
            target_name: "target".to_string(),
            constraint_type: IkConstraintType::Position,
            weight: 1.0,
            chain_length: 3,
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut e = new_ik_constraint_export();
        add_ik_constraint(&mut e, sample_constraint("arm"));
        assert_eq!(ik_constraint_count(&e), 1);
    }

    #[test]
    fn test_find_by_bone() {
        let mut e = new_ik_constraint_export();
        add_ik_constraint(&mut e, sample_constraint("arm"));
        assert!(find_constraint_by_bone(&e, "arm").is_some());
    }

    #[test]
    fn test_find_missing() {
        let e = new_ik_constraint_export();
        assert!(find_constraint_by_bone(&e, "leg").is_none());
    }

    #[test]
    fn test_count_type() {
        let mut e = new_ik_constraint_export();
        add_ik_constraint(&mut e, sample_constraint("arm"));
        assert_eq!(count_constraint_type(&e, IkConstraintType::Position), 1);
        assert_eq!(count_constraint_type(&e, IkConstraintType::Rotation), 0);
    }

    #[test]
    fn test_validate_valid() {
        let mut e = new_ik_constraint_export();
        add_ik_constraint(&mut e, sample_constraint("arm"));
        assert!(validate_ik_weights(&e));
    }

    #[test]
    fn test_validate_invalid() {
        let mut e = new_ik_constraint_export();
        e.constraints.push(IkConstraintEntry {
            bone_name: "x".to_string(),
            target_name: "t".to_string(),
            constraint_type: IkConstraintType::Position,
            weight: 1.5,
            chain_length: 2,
        });
        assert!(!validate_ik_weights(&e));
    }

    #[test]
    fn test_clamp_weights() {
        let mut e = new_ik_constraint_export();
        e.constraints.push(IkConstraintEntry {
            bone_name: "x".to_string(),
            target_name: "t".to_string(),
            constraint_type: IkConstraintType::Position,
            weight: 2.0,
            chain_length: 2,
        });
        clamp_ik_weights(&mut e);
        assert!(validate_ik_weights(&e));
    }

    #[test]
    fn test_avg_chain_length() {
        let mut e = new_ik_constraint_export();
        add_ik_constraint(&mut e, sample_constraint("arm"));
        assert!((avg_chain_length_ik(&e) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_ik_constraint_to_json() {
        let e = new_ik_constraint_export();
        let j = ik_constraint_to_json(&e);
        assert!(j.contains("constraint_count"));
    }

    #[test]
    fn test_empty_avg_chain_zero() {
        let e = new_ik_constraint_export();
        assert!(avg_chain_length_ik(&e).abs() < 1e-6);
    }
}
