// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! DeferredShading — configuration and stub evaluation for deferred rendering.

#![allow(dead_code)]

/// Configuration for the deferred shading pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeferredConfig {
    pub ambient_intensity: f32,
    pub max_lights: u32,
    pub output_count: u32,
    pub pass_name: String,
}

/// Deferred shading state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeferredShading {
    pub config: DeferredConfig,
}

/// Create a new `DeferredConfig` with the given settings.
#[allow(dead_code)]
pub fn new_deferred_config(ambient: f32, max_lights: u32) -> DeferredConfig {
    DeferredConfig {
        ambient_intensity: ambient,
        max_lights,
        output_count: 1,
        pass_name: "deferred".to_owned(),
    }
}

/// Create a default `DeferredConfig`.
#[allow(dead_code)]
pub fn default_deferred_config() -> DeferredConfig {
    new_deferred_config(0.1, 32)
}

/// Shade a single G-buffer point with ambient lighting only (stub).
#[allow(dead_code)]
pub fn shade_gbuffer_point(cfg: &DeferredConfig, albedo: [f32; 4]) -> [f32; 4] {
    let a = cfg.ambient_intensity;
    [albedo[0] * a, albedo[1] * a, albedo[2] * a, albedo[3]]
}

/// Return the ambient contribution for the given albedo.
#[allow(dead_code)]
pub fn deferred_ambient(cfg: &DeferredConfig, albedo: [f32; 3]) -> [f32; 3] {
    let a = cfg.ambient_intensity;
    [albedo[0] * a, albedo[1] * a, albedo[2] * a]
}

/// Return the directional light contribution (stub).
#[allow(dead_code)]
pub fn deferred_directional(normal: [f32; 3], light_dir: [f32; 3], color: [f32; 3]) -> [f32; 3] {
    let ndotl = (normal[0] * light_dir[0] + normal[1] * light_dir[1] + normal[2] * light_dir[2])
        .max(0.0);
    [color[0] * ndotl, color[1] * ndotl, color[2] * ndotl]
}

/// Return a point light contribution (stub — distance attenuation).
#[allow(dead_code)]
pub fn deferred_point_light(pos: [f32; 3], light_pos: [f32; 3], color: [f32; 3], radius: f32) -> [f32; 3] {
    let dx = pos[0] - light_pos[0];
    let dy = pos[1] - light_pos[1];
    let dz = pos[2] - light_pos[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let att = (1.0 - (dist / radius.max(f32::EPSILON)).min(1.0)).max(0.0);
    [color[0] * att, color[1] * att, color[2] * att]
}

/// Return the number of render outputs.
#[allow(dead_code)]
pub fn deferred_output_count(cfg: &DeferredConfig) -> u32 {
    cfg.output_count
}

/// Return the pass name.
#[allow(dead_code)]
pub fn deferred_pass_name(cfg: &DeferredConfig) -> &str {
    &cfg.pass_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_deferred_config();
        assert!(cfg.ambient_intensity > 0.0);
        assert!(cfg.max_lights > 0);
    }

    #[test]
    fn test_new_deferred_config() {
        let cfg = new_deferred_config(0.2, 16);
        assert!((cfg.ambient_intensity - 0.2).abs() < 1e-6);
        assert_eq!(cfg.max_lights, 16);
    }

    #[test]
    fn test_shade_gbuffer_point_scales_albedo() {
        let cfg = new_deferred_config(0.5, 8);
        let result = shade_gbuffer_point(&cfg, [1.0, 1.0, 1.0, 1.0]);
        assert!((result[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_deferred_ambient() {
        let cfg = new_deferred_config(0.3, 8);
        let a = deferred_ambient(&cfg, [1.0, 1.0, 1.0]);
        assert!((a[0] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_deferred_directional_facing() {
        let n = [0.0_f32, 1.0, 0.0];
        let l = [0.0_f32, 1.0, 0.0];
        let c = deferred_directional(n, l, [1.0, 1.0, 1.0]);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deferred_directional_away() {
        let n = [0.0_f32, 1.0, 0.0];
        let l = [0.0_f32, -1.0, 0.0];
        let c = deferred_directional(n, l, [1.0, 1.0, 1.0]);
        assert!(c[0].abs() < 1e-6);
    }

    #[test]
    fn test_deferred_point_light_near() {
        let pos = [0.0_f32; 3];
        let light = [0.0_f32; 3];
        let c = deferred_point_light(pos, light, [1.0, 1.0, 1.0], 5.0);
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deferred_point_light_far() {
        let pos = [100.0_f32, 0.0, 0.0];
        let light = [0.0_f32; 3];
        let c = deferred_point_light(pos, light, [1.0, 1.0, 1.0], 5.0);
        assert!(c[0].abs() < 1e-5);
    }

    #[test]
    fn test_deferred_output_count() {
        let cfg = default_deferred_config();
        assert_eq!(deferred_output_count(&cfg), 1);
    }

    #[test]
    fn test_deferred_pass_name() {
        let cfg = default_deferred_config();
        assert_eq!(deferred_pass_name(&cfg), "deferred");
    }
}
