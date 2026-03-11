// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AoRenderer — ambient-occlusion rendering utilities.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for AO computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoConfig {
    pub radius: f32,
    pub sample_count: u32,
    pub intensity: f32,
}

/// Ambient-occlusion renderer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoRenderer {
    pub config: AoConfig,
    pub pass_name: String,
}

/// Create a new `AoRenderer` with the given config.
#[allow(dead_code)]
pub fn new_ao_renderer(config: AoConfig) -> AoRenderer {
    AoRenderer {
        config,
        pass_name: "ao_pass".to_owned(),
    }
}

/// Return a default `AoConfig`.
#[allow(dead_code)]
pub fn default_ao_config() -> AoConfig {
    AoConfig {
        radius: 0.5,
        sample_count: 16,
        intensity: 1.0,
    }
}

/// Compute a stub AO value at a vertex given a normal and position.
/// Returns a value in `[0, 1]` based on the normal's y-component scaled by config.
#[allow(dead_code)]
pub fn compute_ao_at_vertex(config: &AoConfig, normal: [f32; 3], _position: [f32; 3]) -> f32 {
    let base = (normal[1].abs() * PI / PI).clamp(0.0, 1.0);
    base * config.intensity
}

/// Return the AO sampling radius.
#[allow(dead_code)]
pub fn ao_radius(config: &AoConfig) -> f32 {
    config.radius
}

/// Return the number of AO samples.
#[allow(dead_code)]
pub fn ao_sample_count(config: &AoConfig) -> u32 {
    config.sample_count
}

/// Convert AO values to a flat grayscale texture buffer (stub).
#[allow(dead_code)]
pub fn ao_to_texture(values: &[f32], width: u32, height: u32) -> Vec<u8> {
    let total = (width * height) as usize;
    let mut buf = Vec::with_capacity(total);
    for i in 0..total {
        let v = if i < values.len() { values[i] } else { 1.0 };
        buf.push((v.clamp(0.0, 1.0) * 255.0) as u8);
    }
    buf
}

/// Return the intensity multiplier.
#[allow(dead_code)]
pub fn ao_intensity(config: &AoConfig) -> f32 {
    config.intensity
}

/// Return the render-pass name.
#[allow(dead_code)]
pub fn ao_pass_name(renderer: &AoRenderer) -> &str {
    &renderer.pass_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ao_config() {
        let c = default_ao_config();
        assert_eq!(c.sample_count, 16);
        assert!((c.radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_ao_renderer() {
        let r = new_ao_renderer(default_ao_config());
        assert_eq!(r.pass_name, "ao_pass");
    }

    #[test]
    fn test_compute_ao_at_vertex_up() {
        let c = default_ao_config();
        let ao = compute_ao_at_vertex(&c, [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((0.0..=1.0).contains(&ao));
    }

    #[test]
    fn test_compute_ao_at_vertex_zero() {
        let c = default_ao_config();
        let ao = compute_ao_at_vertex(&c, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((ao - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_ao_radius() {
        let c = default_ao_config();
        assert!((ao_radius(&c) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ao_sample_count() {
        let c = default_ao_config();
        assert_eq!(ao_sample_count(&c), 16);
    }

    #[test]
    fn test_ao_to_texture() {
        let vals = vec![0.0, 0.5, 1.0, 0.25];
        let tex = ao_to_texture(&vals, 2, 2);
        assert_eq!(tex.len(), 4);
        assert_eq!(tex[0], 0);
        assert_eq!(tex[2], 255);
    }

    #[test]
    fn test_ao_to_texture_pad() {
        let vals = vec![0.5];
        let tex = ao_to_texture(&vals, 2, 2);
        assert_eq!(tex.len(), 4);
        assert_eq!(tex[1], 255); // default 1.0
    }

    #[test]
    fn test_ao_intensity() {
        let c = default_ao_config();
        assert!((ao_intensity(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ao_pass_name() {
        let r = new_ao_renderer(default_ao_config());
        assert_eq!(ao_pass_name(&r), "ao_pass");
    }
}
