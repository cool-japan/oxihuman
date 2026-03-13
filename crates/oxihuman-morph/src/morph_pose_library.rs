// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Pose library for storing named morph poses.

#[allow(dead_code)]
pub struct MorphPose {
    pub name: String,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
pub struct PoseLibrary {
    pub poses: Vec<MorphPose>,
}

#[allow(dead_code)]
pub fn new_pose_library() -> PoseLibrary {
    PoseLibrary { poses: Vec::new() }
}

#[allow(dead_code)]
pub fn pl_add_pose(lib: &mut PoseLibrary, name: &str, weights: Vec<f32>) -> usize {
    let idx = lib.poses.len();
    lib.poses.push(MorphPose { name: name.to_string(), weights });
    idx
}

#[allow(dead_code)]
pub fn pl_get_pose<'a>(lib: &'a PoseLibrary, name: &str) -> Option<&'a MorphPose> {
    lib.poses.iter().find(|p| p.name == name)
}

#[allow(dead_code)]
pub fn pl_pose_count(lib: &PoseLibrary) -> usize {
    lib.poses.len()
}

#[allow(dead_code)]
pub fn pl_blend(lib: &PoseLibrary, name_a: &str, name_b: &str, t: f32) -> Option<Vec<f32>> {
    let a = pl_get_pose(lib, name_a)?;
    let b = pl_get_pose(lib, name_b)?;
    if a.weights.len() != b.weights.len() {
        return None;
    }
    let result = a.weights.iter().zip(b.weights.iter())
        .map(|(wa, wb)| wa + (wb - wa) * t)
        .collect();
    Some(result)
}

#[allow(dead_code)]
pub fn pl_remove(lib: &mut PoseLibrary, name: &str) -> bool {
    if let Some(idx) = lib.poses.iter().position(|p| p.name == name) {
        lib.poses.remove(idx);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_pose() {
        let mut lib = new_pose_library();
        let idx = pl_add_pose(&mut lib, "smile", vec![1.0, 0.0]);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_get_pose() {
        let mut lib = new_pose_library();
        pl_add_pose(&mut lib, "sad", vec![0.0, 1.0]);
        assert!(pl_get_pose(&lib, "sad").is_some());
    }

    #[test]
    fn test_get_pose_missing() {
        let lib = new_pose_library();
        assert!(pl_get_pose(&lib, "none").is_none());
    }

    #[test]
    fn test_pose_count() {
        let mut lib = new_pose_library();
        pl_add_pose(&mut lib, "a", vec![]);
        pl_add_pose(&mut lib, "b", vec![]);
        assert_eq!(pl_pose_count(&lib), 2);
    }

    #[test]
    fn test_blend() {
        let mut lib = new_pose_library();
        pl_add_pose(&mut lib, "neutral", vec![0.0]);
        pl_add_pose(&mut lib, "smile", vec![1.0]);
        let v = pl_blend(&lib, "neutral", "smile", 0.5).expect("should succeed");
        assert!((v[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_missing() {
        let lib = new_pose_library();
        assert!(pl_blend(&lib, "a", "b", 0.5).is_none());
    }

    #[test]
    fn test_remove() {
        let mut lib = new_pose_library();
        pl_add_pose(&mut lib, "pose1", vec![]);
        assert!(pl_remove(&mut lib, "pose1"));
        assert_eq!(pl_pose_count(&lib), 0);
    }

    #[test]
    fn test_remove_missing() {
        let mut lib = new_pose_library();
        assert!(!pl_remove(&mut lib, "ghost"));
    }
}
