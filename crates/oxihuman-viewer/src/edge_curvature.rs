// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge curvature — screen-space curvature estimation for stylized rendering.

/// Configuration for edge curvature detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCurvatureConfig {
    pub sample_radius: f32,
    pub curvature_scale: f32,
    pub threshold: f32,
    pub enabled: bool,
}

/// A curvature sample at a pixel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureSample {
    pub x: u32,
    pub y: u32,
    pub curvature: f32,
}

#[allow(dead_code)]
pub fn default_edge_curvature_config() -> EdgeCurvatureConfig {
    EdgeCurvatureConfig {
        sample_radius: 2.0,
        curvature_scale: 1.0,
        threshold: 0.2,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn ec_set_radius(cfg: &mut EdgeCurvatureConfig, r: f32) {
    cfg.sample_radius = r.max(0.5);
}

#[allow(dead_code)]
pub fn ec_set_threshold(cfg: &mut EdgeCurvatureConfig, t: f32) {
    cfg.threshold = t.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ec_set_enabled(cfg: &mut EdgeCurvatureConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn ec_estimate_curvature(normal_a: [f32; 3], normal_b: [f32; 3]) -> f32 {
    let dot = normal_a[0] * normal_b[0] + normal_a[1] * normal_b[1] + normal_a[2] * normal_b[2];
    (1.0 - dot.clamp(-1.0, 1.0)) * 0.5
}

#[allow(dead_code)]
pub fn ec_is_crease(cfg: &EdgeCurvatureConfig, curvature: f32) -> bool {
    cfg.enabled && curvature >= cfg.threshold
}

#[allow(dead_code)]
pub fn ec_scale_curvature(cfg: &EdgeCurvatureConfig, raw: f32) -> f32 {
    (raw * cfg.curvature_scale).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn ec_build_samples(normals: &[[f32; 3]], width: u32) -> Vec<CurvatureSample> {
    let mut out = Vec::new();
    if normals.is_empty() {
        return out;
    }
    let n = normals.len() as u32;
    for i in 1..n {
        let c = ec_estimate_curvature(normals[(i - 1) as usize], normals[i as usize]);
        out.push(CurvatureSample {
            x: i % width.max(1),
            y: i / width.max(1),
            curvature: c,
        });
    }
    out
}

#[allow(dead_code)]
pub fn ec_to_json(cfg: &EdgeCurvatureConfig) -> String {
    format!(
        r#"{{"radius":{:.4},"scale":{:.4},"threshold":{:.4},"enabled":{}}}"#,
        cfg.sample_radius, cfg.curvature_scale, cfg.threshold, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_edge_curvature_config();
        assert!((cfg.sample_radius - 2.0).abs() < 1e-6);
    }

    #[test]
    fn set_radius() {
        let mut cfg = default_edge_curvature_config();
        ec_set_radius(&mut cfg, 3.0);
        assert!((cfg.sample_radius - 3.0).abs() < 1e-6);
    }

    #[test]
    fn parallel_normals_zero_curvature() {
        let n = [0.0_f32, 0.0, 1.0];
        let c = ec_estimate_curvature(n, n);
        assert!(c.abs() < 1e-6);
    }

    #[test]
    fn opposite_normals_max_curvature() {
        let a = [0.0_f32, 0.0, 1.0];
        let b = [0.0_f32, 0.0, -1.0];
        let c = ec_estimate_curvature(a, b);
        assert!((c - 1.0).abs() < 1e-5);
    }

    #[test]
    fn is_crease_above_threshold() {
        let cfg = default_edge_curvature_config();
        assert!(ec_is_crease(&cfg, 0.5));
    }

    #[test]
    fn not_crease_below_threshold() {
        let cfg = default_edge_curvature_config();
        assert!(!ec_is_crease(&cfg, 0.1));
    }

    #[test]
    fn disabled_never_crease() {
        let mut cfg = default_edge_curvature_config();
        ec_set_enabled(&mut cfg, false);
        assert!(!ec_is_crease(&cfg, 1.0));
    }

    #[test]
    fn build_samples_count() {
        let normals = vec![[0.0_f32, 0.0, 1.0]; 5];
        let samples = ec_build_samples(&normals, 4);
        assert_eq!(samples.len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_edge_curvature_config();
        let j = ec_to_json(&cfg);
        assert!(j.contains("radius"));
    }
}
