// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge length heat map visualization.

/// Edge length heat map configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLengthConfig {
    pub min_length: f32,
    pub max_length: f32,
    pub color_short: [f32; 3],
    pub color_long: [f32; 3],
    pub enabled: bool,
}

impl Default for EdgeLengthConfig {
    fn default() -> Self {
        EdgeLengthConfig {
            min_length: 0.0,
            max_length: 1.0,
            color_short: [0.0, 0.0, 1.0],
            color_long: [1.0, 0.0, 0.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_edge_length_config() -> EdgeLengthConfig {
    EdgeLengthConfig::default()
}

#[allow(dead_code)]
pub fn el_enable(cfg: &mut EdgeLengthConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn el_disable(cfg: &mut EdgeLengthConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn el_set_range(cfg: &mut EdgeLengthConfig, min: f32, max: f32) {
    cfg.min_length = min.max(0.0);
    cfg.max_length = max.max(cfg.min_length + 1e-6);
}

/// Compute edge length between two 3D points.
#[allow(dead_code)]
pub fn el_edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Map an edge length to a heat map color.
#[allow(dead_code)]
pub fn el_length_to_color(cfg: &EdgeLengthConfig, length: f32) -> [f32; 3] {
    let range = cfg.max_length - cfg.min_length;
    let t = if range < 1e-10 {
        0.5
    } else {
        ((length - cfg.min_length) / range).clamp(0.0, 1.0)
    };
    [
        cfg.color_short[0] + (cfg.color_long[0] - cfg.color_short[0]) * t,
        cfg.color_short[1] + (cfg.color_long[1] - cfg.color_short[1]) * t,
        cfg.color_short[2] + (cfg.color_long[2] - cfg.color_short[2]) * t,
    ]
}

/// Compute statistics for a list of edge lengths.
#[allow(dead_code)]
pub fn el_stats(lengths: &[f32]) -> (f32, f32, f32) {
    if lengths.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let min = lengths.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = lengths.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let avg = lengths.iter().sum::<f32>() / lengths.len() as f32;
    (min, max, avg)
}

#[allow(dead_code)]
pub fn el_to_json(cfg: &EdgeLengthConfig) -> String {
    format!(
        r#"{{"min_length":{:.4},"max_length":{:.4},"enabled":{}}}"#,
        cfg.min_length, cfg.max_length, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_edge_length_config().enabled);
    }

    #[test]
    fn edge_length_unit() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        assert!((el_edge_length(a, b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn edge_length_diagonal() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 1.0, 1.0];
        let len = el_edge_length(a, b);
        assert!((len - 3.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn color_at_min() {
        let cfg = default_edge_length_config();
        let c = el_length_to_color(&cfg, 0.0);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_at_max() {
        let cfg = default_edge_length_config();
        let c = el_length_to_color(&cfg, 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn stats_empty() {
        let (mn, mx, avg) = el_stats(&[]);
        assert!(mn.abs() < 1e-6 && mx.abs() < 1e-6 && avg.abs() < 1e-6);
    }

    #[test]
    fn stats_single() {
        let (mn, mx, avg) = el_stats(&[0.5]);
        assert!((mn - 0.5).abs() < 1e-6 && (mx - 0.5).abs() < 1e-6 && (avg - 0.5).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_edge_length_config();
        el_enable(&mut cfg);
        assert!(cfg.enabled);
        el_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_min_length() {
        assert!(el_to_json(&default_edge_length_config()).contains("min_length"));
    }
}
