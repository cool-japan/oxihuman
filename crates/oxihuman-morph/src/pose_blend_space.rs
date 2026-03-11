// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A sample in a 2D blend space.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendSample {
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub weight: f32,
}

/// A 2D pose blend space with samples at known positions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseBlendSpace {
    pub samples: Vec<BlendSample>,
}

/// Create a new empty pose blend space.
#[allow(dead_code)]
pub fn new_pose_blend_space() -> PoseBlendSpace {
    PoseBlendSpace {
        samples: Vec::new(),
    }
}

/// Add a sample to the blend space.
#[allow(dead_code)]
pub fn add_blend_sample(space: &mut PoseBlendSpace, name: &str, x: f32, y: f32) {
    space.samples.push(BlendSample {
        x,
        y,
        name: name.to_string(),
        weight: 0.0,
    });
}

/// Evaluate the blend space at (x, y), returning weights for each sample (inverse-distance).
#[allow(dead_code)]
pub fn evaluate_blend_space(space: &PoseBlendSpace, x: f32, y: f32) -> Vec<(String, f32)> {
    if space.samples.is_empty() {
        return Vec::new();
    }
    let dists: Vec<f32> = space
        .samples
        .iter()
        .map(|s| ((s.x - x).powi(2) + (s.y - y).powi(2)).sqrt().max(1e-9))
        .collect();
    // Check for exact match
    for (i, d) in dists.iter().enumerate() {
        if *d < 1e-6 {
            return vec![(space.samples[i].name.clone(), 1.0)];
        }
    }
    let inv_sum: f32 = dists.iter().map(|d| 1.0 / d).sum();
    space
        .samples
        .iter()
        .zip(dists.iter())
        .map(|(s, d)| (s.name.clone(), (1.0 / d) / inv_sum))
        .collect()
}

/// Return the number of samples.
#[allow(dead_code)]
pub fn sample_count_pbs(space: &PoseBlendSpace) -> usize {
    space.samples.len()
}

/// Find the closest sample to (x, y).
#[allow(dead_code)]
pub fn closest_sample(space: &PoseBlendSpace, x: f32, y: f32) -> Option<String> {
    space
        .samples
        .iter()
        .min_by(|a, b| {
            let da = (a.x - x).powi(2) + (a.y - y).powi(2);
            let db = (b.x - x).powi(2) + (b.y - y).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|s| s.name.clone())
}

/// Return the bounding box of the blend space as (min_x, min_y, max_x, max_y).
#[allow(dead_code)]
pub fn blend_space_bounds(space: &PoseBlendSpace) -> (f32, f32, f32, f32) {
    if space.samples.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let min_x = space.samples.iter().map(|s| s.x).fold(f32::INFINITY, f32::min);
    let min_y = space.samples.iter().map(|s| s.y).fold(f32::INFINITY, f32::min);
    let max_x = space.samples.iter().map(|s| s.x).fold(f32::NEG_INFINITY, f32::max);
    let max_y = space.samples.iter().map(|s| s.y).fold(f32::NEG_INFINITY, f32::max);
    (min_x, min_y, max_x, max_y)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn blend_space_to_json(space: &PoseBlendSpace) -> String {
    let entries: Vec<String> = space
        .samples
        .iter()
        .map(|s| format!("{{\"name\":\"{}\",\"x\":{:.4},\"y\":{:.4}}}", s.name, s.x, s.y))
        .collect();
    format!("{{\"samples\":[{}]}}", entries.join(","))
}

/// Remove all samples.
#[allow(dead_code)]
pub fn blend_space_clear(space: &mut PoseBlendSpace) {
    space.samples.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_space_empty() {
        let s = new_pose_blend_space();
        assert_eq!(sample_count_pbs(&s), 0);
    }

    #[test]
    fn add_sample() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "idle", 0.0, 0.0);
        assert_eq!(sample_count_pbs(&s), 1);
    }

    #[test]
    fn evaluate_single() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "idle", 0.0, 0.0);
        let w = evaluate_blend_space(&s, 0.0, 0.0);
        assert_eq!(w.len(), 1);
        assert!((w[0].1 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn evaluate_empty() {
        let s = new_pose_blend_space();
        let w = evaluate_blend_space(&s, 0.0, 0.0);
        assert!(w.is_empty());
    }

    #[test]
    fn closest_single() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "run", 1.0, 0.0);
        assert_eq!(closest_sample(&s, 0.5, 0.0).unwrap(), "run");
    }

    #[test]
    fn bounds_two_samples() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "a", -1.0, -2.0);
        add_blend_sample(&mut s, "b", 3.0, 4.0);
        let (mx, my, mxx, mxy) = blend_space_bounds(&s);
        assert!((mx - (-1.0)).abs() < 1e-6);
        assert!((mxx - 3.0).abs() < 1e-6);
        assert!((my - (-2.0)).abs() < 1e-6);
        assert!((mxy - 4.0).abs() < 1e-6);
    }

    #[test]
    fn clear_space() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "x", 0.0, 0.0);
        blend_space_clear(&mut s);
        assert_eq!(sample_count_pbs(&s), 0);
    }

    #[test]
    fn to_json() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "test", 1.0, 2.0);
        let j = blend_space_to_json(&s);
        assert!(j.contains("\"test\""));
    }

    #[test]
    fn weights_sum_to_one() {
        let mut s = new_pose_blend_space();
        add_blend_sample(&mut s, "a", 0.0, 0.0);
        add_blend_sample(&mut s, "b", 1.0, 0.0);
        add_blend_sample(&mut s, "c", 0.0, 1.0);
        let w = evaluate_blend_space(&s, 0.5, 0.5);
        let sum: f32 = w.iter().map(|(_, v)| v).sum();
        assert!((sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn closest_empty() {
        let s = new_pose_blend_space();
        assert!(closest_sample(&s, 0.0, 0.0).is_none());
    }
}
