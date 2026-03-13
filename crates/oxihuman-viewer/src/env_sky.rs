// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Environment sky — procedural sky model with sun and horizon control.

use std::f32::consts::{FRAC_PI_2, PI};

/// Sky model type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyModelType {
    Gradient,
    Preetham,
    Hosek,
}

/// Configuration for the environment sky.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvSkyConfig {
    pub model: SkyModelType,
    pub sun_elevation_rad: f32,
    pub sun_azimuth_rad: f32,
    pub turbidity: f32,
    pub sun_intensity: f32,
    pub horizon_tint: [f32; 3],
    pub zenith_tint: [f32; 3],
}

#[allow(dead_code)]
pub fn default_env_sky_config() -> EnvSkyConfig {
    EnvSkyConfig {
        model: SkyModelType::Preetham,
        sun_elevation_rad: FRAC_PI_2 * 0.5,
        sun_azimuth_rad: 0.0,
        turbidity: 2.0,
        sun_intensity: 1.0,
        horizon_tint: [0.85, 0.92, 1.0],
        zenith_tint: [0.3, 0.5, 1.0],
    }
}

#[allow(dead_code)]
pub fn esky_sun_direction(cfg: &EnvSkyConfig) -> [f32; 3] {
    let el = cfg.sun_elevation_rad;
    let az = cfg.sun_azimuth_rad;
    [el.cos() * az.sin(), el.sin(), el.cos() * az.cos()]
}

#[allow(dead_code)]
pub fn esky_set_elevation(cfg: &mut EnvSkyConfig, rad: f32) {
    cfg.sun_elevation_rad = rad.clamp(-FRAC_PI_2, FRAC_PI_2);
}

#[allow(dead_code)]
pub fn esky_set_azimuth(cfg: &mut EnvSkyConfig, rad: f32) {
    cfg.sun_azimuth_rad = rad % (2.0 * PI);
}

#[allow(dead_code)]
pub fn esky_set_turbidity(cfg: &mut EnvSkyConfig, t: f32) {
    cfg.turbidity = t.clamp(1.7, 10.0);
}

#[allow(dead_code)]
pub fn esky_set_intensity(cfg: &mut EnvSkyConfig, v: f32) {
    cfg.sun_intensity = v.max(0.0);
}

#[allow(dead_code)]
pub fn esky_sky_color_at_angle(cfg: &EnvSkyConfig, cos_theta: f32) -> [f32; 3] {
    let t = ((1.0 - cos_theta) * 0.5).clamp(0.0, 1.0);
    [
        cfg.zenith_tint[0] + (cfg.horizon_tint[0] - cfg.zenith_tint[0]) * t,
        cfg.zenith_tint[1] + (cfg.horizon_tint[1] - cfg.zenith_tint[1]) * t,
        cfg.zenith_tint[2] + (cfg.horizon_tint[2] - cfg.zenith_tint[2]) * t,
    ]
}

#[allow(dead_code)]
pub fn esky_model_name(cfg: &EnvSkyConfig) -> &'static str {
    match cfg.model {
        SkyModelType::Gradient => "gradient",
        SkyModelType::Preetham => "preetham",
        SkyModelType::Hosek => "hosek",
    }
}

#[allow(dead_code)]
pub fn esky_to_json(cfg: &EnvSkyConfig) -> String {
    format!(
        r#"{{"model":"{}","elevation":{:.4},"azimuth":{:.4},"turbidity":{:.4}}}"#,
        esky_model_name(cfg),
        cfg.sun_elevation_rad,
        cfg.sun_azimuth_rad,
        cfg.turbidity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_env_sky_config();
        assert!((cfg.turbidity - 2.0).abs() < 1e-6);
    }

    #[test]
    fn sun_direction_unit_length() {
        let cfg = default_env_sky_config();
        let d = esky_sun_direction(&cfg);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_elevation_clamps() {
        let mut cfg = default_env_sky_config();
        esky_set_elevation(&mut cfg, 5.0);
        assert!((cfg.sun_elevation_rad - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn set_turbidity_clamps() {
        let mut cfg = default_env_sky_config();
        esky_set_turbidity(&mut cfg, 0.5);
        assert!(cfg.turbidity >= 1.7);
    }

    #[test]
    fn sky_color_zenith_at_one() {
        let cfg = default_env_sky_config();
        let c = esky_sky_color_at_angle(&cfg, 1.0);
        for (a, b) in c.iter().zip(cfg.zenith_tint.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn sky_color_horizon_at_zero() {
        let cfg = default_env_sky_config();
        let c = esky_sky_color_at_angle(&cfg, 0.0);
        let expected_r = (cfg.zenith_tint[0] + cfg.horizon_tint[0]) * 0.5;
        assert!((c[0] - expected_r).abs() < 1e-5);
    }

    #[test]
    fn model_name_preetham() {
        let cfg = default_env_sky_config();
        assert_eq!(esky_model_name(&cfg), "preetham");
    }

    #[test]
    fn set_intensity_no_negative() {
        let mut cfg = default_env_sky_config();
        esky_set_intensity(&mut cfg, -1.0);
        assert!(cfg.sun_intensity >= 0.0);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_env_sky_config();
        let j = esky_to_json(&cfg);
        assert!(j.contains("model"));
        assert!(j.contains("turbidity"));
    }
}
