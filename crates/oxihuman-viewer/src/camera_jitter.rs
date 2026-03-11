// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Camera jitter patterns for temporal anti-aliasing.

use std::f32::consts::PI;

/// Configuration for camera jitter.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraJitterConfig {
    pub amplitude: f32,
    pub frequency: f32,
    pub pattern_index: f32,
    pub enabled: f32,
    pub frame_count: f32,
}

#[allow(dead_code)]
pub fn default_camera_jitter() -> CameraJitterConfig {
    CameraJitterConfig { amplitude: 0.5, frequency: 1.0, pattern_index: 0.0, enabled: 1.0, frame_count: 0.0 }
}

#[allow(dead_code)]
pub fn set_camera_jitter_amplitude(cfg: &mut CameraJitterConfig, value: f32) {
    cfg.amplitude = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_camera_jitter_frequency(cfg: &mut CameraJitterConfig, value: f32) {
    cfg.frequency = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_camera_jitter_pattern_index(cfg: &mut CameraJitterConfig, value: f32) {
    cfg.pattern_index = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_camera_jitter_enabled(cfg: &mut CameraJitterConfig, value: f32) {
    cfg.enabled = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_camera_jitter_frame_count(cfg: &mut CameraJitterConfig, value: f32) {
    cfg.frame_count = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn camera_jitter_weight(cfg: &CameraJitterConfig) -> f32 {
    (cfg.amplitude * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_camera_jitter(a: &CameraJitterConfig, b: &CameraJitterConfig, t: f32) -> CameraJitterConfig {
    let t = t.clamp(0.0, 1.0);
    CameraJitterConfig {
        amplitude: a.amplitude + (b.amplitude - a.amplitude) * t,
        frequency: a.frequency + (b.frequency - a.frequency) * t,
        pattern_index: a.pattern_index + (b.pattern_index - a.pattern_index) * t,
        enabled: a.enabled + (b.enabled - a.enabled) * t,
        frame_count: a.frame_count + (b.frame_count - a.frame_count) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_camera_jitter();
        assert!((cfg.amplitude - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_amplitude() {
        let mut cfg = default_camera_jitter();
        set_camera_jitter_amplitude(&mut cfg, 0.7);
        assert!((cfg.amplitude - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_frequency() {
        let mut cfg = default_camera_jitter();
        set_camera_jitter_frequency(&mut cfg, 0.8);
        assert!((cfg.frequency - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_pattern_index() {
        let mut cfg = default_camera_jitter();
        set_camera_jitter_pattern_index(&mut cfg, 0.6);
        assert!((cfg.pattern_index - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_enabled() {
        let mut cfg = default_camera_jitter();
        set_camera_jitter_enabled(&mut cfg, 0.5);
        assert!((cfg.enabled - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_frame_count() {
        let mut cfg = default_camera_jitter();
        set_camera_jitter_frame_count(&mut cfg, 0.4);
        assert!((cfg.frame_count - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_camera_jitter();
        let w = camera_jitter_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_camera_jitter();
        let mut b = default_camera_jitter();
        b.amplitude = 1.0;
        let mid = blend_camera_jitter(&a, &b, 0.5);
        assert!((mid.amplitude - 0.75_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_camera_jitter();
        let b = default_camera_jitter();
        let r = blend_camera_jitter(&a, &b, 0.0);
        assert!((r.amplitude - a.amplitude).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_camera_jitter();
        let b = default_camera_jitter();
        let r = blend_camera_jitter(&a, &b, 1.0);
        assert!((r.amplitude - b.amplitude).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_camera_jitter();
        let b = default_camera_jitter();
        let r = blend_camera_jitter(&a, &b, 2.0);
        assert!((r.amplitude - b.amplitude).abs() < 1e-6);
    }
}
