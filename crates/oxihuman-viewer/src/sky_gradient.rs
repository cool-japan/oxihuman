// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Sky gradient/atmosphere renderer.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkyGradientConfig {
    pub zenith_color: [f32; 3],
    pub horizon_color: [f32; 3],
    pub ground_color: [f32; 3],
    pub sun_intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkyGradientState {
    pub sun_direction: [f32; 3],
    pub time_of_day: f32,
}

#[allow(dead_code)]
pub fn default_sky_gradient_config() -> SkyGradientConfig {
    SkyGradientConfig {
        zenith_color: [0.1, 0.4, 0.9],
        horizon_color: [0.7, 0.8, 1.0],
        ground_color: [0.3, 0.25, 0.2],
        sun_intensity: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_sky_gradient_state() -> SkyGradientState {
    SkyGradientState {
        sun_direction: [0.0, 1.0, 0.0],
        time_of_day: 0.5,
    }
}

#[allow(dead_code)]
pub fn sky_color_at_angle(
    config: &SkyGradientConfig,
    _state: &SkyGradientState,
    elevation: f32,
) -> [f32; 3] {
    let t = elevation.clamp(-1.0, 1.0);
    if t >= 0.0 {
        // interpolate horizon -> zenith
        let a = config.horizon_color;
        let b = config.zenith_color;
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    } else {
        // interpolate ground -> horizon
        let a = config.ground_color;
        let b = config.horizon_color;
        let tt = 1.0 + t;
        [
            a[0] + (b[0] - a[0]) * tt,
            a[1] + (b[1] - a[1]) * tt,
            a[2] + (b[2] - a[2]) * tt,
        ]
    }
}

#[allow(dead_code)]
pub fn sky_set_time_of_day(state: &mut SkyGradientState, time: f32) {
    state.time_of_day = time.clamp(0.0, 1.0);
    let angle = (time - 0.25) * std::f32::consts::PI * 2.0;
    state.sun_direction = [angle.cos(), angle.sin(), 0.0];
}

#[allow(dead_code)]
pub fn sky_sun_direction(state: &SkyGradientState) -> [f32; 3] {
    state.sun_direction
}

#[allow(dead_code)]
pub fn sky_set_sun_direction(state: &mut SkyGradientState, dir: [f32; 3]) {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len > 1e-6 {
        state.sun_direction = [dir[0] / len, dir[1] / len, dir[2] / len];
    }
}

#[allow(dead_code)]
pub fn sky_to_json(config: &SkyGradientConfig, state: &SkyGradientState) -> String {
    format!(
        r#"{{"time_of_day":{:.4},"sun_intensity":{:.4}}}"#,
        state.time_of_day, config.sun_intensity
    )
}

#[allow(dead_code)]
pub fn sky_is_day(state: &SkyGradientState) -> bool {
    // Sun is above horizon if y component > 0
    state.sun_direction[1] > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_sky_gradient_config();
        assert!((cfg.sun_intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_sky_gradient_state();
        assert!((s.time_of_day - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sky_color_at_zenith() {
        let cfg = default_sky_gradient_config();
        let s = new_sky_gradient_state();
        let c = sky_color_at_angle(&cfg, &s, 1.0);
        assert!((c[0] - cfg.zenith_color[0]).abs() < 1e-5);
    }

    #[test]
    fn test_sky_color_at_horizon() {
        let cfg = default_sky_gradient_config();
        let s = new_sky_gradient_state();
        let c = sky_color_at_angle(&cfg, &s, 0.0);
        assert!((c[0] - cfg.horizon_color[0]).abs() < 1e-5);
    }

    #[test]
    fn test_set_time_of_day_clamps() {
        let mut s = new_sky_gradient_state();
        sky_set_time_of_day(&mut s, 2.0);
        assert!((s.time_of_day - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sky_sun_direction() {
        let s = new_sky_gradient_state();
        let d = sky_sun_direction(&s);
        assert_eq!(d.len(), 3);
    }

    #[test]
    fn test_set_sun_direction_normalizes() {
        let mut s = new_sky_gradient_state();
        sky_set_sun_direction(&mut s, [3.0, 4.0, 0.0]);
        let d = s.sun_direction;
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sky_to_json() {
        let cfg = default_sky_gradient_config();
        let s = new_sky_gradient_state();
        let j = sky_to_json(&cfg, &s);
        assert!(j.contains("time_of_day"));
        assert!(j.contains("sun_intensity"));
    }

    #[test]
    fn test_sky_is_day() {
        let mut s = new_sky_gradient_state();
        s.sun_direction = [0.0, 1.0, 0.0];
        assert!(sky_is_day(&s));
        s.sun_direction = [0.0, -1.0, 0.0];
        assert!(!sky_is_day(&s));
    }
}
