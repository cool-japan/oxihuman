#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export a single pose (bone transforms).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneTransform {
    pub bone_id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseExport {
    pub name: String,
    pub bones: Vec<BoneTransform>,
}

#[allow(dead_code)]
pub fn new_pose_export(name: &str) -> PoseExport {
    PoseExport {
        name: name.to_string(),
        bones: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_bone_transform(
    p: &mut PoseExport,
    id: u32,
    name: &str,
    pos: [f32; 3],
    rot: [f32; 4],
    scale: [f32; 3],
) {
    p.bones.push(BoneTransform {
        bone_id: id,
        name: name.to_string(),
        position: pos,
        rotation: rot,
        scale,
    });
}

#[allow(dead_code)]
pub fn export_pose_to_json(p: &PoseExport) -> String {
    let mut bones_json = String::new();
    for (i, b) in p.bones.iter().enumerate() {
        if i > 0 {
            bones_json.push(',');
        }
        bones_json.push_str(&format!(
            r#"{{"id":{},"name":"{}","position":[{},{},{}],"rotation":[{},{},{},{}],"scale":[{},{},{}]}}"#,
            b.bone_id, b.name,
            b.position[0], b.position[1], b.position[2],
            b.rotation[0], b.rotation[1], b.rotation[2], b.rotation[3],
            b.scale[0], b.scale[1], b.scale[2],
        ));
    }
    format!(r#"{{"name":"{}","bones":[{}]}}"#, p.name, bones_json)
}

#[allow(dead_code)]
pub fn bone_count(p: &PoseExport) -> usize {
    p.bones.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pose_export_empty() {
        let p = new_pose_export("idle");
        assert_eq!(p.name, "idle");
        assert_eq!(bone_count(&p), 0);
    }

    #[test]
    fn add_bone_increases_count() {
        let mut p = new_pose_export("run");
        add_bone_transform(&mut p, 0, "root", [0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]);
        assert_eq!(bone_count(&p), 1);
    }

    #[test]
    fn bone_transform_fields() {
        let mut p = new_pose_export("test");
        add_bone_transform(
            &mut p,
            5,
            "spine",
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0],
            [1.0; 3],
        );
        assert_eq!(p.bones[0].bone_id, 5);
        assert_eq!(p.bones[0].name, "spine");
    }

    #[test]
    fn export_pose_to_json_contains_name() {
        let p = new_pose_export("walk");
        let j = export_pose_to_json(&p);
        assert!(j.contains("walk"));
    }

    #[test]
    fn export_pose_to_json_bones_array() {
        let mut p = new_pose_export("t");
        add_bone_transform(&mut p, 0, "b0", [0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]);
        let j = export_pose_to_json(&p);
        assert!(j.contains("bones"));
        assert!(j.contains("b0"));
    }

    #[test]
    fn multiple_bones() {
        let mut p = new_pose_export("t");
        for i in 0..5 {
            add_bone_transform(&mut p, i, "b", [0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]);
        }
        assert_eq!(bone_count(&p), 5);
    }

    #[test]
    fn export_json_has_position() {
        let mut p = new_pose_export("t");
        add_bone_transform(
            &mut p,
            0,
            "b",
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0],
            [1.0; 3],
        );
        let j = export_pose_to_json(&p);
        assert!(j.contains("position"));
    }

    #[test]
    fn export_json_has_rotation() {
        let mut p = new_pose_export("t");
        add_bone_transform(&mut p, 0, "b", [0.0; 3], [0.1, 0.2, 0.3, 0.9], [1.0; 3]);
        let j = export_pose_to_json(&p);
        assert!(j.contains("rotation"));
    }

    #[test]
    fn export_json_has_scale() {
        let mut p = new_pose_export("t");
        add_bone_transform(
            &mut p,
            0,
            "b",
            [0.0; 3],
            [0.0, 0.0, 0.0, 1.0],
            [2.0, 2.0, 2.0],
        );
        let j = export_pose_to_json(&p);
        assert!(j.contains("scale"));
    }
}
