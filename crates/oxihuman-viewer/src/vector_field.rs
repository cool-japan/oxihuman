// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 3D vector field glyph rendering parameters.

#![allow(dead_code)]

/// Glyph style for 3D vectors.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphStyle {
    Arrow,
    Cone,
    Line,
    Hedgehog,
}

/// 3D vector field visualization config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VectorFieldConfig {
    pub grid_x: u32,
    pub grid_y: u32,
    pub grid_z: u32,
    pub glyph_style: GlyphStyle,
    pub glyph_scale: f32,
    pub normalize_glyphs: bool,
    pub max_magnitude: f32,
}

#[allow(dead_code)]
impl Default for VectorFieldConfig {
    fn default() -> Self {
        Self {
            grid_x: 8,
            grid_y: 8,
            grid_z: 8,
            glyph_style: GlyphStyle::Arrow,
            glyph_scale: 1.0,
            normalize_glyphs: false,
            max_magnitude: 1.0,
        }
    }
}

/// A 3D vector at a point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VectorAt {
    pub position: [f32; 3],
    pub vector: [f32; 3],
}

/// Create default vector field config.
#[allow(dead_code)]
pub fn new_vector_field_config() -> VectorFieldConfig {
    VectorFieldConfig::default()
}

/// Magnitude of a 3D vector.
#[allow(dead_code)]
pub fn vec3_magnitude(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Normalize a 3D vector.
#[allow(dead_code)]
pub fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_magnitude(v);
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Map magnitude to RGBA color (blue low → red high).
#[allow(dead_code)]
pub fn vf_magnitude_color(mag: f32, max_mag: f32) -> [f32; 4] {
    let t = (mag / max_mag.max(1e-10)).clamp(0.0, 1.0);
    [t, 0.0, 1.0 - t, 1.0]
}

/// Generate a uniform curl field in a box.
#[allow(dead_code)]
pub fn curl_vector_field(cfg: &VectorFieldConfig) -> Vec<VectorAt> {
    let total = (cfg.grid_x * cfg.grid_y * cfg.grid_z) as usize;
    let mut out = Vec::with_capacity(total);
    for k in 0..cfg.grid_z {
        for j in 0..cfg.grid_y {
            for i in 0..cfg.grid_x {
                let x = i as f32 / cfg.grid_x as f32 * 2.0 - 1.0;
                let y = j as f32 / cfg.grid_y as f32 * 2.0 - 1.0;
                let z = k as f32 / cfg.grid_z as f32 * 2.0 - 1.0;
                out.push(VectorAt {
                    position: [x, y, z],
                    vector: [-y, x, z * 0.1],
                });
            }
        }
    }
    out
}

/// Set glyph scale.
#[allow(dead_code)]
pub fn vf_set_glyph_scale(cfg: &mut VectorFieldConfig, value: f32) {
    cfg.glyph_scale = value.max(0.0);
}

/// Set glyph style.
#[allow(dead_code)]
pub fn vf_set_glyph_style(cfg: &mut VectorFieldConfig, style: GlyphStyle) {
    cfg.glyph_style = style;
}

/// Total glyph count.
#[allow(dead_code)]
pub fn vf_glyph_count(cfg: &VectorFieldConfig) -> u32 {
    cfg.grid_x * cfg.grid_y * cfg.grid_z
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn vector_field_to_json(cfg: &VectorFieldConfig) -> String {
    format!(
        r#"{{"grid_x":{},"grid_y":{},"grid_z":{},"glyph_scale":{:.4}}}"#,
        cfg.grid_x, cfg.grid_y, cfg.grid_z, cfg.glyph_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = VectorFieldConfig::default();
        assert_eq!(c.grid_x, 8);
        assert_eq!(c.glyph_style, GlyphStyle::Arrow);
    }

    #[test]
    fn test_vec3_magnitude() {
        assert!((vec3_magnitude([3.0, 4.0, 0.0]) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_vec3_normalize_unit() {
        let n = vec3_normalize([0.0, 3.0, 4.0]);
        let len = vec3_magnitude(n);
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vec3_normalize_zero_safe() {
        let n = vec3_normalize([0.0, 0.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_magnitude_color_zero() {
        let c = vf_magnitude_color(0.0, 1.0);
        assert!(c[0].abs() < 1e-6);
    }

    #[test]
    fn test_curl_field_count() {
        let cfg = VectorFieldConfig {
            grid_x: 2,
            grid_y: 2,
            grid_z: 2,
            ..Default::default()
        };
        assert_eq!(curl_vector_field(&cfg).len(), 8);
    }

    #[test]
    fn test_glyph_count() {
        let cfg = VectorFieldConfig {
            grid_x: 3,
            grid_y: 3,
            grid_z: 3,
            ..Default::default()
        };
        assert_eq!(vf_glyph_count(&cfg), 27);
    }

    #[test]
    fn test_set_glyph_scale_clamped() {
        let mut c = VectorFieldConfig::default();
        vf_set_glyph_scale(&mut c, -1.0);
        assert!(c.glyph_scale < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = vector_field_to_json(&VectorFieldConfig::default());
        assert!(j.contains("grid_x"));
    }
}
