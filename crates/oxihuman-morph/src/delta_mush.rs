// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Delta mush smoothing deformer stub.

/// Configuration for the delta mush deformer.
#[derive(Debug, Clone)]
pub struct DeltaMushConfig {
    /// Number of smoothing iterations.
    pub iterations: usize,
    /// Smoothing amount per iteration.
    pub smoothing: f32,
    /// Delta scale factor.
    pub delta_scale: f32,
}

impl Default for DeltaMushConfig {
    fn default() -> Self {
        DeltaMushConfig {
            iterations: 10,
            smoothing: 0.5,
            delta_scale: 1.0,
        }
    }
}

/// Delta mush deformer state.
#[derive(Debug, Clone)]
pub struct DeltaMush {
    pub config: DeltaMushConfig,
    /// Cached smoothed positions per vertex.
    pub smoothed: Vec<[f32; 3]>,
}

impl DeltaMush {
    pub fn new(vertex_count: usize) -> Self {
        DeltaMush {
            config: DeltaMushConfig::default(),
            smoothed: vec![[0.0; 3]; vertex_count],
        }
    }
}

/// Create a new delta mush deformer.
pub fn new_delta_mush(vertex_count: usize) -> DeltaMush {
    DeltaMush::new(vertex_count)
}

/// Smooth positions using a simple Laplacian pass (stub: just blends toward zero).
#[allow(clippy::needless_range_loop)]
pub fn delta_mush_smooth(dm: &mut DeltaMush, positions: &[[f32; 3]]) {
    let n = dm.smoothed.len().min(positions.len());
    for i in 0..n {
        let p = positions[i];
        let s = dm.smoothed[i];
        let t = dm.config.smoothing;
        dm.smoothed[i] = [
            s[0] + t * (p[0] - s[0]),
            s[1] + t * (p[1] - s[1]),
            s[2] + t * (p[2] - s[2]),
        ];
    }
}

/// Return the vertex count.
pub fn delta_mush_vertex_count(dm: &DeltaMush) -> usize {
    dm.smoothed.len()
}

/// Return a JSON-like string.
pub fn delta_mush_to_json(dm: &DeltaMush) -> String {
    format!(
        r#"{{"iterations":{},"smoothing":{:.4},"delta_scale":{:.4},"vertices":{}}}"#,
        dm.config.iterations,
        dm.config.smoothing,
        dm.config.delta_scale,
        dm.smoothed.len()
    )
}

/// Reset all smoothed positions to zero.
pub fn delta_mush_reset(dm: &mut DeltaMush) {
    for s in &mut dm.smoothed {
        *s = [0.0; 3];
    }
}

/// Set the smoothing amount.
pub fn delta_mush_set_smoothing(dm: &mut DeltaMush, smoothing: f32) {
    dm.config.smoothing = smoothing.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_delta_mush_vertex_count() {
        let dm = new_delta_mush(20);
        assert_eq!(
            delta_mush_vertex_count(&dm),
            20, /* vertex count must match */
        );
    }

    #[test]
    fn test_default_iterations() {
        let dm = new_delta_mush(5);
        assert_eq!(dm.config.iterations, 10 /* default iterations is 10 */,);
    }

    #[test]
    fn test_smooth_moves_toward_target() {
        let mut dm = new_delta_mush(1);
        delta_mush_smooth(&mut dm, &[[2.0, 0.0, 0.0]]);
        assert!(dm.smoothed[0][0] > 0.0, /* smoothing should move toward target */);
    }

    #[test]
    fn test_reset_zeroes_positions() {
        let mut dm = new_delta_mush(3);
        delta_mush_smooth(
            &mut dm,
            &[[1.0, 1.0, 1.0], [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]],
        );
        delta_mush_reset(&mut dm);
        for s in &dm.smoothed {
            assert!((s[0]).abs() < 1e-6, /* reset should zero all positions */);
        }
    }

    #[test]
    fn test_set_smoothing_clamps() {
        let mut dm = new_delta_mush(2);
        delta_mush_set_smoothing(&mut dm, 2.0);
        assert!((dm.config.smoothing - 1.0).abs() < 1e-5, /* smoothing clamped to 1 */);
    }

    #[test]
    fn test_set_smoothing_negative_clamps() {
        let mut dm = new_delta_mush(2);
        delta_mush_set_smoothing(&mut dm, -1.0);
        assert!((dm.config.smoothing).abs() < 1e-6, /* negative smoothing clamped to 0 */);
    }

    #[test]
    fn test_to_json_contains_iterations() {
        let dm = new_delta_mush(4);
        let j = delta_mush_to_json(&dm);
        assert!(j.contains("iterations"), /* JSON must contain iterations */);
    }

    #[test]
    fn test_smoothed_initialized_zero() {
        let dm = new_delta_mush(5);
        for s in &dm.smoothed {
            assert!((s[0]).abs() < 1e-6, /* initial smoothed values are zero */);
        }
    }

    #[test]
    fn test_smooth_ignores_extra_positions() {
        let mut dm = new_delta_mush(2);
        delta_mush_smooth(&mut dm, &[[1.0, 0.0, 0.0]; 10]);
        assert_eq!(
            delta_mush_vertex_count(&dm),
            2, /* vertex count unchanged */
        );
    }

    #[test]
    fn test_delta_scale_default_one() {
        let dm = new_delta_mush(1);
        assert!((dm.config.delta_scale - 1.0).abs() < 1e-5, /* default delta scale is 1.0 */);
    }

    #[test]
    fn test_to_json_contains_vertices() {
        let dm = new_delta_mush(7);
        let j = delta_mush_to_json(&dm);
        assert!(j.contains("7") /* JSON should contain vertex count */,);
    }
}
