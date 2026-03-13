// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A mapping from source joint name to target joint name.
#[allow(dead_code)]
pub struct RetargetMap {
    /// (source_name, target_name) pairs.
    pub pairs: Vec<(String, String)>,
}

#[allow(dead_code)]
impl RetargetMap {
    /// Create an empty retarget map.
    pub fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// Add a (source, target) joint name pair to the map.
    pub fn add(&mut self, src: &str, tgt: &str) {
        self.pairs.push((src.to_string(), tgt.to_string()));
    }

    /// Build an identity map from a list of joint names (src == tgt).
    pub fn identity(joint_names: &[&str]) -> Self {
        let pairs = joint_names
            .iter()
            .map(|name| (name.to_string(), name.to_string()))
            .collect();
        Self { pairs }
    }

    /// Look up target joint index given source joint name and target skeleton.
    pub fn resolve(&self, src_name: &str, target_names: &[String]) -> Option<usize> {
        // Find the target name mapped from src_name
        let tgt_name = self
            .pairs
            .iter()
            .find(|(src, _)| src == src_name)
            .map(|(_, tgt)| tgt.as_str())?;

        // Find the index of that target name in the target skeleton
        target_names.iter().position(|n| n == tgt_name)
    }
}

impl Default for RetargetMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Retarget a pose (Vec of quaternions, one per joint) from a source skeleton
/// to a target skeleton using the retarget map.
///
/// - `src_quats`: source joint rotations as [f32; 4] (xyzw quaternions), indexed by source joint index
/// - `src_names`: source joint names
/// - `tgt_names`: target joint names
/// - Returns a `Vec<[f32; 4]>` of length `tgt_names.len()`, with identity quat `[0,0,0,1]` for unmapped joints.
#[allow(dead_code)]
pub fn retarget_pose(
    src_quats: &[[f32; 4]],
    src_names: &[String],
    tgt_names: &[String],
    map: &RetargetMap,
) -> Vec<[f32; 4]> {
    const IDENTITY: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    let mut result = vec![IDENTITY; tgt_names.len()];

    for (src_idx, src_name) in src_names.iter().enumerate() {
        if let Some(tgt_idx) = map.resolve(src_name, tgt_names) {
            if let Some(quat) = src_quats.get(src_idx) {
                result[tgt_idx] = *quat;
            }
        }
    }

    result
}

/// Retarget a sequence of poses (each pose is `Vec<[f32;4]>`).
#[allow(dead_code)]
pub fn retarget_sequence(
    src_sequence: &[Vec<[f32; 4]>],
    src_names: &[String],
    tgt_names: &[String],
    map: &RetargetMap,
) -> Vec<Vec<[f32; 4]>> {
    src_sequence
        .iter()
        .map(|pose| retarget_pose(pose, src_names, tgt_names, map))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(ns: &[&str]) -> Vec<String> {
        ns.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn retarget_map_new_is_empty() {
        let map = RetargetMap::new();
        assert!(map.pairs.is_empty());
    }

    #[test]
    fn retarget_map_add_and_resolve() {
        let mut map = RetargetMap::new();
        map.add("hip", "pelvis");
        let tgt = names(&["pelvis", "spine"]);
        assert_eq!(map.resolve("hip", &tgt), Some(0));
    }

    #[test]
    fn retarget_map_identity_resolves_same_name() {
        let map = RetargetMap::identity(&["hip", "spine", "head"]);
        let tgt = names(&["hip", "spine", "head"]);
        assert_eq!(map.resolve("spine", &tgt), Some(1));
        assert_eq!(map.resolve("head", &tgt), Some(2));
    }

    #[test]
    fn retarget_map_resolve_missing_returns_none() {
        let map = RetargetMap::identity(&["hip", "spine"]);
        let tgt = names(&["hip", "spine"]);
        assert_eq!(map.resolve("nonexistent", &tgt), None);
    }

    #[test]
    fn retarget_pose_identity_map_preserves_quats() {
        let map = RetargetMap::identity(&["hip", "spine"]);
        let src_names = names(&["hip", "spine"]);
        let tgt_names = names(&["hip", "spine"]);
        let src_quats = vec![[0.1, 0.2, 0.3, 0.9], [0.0, 0.5, 0.5, 0.7]];
        let result = retarget_pose(&src_quats, &src_names, &tgt_names, &map);
        assert_eq!(result.len(), 2);
        for i in 0..2 {
            for j in 0..4 {
                assert!((result[i][j] - src_quats[i][j]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn retarget_pose_mapped_joint_transfers_correctly() {
        let mut map = RetargetMap::new();
        map.add("src_hip", "tgt_pelvis");
        let src_names = names(&["src_hip", "src_spine"]);
        let tgt_names = names(&["tgt_pelvis", "tgt_chest"]);
        let src_quats = vec![[0.1, 0.2, 0.3, 0.9], [0.0, 0.0, 0.0, 1.0]];
        let result = retarget_pose(&src_quats, &src_names, &tgt_names, &map);
        // tgt_pelvis should receive src_hip's rotation
        assert!((result[0][0] - 0.1).abs() < 1e-6);
        assert!((result[0][1] - 0.2).abs() < 1e-6);
        assert!((result[0][2] - 0.3).abs() < 1e-6);
        assert!((result[0][3] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn retarget_pose_unmapped_joint_is_identity_quat() {
        let map = RetargetMap::new(); // empty map — nothing maps
        let src_names = names(&["hip"]);
        let tgt_names = names(&["pelvis", "spine"]);
        let src_quats = vec![[0.1, 0.2, 0.3, 0.9]];
        let result = retarget_pose(&src_quats, &src_names, &tgt_names, &map);
        // Both target joints should be identity [0,0,0,1]
        assert_eq!(result[0], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(result[1], [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn retarget_pose_length_equals_target_count() {
        let map = RetargetMap::identity(&["a", "b", "c"]);
        let src_names = names(&["a", "b", "c"]);
        let tgt_names = names(&["a", "b", "c", "d", "e"]);
        let src_quats = vec![[0.0, 0.0, 0.0, 1.0]; 3];
        let result = retarget_pose(&src_quats, &src_names, &tgt_names, &map);
        assert_eq!(result.len(), tgt_names.len());
    }

    #[test]
    fn retarget_sequence_length_matches_input() {
        let map = RetargetMap::identity(&["hip"]);
        let src_names = names(&["hip"]);
        let tgt_names = names(&["hip"]);
        let src_sequence: Vec<Vec<[f32; 4]>> = vec![
            vec![[0.0, 0.0, 0.0, 1.0]],
            vec![[0.1, 0.0, 0.0, 1.0]],
            vec![[0.2, 0.0, 0.0, 1.0]],
        ];
        let result = retarget_sequence(&src_sequence, &src_names, &tgt_names, &map);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn retarget_sequence_each_pose_correct_length() {
        let map = RetargetMap::identity(&["hip", "spine"]);
        let src_names = names(&["hip", "spine"]);
        let tgt_names = names(&["hip", "spine", "extra"]);
        let src_sequence: Vec<Vec<[f32; 4]>> = vec![
            vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]],
            vec![[0.1, 0.0, 0.0, 1.0], [0.0, 0.1, 0.0, 1.0]],
        ];
        let result = retarget_sequence(&src_sequence, &src_names, &tgt_names, &map);
        for pose in &result {
            assert_eq!(pose.len(), tgt_names.len());
        }
    }
}
