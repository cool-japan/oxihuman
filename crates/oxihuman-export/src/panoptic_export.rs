// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct PanopticBody {
    pub body_25_keypoints: Vec<[f32; 3]>,
    pub confidence: Vec<f32>,
}

pub fn new_panoptic_body() -> PanopticBody {
    PanopticBody {
        body_25_keypoints: Vec::new(),
        confidence: Vec::new(),
    }
}

pub fn panoptic_push_keypoint(b: &mut PanopticBody, pos: [f32; 3], conf: f32) {
    b.body_25_keypoints.push(pos);
    b.confidence.push(conf);
}

pub fn panoptic_to_json(b: &PanopticBody) -> String {
    let kps: Vec<_> = b
        .body_25_keypoints
        .iter()
        .zip(b.confidence.iter())
        .map(|(p, c)| format!(r#"[{},{},{},{}]"#, p[0], p[1], p[2], c))
        .collect();
    format!(r#"{{"keypoints":[{}]}}"#, kps.join(","))
}

pub fn panoptic_keypoint_count(b: &PanopticBody) -> usize {
    b.body_25_keypoints.len()
}

pub fn panoptic_is_body25(b: &PanopticBody) -> bool {
    b.body_25_keypoints.len() == 25
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_panoptic_body_empty() {
        /* starts empty */
        let b = new_panoptic_body();
        assert_eq!(panoptic_keypoint_count(&b), 0);
    }

    #[test]
    fn test_panoptic_push_keypoint() {
        /* push increases count */
        let mut b = new_panoptic_body();
        panoptic_push_keypoint(&mut b, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(panoptic_keypoint_count(&b), 1);
    }

    #[test]
    fn test_panoptic_is_body25_false() {
        /* not 25 = false */
        let b = new_panoptic_body();
        assert!(!panoptic_is_body25(&b));
    }

    #[test]
    fn test_panoptic_is_body25_true() {
        /* 25 keypoints = body25 */
        let mut b = new_panoptic_body();
        for _ in 0..25 {
            panoptic_push_keypoint(&mut b, [0.0, 0.0, 0.0], 1.0);
        }
        assert!(panoptic_is_body25(&b));
    }

    #[test]
    fn test_panoptic_to_json_not_empty() {
        /* json not empty */
        let mut b = new_panoptic_body();
        panoptic_push_keypoint(&mut b, [1.0, 2.0, 3.0], 0.9);
        let j = panoptic_to_json(&b);
        assert!(!j.is_empty());
    }
}
