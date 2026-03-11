// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Indirect specular: prefiltered environment map sampling for PBR rendering.

use std::f32::consts::PI;

/// Configuration for indirect specular.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectSpecularConfig {
    pub max_mip_level: u32,
    pub intensity: f32,
    pub roughness_offset: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_indirect_specular_config() -> IndirectSpecularConfig {
    IndirectSpecularConfig {
        max_mip_level: 8,
        intensity: 1.0,
        roughness_offset: 0.0,
        enabled: true,
    }
}

/// Map roughness to mip level for prefiltered env map.
#[allow(dead_code)]
pub fn roughness_to_mip_level(roughness: f32, cfg: &IndirectSpecularConfig) -> f32 {
    let r = (roughness + cfg.roughness_offset).clamp(0.0, 1.0);
    r * cfg.max_mip_level as f32
}

/// Fresnel-Schlick approximation.
#[allow(dead_code)]
pub fn fresnel_schlick(cos_theta: f32, f0: f32) -> f32 {
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

/// Fresnel-Schlick with roughness attenuation.
#[allow(dead_code)]
pub fn fresnel_schlick_roughness(cos_theta: f32, f0: f32, roughness: f32) -> f32 {
    let max_f = (1.0 - roughness).max(f0);
    f0 + (max_f - f0) * (1.0 - cos_theta).powi(5)
}

/// GGX distribution for a half-angle.
#[allow(dead_code)]
pub fn ggx_distribution(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    a2 / (PI * denom * denom)
}

#[allow(dead_code)]
pub fn set_indirect_intensity(cfg: &mut IndirectSpecularConfig, v: f32) {
    cfg.intensity = v.max(0.0);
}

#[allow(dead_code)]
pub fn set_roughness_offset(cfg: &mut IndirectSpecularConfig, v: f32) {
    cfg.roughness_offset = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn indirect_specular_to_json(cfg: &IndirectSpecularConfig) -> String {
    format!(
        r#"{{"max_mip":{},"intensity":{:.4},"roughness_offset":{:.4},"enabled":{}}}"#,
        cfg.max_mip_level, cfg.intensity, cfg.roughness_offset, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_indirect_specular_config();
        assert_eq!(cfg.max_mip_level, 8);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_roughness_to_mip_smooth() {
        let cfg = default_indirect_specular_config();
        let mip = roughness_to_mip_level(0.0, &cfg);
        assert!(mip.abs() < 1e-6);
    }

    #[test]
    fn test_roughness_to_mip_rough() {
        let cfg = default_indirect_specular_config();
        let mip = roughness_to_mip_level(1.0, &cfg);
        assert!((mip - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_fresnel_schlick_zero() {
        let f = fresnel_schlick(1.0, 0.04);
        assert!((f - 0.04).abs() < 1e-6);
    }

    #[test]
    fn test_fresnel_schlick_grazing() {
        let f = fresnel_schlick(0.0, 0.04);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fresnel_roughness() {
        let f = fresnel_schlick_roughness(1.0, 0.04, 0.5);
        assert!((f - 0.04).abs() < 1e-6);
    }

    #[test]
    fn test_ggx_distribution() {
        let d = ggx_distribution(1.0, 0.5);
        assert!(d > 0.0);
    }

    #[test]
    fn test_set_intensity() {
        let mut cfg = default_indirect_specular_config();
        set_indirect_intensity(&mut cfg, 3.0);
        assert!((cfg.intensity - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_roughness_offset() {
        let mut cfg = default_indirect_specular_config();
        set_roughness_offset(&mut cfg, 0.1);
        assert!((cfg.roughness_offset - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_indirect_specular_config();
        let j = indirect_specular_to_json(&cfg);
        assert!(j.contains("max_mip"));
    }
}
