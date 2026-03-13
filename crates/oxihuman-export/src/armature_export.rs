// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export armature (bone hierarchy) data.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArmatureBone {
    pub name: String,
    pub parent: Option<usize>,
    pub head: [f32; 3],
    pub tail: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArmatureExport {
    pub name: String,
    pub bones: Vec<ArmatureBone>,
}

#[allow(dead_code)]
pub fn new_armature_export(name: &str) -> ArmatureExport {
    ArmatureExport { name: name.to_string(), bones: Vec::new() }
}

#[allow(dead_code)]
pub fn armature_add_bone(ae: &mut ArmatureExport, name: &str, parent: Option<usize>, head: [f32; 3], tail: [f32; 3]) {
    ae.bones.push(ArmatureBone { name: name.to_string(), parent, head, tail });
}

#[allow(dead_code)]
pub fn armature_bone_count(ae: &ArmatureExport) -> usize { ae.bones.len() }

#[allow(dead_code)]
pub fn armature_root_bones(ae: &ArmatureExport) -> Vec<usize> {
    ae.bones.iter().enumerate().filter(|(_, b)| b.parent.is_none()).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn armature_bone_length(bone: &ArmatureBone) -> f32 {
    let d = [bone.tail[0]-bone.head[0], bone.tail[1]-bone.head[1], bone.tail[2]-bone.head[2]];
    (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
pub fn armature_children(ae: &ArmatureExport, parent: usize) -> Vec<usize> {
    ae.bones.iter().enumerate().filter(|(_, b)| b.parent.is_some_and(|p| p == parent)).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn armature_validate(ae: &ArmatureExport) -> bool {
    ae.bones.iter().all(|b| b.parent.is_none_or(|p| p < ae.bones.len()))
}

#[allow(dead_code)]
pub fn armature_to_json(ae: &ArmatureExport) -> String {
    let bones: Vec<String> = ae.bones.iter().map(|b| format!("{{\"name\":\"{}\",\"length\":{:.4}}}", b.name, armature_bone_length(b))).collect();
    format!("{{\"armature\":\"{}\",\"bones\":[{}]}}", ae.name, bones.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm() -> ArmatureExport {
        let mut a = new_armature_export("skeleton");
        armature_add_bone(&mut a, "root", None, [0.0,0.0,0.0], [0.0,1.0,0.0]);
        armature_add_bone(&mut a, "spine", Some(0), [0.0,1.0,0.0], [0.0,2.0,0.0]);
        a
    }

    #[test] fn test_new() { let a = new_armature_export("test"); assert_eq!(a.bones.len(), 0); }
    #[test] fn test_add_bone() { let a = arm(); assert_eq!(armature_bone_count(&a), 2); }
    #[test] fn test_root_bones() { let a = arm(); assert_eq!(armature_root_bones(&a).len(), 1); }
    #[test] fn test_bone_length() { let a = arm(); assert!((armature_bone_length(&a.bones[0]) - 1.0).abs() < 1e-5); }
    #[test] fn test_children() { let a = arm(); assert_eq!(armature_children(&a, 0).len(), 1); }
    #[test] fn test_validate() { let a = arm(); assert!(armature_validate(&a)); }
    #[test] fn test_to_json() { let a = arm(); assert!(armature_to_json(&a).contains("skeleton")); }
    #[test] fn test_no_children() { let a = arm(); assert!(armature_children(&a, 1).is_empty()); }
    #[test] fn test_name() { let a = arm(); assert_eq!(a.name, "skeleton"); }
    #[test] fn test_empty() { let a = new_armature_export("e"); assert!(armature_validate(&a)); }
}
