// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An anatomical landmark with name, position and confidence.
pub struct Landmark {
    pub name: String,
    pub pos: [f32; 3],
    pub confidence: f32,
}

pub fn new_landmark(name: &str, pos: [f32; 3]) -> Landmark {
    Landmark {
        name: name.to_string(),
        pos,
        confidence: 1.0,
    }
}

pub fn landmark_to_json_line(l: &Landmark) -> String {
    format!(
        "{{\"name\":\"{}\",\"pos\":[{:.4},{:.4},{:.4}],\"confidence\":{:.4}}}",
        l.name, l.pos[0], l.pos[1], l.pos[2], l.confidence
    )
}

pub fn landmarks_to_json(ls: &[Landmark]) -> String {
    let items: Vec<String> = ls.iter().map(landmark_to_json_line).collect();
    format!("[{}]", items.join(","))
}

pub fn landmark_distance(a: &Landmark, b: &Landmark) -> f32 {
    let dx = b.pos[0] - a.pos[0];
    let dy = b.pos[1] - a.pos[1];
    let dz = b.pos[2] - a.pos[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn landmarks_bounding_box(ls: &[Landmark]) -> ([f32; 3], [f32; 3]) {
    if ls.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = ls[0].pos;
    let mut mx = ls[0].pos;
    for l in ls {
        for i in 0..3 {
            mn[i] = mn[i].min(l.pos[i]);
            mx[i] = mx[i].max(l.pos[i]);
        }
    }
    (mn, mx)
}

pub fn landmark_centroid(ls: &[Landmark]) -> [f32; 3] {
    let n = ls.len();
    if n == 0 {
        return [0.0; 3];
    }
    let sum = ls.iter().fold([0.0f32; 3], |acc, l| {
        [acc[0] + l.pos[0], acc[1] + l.pos[1], acc[2] + l.pos[2]]
    });
    [sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_landmark() {
        let l = new_landmark("nose_tip", [0.0, 0.0, 1.0]);
        assert_eq!(l.name, "nose_tip");
        assert!((l.confidence - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_landmark_to_json_line() {
        let l = new_landmark("ear_l", [1.0, 0.0, 0.0]);
        let json = landmark_to_json_line(&l);
        assert!(json.contains("\"ear_l\""));
    }

    #[test]
    fn test_landmarks_to_json_array() {
        let ls = vec![new_landmark("a", [0.0; 3]), new_landmark("b", [1.0; 3])];
        let json = landmarks_to_json(&ls);
        assert!(json.starts_with('[') && json.ends_with(']'));
    }

    #[test]
    fn test_landmark_distance() {
        let a = new_landmark("a", [0.0, 0.0, 0.0]);
        let b = new_landmark("b", [3.0, 4.0, 0.0]);
        assert!((landmark_distance(&a, &b) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_landmarks_bounding_box() {
        let ls = vec![
            new_landmark("a", [0.0, 0.0, 0.0]),
            new_landmark("b", [2.0, 3.0, 1.0]),
        ];
        let (mn, mx) = landmarks_bounding_box(&ls);
        assert!((mx[0] - 2.0).abs() < 1e-5);
        assert!((mn[1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_landmark_centroid() {
        let ls = vec![
            new_landmark("a", [0.0, 0.0, 0.0]),
            new_landmark("b", [2.0, 0.0, 0.0]),
        ];
        let c = landmark_centroid(&ls);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_landmarks_to_json_empty() {
        let json = landmarks_to_json(&[]);
        assert_eq!(json, "[]");
    }
}
