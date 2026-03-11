// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2D flow field arrow glyph visualization data.

#![allow(dead_code)]

/// Configuration for 2D flow field visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlowFieldConfig {
    /// Grid resolution in X.
    pub grid_x: u32,
    /// Grid resolution in Y.
    pub grid_y: u32,
    /// Arrow scale factor.
    pub arrow_scale: f32,
    /// Color by magnitude.
    pub color_by_magnitude: bool,
    /// Max magnitude for normalization.
    pub max_magnitude: f32,
}

#[allow(dead_code)]
impl Default for FlowFieldConfig {
    fn default() -> Self {
        Self {
            grid_x: 16,
            grid_y: 16,
            arrow_scale: 1.0,
            color_by_magnitude: true,
            max_magnitude: 1.0,
        }
    }
}

/// A single 2D flow vector at a grid point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FlowVector {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

/// Create default flow field config.
#[allow(dead_code)]
pub fn new_flow_field_config() -> FlowFieldConfig {
    FlowFieldConfig::default()
}

/// Compute the magnitude of a flow vector.
#[allow(dead_code)]
pub fn flow_magnitude(v: &FlowVector) -> f32 {
    (v.velocity[0] * v.velocity[0] + v.velocity[1] * v.velocity[1]).sqrt()
}

/// Normalize a velocity to unit length.
#[allow(dead_code)]
pub fn normalize_velocity(vel: [f32; 2]) -> [f32; 2] {
    let len = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0];
    }
    [vel[0] / len, vel[1] / len]
}

/// Map magnitude to a color (blue=low, red=high).
#[allow(dead_code)]
pub fn magnitude_to_color(mag: f32, max_mag: f32) -> [f32; 3] {
    let t = (mag / max_mag.max(1e-10)).clamp(0.0, 1.0);
    [t, 0.0, 1.0 - t]
}

/// Generate a simple vortex flow field for testing.
#[allow(dead_code)]
pub fn vortex_flow_field(cfg: &FlowFieldConfig) -> Vec<FlowVector> {
    let mut out = Vec::with_capacity((cfg.grid_x * cfg.grid_y) as usize);
    for j in 0..cfg.grid_y {
        for i in 0..cfg.grid_x {
            let x = i as f32 / cfg.grid_x as f32 * 2.0 - 1.0;
            let y = j as f32 / cfg.grid_y as f32 * 2.0 - 1.0;
            out.push(FlowVector {
                position: [x, y],
                velocity: [-y, x],
            });
        }
    }
    out
}

/// Set arrow scale.
#[allow(dead_code)]
pub fn ff_set_arrow_scale(cfg: &mut FlowFieldConfig, value: f32) {
    cfg.arrow_scale = value.max(0.0);
}

/// Set max magnitude.
#[allow(dead_code)]
pub fn ff_set_max_magnitude(cfg: &mut FlowFieldConfig, value: f32) {
    cfg.max_magnitude = value.max(1e-10);
}

/// Compute divergence at a grid index (finite difference placeholder).
#[allow(dead_code)]
pub fn flow_divergence(vectors: &[FlowVector], idx: usize, grid_x: u32) -> f32 {
    let n = vectors.len();
    if idx == 0 || idx + 1 >= n {
        return 0.0;
    }
    let right = if idx + 1 < n {
        vectors[idx + 1].velocity[0]
    } else {
        vectors[idx].velocity[0]
    };
    let left = vectors[idx - 1].velocity[0];
    let dx = if idx < grid_x as usize {
        1.0
    } else {
        vectors[idx].position[0] - vectors[idx - 1].position[0]
    };
    (right - left) / (2.0 * dx.abs().max(1e-10))
}

/// Serialize config to JSON.
#[allow(dead_code)]
pub fn flow_field_to_json(cfg: &FlowFieldConfig) -> String {
    format!(
        r#"{{"grid_x":{},"grid_y":{},"arrow_scale":{:.4},"max_magnitude":{:.4}}}"#,
        cfg.grid_x, cfg.grid_y, cfg.arrow_scale, cfg.max_magnitude
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = FlowFieldConfig::default();
        assert_eq!(c.grid_x, 16);
        assert!(c.color_by_magnitude);
    }

    #[test]
    fn test_flow_magnitude_zero() {
        let v = FlowVector {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
        };
        assert!(flow_magnitude(&v) < 1e-6);
    }

    #[test]
    fn test_flow_magnitude_unit() {
        let v = FlowVector {
            position: [0.0, 0.0],
            velocity: [1.0, 0.0],
        };
        assert!((flow_magnitude(&v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_velocity() {
        let n = normalize_velocity([3.0, 4.0]);
        let len = (n[0] * n[0] + n[1] * n[1]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_zero_safe() {
        let n = normalize_velocity([0.0, 0.0]);
        assert!(n[0].abs() < 1e-6 && n[1].abs() < 1e-6);
    }

    #[test]
    fn test_magnitude_to_color_zero() {
        let c = magnitude_to_color(0.0, 1.0);
        assert!(c[0].abs() < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vortex_field_count() {
        let cfg = FlowFieldConfig {
            grid_x: 4,
            grid_y: 4,
            ..Default::default()
        };
        let v = vortex_flow_field(&cfg);
        assert_eq!(v.len(), 16);
    }

    #[test]
    fn test_set_arrow_scale() {
        let mut c = FlowFieldConfig::default();
        ff_set_arrow_scale(&mut c, -1.0);
        assert!(c.arrow_scale < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = flow_field_to_json(&FlowFieldConfig::default());
        assert!(j.contains("grid_x"));
        assert!(j.contains("arrow_scale"));
    }
}
