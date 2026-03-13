// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Datamosh effect — datamosh/compression-artifact effect parameters.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DatamoshEffectConfig {
    /// I-frame interval: number of frames between full resets.
    pub iframe_interval: u32,
    /// Block size in pixels for motion copying (typical 4..=32).
    pub block_size_px: u32,
    /// Blur amount applied to accumulated error 0..=1.
    pub error_blur: f32,
    /// Intensity of the artifact overlay 0..=1.
    pub intensity: f32,
    /// Feedback factor: how much of the previous frame bleeds through 0..=1.
    pub feedback: f32,
    pub enabled: bool,
}

impl Default for DatamoshEffectConfig {
    fn default() -> Self {
        Self {
            iframe_interval: 30,
            block_size_px: 16,
            error_blur: 0.3,
            intensity: 0.6,
            feedback: 0.85,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_datamosh_effect_config() -> DatamoshEffectConfig {
    DatamoshEffectConfig::default()
}

#[allow(dead_code)]
pub fn dm_set_intensity(cfg: &mut DatamoshEffectConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn dm_set_feedback(cfg: &mut DatamoshEffectConfig, v: f32) {
    cfg.feedback = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn dm_set_block_size(cfg: &mut DatamoshEffectConfig, px: u32) {
    cfg.block_size_px = px.clamp(2, 64);
}

#[allow(dead_code)]
pub fn dm_set_iframe_interval(cfg: &mut DatamoshEffectConfig, frames: u32) {
    cfg.iframe_interval = frames.max(1);
}

/// Returns true if the given frame number is an I-frame.
#[allow(dead_code)]
pub fn dm_is_iframe(frame: u32, cfg: &DatamoshEffectConfig) -> bool {
    cfg.iframe_interval > 0 && frame.is_multiple_of(cfg.iframe_interval)
}

/// Simulate accumulating error: applies feedback blend between current and accumulated.
#[allow(dead_code)]
pub fn dm_accumulate(current: f32, accumulated: f32, cfg: &DatamoshEffectConfig) -> f32 {
    if !cfg.enabled {
        return current;
    }
    let feedback = cfg.feedback;
    (accumulated * feedback + current * (1.0 - feedback)).clamp(0.0, 1.0)
}

/// Returns the effective artifact strength for a given frame.
#[allow(dead_code)]
pub fn dm_artifact_strength(frame: u32, cfg: &DatamoshEffectConfig) -> f32 {
    if !cfg.enabled || dm_is_iframe(frame, cfg) {
        0.0
    } else {
        cfg.intensity
    }
}

/// Compute total blocks for given resolution.
#[allow(dead_code)]
pub fn dm_block_count(width_px: u32, height_px: u32, cfg: &DatamoshEffectConfig) -> u32 {
    let bx = width_px.div_ceil(cfg.block_size_px);
    let by = height_px.div_ceil(cfg.block_size_px);
    bx * by
}

#[allow(dead_code)]
pub fn dm_to_json(cfg: &DatamoshEffectConfig) -> String {
    format!(
        "{{\"iframe_interval\":{},\"intensity\":{:.3},\"feedback\":{:.3},\"enabled\":{}}}",
        cfg.iframe_interval, cfg.intensity, cfg.feedback, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iframe_on_interval() {
        let cfg = new_datamosh_effect_config();
        assert!(dm_is_iframe(0, &cfg));
        assert!(dm_is_iframe(30, &cfg));
    }

    #[test]
    fn not_iframe_between() {
        let cfg = new_datamosh_effect_config();
        assert!(!dm_is_iframe(15, &cfg));
    }

    #[test]
    fn artifact_zero_on_iframe() {
        let cfg = new_datamosh_effect_config();
        assert!(dm_artifact_strength(0, &cfg) < 1e-6);
    }

    #[test]
    fn artifact_nonzero_between_iframes() {
        let cfg = new_datamosh_effect_config();
        assert!(dm_artifact_strength(5, &cfg) > 0.0);
    }

    #[test]
    fn artifact_zero_when_disabled() {
        let mut cfg = new_datamosh_effect_config();
        cfg.enabled = false;
        assert!(dm_artifact_strength(5, &cfg) < 1e-6);
    }

    #[test]
    fn accumulate_moves_toward_current() {
        let cfg = new_datamosh_effect_config();
        let acc = dm_accumulate(0.0, 1.0, &cfg);
        assert!(acc < 1.0); // Pulled toward current (0)
    }

    #[test]
    fn block_count_correct() {
        let cfg = new_datamosh_effect_config(); // block_size=16
        let count = dm_block_count(64, 32, &cfg);
        assert_eq!(count, 4 * 2);
    }

    #[test]
    fn block_size_clamps() {
        let mut cfg = new_datamosh_effect_config();
        dm_set_block_size(&mut cfg, 1);
        assert_eq!(cfg.block_size_px, 2);
    }

    #[test]
    fn intensity_clamps() {
        let mut cfg = new_datamosh_effect_config();
        dm_set_intensity(&mut cfg, 5.0);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn json_has_keys() {
        let j = dm_to_json(&new_datamosh_effect_config());
        assert!(j.contains("iframe_interval") && j.contains("enabled"));
    }
}
