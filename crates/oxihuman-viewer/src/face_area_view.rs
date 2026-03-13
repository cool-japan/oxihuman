// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face area heat map visualization.

/// Face area heat map config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceAreaConfig {
    pub min_area: f32,
    pub max_area: f32,
    pub color_small: [f32; 3],
    pub color_large: [f32; 3],
    pub enabled: bool,
}

impl Default for FaceAreaConfig {
    fn default() -> Self {
        FaceAreaConfig {
            min_area: 0.0,
            max_area: 1.0,
            color_small: [0.0, 0.0, 1.0],
            color_large: [1.0, 0.5, 0.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_face_area_config() -> FaceAreaConfig {
    FaceAreaConfig::default()
}

#[allow(dead_code)]
pub fn fa_enable(cfg: &mut FaceAreaConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn fa_disable(cfg: &mut FaceAreaConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn fa_set_range(cfg: &mut FaceAreaConfig, min: f32, max: f32) {
    cfg.min_area = min.max(0.0);
    cfg.max_area = max.max(cfg.min_area + 1e-9);
}

/// Compute the area of a triangle given three 3D vertices.
#[allow(dead_code)]
pub fn fa_triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    len * 0.5
}

/// Map face area to heat map color.
#[allow(dead_code)]
pub fn fa_area_to_color(cfg: &FaceAreaConfig, area: f32) -> [f32; 3] {
    let range = cfg.max_area - cfg.min_area;
    let t = if range < 1e-10 {
        0.5
    } else {
        ((area - cfg.min_area) / range).clamp(0.0, 1.0)
    };
    [
        cfg.color_small[0] + (cfg.color_large[0] - cfg.color_small[0]) * t,
        cfg.color_small[1] + (cfg.color_large[1] - cfg.color_small[1]) * t,
        cfg.color_small[2] + (cfg.color_large[2] - cfg.color_small[2]) * t,
    ]
}

/// Total mesh area from a list of triangle areas.
#[allow(dead_code)]
pub fn fa_total_area(areas: &[f32]) -> f32 {
    areas.iter().sum()
}

/// Average face area.
#[allow(dead_code)]
pub fn fa_average_area(areas: &[f32]) -> f32 {
    if areas.is_empty() {
        return 0.0;
    }
    fa_total_area(areas) / areas.len() as f32
}

#[allow(dead_code)]
pub fn fa_to_json(cfg: &FaceAreaConfig) -> String {
    format!(
        r#"{{"min_area":{:.6},"max_area":{:.6},"enabled":{}}}"#,
        cfg.min_area, cfg.max_area, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_face_area_config().enabled);
    }

    #[test]
    fn unit_triangle_area() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let c = [0.0f32, 1.0, 0.0];
        assert!((fa_triangle_area(a, b, c) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn degenerate_triangle_zero_area() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        let c = [2.0f32, 0.0, 0.0];
        assert!(fa_triangle_area(a, b, c).abs() < 1e-6);
    }

    #[test]
    fn color_at_min_area() {
        let cfg = default_face_area_config();
        let c = fa_area_to_color(&cfg, 0.0);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn total_area_sum() {
        let areas = vec![0.5f32, 0.3, 0.2];
        assert!((fa_total_area(&areas) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_area_empty() {
        assert!(fa_average_area(&[]).abs() < 1e-6);
    }

    #[test]
    fn average_area_uniform() {
        let areas = vec![0.5f32, 0.5, 0.5];
        assert!((fa_average_area(&areas) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_face_area_config();
        fa_enable(&mut cfg);
        assert!(cfg.enabled);
        fa_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_has_min_area() {
        assert!(fa_to_json(&default_face_area_config()).contains("min_area"));
    }
}
