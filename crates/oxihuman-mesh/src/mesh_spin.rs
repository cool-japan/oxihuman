// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Spin-around-axis modifier.

use std::f32::consts::PI;

/// Configuration for the spin modifier.
#[derive(Debug, Clone)]
pub struct SpinConfig {
    pub angle_degrees: f32,
    pub steps: usize,
    pub axis: [f32; 3],
    pub center: [f32; 3],
    pub merge_first_last: bool,
}

impl SpinConfig {
    pub fn new(angle_degrees: f32, steps: usize) -> Self {
        Self {
            angle_degrees,
            steps,
            axis: [0.0, 0.0, 1.0],
            center: [0.0, 0.0, 0.0],
            merge_first_last: false,
        }
    }
}

impl Default for SpinConfig {
    fn default() -> Self {
        Self::new(360.0, 16)
    }
}

/// Result of a spin operation.
#[derive(Debug, Clone, Default)]
pub struct SpinResult {
    pub profile_vertex_count: usize,
    pub total_vertex_count: usize,
    pub total_face_count: usize,
}

/// Rotate a single point around an axis by angle_radians.
pub fn rotate_point_around_axis(point: [f32; 3], axis: [f32; 3], angle_rad: f32) -> [f32; 3] {
    let (ax, ay, az) = (axis[0], axis[1], axis[2]);
    let len = (ax * ax + ay * ay + az * az).sqrt().max(1e-12);
    let (ux, uy, uz) = (ax / len, ay / len, az / len);
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let (px, py, pz) = (point[0], point[1], point[2]);
    let dot = ux * px + uy * py + uz * pz;
    [
        c * px + s * (uy * pz - uz * py) + (1.0 - c) * dot * ux,
        c * py + s * (uz * px - ux * pz) + (1.0 - c) * dot * uy,
        c * pz + s * (ux * py - uy * px) + (1.0 - c) * dot * uz,
    ]
}

/// Spin a profile (set of vertices) around an axis.
pub fn spin_profile(profile: &[[f32; 3]], cfg: &SpinConfig) -> SpinResult {
    let n = profile.len();
    let steps = cfg.steps.max(1);
    SpinResult {
        profile_vertex_count: n,
        total_vertex_count: n * (steps + if cfg.merge_first_last { 0 } else { 1 }),
        total_face_count: n.saturating_sub(1) * steps,
    }
}

/// Compute total angle in radians.
pub fn spin_angle_radians(cfg: &SpinConfig) -> f32 {
    cfg.angle_degrees * PI / 180.0
}

/// Validate spin config.
pub fn validate_spin_config(cfg: &SpinConfig) -> bool {
    cfg.steps > 0
        && cfg.angle_degrees.abs() > 0.0
        && (cfg.axis[0].powi(2) + cfg.axis[1].powi(2) + cfg.axis[2].powi(2)).sqrt() > 1e-6
}

/// Compute per-step angle increment in radians.
pub fn spin_step_angle(cfg: &SpinConfig) -> f32 {
    spin_angle_radians(cfg) / cfg.steps as f32
}

// ---- New API required by lib.rs ----

/// Spin modifier parameters (new API).
pub struct SpinParams {
    pub axis: [f32; 3],
    pub center: [f32; 3],
    pub angle_deg: f32,
    pub steps: usize,
}

pub fn new_spin_params(steps: usize, angle_deg: f32) -> SpinParams {
    SpinParams {
        axis: [0.0, 0.0, 1.0],
        center: [0.0; 3],
        angle_deg,
        steps: steps.max(1),
    }
}

pub fn spin_vertex(p: [f32; 3], params: &SpinParams, step: usize) -> [f32; 3] {
    let angle_rad = params.angle_deg * PI / 180.0 * step as f32 / params.steps.max(1) as f32;
    let ax = params.axis;
    let len = (ax[0] * ax[0] + ax[1] * ax[1] + ax[2] * ax[2])
        .sqrt()
        .max(1e-9);
    let ux = ax[0] / len;
    let uy = ax[1] / len;
    let uz = ax[2] / len;
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let dot = p[0] * ux + p[1] * uy + p[2] * uz;
    let cross = [
        uy * p[2] - uz * p[1],
        uz * p[0] - ux * p[2],
        ux * p[1] - uy * p[0],
    ];
    [
        p[0] * c + cross[0] * s + ux * dot * (1.0 - c),
        p[1] * c + cross[1] * s + uy * dot * (1.0 - c),
        p[2] * c + cross[2] * s + uz * dot * (1.0 - c),
    ]
}

pub fn spin_vertex_count(profile_count: usize, params: &SpinParams) -> usize {
    profile_count * (params.steps + 1)
}

pub fn spin_face_count(profile_count: usize, params: &SpinParams) -> usize {
    if profile_count < 2 {
        return 0;
    }
    (profile_count - 1) * params.steps
}

pub fn spin_is_closed(params: &SpinParams) -> bool {
    (params.angle_deg - 360.0).abs() < 0.5 || (params.angle_deg.abs() - 360.0).abs() < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spin_config_default() {
        let cfg = SpinConfig::default();
        assert_eq!(cfg.steps, 16);
        assert!((cfg.angle_degrees - 360.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotate_identity_zero_angle() {
        let p = [1.0_f32, 0.0, 0.0];
        let r = rotate_point_around_axis(p, [0.0, 0.0, 1.0], 0.0);
        assert!((r[0] - 1.0).abs() < 1e-5);
        assert!(r[1].abs() < 1e-5);
    }

    #[test]
    fn test_rotate_90_degrees_z() {
        let p = [1.0_f32, 0.0, 0.0];
        let r = rotate_point_around_axis(p, [0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        assert!(r[0].abs() < 1e-5);
        assert!((r[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_spin_profile_counts() {
        let profile = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let cfg = SpinConfig::new(360.0, 8);
        let res = spin_profile(&profile, &cfg);
        assert_eq!(res.profile_vertex_count, 3);
        assert_eq!(res.total_face_count, 2 * 8);
    }

    #[test]
    fn test_spin_angle_radians() {
        let cfg = SpinConfig::new(180.0, 8);
        let rad = spin_angle_radians(&cfg);
        assert!((rad - PI).abs() < 1e-5);
    }

    #[test]
    fn test_validate_spin_config_valid() {
        let cfg = SpinConfig::default();
        assert!(validate_spin_config(&cfg));
    }

    #[test]
    fn test_validate_spin_config_zero_steps() {
        let cfg = SpinConfig {
            steps: 0,
            ..SpinConfig::default()
        };
        assert!(!validate_spin_config(&cfg));
    }

    #[test]
    fn test_spin_step_angle() {
        let cfg = SpinConfig::new(360.0, 4);
        let step = spin_step_angle(&cfg);
        assert!((step - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_spin_merge_first_last() {
        let profile = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let cfg = SpinConfig {
            merge_first_last: true,
            ..SpinConfig::new(360.0, 4)
        };
        let res = spin_profile(&profile, &cfg);
        assert_eq!(res.total_vertex_count, 2 * 4);
    }

    #[test]
    fn test_spin_profile_single_vertex() {
        let profile = vec![[0.5_f32, 0.0, 0.0]];
        let cfg = SpinConfig::new(180.0, 3);
        let res = spin_profile(&profile, &cfg);
        assert_eq!(res.profile_vertex_count, 1);
        assert_eq!(res.total_face_count, 0);
    }
}
