// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Barycentric coordinate debug visualization.

/// Barycentric coordinate visualization config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BarycentricConfig {
    pub color_v0: [f32; 3],
    pub color_v1: [f32; 3],
    pub color_v2: [f32; 3],
    pub show_edges: bool,
    pub edge_width: f32,
    pub enabled: bool,
}

impl Default for BarycentricConfig {
    fn default() -> Self {
        BarycentricConfig {
            color_v0: [1.0, 0.0, 0.0],
            color_v1: [0.0, 1.0, 0.0],
            color_v2: [0.0, 0.0, 1.0],
            show_edges: false,
            edge_width: 1.0,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_barycentric_config() -> BarycentricConfig {
    BarycentricConfig::default()
}

#[allow(dead_code)]
pub fn bary_enable(cfg: &mut BarycentricConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn bary_disable(cfg: &mut BarycentricConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn bary_set_edge_width(cfg: &mut BarycentricConfig, w: f32) {
    cfg.edge_width = w.clamp(0.1, 10.0);
}

/// Interpolate a color using barycentric coordinates (u, v, w) — must sum to 1.
#[allow(dead_code)]
pub fn bary_interpolate_color(cfg: &BarycentricConfig, u: f32, v: f32, w: f32) -> [f32; 3] {
    [
        cfg.color_v0[0] * u + cfg.color_v1[0] * v + cfg.color_v2[0] * w,
        cfg.color_v0[1] * u + cfg.color_v1[1] * v + cfg.color_v2[1] * w,
        cfg.color_v0[2] * u + cfg.color_v1[2] * v + cfg.color_v2[2] * w,
    ]
}

/// Compute barycentric coordinates of point P relative to triangle (A, B, C).
#[allow(dead_code)]
pub fn bary_coords(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> [f32; 3] {
    let v0 = [b[0] - a[0], b[1] - a[1]];
    let v1 = [c[0] - a[0], c[1] - a[1]];
    let v2 = [p[0] - a[0], p[1] - a[1]];
    let d00 = v0[0] * v0[0] + v0[1] * v0[1];
    let d01 = v0[0] * v1[0] + v0[1] * v1[1];
    let d11 = v1[0] * v1[0] + v1[1] * v1[1];
    let d20 = v2[0] * v0[0] + v2[1] * v0[1];
    let d21 = v2[0] * v1[0] + v2[1] * v1[1];
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    [u, v, w]
}

/// Check if barycentric coords are inside the triangle.
#[allow(dead_code)]
pub fn bary_is_inside(u: f32, v: f32, w: f32) -> bool {
    u >= 0.0 && v >= 0.0 && w >= 0.0
}

#[allow(dead_code)]
pub fn bary_to_json(cfg: &BarycentricConfig) -> String {
    format!(
        r#"{{"show_edges":{},"edge_width":{:.4},"enabled":{}}}"#,
        cfg.show_edges, cfg.edge_width, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_barycentric_config().enabled);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_barycentric_config();
        bary_enable(&mut cfg);
        assert!(cfg.enabled);
        bary_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn interpolate_at_v0() {
        let cfg = default_barycentric_config();
        let c = bary_interpolate_color(&cfg, 1.0, 0.0, 0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!(c[1].abs() < 1e-6);
    }

    #[test]
    fn interpolate_at_center() {
        let cfg = default_barycentric_config();
        let c = bary_interpolate_color(&cfg, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
        assert!((c[0] - c[1]).abs() < 1e-5);
    }

    #[test]
    fn coords_at_vertex_a() {
        let a = [0.0f32, 0.0];
        let b = [1.0f32, 0.0];
        let c = [0.0f32, 1.0];
        let coords = bary_coords(a, a, b, c);
        assert!((coords[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn coords_inside() {
        let a = [0.0f32, 0.0];
        let b = [1.0f32, 0.0];
        let c = [0.0f32, 1.0];
        let p = [0.25f32, 0.25];
        let coords = bary_coords(p, a, b, c);
        assert!(bary_is_inside(coords[0], coords[1], coords[2]));
    }

    #[test]
    fn coords_outside() {
        let a = [0.0f32, 0.0];
        let b = [1.0f32, 0.0];
        let c = [0.0f32, 1.0];
        let p = [2.0f32, 0.0];
        let coords = bary_coords(p, a, b, c);
        assert!(!bary_is_inside(coords[0], coords[1], coords[2]));
    }

    #[test]
    fn edge_width_clamps() {
        let mut cfg = default_barycentric_config();
        bary_set_edge_width(&mut cfg, 0.0);
        assert!((cfg.edge_width - 0.1).abs() < 1e-6);
    }

    #[test]
    fn to_json_has_enabled() {
        assert!(bary_to_json(&default_barycentric_config()).contains("enabled"));
    }
}
