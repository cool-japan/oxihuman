// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh pole (high-valence vertex) visualization.

/// Pole view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoleViewConfig {
    pub normal_valence: u32,
    pub high_valence_threshold: u32,
    pub color_normal: [f32; 3],
    pub color_high: [f32; 3],
    pub color_low: [f32; 3],
    pub point_size: f32,
    pub enabled: bool,
}

impl Default for PoleViewConfig {
    fn default() -> Self {
        PoleViewConfig {
            normal_valence: 4,
            high_valence_threshold: 5,
            color_normal: [0.3, 0.3, 0.3],
            color_high: [1.0, 0.2, 0.0],
            color_low: [0.0, 0.4, 1.0],
            point_size: 6.0,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_pole_view_config() -> PoleViewConfig {
    PoleViewConfig::default()
}

#[allow(dead_code)]
pub fn pv_enable(cfg: &mut PoleViewConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn pv_disable(cfg: &mut PoleViewConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn pv_set_point_size(cfg: &mut PoleViewConfig, s: f32) {
    cfg.point_size = s.clamp(1.0, 50.0);
}

#[allow(dead_code)]
pub fn pv_set_high_threshold(cfg: &mut PoleViewConfig, n: u32) {
    cfg.high_valence_threshold = n.max(1);
}

/// Determine pole type for a vertex with given valence.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoleKind {
    Normal,
    High,
    Low,
}

#[allow(dead_code)]
pub fn pv_classify_pole(cfg: &PoleViewConfig, valence: u32) -> PoleKind {
    if valence >= cfg.high_valence_threshold {
        PoleKind::High
    } else if valence < cfg.normal_valence {
        PoleKind::Low
    } else {
        PoleKind::Normal
    }
}

/// Color for a vertex based on its pole kind.
#[allow(dead_code)]
pub fn pv_pole_color(cfg: &PoleViewConfig, kind: PoleKind) -> [f32; 3] {
    match kind {
        PoleKind::Normal => cfg.color_normal,
        PoleKind::High => cfg.color_high,
        PoleKind::Low => cfg.color_low,
    }
}

/// Count high-valence poles in a valence list.
#[allow(dead_code)]
pub fn pv_count_poles(cfg: &PoleViewConfig, valences: &[u32]) -> (usize, usize) {
    let high = valences
        .iter()
        .filter(|&&v| v >= cfg.high_valence_threshold)
        .count();
    let low = valences.iter().filter(|&&v| v < cfg.normal_valence).count();
    (high, low)
}

#[allow(dead_code)]
pub fn pv_to_json(cfg: &PoleViewConfig) -> String {
    format!(
        r#"{{"high_valence_threshold":{},"point_size":{:.4},"enabled":{}}}"#,
        cfg.high_valence_threshold, cfg.point_size, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_pole_view_config().enabled);
    }

    #[test]
    fn classify_normal() {
        let cfg = default_pole_view_config();
        assert_eq!(pv_classify_pole(&cfg, 4), PoleKind::Normal);
    }

    #[test]
    fn classify_high() {
        let cfg = default_pole_view_config();
        assert_eq!(pv_classify_pole(&cfg, 6), PoleKind::High);
    }

    #[test]
    fn classify_low() {
        let cfg = default_pole_view_config();
        assert_eq!(pv_classify_pole(&cfg, 3), PoleKind::Low);
    }

    #[test]
    fn pole_color_high() {
        let cfg = default_pole_view_config();
        let c = pv_pole_color(&cfg, PoleKind::High);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn count_poles() {
        let cfg = default_pole_view_config();
        let valences = vec![3u32, 4, 5, 6, 4, 3];
        let (high, low) = pv_count_poles(&cfg, &valences);
        assert_eq!(high, 2);
        assert_eq!(low, 2);
    }

    #[test]
    fn point_size_clamps() {
        let mut cfg = default_pole_view_config();
        pv_set_point_size(&mut cfg, 0.0);
        assert!((cfg.point_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_pole_view_config();
        pv_enable(&mut cfg);
        assert!(cfg.enabled);
        pv_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_threshold() {
        assert!(pv_to_json(&default_pole_view_config()).contains("high_valence_threshold"));
    }
}
