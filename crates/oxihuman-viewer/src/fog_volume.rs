// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fog volume — volumetric fog density and color parameters.

/// Fog type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FogKind {
    Linear,
    #[default]
    Exponential,
    ExponentialSquared,
}

/// Fog volume config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FogVolumeConfig {
    pub kind: FogKind,
    /// Fog density (used by exponential modes).
    pub density: f32,
    /// Linear fog start distance (world units).
    pub linear_start: f32,
    /// Linear fog end distance.
    pub linear_end: f32,
    /// Fog color RGBA (0..=1 each).
    pub color: [f32; 4],
    /// Height falloff: fog thins above this world-Y.
    pub height_falloff: f32,
    pub enabled: bool,
}

impl Default for FogVolumeConfig {
    fn default() -> Self {
        Self {
            kind: FogKind::Exponential,
            density: 0.02,
            linear_start: 10.0,
            linear_end: 100.0,
            color: [0.8, 0.85, 0.9, 1.0],
            height_falloff: 50.0,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_fog_volume_config() -> FogVolumeConfig {
    FogVolumeConfig::default()
}

/// Compute fog factor (0 = no fog, 1 = fully fogged) for the given distance.
#[allow(dead_code)]
pub fn fog_factor(distance: f32, cfg: &FogVolumeConfig) -> f32 {
    if !cfg.enabled {
        return 0.0;
    }
    let f = match cfg.kind {
        FogKind::Linear => {
            let range = (cfg.linear_end - cfg.linear_start).max(1e-6);
            ((distance - cfg.linear_start) / range).clamp(0.0, 1.0)
        }
        FogKind::Exponential => {
            let d = cfg.density * distance;
            1.0 - (-d).exp()
        }
        FogKind::ExponentialSquared => {
            let d = cfg.density * distance;
            1.0 - (-(d * d)).exp()
        }
    };
    f.clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn fv_set_density(cfg: &mut FogVolumeConfig, v: f32) {
    cfg.density = v.clamp(0.0, 10.0);
}

#[allow(dead_code)]
pub fn fv_set_color(cfg: &mut FogVolumeConfig, r: f32, g: f32, b: f32, a: f32) {
    cfg.color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

#[allow(dead_code)]
pub fn fv_set_kind(cfg: &mut FogVolumeConfig, kind: FogKind) {
    cfg.kind = kind;
}

#[allow(dead_code)]
pub fn fv_blend_color(from: &FogVolumeConfig, to: &FogVolumeConfig, t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    [
        from.color[0] * inv + to.color[0] * t,
        from.color[1] * inv + to.color[1] * t,
        from.color[2] * inv + to.color[2] * t,
        from.color[3] * inv + to.color[3] * t,
    ]
}

#[allow(dead_code)]
pub fn fv_to_json(cfg: &FogVolumeConfig) -> String {
    format!(
        "{{\"density\":{:.4},\"enabled\":{}}}",
        cfg.density, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_fog_zero() {
        let mut cfg = new_fog_volume_config();
        cfg.enabled = false;
        assert!(fog_factor(100.0, &cfg) < 1e-8);
    }

    #[test]
    fn exponential_factor_increases_with_distance() {
        let cfg = new_fog_volume_config();
        let a = fog_factor(10.0, &cfg);
        let b = fog_factor(50.0, &cfg);
        assert!(b > a);
    }

    #[test]
    fn linear_fog_before_start_zero() {
        let mut cfg = new_fog_volume_config();
        fv_set_kind(&mut cfg, FogKind::Linear);
        let f = fog_factor(5.0, &cfg); // before linear_start=10
        assert!(f < 1e-6);
    }

    #[test]
    fn linear_fog_after_end_one() {
        let mut cfg = new_fog_volume_config();
        fv_set_kind(&mut cfg, FogKind::Linear);
        let f = fog_factor(200.0, &cfg);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn exp_squared_increases() {
        let mut cfg = new_fog_volume_config();
        fv_set_kind(&mut cfg, FogKind::ExponentialSquared);
        let a = fog_factor(1.0, &cfg);
        let b = fog_factor(10.0, &cfg);
        assert!(b > a);
    }

    #[test]
    fn density_clamped() {
        let mut cfg = new_fog_volume_config();
        fv_set_density(&mut cfg, 100.0);
        assert!((cfg.density - 10.0).abs() < 1e-6);
    }

    #[test]
    fn color_set_clamped() {
        let mut cfg = new_fog_volume_config();
        fv_set_color(&mut cfg, 2.0, -1.0, 0.5, 1.0);
        assert!((cfg.color[0] - 1.0).abs() < 1e-6);
        assert!(cfg.color[1] < 1e-6);
    }

    #[test]
    fn blend_color_midpoint() {
        let a = new_fog_volume_config();
        let mut b = new_fog_volume_config();
        fv_set_color(&mut b, 0.0, 0.0, 0.0, 0.0);
        let m = fv_blend_color(&a, &b, 0.5);
        assert!((m[0] - a.color[0] * 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = fv_to_json(&new_fog_volume_config());
        assert!(j.contains("density") && j.contains("enabled"));
    }
}
