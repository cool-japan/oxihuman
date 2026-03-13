// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Scalar field isosurface overlay settings.

#![allow(dead_code)]

/// Config for scalar field isosurface visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScalarFieldConfig {
    /// Isovalue for the primary isosurface.
    pub isovalue: f32,
    /// Number of additional isosurface levels.
    pub num_isosurfaces: u32,
    /// Spacing between additional isosurfaces.
    pub isosurface_spacing: f32,
    /// Opacity of the isosurface overlay.
    pub opacity: f32,
    /// Color of the isosurface.
    pub color: [f32; 3],
}

#[allow(dead_code)]
impl Default for ScalarFieldConfig {
    fn default() -> Self {
        Self {
            isovalue: 0.5,
            num_isosurfaces: 1,
            isosurface_spacing: 0.1,
            opacity: 0.5,
            color: [0.2, 0.6, 1.0],
        }
    }
}

/// A scalar sample at a 3D point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ScalarSample {
    pub position: [f32; 3],
    pub value: f32,
}

/// Create default scalar field config.
#[allow(dead_code)]
pub fn new_scalar_field_config() -> ScalarFieldConfig {
    ScalarFieldConfig::default()
}

/// Set the primary isovalue.
#[allow(dead_code)]
pub fn sf_set_isovalue(cfg: &mut ScalarFieldConfig, value: f32) {
    cfg.isovalue = value;
}

/// Set opacity.
#[allow(dead_code)]
pub fn sf_set_opacity(cfg: &mut ScalarFieldConfig, value: f32) {
    cfg.opacity = value.clamp(0.0, 1.0);
}

/// Check if a sample is above the isovalue.
#[allow(dead_code)]
pub fn sf_is_above_iso(sample: &ScalarSample, isovalue: f32) -> bool {
    sample.value >= isovalue
}

/// Trilinear interpolate (placeholder: linear in first coord only).
#[allow(dead_code)]
pub fn sf_interpolate(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Return all isovalue levels for the config.
#[allow(dead_code)]
pub fn sf_isovalues(cfg: &ScalarFieldConfig) -> Vec<f32> {
    let mut out = vec![cfg.isovalue];
    for i in 1..=cfg.num_isosurfaces {
        out.push(cfg.isovalue + i as f32 * cfg.isosurface_spacing);
        out.push(cfg.isovalue - i as f32 * cfg.isosurface_spacing);
    }
    out
}

/// Compute gradient by finite differences (1D placeholder).
#[allow(dead_code)]
pub fn sf_gradient_1d(values: &[f32], idx: usize, dx: f32) -> f32 {
    if values.is_empty() || dx.abs() < 1e-10 {
        return 0.0;
    }
    let n = values.len();
    if idx == 0 {
        return (values[1.min(n - 1)] - values[0]) / dx;
    }
    if idx + 1 >= n {
        return (values[n - 1] - values[n - 2]) / dx;
    }
    (values[idx + 1] - values[idx - 1]) / (2.0 * dx)
}

/// Normalize color channels to 0..1.
#[allow(dead_code)]
pub fn sf_normalize_color(color: [f32; 3]) -> [f32; 3] {
    [
        color[0].clamp(0.0, 1.0),
        color[1].clamp(0.0, 1.0),
        color[2].clamp(0.0, 1.0),
    ]
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn scalar_field_to_json(cfg: &ScalarFieldConfig) -> String {
    format!(
        r#"{{"isovalue":{:.4},"num_isosurfaces":{},"opacity":{:.4}}}"#,
        cfg.isovalue, cfg.num_isosurfaces, cfg.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = ScalarFieldConfig::default();
        assert!((c.isovalue - 0.5).abs() < 1e-6);
        assert!((c.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_opacity_clamped() {
        let mut c = ScalarFieldConfig::default();
        sf_set_opacity(&mut c, 2.0);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_above_iso_true() {
        let s = ScalarSample {
            position: [0.0, 0.0, 0.0],
            value: 0.8,
        };
        assert!(sf_is_above_iso(&s, 0.5));
    }

    #[test]
    fn test_is_above_iso_false() {
        let s = ScalarSample {
            position: [0.0, 0.0, 0.0],
            value: 0.3,
        };
        assert!(!sf_is_above_iso(&s, 0.5));
    }

    #[test]
    fn test_interpolate() {
        assert!((sf_interpolate(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_isovalues_count() {
        let c = ScalarFieldConfig {
            num_isosurfaces: 2,
            ..Default::default()
        };
        let v = sf_isovalues(&c);
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn test_gradient_1d() {
        let values = [0.0f32, 1.0, 2.0, 3.0];
        let g = sf_gradient_1d(&values, 1, 1.0);
        assert!((g - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_color() {
        let c = sf_normalize_color([2.0, -0.5, 0.5]);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!(c[1] < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = scalar_field_to_json(&ScalarFieldConfig::default());
        assert!(j.contains("isovalue"));
    }
}
