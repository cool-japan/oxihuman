// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV warp modifier — warps UV coordinates using two control objects.

/// Configuration for UV warp.
#[derive(Debug, Clone)]
pub struct UvWarpConfig {
    pub from_uv: [f32; 2],
    pub to_uv: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32, /* radians */
    pub mix: f32,
}

impl Default for UvWarpConfig {
    fn default() -> Self {
        Self {
            from_uv: [0.0; 2],
            to_uv: [0.0; 2],
            scale: [1.0; 2],
            rotation: 0.0,
            mix: 1.0,
        }
    }
}

impl UvWarpConfig {
    pub fn new(from: [f32; 2], to: [f32; 2]) -> Self {
        Self { from_uv: from, to_uv: to, ..Self::default() }
    }

    pub fn with_rotation(mut self, rot: f32) -> Self {
        self.rotation = rot;
        self
    }
}

/// Apply UV warp to a single UV coordinate.
pub fn warp_uv(uv: [f32; 2], cfg: &UvWarpConfig) -> [f32; 2] {
    let centered = [uv[0] - cfg.from_uv[0], uv[1] - cfg.from_uv[1]];
    let c = cfg.rotation.cos();
    let s = cfg.rotation.sin();
    let rotated = [centered[0] * c - centered[1] * s, centered[0] * s + centered[1] * c];
    let scaled = [rotated[0] * cfg.scale[0], rotated[1] * cfg.scale[1]];
    let target = [scaled[0] + cfg.to_uv[0], scaled[1] + cfg.to_uv[1]];
    [uv[0] + (target[0] - uv[0]) * cfg.mix, uv[1] + (target[1] - uv[1]) * cfg.mix]
}

/// Apply UV warp to a slice of UV coordinates.
pub fn apply_uv_warp(uvs: &mut [[f32; 2]], cfg: &UvWarpConfig) {
    for uv in uvs.iter_mut() {
        *uv = warp_uv(*uv, cfg);
    }
}

/// Compute the UV delta between from and to centers.
pub fn uv_warp_delta(cfg: &UvWarpConfig) -> [f32; 2] {
    [cfg.to_uv[0] - cfg.from_uv[0], cfg.to_uv[1] - cfg.from_uv[1]]
}

/// Validate UV warp config.
pub fn validate_uv_warp_config(cfg: &UvWarpConfig) -> bool {
    (0.0..=1.0).contains(&cfg.mix) && cfg.scale[0] > 0.0 && cfg.scale[1] > 0.0
}

/// Reset UV warp to identity.
pub fn uv_warp_identity() -> UvWarpConfig {
    UvWarpConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_warp_config_default() {
        let cfg = UvWarpConfig::default();
        assert!((cfg.mix - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_warp_uv_identity() {
        let cfg = uv_warp_identity();
        let uv = warp_uv([0.5, 0.5], &cfg);
        assert!((uv[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_warp_uv_translation() {
        let cfg = UvWarpConfig::new([0.0; 2], [0.1, 0.2]);
        let uv = warp_uv([0.0, 0.0], &cfg);
        assert!((uv[0] - 0.1).abs() < 1e-5);
        assert!((uv[1] - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_warp_uv_zero_mix() {
        let cfg = UvWarpConfig { mix: 0.0, to_uv: [1.0, 1.0], ..UvWarpConfig::default() };
        let uv = warp_uv([0.5, 0.5], &cfg);
        assert!((uv[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_uv_warp_count() {
        let mut uvs = vec![[0.0_f32; 2]; 5];
        let cfg = UvWarpConfig::default();
        apply_uv_warp(&mut uvs, &cfg);
        assert_eq!(uvs.len(), 5);
    }

    #[test]
    fn test_uv_warp_delta() {
        let cfg = UvWarpConfig::new([0.0; 2], [0.5, 0.5]);
        let d = uv_warp_delta(&cfg);
        assert!((d[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = UvWarpConfig::default();
        assert!(validate_uv_warp_config(&cfg));
    }

    #[test]
    fn test_validate_config_invalid_scale() {
        let cfg = UvWarpConfig { scale: [0.0, 1.0], ..UvWarpConfig::default() };
        assert!(!validate_uv_warp_config(&cfg));
    }

    #[test]
    fn test_with_rotation() {
        let cfg = UvWarpConfig::default().with_rotation(std::f32::consts::FRAC_PI_2);
        assert!((cfg.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }
}
