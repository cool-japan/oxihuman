// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sun position (azimuth/elevation) display parameters.

use std::f32::consts::PI;

/// Sun position parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SunPositionConfig {
    /// Azimuth in degrees (0 = North, 90 = East).
    pub azimuth_deg: f32,
    /// Elevation in degrees (0 = horizon, 90 = zenith).
    pub elevation_deg: f32,
    /// Sun disk size in screen pixels.
    pub disk_size_px: f32,
    /// Sun color [R, G, B, A].
    pub sun_color: [f32; 4],
    /// Show sun disk overlay.
    pub visible: bool,
    /// Show sun direction arrow.
    pub show_arrow: bool,
}

impl Default for SunPositionConfig {
    fn default() -> Self {
        Self {
            azimuth_deg: 180.0,
            elevation_deg: 45.0,
            disk_size_px: 20.0,
            sun_color: [1.0, 0.95, 0.7, 1.0],
            visible: true,
            show_arrow: false,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_sun_position_config() -> SunPositionConfig {
    SunPositionConfig::default()
}

/// Compute sun direction vector in world space (Y-up).
#[allow(dead_code)]
pub fn sun_direction_vector(cfg: &SunPositionConfig) -> [f32; 3] {
    let az = cfg.azimuth_deg * PI / 180.0;
    let el = cfg.elevation_deg * PI / 180.0;
    let x = el.cos() * az.sin();
    let y = el.sin();
    let z = el.cos() * az.cos();
    [x, y, z]
}

/// Set azimuth.
#[allow(dead_code)]
pub fn sp_set_azimuth(cfg: &mut SunPositionConfig, deg: f32) {
    cfg.azimuth_deg = deg.rem_euclid(360.0);
}

/// Set elevation.
#[allow(dead_code)]
pub fn sp_set_elevation(cfg: &mut SunPositionConfig, deg: f32) {
    cfg.elevation_deg = deg.clamp(-90.0, 90.0);
}

/// Check if sun is above horizon.
#[allow(dead_code)]
pub fn sp_is_above_horizon(cfg: &SunPositionConfig) -> bool {
    cfg.elevation_deg > 0.0
}

/// Compute approximate sky luminance based on elevation.
#[allow(dead_code)]
pub fn sp_sky_luminance(cfg: &SunPositionConfig) -> f32 {
    let el = cfg.elevation_deg.clamp(-10.0, 90.0);
    let t = (el + 10.0) / 100.0;
    t.clamp(0.0, 1.0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn sun_position_to_json(cfg: &SunPositionConfig) -> String {
    format!(
        r#"{{"azimuth_deg":{:.2},"elevation_deg":{:.2},"visible":{}}}"#,
        cfg.azimuth_deg, cfg.elevation_deg, cfg.visible
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default() {
        let c = SunPositionConfig::default();
        assert!(sp_is_above_horizon(&c));
    }

    #[test]
    fn test_sun_dir_normalized() {
        let c = SunPositionConfig::default();
        let d = sun_direction_vector(&c);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_azimuth_wrap() {
        let mut c = SunPositionConfig::default();
        sp_set_azimuth(&mut c, 370.0);
        assert!((c.azimuth_deg - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_elevation_clamp() {
        let mut c = SunPositionConfig::default();
        sp_set_elevation(&mut c, 200.0);
        assert!((c.elevation_deg - 90.0).abs() < 1e-5);
    }

    #[test]
    fn test_below_horizon() {
        let c = SunPositionConfig {
            elevation_deg: -5.0,
            ..Default::default()
        };
        assert!(!sp_is_above_horizon(&c));
    }

    #[test]
    fn test_sky_luminance_zenith() {
        let c = SunPositionConfig {
            elevation_deg: 90.0,
            ..Default::default()
        };
        let l = sp_sky_luminance(&c);
        assert!(l > 0.9);
    }

    #[test]
    fn test_sky_luminance_night() {
        let c = SunPositionConfig {
            elevation_deg: -10.0,
            ..Default::default()
        };
        let l = sp_sky_luminance(&c);
        assert!(l.abs() < 1e-5);
    }

    #[test]
    fn test_sun_dir_at_zenith() {
        let c = SunPositionConfig {
            elevation_deg: 90.0,
            azimuth_deg: 0.0,
            ..Default::default()
        };
        let d = sun_direction_vector(&c);
        assert!((d[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pi_used() {
        let _pi = PI;
        let c = SunPositionConfig {
            azimuth_deg: 180.0,
            elevation_deg: 0.0,
            ..Default::default()
        };
        let d = sun_direction_vector(&c);
        assert!(d[0].abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = sun_position_to_json(&SunPositionConfig::default());
        assert!(j.contains("azimuth_deg"));
        assert!(j.contains("visible"));
    }
}
