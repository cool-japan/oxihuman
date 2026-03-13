// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct MediapipeLandmark {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub visibility: f32,
}

pub struct MediapipePose {
    pub landmarks: Vec<MediapipeLandmark>,
}

pub fn new_mediapipe_landmark(x: f32, y: f32, z: f32) -> MediapipeLandmark {
    MediapipeLandmark {
        x,
        y,
        z,
        visibility: 1.0,
    }
}

pub fn new_mediapipe_pose() -> MediapipePose {
    MediapipePose {
        landmarks: Vec::new(),
    }
}

pub fn pose_push_landmark(p: &mut MediapipePose, lm: MediapipeLandmark) {
    p.landmarks.push(lm);
}

pub fn pose_to_json(p: &MediapipePose) -> String {
    let inner: Vec<_> = p
        .landmarks
        .iter()
        .map(|lm| {
            format!(
                r#"{{"x":{},"y":{},"z":{},"visibility":{}}}"#,
                lm.x, lm.y, lm.z, lm.visibility
            )
        })
        .collect();
    format!(r#"{{"landmarks":[{}]}}"#, inner.join(","))
}

static LANDMARK_NAMES: &[&str] = &[
    "nose",
    "left_eye_inner",
    "left_eye",
    "left_eye_outer",
    "right_eye_inner",
    "right_eye",
    "right_eye_outer",
    "left_ear",
    "right_ear",
    "mouth_left",
    "mouth_right",
    "left_shoulder",
    "right_shoulder",
    "left_elbow",
    "right_elbow",
    "left_wrist",
    "right_wrist",
    "left_pinky",
    "right_pinky",
    "left_index",
    "right_index",
    "left_thumb",
    "right_thumb",
    "left_hip",
    "right_hip",
    "left_knee",
    "right_knee",
    "left_ankle",
    "right_ankle",
    "left_heel",
    "right_heel",
    "left_foot_index",
    "right_foot_index",
];

pub fn pose_landmark_name(idx: usize) -> &'static str {
    LANDMARK_NAMES.get(idx).copied().unwrap_or("unknown")
}

pub fn pose_is_complete(p: &MediapipePose) -> bool {
    p.landmarks.len() == 33
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mediapipe_pose_empty() {
        /* starts empty */
        let p = new_mediapipe_pose();
        assert_eq!(p.landmarks.len(), 0);
    }

    #[test]
    fn test_push_landmark() {
        /* push increases count */
        let mut p = new_mediapipe_pose();
        pose_push_landmark(&mut p, new_mediapipe_landmark(0.5, 0.5, 0.0));
        assert_eq!(p.landmarks.len(), 1);
    }

    #[test]
    fn test_pose_landmark_name_zero() {
        /* idx 0 = nose */
        assert_eq!(pose_landmark_name(0), "nose");
    }

    #[test]
    fn test_pose_is_complete_false() {
        /* not complete with 0 landmarks */
        let p = new_mediapipe_pose();
        assert!(!pose_is_complete(&p));
    }

    #[test]
    fn test_pose_to_json_not_empty() {
        /* json not empty */
        let mut p = new_mediapipe_pose();
        pose_push_landmark(&mut p, new_mediapipe_landmark(0.0, 0.0, 0.0));
        let j = pose_to_json(&p);
        assert!(!j.is_empty());
    }
}
