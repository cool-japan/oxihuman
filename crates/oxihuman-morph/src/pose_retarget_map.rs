#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A single bone retargeting pair.
#[derive(Debug, Clone)]
pub struct RetargetPair {
    pub source_bone: String,
    pub target_bone: String,
    pub scale: f32,
}

/// Pose retargeting bone-name mapping.
#[derive(Debug, Clone)]
pub struct PoseRetargetMap {
    pub pairs: Vec<RetargetPair>,
}

#[allow(dead_code)]
pub fn new_pose_retarget_map() -> PoseRetargetMap {
    PoseRetargetMap { pairs: Vec::new() }
}

#[allow(dead_code)]
pub fn add_pair(map: &mut PoseRetargetMap, src: &str, tgt: &str, scale: f32) {
    map.pairs.push(RetargetPair {
        source_bone: src.to_string(),
        target_bone: tgt.to_string(),
        scale,
    });
}

#[allow(dead_code)]
pub fn retarget_weight(map: &PoseRetargetMap, src: &str) -> Option<(String, f32)> {
    map.pairs
        .iter()
        .find(|p| p.source_bone == src)
        .map(|p| (p.target_bone.clone(), p.scale))
}

#[allow(dead_code)]
pub fn pair_count(map: &PoseRetargetMap) -> usize {
    map.pairs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let m = new_pose_retarget_map();
        assert!(m.pairs.is_empty());
    }

    #[test]
    fn test_add_pair() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "Hips", "pelvis", 1.0);
        assert_eq!(pair_count(&m), 1);
    }

    #[test]
    fn test_retarget_weight_found() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "Hips", "pelvis", 1.5);
        let r = retarget_weight(&m, "Hips");
        assert!(r.is_some());
        let (tgt, scale) = r.unwrap();
        assert_eq!(tgt, "pelvis");
        assert!((scale - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_retarget_weight_not_found() {
        let m = new_pose_retarget_map();
        assert!(retarget_weight(&m, "Unknown").is_none());
    }

    #[test]
    fn test_pair_count_multiple() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "Spine", "spine1", 1.0);
        add_pair(&mut m, "Neck", "neck", 1.0);
        assert_eq!(pair_count(&m), 2);
    }

    #[test]
    fn test_source_bone_stored() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "LeftArm", "upper_arm_l", 1.0);
        assert_eq!(m.pairs[0].source_bone, "LeftArm");
    }

    #[test]
    fn test_target_bone_stored() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "LeftArm", "upper_arm_l", 1.0);
        assert_eq!(m.pairs[0].target_bone, "upper_arm_l");
    }

    #[test]
    fn test_scale_stored() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "Head", "head", 0.8);
        assert!((m.pairs[0].scale - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_pairs_independent() {
        let mut m = new_pose_retarget_map();
        add_pair(&mut m, "A", "a", 1.0);
        add_pair(&mut m, "B", "b", 2.0);
        let ra = retarget_weight(&m, "A").unwrap();
        let rb = retarget_weight(&m, "B").unwrap();
        assert_eq!(ra.0, "a");
        assert_eq!(rb.0, "b");
    }
}
