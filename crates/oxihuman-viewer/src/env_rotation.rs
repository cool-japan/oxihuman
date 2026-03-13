// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Environment map rotation controls.

use std::f32::consts::PI;

/// Configuration for env rotation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvRotationConfig {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub intensity: f32,
    pub blur_level: f32,
}

#[allow(dead_code)]
pub fn default_env_rotation() -> EnvRotationConfig {
    EnvRotationConfig { yaw: 0.0, pitch: 0.0, roll: 0.0, intensity: 0.5, blur_level: 0.0 }
}

#[allow(dead_code)]
pub fn set_env_rotation_yaw(cfg: &mut EnvRotationConfig, value: f32) {
    cfg.yaw = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_env_rotation_pitch(cfg: &mut EnvRotationConfig, value: f32) {
    cfg.pitch = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_env_rotation_roll(cfg: &mut EnvRotationConfig, value: f32) {
    cfg.roll = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_env_rotation_intensity(cfg: &mut EnvRotationConfig, value: f32) {
    cfg.intensity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_env_rotation_blur_level(cfg: &mut EnvRotationConfig, value: f32) {
    cfg.blur_level = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn env_rotation_weight(cfg: &EnvRotationConfig) -> f32 {
    (cfg.yaw * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_env_rotation(a: &EnvRotationConfig, b: &EnvRotationConfig, t: f32) -> EnvRotationConfig {
    let t = t.clamp(0.0, 1.0);
    EnvRotationConfig {
        yaw: a.yaw + (b.yaw - a.yaw) * t,
        pitch: a.pitch + (b.pitch - a.pitch) * t,
        roll: a.roll + (b.roll - a.roll) * t,
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        blur_level: a.blur_level + (b.blur_level - a.blur_level) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_env_rotation();
        assert!((cfg.yaw - 0.0_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_yaw() {
        let mut cfg = default_env_rotation();
        set_env_rotation_yaw(&mut cfg, 0.7);
        assert!((cfg.yaw - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_pitch() {
        let mut cfg = default_env_rotation();
        set_env_rotation_pitch(&mut cfg, 0.8);
        assert!((cfg.pitch - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_roll() {
        let mut cfg = default_env_rotation();
        set_env_rotation_roll(&mut cfg, 0.6);
        assert!((cfg.roll - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity() {
        let mut cfg = default_env_rotation();
        set_env_rotation_intensity(&mut cfg, 0.5);
        assert!((cfg.intensity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_blur_level() {
        let mut cfg = default_env_rotation();
        set_env_rotation_blur_level(&mut cfg, 0.4);
        assert!((cfg.blur_level - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_env_rotation();
        let w = env_rotation_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_env_rotation();
        let mut b = default_env_rotation();
        b.yaw = 1.0;
        let mid = blend_env_rotation(&a, &b, 0.5);
        assert!((mid.yaw - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_env_rotation();
        let b = default_env_rotation();
        let r = blend_env_rotation(&a, &b, 0.0);
        assert!((r.yaw - a.yaw).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_env_rotation();
        let b = default_env_rotation();
        let r = blend_env_rotation(&a, &b, 1.0);
        assert!((r.yaw - b.yaw).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_env_rotation();
        let b = default_env_rotation();
        let r = blend_env_rotation(&a, &b, 2.0);
        assert!((r.yaw - b.yaw).abs() < 1e-6);
    }
}
