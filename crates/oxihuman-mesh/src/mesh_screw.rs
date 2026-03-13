// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Screw/lathe modifier — revolve a profile around an axis.

/// Parameters for the screw operation.
#[derive(Debug, Clone)]
pub struct ScrewParams {
    /// Total angle of revolution in radians.
    pub angle: f32,
    /// Pitch (height advance per full revolution).
    pub pitch: f32,
    /// Number of steps around the axis.
    pub steps: usize,
    /// Axis index: 0=X, 1=Y, 2=Z.
    pub axis: usize,
}

impl Default for ScrewParams {
    fn default() -> Self {
        Self { angle: std::f32::consts::TAU, pitch: 0.0, steps: 16, axis: 1 }
    }
}

/// Result of the screw operation.
#[derive(Debug, Clone, Default)]
pub struct ScrewResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Revolve a 2D profile (list of positions) around the specified axis.
pub fn screw_profile(profile: &[[f32; 3]], params: &ScrewParams) -> ScrewResult {
    let n = profile.len();
    if n == 0 || params.steps == 0 {
        return ScrewResult::default();
    }
    let steps = params.steps;
    let mut out_pos: Vec<[f32; 3]> = Vec::with_capacity(n * (steps + 1));
    let mut out_idx: Vec<u32> = Vec::new();

    for s in 0..=steps {
        let frac = s as f32 / steps as f32;
        let theta = params.angle * frac;
        let height = params.pitch * frac;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        for &p in profile {
            let rotated = rotate_around_axis(p, params.axis, cos_t, sin_t, height);
            out_pos.push(rotated);
        }
    }

    /* generate quad faces between consecutive rings */
    for s in 0..steps {
        let ring0 = (s * n) as u32;
        let ring1 = ((s + 1) * n) as u32;
        for k in 0..(n.saturating_sub(1)) {
            let a = ring0 + k as u32;
            let b = ring0 + k as u32 + 1;
            let c = ring1 + k as u32 + 1;
            let d = ring1 + k as u32;
            out_idx.extend_from_slice(&[a, b, c]);
            out_idx.extend_from_slice(&[a, c, d]);
        }
    }

    ScrewResult { positions: out_pos, indices: out_idx }
}

/// Default screw parameters.
pub fn default_screw_params() -> ScrewParams {
    ScrewParams::default()
}

/// Number of vertices in the result.
pub fn screw_vertex_count(r: &ScrewResult) -> usize {
    r.positions.len()
}

/// Number of triangles in the result.
pub fn screw_triangle_count(r: &ScrewResult) -> usize {
    r.indices.len() / 3
}

/// Approximate height extent of the screw result.
pub fn screw_height_extent(r: &ScrewResult) -> f32 {
    if r.positions.is_empty() { return 0.0; }
    let min_y: f32 = r.positions.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let max_y: f32 = r.positions.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
    max_y - min_y
}

fn rotate_around_axis(p: [f32; 3], axis: usize, cos_t: f32, sin_t: f32, height: f32) -> [f32; 3] {
    match axis % 3 {
        0 => {
            /* rotate around X: y/z plane */
            [p[0] + height, p[1] * cos_t - p[2] * sin_t, p[1] * sin_t + p[2] * cos_t]
        }
        1 => {
            /* rotate around Y: x/z plane */
            [p[0] * cos_t + p[2] * sin_t, p[1] + height, -p[0] * sin_t + p[2] * cos_t]
        }
        _ => {
            /* rotate around Z: x/y plane */
            [p[0] * cos_t - p[1] * sin_t, p[0] * sin_t + p[1] * cos_t, p[2] + height]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_profile() -> Vec<[f32; 3]> {
        vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 2.0, 0.0]]
    }

    #[test]
    fn test_screw_vertex_count() {
        /* profile of 3 × (steps+1) rings */
        let p = simple_profile();
        let params = ScrewParams { steps: 8, ..ScrewParams::default() };
        let r = screw_profile(&p, &params);
        assert_eq!(screw_vertex_count(&r), 3 * 9);
    }

    #[test]
    fn test_screw_triangle_count_positive() {
        /* should have non-zero faces */
        let p = simple_profile();
        let r = screw_profile(&p, &default_screw_params());
        assert!(screw_triangle_count(&r) > 0);
    }

    #[test]
    fn test_screw_empty_profile() {
        /* empty profile yields empty result */
        let r = screw_profile(&[], &default_screw_params());
        assert_eq!(screw_vertex_count(&r), 0);
    }

    #[test]
    fn test_screw_zero_steps() {
        /* zero steps yields empty result */
        let p = simple_profile();
        let params = ScrewParams { steps: 0, ..ScrewParams::default() };
        let r = screw_profile(&p, &params);
        assert_eq!(screw_vertex_count(&r), 0);
    }

    #[test]
    fn test_screw_with_pitch_increases_height() {
        /* pitch > 0 should increase height extent */
        let p = simple_profile();
        let no_pitch = ScrewParams { pitch: 0.0, ..ScrewParams::default() };
        let with_pitch = ScrewParams { pitch: 2.0, ..ScrewParams::default() };
        let r0 = screw_profile(&p, &no_pitch);
        let r1 = screw_profile(&p, &with_pitch);
        assert!(screw_height_extent(&r1) > screw_height_extent(&r0));
    }

    #[test]
    fn test_screw_default_params() {
        /* default params are reasonable */
        let dp = default_screw_params();
        assert!(dp.steps > 0);
        assert!((dp.angle - std::f32::consts::TAU).abs() < 1e-5);
    }

    #[test]
    fn test_rotate_around_y_identity_at_zero() {
        /* at theta=0, rotation is identity */
        let p = [2.0, 0.0, 0.0];
        let r = rotate_around_axis(p, 1, 1.0, 0.0, 0.0);
        assert!((r[0] - 2.0).abs() < 1e-5);
        assert!(r[1].abs() < 1e-5);
    }

    #[test]
    fn test_screw_axis_z() {
        /* axis=2 should also work without panic */
        let p = simple_profile();
        let params = ScrewParams { axis: 2, steps: 4, ..ScrewParams::default() };
        let r = screw_profile(&p, &params);
        assert!(screw_vertex_count(&r) > 0);
    }

    #[test]
    fn test_height_extent_no_pitch_near_zero() {
        /* no-pitch full revolution has near-zero Y change for XY profile */
        let p = vec![[1.0, 0.0, 0.0]];
        let params = ScrewParams { pitch: 0.0, steps: 32, angle: std::f32::consts::TAU, axis: 1 };
        let r = screw_profile(&p, &params);
        assert!(screw_height_extent(&r) < 1e-4);
    }
}
