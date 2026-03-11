// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV stretch distortion visualization.

/// Stretch metric type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StretchMetric {
    Linear,
    Area,
    Conformal,
    Isometric,
}

impl StretchMetric {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            StretchMetric::Linear => "linear",
            StretchMetric::Area => "area",
            StretchMetric::Conformal => "conformal",
            StretchMetric::Isometric => "isometric",
        }
    }
}

/// UV stretch visualization config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StretchConfig {
    pub metric: StretchMetric,
    pub threshold: f32,
    pub color_ok: [f32; 3],
    pub color_stretch: [f32; 3],
    pub color_compress: [f32; 3],
    pub enabled: bool,
}

impl Default for StretchConfig {
    fn default() -> Self {
        StretchConfig {
            metric: StretchMetric::Linear,
            threshold: 0.1,
            color_ok: [0.2, 0.8, 0.2],
            color_stretch: [1.0, 0.2, 0.0],
            color_compress: [0.0, 0.2, 1.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_stretch_config() -> StretchConfig {
    StretchConfig::default()
}

#[allow(dead_code)]
pub fn sv_enable(cfg: &mut StretchConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn sv_disable(cfg: &mut StretchConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn sv_set_threshold(cfg: &mut StretchConfig, v: f32) {
    cfg.threshold = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sv_set_metric(cfg: &mut StretchConfig, m: StretchMetric) {
    cfg.metric = m;
}

/// Compute area stretch ratio: 3D area / UV area.
#[allow(dead_code)]
pub fn sv_area_ratio(area_3d: f32, area_uv: f32) -> f32 {
    if area_uv < 1e-10 {
        1.0
    } else {
        area_3d / area_uv
    }
}

/// Map stretch value to color.
#[allow(dead_code)]
pub fn sv_stretch_to_color(cfg: &StretchConfig, ratio: f32) -> [f32; 3] {
    let deviation = (ratio - 1.0).abs();
    if deviation <= cfg.threshold {
        return cfg.color_ok;
    }
    let t = ((deviation - cfg.threshold) / (1.0 - cfg.threshold)).clamp(0.0, 1.0);
    if ratio > 1.0 {
        let s = t;
        [
            cfg.color_ok[0] + (cfg.color_stretch[0] - cfg.color_ok[0]) * s,
            cfg.color_ok[1] + (cfg.color_stretch[1] - cfg.color_ok[1]) * s,
            cfg.color_ok[2] + (cfg.color_stretch[2] - cfg.color_ok[2]) * s,
        ]
    } else {
        let s = t;
        [
            cfg.color_ok[0] + (cfg.color_compress[0] - cfg.color_ok[0]) * s,
            cfg.color_ok[1] + (cfg.color_compress[1] - cfg.color_ok[1]) * s,
            cfg.color_ok[2] + (cfg.color_compress[2] - cfg.color_ok[2]) * s,
        ]
    }
}

/// Maximum stretch in a list.
#[allow(dead_code)]
pub fn sv_max_stretch(ratios: &[f32]) -> f32 {
    ratios.iter().cloned().fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn sv_to_json(cfg: &StretchConfig) -> String {
    format!(
        r#"{{"metric":"{}","threshold":{:.4},"enabled":{}}}"#,
        cfg.metric.name(),
        cfg.threshold,
        cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_stretch_config().enabled);
    }

    #[test]
    fn area_ratio_no_uv() {
        assert!((sv_area_ratio(1.0, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn area_ratio_unity() {
        assert!((sv_area_ratio(0.5, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_ok_at_ratio_one() {
        let cfg = default_stretch_config();
        let c = sv_stretch_to_color(&cfg, 1.0);
        assert!((c[1] - cfg.color_ok[1]).abs() < 1e-6);
    }

    #[test]
    fn color_stretch_at_high_ratio() {
        let cfg = default_stretch_config();
        let c = sv_stretch_to_color(&cfg, 10.0);
        assert!(c[0] > cfg.color_ok[0]);
    }

    #[test]
    fn color_compress_at_low_ratio() {
        let cfg = default_stretch_config();
        let c = sv_stretch_to_color(&cfg, 0.01);
        assert!(c[2] > cfg.color_ok[2]);
    }

    #[test]
    fn max_stretch_empty() {
        assert!(sv_max_stretch(&[]).abs() < 1e-6);
    }

    #[test]
    fn max_stretch_value() {
        let r = vec![0.5f32, 1.0, 2.0, 1.5];
        assert!((sv_max_stretch(&r) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_stretch_config();
        sv_enable(&mut cfg);
        assert!(cfg.enabled);
        sv_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_metric() {
        assert!(sv_to_json(&default_stretch_config()).contains("metric"));
    }
}
