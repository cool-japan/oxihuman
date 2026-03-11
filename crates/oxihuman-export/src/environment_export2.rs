//! Environment and sky parameter export.
#![allow(dead_code)]

use std::f32::consts::PI;

/// Sky/environment parameters.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SkyParams2 {
    pub zenith_color: [f32; 3],
    pub horizon_color: [f32; 3],
    pub brightness: f32,
    pub exposure: f32,
}

/// Environment export struct.
#[allow(dead_code)]
pub struct EnvironmentExport2 {
    pub sky: SkyParams2,
}

/// Default sky parameters.
#[allow(dead_code)]
pub fn default_sky_params2() -> SkyParams2 {
    SkyParams2 {
        zenith_color: [0.1, 0.3, 0.8],
        horizon_color: [0.8, 0.7, 0.6],
        brightness: 1.0,
        exposure: 1.0,
    }
}

/// Export sky parameters to JSON.
#[allow(dead_code)]
pub fn export_sky_params2(sky: &SkyParams2) -> String {
    format!(
        r#"{{"zenith":[{},{},{}],"horizon":[{},{},{}],"brightness":{},"exposure":{}}}"#,
        sky.zenith_color[0], sky.zenith_color[1], sky.zenith_color[2],
        sky.horizon_color[0], sky.horizon_color[1], sky.horizon_color[2],
        sky.brightness, sky.exposure
    )
}

/// Get sky color at a given angle from horizon (0 = horizon, PI/2 = zenith).
#[allow(dead_code)]
pub fn sky_color_at_angle2(sky: &SkyParams2, angle: f32) -> [f32; 3] {
    let t = (angle / (PI * 0.5)).clamp(0.0, 1.0);
    [
        sky.horizon_color[0] + (sky.zenith_color[0] - sky.horizon_color[0]) * t,
        sky.horizon_color[1] + (sky.zenith_color[1] - sky.horizon_color[1]) * t,
        sky.horizon_color[2] + (sky.zenith_color[2] - sky.horizon_color[2]) * t,
    ]
}

/// Get zenith color.
#[allow(dead_code)]
pub fn sky2_zenith_color(sky: &SkyParams2) -> [f32; 3] { sky.zenith_color }

/// Get horizon color.
#[allow(dead_code)]
pub fn sky2_horizon_color(sky: &SkyParams2) -> [f32; 3] { sky.horizon_color }

/// Convert environment to JSON.
#[allow(dead_code)]
pub fn env2_to_json(env: &EnvironmentExport2) -> String { export_sky_params2(&env.sky) }

/// Get environment brightness.
#[allow(dead_code)]
pub fn env2_brightness(env: &EnvironmentExport2) -> f32 { env.sky.brightness }

/// Get environment exposure.
#[allow(dead_code)]
pub fn env2_exposure(env: &EnvironmentExport2) -> f32 { env.sky.exposure }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sky_brightness() {
        let s = default_sky_params2();
        assert!((s.brightness - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_default_sky_exposure() {
        let s = default_sky_params2();
        assert!((s.exposure - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_export_sky_json() {
        let s = default_sky_params2();
        let j = export_sky_params2(&s);
        assert!(j.contains("zenith"));
    }

    #[test]
    fn test_sky_color_at_horizon() {
        let s = default_sky_params2();
        let c = sky_color_at_angle2(&s, 0.0);
        assert!((c[0] - s.horizon_color[0]).abs() < 1e-5);
    }

    #[test]
    fn test_sky_color_at_zenith() {
        let s = default_sky_params2();
        let c = sky_color_at_angle2(&s, PI * 0.5);
        assert!((c[0] - s.zenith_color[0]).abs() < 1e-4);
    }

    #[test]
    fn test_sky2_zenith_color() {
        let s = default_sky_params2();
        let z = sky2_zenith_color(&s);
        assert!((z[2] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_sky2_horizon_color() {
        let s = default_sky_params2();
        let h = sky2_horizon_color(&s);
        assert!((h[0] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_env_brightness() {
        let env = EnvironmentExport2 { sky: default_sky_params2() };
        assert!((env2_brightness(&env) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_env_to_json() {
        let env = EnvironmentExport2 { sky: default_sky_params2() };
        let j = env2_to_json(&env);
        assert!(j.contains("horizon"));
    }
}
