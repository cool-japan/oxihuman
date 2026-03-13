// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! TorsoShape — chest/hip width and depth control.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Torso shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorsoShape {
    pub chest_width: f32,
    pub chest_depth: f32,
    pub hip_width: f32,
    pub hip_depth: f32,
    pub torso_height: f32,
}

impl Default for TorsoShape {
    fn default() -> Self {
        TorsoShape {
            chest_width: 1.0,
            chest_depth: 0.5,
            hip_width: 0.9,
            hip_depth: 0.45,
            torso_height: 2.0,
        }
    }
}

/// Create a default `TorsoShape`.
#[allow(dead_code)]
pub fn new_torso_shape() -> TorsoShape {
    TorsoShape::default()
}

/// Set chest width.
#[allow(dead_code)]
pub fn set_chest_width(ts: &mut TorsoShape, w: f32) {
    ts.chest_width = w;
}

/// Set chest depth.
#[allow(dead_code)]
pub fn set_chest_depth(ts: &mut TorsoShape, d: f32) {
    ts.chest_depth = d;
}

/// Set hip width.
#[allow(dead_code)]
pub fn set_hip_width(ts: &mut TorsoShape, w: f32) {
    ts.hip_width = w;
}

/// Approximate torso volume as a frustum with elliptical cross-sections.
#[allow(dead_code)]
pub fn torso_volume_approx(ts: &TorsoShape) -> f32 {
    let a_chest = PI * (ts.chest_width / 2.0) * (ts.chest_depth / 2.0);
    let a_hip = PI * (ts.hip_width / 2.0) * (ts.hip_depth / 2.0);
    // Prismatoid formula: V = h/6 * (A_top + A_bot + 4*A_mid)
    let a_mid = PI * ((ts.chest_width + ts.hip_width) / 4.0) * ((ts.chest_depth + ts.hip_depth) / 4.0);
    ts.torso_height / 6.0 * (a_chest + a_hip + 4.0 * a_mid)
}

/// Return the chest-width to hip-width aspect ratio.
#[allow(dead_code)]
pub fn torso_aspect(ts: &TorsoShape) -> f32 {
    ts.chest_width / ts.hip_width.max(f32::EPSILON)
}

/// Apply torso shape to a 6-element weight array.
/// Layout: [chest_w, chest_d, hip_w, hip_d, torso_h, aspect].
#[allow(dead_code)]
pub fn apply_torso_shape(ts: &TorsoShape, weights: &mut [f32]) {
    if weights.len() >= 5 {
        weights[0] = ts.chest_width;
        weights[1] = ts.chest_depth;
        weights[2] = ts.hip_width;
        weights[3] = ts.hip_depth;
        weights[4] = ts.torso_height;
    }
}

/// Serialise torso shape to a minimal JSON string.
#[allow(dead_code)]
pub fn torso_to_json(ts: &TorsoShape) -> String {
    format!(
        r#"{{"chest_width":{:.4},"chest_depth":{:.4},"hip_width":{:.4},"hip_depth":{:.4},"torso_height":{:.4}}}"#,
        ts.chest_width, ts.chest_depth, ts.hip_width, ts.hip_depth, ts.torso_height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_torso_defaults() {
        let ts = new_torso_shape();
        assert_eq!(ts.chest_width, 1.0);
    }

    #[test]
    fn test_set_chest_width() {
        let mut ts = new_torso_shape();
        set_chest_width(&mut ts, 1.3);
        assert!((ts.chest_width - 1.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_chest_depth() {
        let mut ts = new_torso_shape();
        set_chest_depth(&mut ts, 0.6);
        assert!((ts.chest_depth - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_hip_width() {
        let mut ts = new_torso_shape();
        set_hip_width(&mut ts, 1.1);
        assert!((ts.hip_width - 1.1).abs() < 1e-6);
    }

    #[test]
    fn test_torso_volume_positive() {
        let ts = new_torso_shape();
        assert!(torso_volume_approx(&ts) > 0.0);
    }

    #[test]
    fn test_torso_aspect_default() {
        let ts = new_torso_shape();
        let a = torso_aspect(&ts);
        assert!(a > 0.0);
    }

    #[test]
    fn test_apply_torso_shape() {
        let ts = new_torso_shape();
        let mut w = vec![0.0_f32; 5];
        apply_torso_shape(&ts, &mut w);
        assert!((w[0] - ts.chest_width).abs() < 1e-6);
    }

    #[test]
    fn test_torso_to_json_nonempty() {
        let ts = new_torso_shape();
        let j = torso_to_json(&ts);
        assert!(!j.is_empty());
        assert!(j.contains("chest_width"));
    }

    #[test]
    fn test_torso_volume_increases_with_width() {
        let ts1 = new_torso_shape();
        let v1 = torso_volume_approx(&ts1);
        let mut ts2 = new_torso_shape();
        set_chest_width(&mut ts2, 2.0);
        let v2 = torso_volume_approx(&ts2);
        assert!(v2 > v1);
    }
}
