// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Depth resolve — resolving multi-sample depth buffers to single-sample.

use std::f32::consts::FRAC_PI_8;

/// Resolve method.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthResolveMethod {
    /// Take the minimum (closest) sample.
    Min,
    /// Take the maximum (farthest) sample.
    Max,
    /// Average all samples.
    Average,
}

/// Configuration for depth resolve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthResolveConfig {
    pub method: DepthResolveMethod,
    /// Number of MSAA samples (2, 4, 8).
    pub sample_count: u32,
    /// Whether reversed-Z is used.
    pub reversed_z: bool,
    /// Reference angle (informational).
    pub ref_angle_rad: f32,
}

impl Default for DepthResolveConfig {
    fn default() -> Self {
        DepthResolveConfig {
            method: DepthResolveMethod::Min,
            sample_count: 4,
            reversed_z: false,
            ref_angle_rad: FRAC_PI_8,
        }
    }
}

/// Default config.
pub fn default_depth_resolve_config() -> DepthResolveConfig {
    DepthResolveConfig::default()
}

/// New config with a given sample count and method.
pub fn new_depth_resolve(sample_count: u32, method: DepthResolveMethod) -> DepthResolveConfig {
    DepthResolveConfig {
        sample_count,
        method,
        ..DepthResolveConfig::default()
    }
}

/// Set the resolve method.
pub fn dr_set_method(cfg: &mut DepthResolveConfig, method: DepthResolveMethod) {
    cfg.method = method;
}

/// Set reversed-Z.
pub fn dr_set_reversed_z(cfg: &mut DepthResolveConfig, v: bool) {
    cfg.reversed_z = v;
}

/// Return method name.
pub fn dr_method_name(cfg: &DepthResolveConfig) -> &'static str {
    match cfg.method {
        DepthResolveMethod::Min => "min",
        DepthResolveMethod::Max => "max",
        DepthResolveMethod::Average => "average",
    }
}

/// Resolve a slice of depth samples using the configured method.
pub fn dr_resolve(cfg: &DepthResolveConfig, samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return if cfg.reversed_z { 0.0 } else { 1.0 };
    }
    match cfg.method {
        DepthResolveMethod::Min => samples.iter().cloned().fold(f32::MAX, f32::min),
        DepthResolveMethod::Max => samples.iter().cloned().fold(f32::MIN, f32::max),
        DepthResolveMethod::Average => samples.iter().sum::<f32>() / samples.len() as f32,
    }
}

/// Linearise a resolved depth value.
///
/// `near` and `far` are clip plane distances.
pub fn dr_linearize(cfg: &DepthResolveConfig, depth: f32, near: f32, far: f32) -> f32 {
    let d = depth.clamp(0.0, 1.0);
    let d = if cfg.reversed_z { 1.0 - d } else { d };
    near * far / (far - d * (far - near))
}

/// Estimate memory saved vs. storing all samples (bytes).
pub fn dr_memory_saved(cfg: &DepthResolveConfig, pixel_count: u64) -> u64 {
    let bytes_per_sample = 4u64; // f32 depth
    pixel_count * bytes_per_sample * (cfg.sample_count as u64 - 1)
}

/// Serialise.
pub fn dr_to_json(cfg: &DepthResolveConfig) -> String {
    format!(
        r#"{{"method":"{}","samples":{},"reversed_z":{}}}"#,
        dr_method_name(cfg),
        cfg.sample_count,
        cfg.reversed_z
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_method_is_min() {
        assert_eq!(
            default_depth_resolve_config().method,
            DepthResolveMethod::Min
        );
    }

    #[test]
    fn resolve_min() {
        let cfg = new_depth_resolve(4, DepthResolveMethod::Min);
        let v = dr_resolve(&cfg, &[0.9, 0.5, 0.7]);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn resolve_max() {
        let cfg = new_depth_resolve(4, DepthResolveMethod::Max);
        let v = dr_resolve(&cfg, &[0.9, 0.5, 0.7]);
        assert!((v - 0.9).abs() < 1e-5);
    }

    #[test]
    fn resolve_average() {
        let cfg = new_depth_resolve(4, DepthResolveMethod::Average);
        let v = dr_resolve(&cfg, &[0.0, 1.0]);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn empty_samples_returns_far() {
        let cfg = default_depth_resolve_config();
        assert!((dr_resolve(&cfg, &[]) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn linearize_in_range() {
        let cfg = default_depth_resolve_config();
        let v = dr_linearize(&cfg, 0.5, 0.1, 100.0);
        assert!(v > 0.0 && v < 100.0);
    }

    #[test]
    fn method_name_correct() {
        let cfg = new_depth_resolve(4, DepthResolveMethod::Average);
        assert_eq!(dr_method_name(&cfg), "average");
    }

    #[test]
    fn memory_saved_nonzero() {
        let cfg = new_depth_resolve(4, DepthResolveMethod::Min);
        assert!(dr_memory_saved(&cfg, 1920 * 1080) > 0);
    }

    #[test]
    fn json_has_method() {
        assert!(dr_to_json(&default_depth_resolve_config()).contains("method"));
    }
}
