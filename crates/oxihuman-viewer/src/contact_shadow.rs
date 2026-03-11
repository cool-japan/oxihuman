// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Contact shadow rendering for ground-proximity shadows.

use std::f32::consts::PI;

/// Contact shadow settings.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactShadowConfig {
    pub enabled: bool,
    pub max_distance: f32,
    pub intensity: f32,
    pub step_count: u32,
    pub thickness: f32,
}

#[allow(dead_code)]
pub fn default_contact_shadow() -> ContactShadowConfig {
    ContactShadowConfig {
        enabled: true,
        max_distance: 0.5,
        intensity: 0.8,
        step_count: 8,
        thickness: 0.02,
    }
}

#[allow(dead_code)]
pub fn enable_contact_shadow(cfg: &mut ContactShadowConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn disable_contact_shadow(cfg: &mut ContactShadowConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn set_shadow_intensity(cfg: &mut ContactShadowConfig, value: f32) {
    cfg.intensity = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_shadow_max_distance(cfg: &mut ContactShadowConfig, value: f32) {
    cfg.max_distance = value.max(0.01);
}

#[allow(dead_code)]
pub fn set_shadow_step_count(cfg: &mut ContactShadowConfig, count: u32) {
    cfg.step_count = count.clamp(1, 64);
}

#[allow(dead_code)]
pub fn set_shadow_thickness(cfg: &mut ContactShadowConfig, value: f32) {
    cfg.thickness = value.clamp(0.001, 0.5);
}

#[allow(dead_code)]
pub fn shadow_falloff(distance: f32, max_distance: f32) -> f32 {
    if max_distance <= 0.0 { return 0.0; }
    let ratio = (distance / max_distance).clamp(0.0, 1.0);
    let angle = ratio * PI * 0.5;
    1.0 - angle.sin()
}

#[allow(dead_code)]
pub fn shadow_step_size(cfg: &ContactShadowConfig) -> f32 {
    if cfg.step_count == 0 { return 0.0; }
    cfg.max_distance / cfg.step_count as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_contact_shadow();
        assert!(cfg.enabled);
        assert!((cfg.intensity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        let mut cfg = default_contact_shadow();
        disable_contact_shadow(&mut cfg);
        assert!(!cfg.enabled);
        enable_contact_shadow(&mut cfg);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_set_intensity_clamp() {
        let mut cfg = default_contact_shadow();
        set_shadow_intensity(&mut cfg, 1.5);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_distance() {
        let mut cfg = default_contact_shadow();
        set_shadow_max_distance(&mut cfg, 2.0);
        assert!((cfg.max_distance - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_step_count_clamp() {
        let mut cfg = default_contact_shadow();
        set_shadow_step_count(&mut cfg, 100);
        assert_eq!(cfg.step_count, 64);
    }

    #[test]
    fn test_set_thickness() {
        let mut cfg = default_contact_shadow();
        set_shadow_thickness(&mut cfg, 0.1);
        assert!((cfg.thickness - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_shadow_falloff_zero() {
        let f = shadow_falloff(0.0, 1.0);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_shadow_falloff_max() {
        let f = shadow_falloff(1.0, 1.0);
        assert!(f.abs() < 1e-6);
    }

    #[test]
    fn test_shadow_step_size() {
        let cfg = default_contact_shadow();
        let step = shadow_step_size(&cfg);
        assert!(step > 0.0);
    }

    #[test]
    fn test_shadow_falloff_negative_max() {
        let f = shadow_falloff(0.5, 0.0);
        assert!(f.abs() < 1e-6);
    }
}
