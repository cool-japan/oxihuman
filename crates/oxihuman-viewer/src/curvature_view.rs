// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mean/Gaussian curvature visualization.

/// Curvature type to display.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurvatureType {
    Mean,
    Gaussian,
    MinPrincipal,
    MaxPrincipal,
}

impl CurvatureType {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            CurvatureType::Mean => "mean",
            CurvatureType::Gaussian => "gaussian",
            CurvatureType::MinPrincipal => "min_principal",
            CurvatureType::MaxPrincipal => "max_principal",
        }
    }
}

/// Curvature view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureConfig {
    pub curvature_type: CurvatureType,
    pub scale: f32,
    pub color_negative: [f32; 3],
    pub color_zero: [f32; 3],
    pub color_positive: [f32; 3],
    pub enabled: bool,
}

impl Default for CurvatureConfig {
    fn default() -> Self {
        CurvatureConfig {
            curvature_type: CurvatureType::Mean,
            scale: 1.0,
            color_negative: [0.0, 0.0, 1.0],
            color_zero: [0.5, 0.5, 0.5],
            color_positive: [1.0, 0.0, 0.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_curvature_config() -> CurvatureConfig {
    CurvatureConfig::default()
}

#[allow(dead_code)]
pub fn cv_enable(cfg: &mut CurvatureConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn cv_disable(cfg: &mut CurvatureConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn cv_set_type(cfg: &mut CurvatureConfig, t: CurvatureType) {
    cfg.curvature_type = t;
}

#[allow(dead_code)]
pub fn cv_set_scale(cfg: &mut CurvatureConfig, s: f32) {
    cfg.scale = s.clamp(0.001, 1000.0);
}

/// Map a curvature value to a color (diverging blue-gray-red).
#[allow(dead_code)]
pub fn cv_curvature_to_color(cfg: &CurvatureConfig, k: f32) -> [f32; 3] {
    let t = (k * cfg.scale).tanh();
    if t >= 0.0 {
        [
            cfg.color_zero[0] + (cfg.color_positive[0] - cfg.color_zero[0]) * t,
            cfg.color_zero[1] + (cfg.color_positive[1] - cfg.color_zero[1]) * t,
            cfg.color_zero[2] + (cfg.color_positive[2] - cfg.color_zero[2]) * t,
        ]
    } else {
        let s = -t;
        [
            cfg.color_zero[0] + (cfg.color_negative[0] - cfg.color_zero[0]) * s,
            cfg.color_zero[1] + (cfg.color_negative[1] - cfg.color_zero[1]) * s,
            cfg.color_zero[2] + (cfg.color_negative[2] - cfg.color_zero[2]) * s,
        ]
    }
}

/// Approximate mean curvature from principal curvatures.
#[allow(dead_code)]
pub fn cv_mean_curvature(k1: f32, k2: f32) -> f32 {
    (k1 + k2) * 0.5
}

/// Compute Gaussian curvature from principal curvatures.
#[allow(dead_code)]
pub fn cv_gaussian_curvature(k1: f32, k2: f32) -> f32 {
    k1 * k2
}

#[allow(dead_code)]
pub fn cv_to_json(cfg: &CurvatureConfig) -> String {
    format!(
        r#"{{"type":"{}","scale":{:.4},"enabled":{}}}"#,
        cfg.curvature_type.name(),
        cfg.scale,
        cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_curvature_config().enabled);
    }

    #[test]
    fn zero_curvature_gray() {
        let cfg = default_curvature_config();
        let c = cv_curvature_to_color(&cfg, 0.0);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn positive_curvature_red_component() {
        let cfg = default_curvature_config();
        let c = cv_curvature_to_color(&cfg, 10.0);
        assert!(c[0] > 0.5);
    }

    #[test]
    fn negative_curvature_blue_component() {
        let cfg = default_curvature_config();
        let c = cv_curvature_to_color(&cfg, -10.0);
        assert!(c[2] > 0.5);
    }

    #[test]
    fn mean_curvature_average() {
        assert!((cv_mean_curvature(1.0, 3.0) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn gaussian_curvature_product() {
        assert!((cv_gaussian_curvature(2.0, 3.0) - 6.0).abs() < 1e-6);
    }

    #[test]
    fn set_scale_clamps() {
        let mut cfg = default_curvature_config();
        cv_set_scale(&mut cfg, 0.0);
        assert!((cfg.scale - 0.001).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_curvature_config();
        cv_enable(&mut cfg);
        assert!(cfg.enabled);
        cv_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_type() {
        assert!(cv_to_json(&default_curvature_config()).contains("type"));
    }
}
