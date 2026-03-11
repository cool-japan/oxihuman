// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct OpenPoseKeypoint {
    pub x: f32,
    pub y: f32,
    pub confidence: f32,
}

pub struct OpenPoseBody {
    pub keypoints: Vec<OpenPoseKeypoint>,
}

pub fn new_openpose_keypoint(x: f32, y: f32, conf: f32) -> OpenPoseKeypoint {
    OpenPoseKeypoint {
        x,
        y,
        confidence: conf,
    }
}

pub fn new_openpose_body() -> OpenPoseBody {
    OpenPoseBody {
        keypoints: Vec::new(),
    }
}

pub fn body_push_keypoint(b: &mut OpenPoseBody, kp: OpenPoseKeypoint) {
    b.keypoints.push(kp);
}

pub fn body_to_json(b: &OpenPoseBody) -> String {
    let inner: Vec<_> = b
        .keypoints
        .iter()
        .map(|kp| format!(r#"[{},{},{}]"#, kp.x, kp.y, kp.confidence))
        .collect();
    format!(r#"{{"keypoints":[{}]}}"#, inner.join(","))
}

static COCO_NAMES: &[&str] = &[
    "nose",
    "neck",
    "right_shoulder",
    "right_elbow",
    "right_wrist",
    "left_shoulder",
    "left_elbow",
    "left_wrist",
    "right_hip",
    "right_knee",
    "right_ankle",
    "left_hip",
    "left_knee",
    "left_ankle",
    "right_eye",
    "left_eye",
    "right_ear",
    "left_ear",
];

pub fn keypoint_name_coco(idx: usize) -> &'static str {
    COCO_NAMES.get(idx).copied().unwrap_or("unknown")
}

pub fn body_is_valid_coco(b: &OpenPoseBody) -> bool {
    b.keypoints.len() == 18
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_openpose_body_empty() {
        /* starts empty */
        let b = new_openpose_body();
        assert_eq!(b.keypoints.len(), 0);
    }

    #[test]
    fn test_body_push_keypoint() {
        /* push increases count */
        let mut b = new_openpose_body();
        body_push_keypoint(&mut b, new_openpose_keypoint(0.5, 0.3, 0.9));
        assert_eq!(b.keypoints.len(), 1);
    }

    #[test]
    fn test_keypoint_name_coco_nose() {
        /* idx 0 = nose */
        assert_eq!(keypoint_name_coco(0), "nose");
    }

    #[test]
    fn test_body_is_valid_coco_false() {
        /* not 18 = invalid */
        let b = new_openpose_body();
        assert!(!body_is_valid_coco(&b));
    }

    #[test]
    fn test_body_to_json_not_empty() {
        /* json not empty */
        let mut b = new_openpose_body();
        body_push_keypoint(&mut b, new_openpose_keypoint(0.0, 0.0, 1.0));
        let j = body_to_json(&b);
        assert!(!j.is_empty());
    }
}
