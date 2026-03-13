// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Depth bias — polygon offset settings to reduce z-fighting artifacts.

/// Depth bias configuration (polygon offset).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthBiasConfig {
    /// Constant depth bias added to each fragment's depth.
    pub constant: f32,
    /// Slope-scaled depth bias.
    pub slope_scale: f32,
    /// Clamp for the maximum depth bias.
    pub clamp: f32,
    pub enabled: bool,
}

/// Preset depth bias for shadow maps.
#[allow(dead_code)]
pub fn shadow_map_depth_bias() -> DepthBiasConfig {
    DepthBiasConfig {
        constant: 1.0,
        slope_scale: 2.5,
        clamp: 0.0,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn default_depth_bias() -> DepthBiasConfig {
    DepthBiasConfig {
        constant: 0.0,
        slope_scale: 0.0,
        clamp: 0.0,
        enabled: false,
    }
}

#[allow(dead_code)]
pub fn db_set_constant(cfg: &mut DepthBiasConfig, v: f32) {
    cfg.constant = v;
}

#[allow(dead_code)]
pub fn db_set_slope_scale(cfg: &mut DepthBiasConfig, v: f32) {
    cfg.slope_scale = v.clamp(-16.0, 16.0);
}

#[allow(dead_code)]
pub fn db_set_clamp(cfg: &mut DepthBiasConfig, v: f32) {
    cfg.clamp = v;
}

#[allow(dead_code)]
pub fn db_set_enabled(cfg: &mut DepthBiasConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn db_is_neutral(cfg: &DepthBiasConfig) -> bool {
    !cfg.enabled || (cfg.constant.abs() < 1e-6 && cfg.slope_scale.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn db_estimated_bias_at_slope(cfg: &DepthBiasConfig, slope_deg: f32) -> f32 {
    if !cfg.enabled {
        return 0.0;
    }
    let slope = slope_deg.to_radians().tan().abs();
    cfg.constant + cfg.slope_scale * slope
}

#[allow(dead_code)]
pub fn db_reset(cfg: &mut DepthBiasConfig) {
    *cfg = default_depth_bias();
}

#[allow(dead_code)]
pub fn db_to_json(cfg: &DepthBiasConfig) -> String {
    format!(
        r#"{{"constant":{:.4},"slope_scale":{:.4},"clamp":{:.4},"enabled":{}}}"#,
        cfg.constant, cfg.slope_scale, cfg.clamp, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        let cfg = default_depth_bias();
        assert!(!cfg.enabled);
    }

    #[test]
    fn shadow_map_preset_enabled() {
        let cfg = shadow_map_depth_bias();
        assert!(cfg.enabled);
        assert!(cfg.constant > 0.0);
    }

    #[test]
    fn set_constant() {
        let mut cfg = default_depth_bias();
        db_set_constant(&mut cfg, 2.0);
        assert!((cfg.constant - 2.0).abs() < 1e-6);
    }

    #[test]
    fn set_slope_scale_clamps() {
        let mut cfg = default_depth_bias();
        db_set_slope_scale(&mut cfg, 100.0);
        assert!((cfg.slope_scale - 16.0).abs() < 1e-6);
    }

    #[test]
    fn neutral_when_disabled() {
        let cfg = default_depth_bias();
        assert!(db_is_neutral(&cfg));
    }

    #[test]
    fn estimated_bias_disabled() {
        let cfg = default_depth_bias();
        assert!((db_estimated_bias_at_slope(&cfg, 45.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn estimated_bias_enabled() {
        let mut cfg = shadow_map_depth_bias();
        cfg.slope_scale = 0.0;
        cfg.constant = 1.0;
        assert!((db_estimated_bias_at_slope(&cfg, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut cfg = shadow_map_depth_bias();
        db_reset(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn to_json_fields() {
        let cfg = shadow_map_depth_bias();
        let j = db_to_json(&cfg);
        assert!(j.contains("constant"));
        assert!(j.contains("slope_scale"));
    }
}
