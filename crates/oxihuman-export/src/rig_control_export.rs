// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rig control/handle export.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ControlType {
    FK,
    IK,
    Blend,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigControl {
    pub name: String,
    pub control_type: ControlType,
    pub bone_target: String,
    pub value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigControlExport {
    pub controls: Vec<RigControl>,
}

#[allow(dead_code)]
pub fn new_rig_control_export() -> RigControlExport {
    RigControlExport { controls: Vec::new() }
}

#[allow(dead_code)]
pub fn rc_add_control(exp: &mut RigControlExport, ctrl: RigControl) {
    exp.controls.push(ctrl);
}

#[allow(dead_code)]
pub fn rc_get_control<'a>(exp: &'a RigControlExport, name: &str) -> Option<&'a RigControl> {
    exp.controls.iter().find(|c| c.name == name)
}

#[allow(dead_code)]
pub fn rc_remove_control(exp: &mut RigControlExport, name: &str) -> bool {
    let before = exp.controls.len();
    exp.controls.retain(|c| c.name != name);
    exp.controls.len() < before
}

#[allow(dead_code)]
pub fn rc_count(exp: &RigControlExport) -> usize {
    exp.controls.len()
}

#[allow(dead_code)]
pub fn rc_set_value(exp: &mut RigControlExport, name: &str, value: f32) -> bool {
    if let Some(c) = exp.controls.iter_mut().find(|c| c.name == name) {
        c.value = value;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn rc_to_json(exp: &RigControlExport) -> String {
    format!(r#"{{"control_count":{}}}"#, exp.controls.len())
}

#[allow(dead_code)]
pub fn rc_control_type_name(ct: &ControlType) -> &'static str {
    match ct {
        ControlType::FK => "FK",
        ControlType::IK => "IK",
        ControlType::Blend => "Blend",
    }
}

#[allow(dead_code)]
pub fn rc_validate(exp: &RigControlExport) -> bool {
    exp.controls.iter().all(|c| !c.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctrl(name: &str, ct: ControlType) -> RigControl {
        RigControl { name: name.to_string(), control_type: ct, bone_target: "Spine".to_string(), value: 0.0 }
    }

    #[test]
    fn new_export_is_empty() {
        let exp = new_rig_control_export();
        assert_eq!(rc_count(&exp), 0);
    }

    #[test]
    fn add_control_increments_count() {
        let mut exp = new_rig_control_export();
        rc_add_control(&mut exp, make_ctrl("arm_ik", ControlType::IK));
        assert_eq!(rc_count(&exp), 1);
    }

    #[test]
    fn get_control_by_name() {
        let mut exp = new_rig_control_export();
        rc_add_control(&mut exp, make_ctrl("leg_fk", ControlType::FK));
        assert!(rc_get_control(&exp, "leg_fk").is_some());
        assert!(rc_get_control(&exp, "missing").is_none());
    }

    #[test]
    fn remove_control() {
        let mut exp = new_rig_control_export();
        rc_add_control(&mut exp, make_ctrl("ctrl_a", ControlType::Blend));
        let removed = rc_remove_control(&mut exp, "ctrl_a");
        assert!(removed);
        assert_eq!(rc_count(&exp), 0);
    }

    #[test]
    fn set_value_updates() {
        let mut exp = new_rig_control_export();
        rc_add_control(&mut exp, make_ctrl("blend_ctrl", ControlType::Blend));
        let ok = rc_set_value(&mut exp, "blend_ctrl", 0.75);
        assert!(ok);
        assert!((rc_get_control(&exp, "blend_ctrl").unwrap().value - 0.75).abs() < 1e-6);
    }

    #[test]
    fn type_names_correct() {
        assert_eq!(rc_control_type_name(&ControlType::FK), "FK");
        assert_eq!(rc_control_type_name(&ControlType::IK), "IK");
        assert_eq!(rc_control_type_name(&ControlType::Blend), "Blend");
    }

    #[test]
    fn validate_empty_export() {
        let exp = new_rig_control_export();
        assert!(rc_validate(&exp));
    }

    #[test]
    fn to_json_has_count() {
        let exp = new_rig_control_export();
        let json = rc_to_json(&exp);
        assert!(json.contains("control_count"));
    }
}
