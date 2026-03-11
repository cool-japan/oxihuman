// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Motion stretch — velocity-based object stretching for motion blur stylisation.

/// Configuration for motion stretch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionStretchConfig {
    pub max_stretch: f32,
    pub velocity_threshold: f32,
    pub falloff: f32,
    pub enabled: bool,
}

/// A stretch instance for one object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StretchInstance {
    pub object_id: u32,
    pub velocity: [f32; 3],
    pub stretch_factor: f32,
}

#[allow(dead_code)]
pub fn default_motion_stretch_config() -> MotionStretchConfig {
    MotionStretchConfig {
        max_stretch: 2.0,
        velocity_threshold: 0.5,
        falloff: 1.5,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn ms_compute_stretch(cfg: &MotionStretchConfig, velocity_mag: f32) -> f32 {
    if !cfg.enabled || velocity_mag < cfg.velocity_threshold {
        return 1.0;
    }
    let excess = velocity_mag - cfg.velocity_threshold;
    (1.0 + excess * cfg.falloff).min(cfg.max_stretch)
}

#[allow(dead_code)]
pub fn ms_velocity_magnitude(velocity: [f32; 3]) -> f32 {
    (velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2]).sqrt()
}

#[allow(dead_code)]
pub fn ms_stretch_direction(velocity: [f32; 3]) -> [f32; 3] {
    let mag = ms_velocity_magnitude(velocity);
    if mag < 1e-6 {
        return [0.0, 0.0, 1.0];
    }
    [velocity[0] / mag, velocity[1] / mag, velocity[2] / mag]
}

#[allow(dead_code)]
pub fn ms_build_instance(
    cfg: &MotionStretchConfig,
    object_id: u32,
    velocity: [f32; 3],
) -> StretchInstance {
    let mag = ms_velocity_magnitude(velocity);
    let stretch_factor = ms_compute_stretch(cfg, mag);
    StretchInstance {
        object_id,
        velocity,
        stretch_factor,
    }
}

#[allow(dead_code)]
pub fn ms_set_max_stretch(cfg: &mut MotionStretchConfig, v: f32) {
    cfg.max_stretch = v.max(1.0);
}

#[allow(dead_code)]
pub fn ms_set_enabled(cfg: &mut MotionStretchConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn ms_to_json(cfg: &MotionStretchConfig) -> String {
    format!(
        r#"{{"max_stretch":{:.4},"threshold":{:.4},"falloff":{:.4},"enabled":{}}}"#,
        cfg.max_stretch, cfg.velocity_threshold, cfg.falloff, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_motion_stretch_config();
        assert!((cfg.max_stretch - 2.0).abs() < 1e-6);
    }

    #[test]
    fn no_stretch_below_threshold() {
        let cfg = default_motion_stretch_config();
        let s = ms_compute_stretch(&cfg, 0.1);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn stretch_above_threshold() {
        let cfg = default_motion_stretch_config();
        let s = ms_compute_stretch(&cfg, 1.0);
        assert!(s > 1.0);
    }

    #[test]
    fn stretch_clamped_to_max() {
        let cfg = default_motion_stretch_config();
        let s = ms_compute_stretch(&cfg, 100.0);
        assert!((s - cfg.max_stretch).abs() < 1e-6);
    }

    #[test]
    fn disabled_always_one() {
        let mut cfg = default_motion_stretch_config();
        ms_set_enabled(&mut cfg, false);
        let s = ms_compute_stretch(&cfg, 10.0);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn velocity_magnitude() {
        let mag = ms_velocity_magnitude([3.0, 4.0, 0.0]);
        assert!((mag - 5.0).abs() < 1e-5);
    }

    #[test]
    fn stretch_direction_unit() {
        let dir = ms_stretch_direction([1.0, 0.0, 0.0]);
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn build_instance() {
        let cfg = default_motion_stretch_config();
        let inst = ms_build_instance(&cfg, 42, [1.0, 0.0, 0.0]);
        assert_eq!(inst.object_id, 42);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_motion_stretch_config();
        let j = ms_to_json(&cfg);
        assert!(j.contains("max_stretch"));
    }
}
