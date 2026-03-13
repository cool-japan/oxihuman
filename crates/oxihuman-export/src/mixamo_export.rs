// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct MixamoBone {
    pub name: String,
    pub pos: [f32; 3],
    pub rotation: [f32; 4],
}

pub fn new_mixamo_bone(name: &str, pos: [f32; 3]) -> MixamoBone {
    MixamoBone {
        name: name.to_string(),
        pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

pub fn mixamo_bone_to_json(b: &MixamoBone) -> String {
    let p = b.pos;
    let r = b.rotation;
    format!(
        r#"{{"name":"{}","pos":[{},{},{}],"rot":[{},{},{},{}]}}"#,
        b.name, p[0], p[1], p[2], r[0], r[1], r[2], r[3]
    )
}

pub fn mixamo_rig_to_json(bones: &[MixamoBone]) -> String {
    let inner: Vec<_> = bones.iter().map(mixamo_bone_to_json).collect();
    format!(r#"{{"bones":[{}]}}"#, inner.join(","))
}

static STANDARD_BONES: &[&str] = &[
    "Hips",
    "Spine",
    "Spine1",
    "Spine2",
    "Neck",
    "Head",
    "LeftShoulder",
    "LeftArm",
    "LeftForeArm",
    "LeftHand",
    "RightShoulder",
    "RightArm",
    "RightForeArm",
    "RightHand",
    "LeftUpLeg",
    "LeftLeg",
    "LeftFoot",
    "LeftToeBase",
    "RightUpLeg",
    "RightLeg",
    "RightFoot",
    "RightToeBase",
];

pub fn mixamo_is_standard_bone(name: &str) -> bool {
    STANDARD_BONES.contains(&name)
}

pub fn mixamo_standard_bones() -> Vec<&'static str> {
    STANDARD_BONES.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mixamo_bone() {
        /* name set */
        let b = new_mixamo_bone("Hips", [0.0, 0.0, 0.0]);
        assert_eq!(b.name, "Hips");
    }

    #[test]
    fn test_mixamo_bone_to_json_contains_name() {
        /* json contains name */
        let b = new_mixamo_bone("Spine", [0.0, 1.0, 0.0]);
        let j = mixamo_bone_to_json(&b);
        assert!(j.contains("Spine"));
    }

    #[test]
    fn test_mixamo_is_standard_bone_true() {
        /* Hips is standard */
        assert!(mixamo_is_standard_bone("Hips"));
    }

    #[test]
    fn test_mixamo_is_standard_bone_false() {
        /* random name is not standard */
        assert!(!mixamo_is_standard_bone("FancyBone123"));
    }

    #[test]
    fn test_mixamo_standard_bones_nonempty() {
        /* standard list not empty */
        assert!(!mixamo_standard_bones().is_empty());
    }
}
