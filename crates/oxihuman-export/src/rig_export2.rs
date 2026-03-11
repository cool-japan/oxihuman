#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export rig/armature data (new module to avoid conflict with existing rig_export).

#[allow(dead_code)]
pub struct BoneExport {
    pub name: String,
    pub parent: Option<String>,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub roll: f32,
}

#[allow(dead_code)]
pub struct RigExport2 {
    pub name: String,
    pub bones: Vec<BoneExport>,
}

#[allow(dead_code)]
pub fn new_rig_export2(name: &str) -> RigExport2 {
    RigExport2 { name: name.to_string(), bones: Vec::new() }
}

#[allow(dead_code)]
pub fn add_bone(rig: &mut RigExport2, name: &str, parent: Option<&str>, head: [f32; 3], tail: [f32; 3], roll: f32) {
    rig.bones.push(BoneExport {
        name: name.to_string(),
        parent: parent.map(|s| s.to_string()),
        head,
        tail,
        roll,
    });
}

#[allow(dead_code)]
pub fn bone_count(rig: &RigExport2) -> usize {
    rig.bones.len()
}

#[allow(dead_code)]
pub fn export_rig2_to_json(rig: &RigExport2) -> String {
    let mut s = format!("{{\"name\":\"{}\",\"bones\":[", rig.name);
    for (i, b) in rig.bones.iter().enumerate() {
        if i > 0 { s.push(','); }
        let parent = b.parent.as_deref().unwrap_or("null");
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"parent\":\"{}\",\"roll\":{}}}",
            b.name, parent, b.roll
        ));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rig_empty() {
        let r = new_rig_export2("Armature");
        assert_eq!(r.name, "Armature");
        assert!(r.bones.is_empty());
    }

    #[test]
    fn add_bone_stored() {
        let mut r = new_rig_export2("A");
        add_bone(&mut r, "root", None, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0);
        assert_eq!(r.bones.len(), 1);
    }

    #[test]
    fn bone_count_correct() {
        let mut r = new_rig_export2("A");
        add_bone(&mut r, "b1", None, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0);
        add_bone(&mut r, "b2", Some("b1"), [0.0, 1.0, 0.0], [0.0, 2.0, 0.0], 0.0);
        assert_eq!(bone_count(&r), 2);
    }

    #[test]
    fn bone_parent_stored() {
        let mut r = new_rig_export2("A");
        add_bone(&mut r, "child", Some("root"), [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0);
        assert_eq!(r.bones[0].parent.as_deref(), Some("root"));
    }

    #[test]
    fn bone_head_stored() {
        let mut r = new_rig_export2("A");
        add_bone(&mut r, "b", None, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0], 0.0);
        assert!((r.bones[0].head[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bone_roll_stored() {
        let mut r = new_rig_export2("A");
        add_bone(&mut r, "b", None, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!((r.bones[0].roll - 0.5).abs() < 1e-6);
    }

    #[test]
    fn export_json_contains_name() {
        let r = new_rig_export2("MyRig");
        let j = export_rig2_to_json(&r);
        assert!(j.contains("MyRig"));
    }

    #[test]
    fn export_json_contains_bone_name() {
        let mut r = new_rig_export2("R");
        add_bone(&mut r, "spine", None, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0);
        let j = export_rig2_to_json(&r);
        assert!(j.contains("spine"));
    }

    #[test]
    fn bone_tail_stored() {
        let mut r = new_rig_export2("A");
        add_bone(&mut r, "b", None, [0.0, 0.0, 0.0], [0.0, 5.0, 0.0], 0.0);
        assert!((r.bones[0].tail[1] - 5.0).abs() < 1e-6);
    }
}
