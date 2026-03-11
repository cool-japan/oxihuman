// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Anti-aliasing mode selection and configuration.

/// Anti-aliasing technique.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AAMode {
    None,
    Msaa2x,
    Msaa4x,
    Msaa8x,
    Fxaa,
    Smaa,
    Taa,
}

/// Anti-aliasing configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AAConfig {
    pub mode: AAMode,
    pub sharpness: f32,
    pub jitter_scale: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_aa_config() -> AAConfig {
    AAConfig { mode: AAMode::Msaa4x, sharpness: 0.5, jitter_scale: 1.0, enabled: true }
}

#[allow(dead_code)]
pub fn set_aa_mode(config: &mut AAConfig, mode: AAMode) {
    config.mode = mode;
}

#[allow(dead_code)]
pub fn set_aa_sharpness(config: &mut AAConfig, value: f32) {
    config.sharpness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn enable_aa(config: &mut AAConfig) {
    config.enabled = true;
}

#[allow(dead_code)]
pub fn disable_aa(config: &mut AAConfig) {
    config.enabled = false;
}

#[allow(dead_code)]
pub fn msaa_sample_count(mode: AAMode) -> u32 {
    match mode {
        AAMode::None | AAMode::Fxaa | AAMode::Smaa | AAMode::Taa => 1,
        AAMode::Msaa2x => 2,
        AAMode::Msaa4x => 4,
        AAMode::Msaa8x => 8,
    }
}

#[allow(dead_code)]
pub fn is_post_process_aa(mode: AAMode) -> bool {
    matches!(mode, AAMode::Fxaa | AAMode::Smaa | AAMode::Taa)
}

#[allow(dead_code)]
pub fn aa_mode_name(mode: AAMode) -> &'static str {
    match mode {
        AAMode::None => "None",
        AAMode::Msaa2x => "MSAA 2x",
        AAMode::Msaa4x => "MSAA 4x",
        AAMode::Msaa8x => "MSAA 8x",
        AAMode::Fxaa => "FXAA",
        AAMode::Smaa => "SMAA",
        AAMode::Taa => "TAA",
    }
}

#[allow(dead_code)]
pub fn aa_quality_cost(mode: AAMode) -> f32 {
    match mode {
        AAMode::None => 0.0,
        AAMode::Fxaa => 0.1,
        AAMode::Msaa2x => 0.2,
        AAMode::Smaa => 0.3,
        AAMode::Msaa4x => 0.4,
        AAMode::Taa => 0.5,
        AAMode::Msaa8x => 0.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_aa_config();
        assert_eq!(cfg.mode, AAMode::Msaa4x);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_set_mode() {
        let mut cfg = default_aa_config();
        set_aa_mode(&mut cfg, AAMode::Fxaa);
        assert_eq!(cfg.mode, AAMode::Fxaa);
    }

    #[test]
    fn test_set_sharpness_clamp() {
        let mut cfg = default_aa_config();
        set_aa_sharpness(&mut cfg, 1.5);
        assert!((cfg.sharpness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        let mut cfg = default_aa_config();
        disable_aa(&mut cfg);
        assert!(!cfg.enabled);
        enable_aa(&mut cfg);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_msaa_sample_count() {
        assert_eq!(msaa_sample_count(AAMode::Msaa4x), 4);
        assert_eq!(msaa_sample_count(AAMode::None), 1);
    }

    #[test]
    fn test_is_post_process_aa() {
        assert!(is_post_process_aa(AAMode::Fxaa));
        assert!(!is_post_process_aa(AAMode::Msaa4x));
    }

    #[test]
    fn test_aa_mode_name() {
        assert_eq!(aa_mode_name(AAMode::Taa), "TAA");
    }

    #[test]
    fn test_aa_quality_cost() {
        assert!(aa_quality_cost(AAMode::None) < aa_quality_cost(AAMode::Msaa8x));
    }

    #[test]
    fn test_msaa8x_samples() {
        assert_eq!(msaa_sample_count(AAMode::Msaa8x), 8);
    }

    #[test]
    fn test_smaa_is_post_process() {
        assert!(is_post_process_aa(AAMode::Smaa));
    }
}
