#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export bone constraints (IK, copy_transform, etc.).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintEntry {
    pub name: String,
    pub constraint_type: String,
    pub target_bone: Option<String>,
    pub influence: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneConstraintsExport {
    pub bone_name: String,
    pub constraints: Vec<ConstraintEntry>,
}

#[allow(dead_code)]
pub fn new_bone_constraints_export(bone: &str) -> BoneConstraintsExport {
    BoneConstraintsExport { bone_name: bone.to_string(), constraints: Vec::new() }
}

#[allow(dead_code)]
pub fn add_bone_constraint(
    exp: &mut BoneConstraintsExport,
    name: &str,
    type_: &str,
    target: Option<&str>,
    influence: f32,
) {
    exp.constraints.push(ConstraintEntry {
        name: name.to_string(),
        constraint_type: type_.to_string(),
        target_bone: target.map(|s| s.to_string()),
        influence,
        enabled: true,
    });
}

#[allow(dead_code)]
pub fn export_bone_constraints_to_json(exp: &BoneConstraintsExport) -> String {
    let mut cs_json = String::new();
    for (i, c) in exp.constraints.iter().enumerate() {
        if i > 0 {
            cs_json.push(',');
        }
        let target = match &c.target_bone {
            Some(t) => format!(r#""{}""#, t),
            None => "null".to_string(),
        };
        cs_json.push_str(&format!(
            r#"{{"name":"{}","type":"{}","target":{},"influence":{},"enabled":{}}}"#,
            c.name, c.constraint_type, target, c.influence, c.enabled
        ));
    }
    format!(r#"{{"bone":"{}","constraints":[{}]}}"#, exp.bone_name, cs_json)
}

#[allow(dead_code)]
pub fn bone_constraint_count(exp: &BoneConstraintsExport) -> usize {
    exp.constraints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_bone_constraints_export("spine");
        assert_eq!(e.bone_name, "spine");
        assert_eq!(bone_constraint_count(&e), 0);
    }

    #[test]
    fn add_constraint_increases_count() {
        let mut e = new_bone_constraints_export("arm");
        add_bone_constraint(&mut e, "ik", "IK", Some("hand"), 1.0);
        assert_eq!(bone_constraint_count(&e), 1);
    }

    #[test]
    fn constraint_fields_stored() {
        let mut e = new_bone_constraints_export("leg");
        add_bone_constraint(&mut e, "copy_loc", "COPY_LOCATION", None, 0.5);
        assert_eq!(e.constraints[0].name, "copy_loc");
        assert!((e.constraints[0].influence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn target_none_stored() {
        let mut e = new_bone_constraints_export("b");
        add_bone_constraint(&mut e, "limit_rot", "LIMIT_ROTATION", None, 1.0);
        assert!(e.constraints[0].target_bone.is_none());
    }

    #[test]
    fn target_some_stored() {
        let mut e = new_bone_constraints_export("b");
        add_bone_constraint(&mut e, "ik", "IK", Some("foot"), 1.0);
        assert_eq!(e.constraints[0].target_bone.as_deref(), Some("foot"));
    }

    #[test]
    fn export_json_has_bone_name() {
        let e = new_bone_constraints_export("head");
        let j = export_bone_constraints_to_json(&e);
        assert!(j.contains("head"));
    }

    #[test]
    fn export_json_has_constraint_name() {
        let mut e = new_bone_constraints_export("b");
        add_bone_constraint(&mut e, "my_ik", "IK", None, 1.0);
        let j = export_bone_constraints_to_json(&e);
        assert!(j.contains("my_ik"));
    }

    #[test]
    fn enabled_defaults_true() {
        let mut e = new_bone_constraints_export("b");
        add_bone_constraint(&mut e, "c", "T", None, 1.0);
        assert!(e.constraints[0].enabled);
    }

    #[test]
    fn multiple_constraints() {
        let mut e = new_bone_constraints_export("b");
        for _ in 0..3 {
            add_bone_constraint(&mut e, "c", "T", None, 1.0);
        }
        assert_eq!(bone_constraint_count(&e), 3);
    }
}
