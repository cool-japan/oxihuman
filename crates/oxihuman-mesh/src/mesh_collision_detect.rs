// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Mesh self-collision detection stub.

#[allow(dead_code)]
pub struct CollisionPair {
    pub face_a: usize,
    pub face_b: usize,
    pub dist: f32,
}

#[allow(dead_code)]
pub struct CollisionDetector {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub threshold: f32,
}

#[allow(dead_code)]
pub fn new_collision_detector(
    positions: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
    threshold: f32,
) -> CollisionDetector {
    CollisionDetector { positions, indices, threshold }
}

#[allow(dead_code)]
pub fn cd_detect(_det: &CollisionDetector) -> Vec<CollisionPair> {
    Vec::new()
}

#[allow(dead_code)]
pub fn cd_face_count(det: &CollisionDetector) -> usize {
    det.indices.len()
}

#[allow(dead_code)]
pub fn cd_threshold(det: &CollisionDetector) -> f32 {
    det.threshold
}

#[allow(dead_code)]
pub fn cd_set_threshold(det: &mut CollisionDetector, t: f32) {
    det.threshold = t;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_detector() -> CollisionDetector {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.5, 1.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2], [3, 4, 5]];
        new_collision_detector(positions, indices, 0.01)
    }

    #[test]
    fn test_face_count() {
        let det = make_detector();
        assert_eq!(cd_face_count(&det), 2);
    }

    #[test]
    fn test_threshold() {
        let det = make_detector();
        assert!((cd_threshold(&det) - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_set_threshold() {
        let mut det = make_detector();
        cd_set_threshold(&mut det, 0.05);
        assert!((cd_threshold(&det) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_detect_no_crash() {
        let det = make_detector();
        let _ = cd_detect(&det);
    }

    #[test]
    fn test_detect_returns_vec() {
        let det = make_detector();
        let pairs = cd_detect(&det);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_empty_detector() {
        let det = new_collision_detector(vec![], vec![], 0.01);
        assert_eq!(cd_face_count(&det), 0);
    }

    #[test]
    fn test_new_stores_threshold() {
        let det = new_collision_detector(vec![], vec![], 0.1);
        assert!((det.threshold - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_detect_empty() {
        let det = new_collision_detector(vec![], vec![], 0.01);
        let pairs = cd_detect(&det);
        assert_eq!(pairs.len(), 0);
    }
}
