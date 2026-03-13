// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Light falloff — attenuation models for point and spot lights.

/// Light attenuation model.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffModel {
    /// No falloff (constant).
    None,
    /// 1 / d (linear).
    Linear,
    /// 1 / d² (physically correct inverse-square).
    InverseSquare,
    /// Smooth window function (Lagarde & de Rousiers).
    Smooth,
}

/// Light falloff configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightFalloffConfig {
    pub model: FalloffModel,
    pub range: f32,
    pub inner_angle_cos: f32,
    pub outer_angle_cos: f32,
}

#[allow(dead_code)]
pub fn default_light_falloff() -> LightFalloffConfig {
    LightFalloffConfig {
        model: FalloffModel::InverseSquare,
        range: 10.0,
        inner_angle_cos: 0.9,
        outer_angle_cos: 0.7,
    }
}

#[allow(dead_code)]
pub fn lfo_set_range(cfg: &mut LightFalloffConfig, v: f32) {
    cfg.range = v.clamp(0.001, 1000.0);
}

#[allow(dead_code)]
pub fn lfo_set_model(cfg: &mut LightFalloffConfig, model: FalloffModel) {
    cfg.model = model;
}

#[allow(dead_code)]
pub fn lfo_attenuation(cfg: &LightFalloffConfig, distance: f32) -> f32 {
    let d = distance.max(0.0001);
    match cfg.model {
        FalloffModel::None => 1.0,
        FalloffModel::Linear => (1.0 - d / cfg.range).clamp(0.0, 1.0),
        FalloffModel::InverseSquare => {
            let ratio = (d / cfg.range).clamp(0.0, 1.0);
            let window = (1.0 - ratio * ratio * ratio * ratio).max(0.0).powi(2);
            window / (d * d + 1.0)
        }
        FalloffModel::Smooth => {
            let s = (d / cfg.range).clamp(0.0, 1.0);
            (1.0 - s * s).powi(2)
        }
    }
}

#[allow(dead_code)]
pub fn lfo_spot_attenuation(cfg: &LightFalloffConfig, cos_angle: f32) -> f32 {
    let t = ((cos_angle - cfg.outer_angle_cos)
        / (cfg.inner_angle_cos - cfg.outer_angle_cos).max(1e-5))
    .clamp(0.0, 1.0);
    t * t
}

#[allow(dead_code)]
pub fn lfo_at_range_is_zero(cfg: &LightFalloffConfig) -> bool {
    let v = lfo_attenuation(cfg, cfg.range);
    v < 1e-3
}

#[allow(dead_code)]
pub fn lfo_reset(cfg: &mut LightFalloffConfig) {
    *cfg = default_light_falloff();
}

#[allow(dead_code)]
pub fn lfo_to_json(cfg: &LightFalloffConfig) -> String {
    let model_str = match cfg.model {
        FalloffModel::None => "none",
        FalloffModel::Linear => "linear",
        FalloffModel::InverseSquare => "inverse_square",
        FalloffModel::Smooth => "smooth",
    };
    format!(r#"{{"model":"{}","range":{:.4}}}"#, model_str, cfg.range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_inverse_square() {
        let cfg = default_light_falloff();
        assert_eq!(cfg.model, FalloffModel::InverseSquare);
    }

    #[test]
    fn set_range_clamps() {
        let mut cfg = default_light_falloff();
        lfo_set_range(&mut cfg, 0.0);
        assert!(cfg.range > 0.0);
    }

    #[test]
    fn none_constant() {
        let mut cfg = default_light_falloff();
        lfo_set_model(&mut cfg, FalloffModel::None);
        assert!((lfo_attenuation(&cfg, 5.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn linear_at_range_zero() {
        let mut cfg = default_light_falloff();
        lfo_set_model(&mut cfg, FalloffModel::Linear);
        let v = lfo_attenuation(&cfg, cfg.range);
        assert!(v < 1e-5);
    }

    #[test]
    fn inverse_square_decreases_with_distance() {
        let cfg = default_light_falloff();
        let a1 = lfo_attenuation(&cfg, 1.0);
        let a2 = lfo_attenuation(&cfg, 5.0);
        assert!(a1 > a2);
    }

    #[test]
    fn smooth_at_zero_is_one() {
        let mut cfg = default_light_falloff();
        lfo_set_model(&mut cfg, FalloffModel::Smooth);
        let v = lfo_attenuation(&cfg, 0.0001);
        assert!(v > 0.9);
    }

    #[test]
    fn spot_attenuation_inside_inner() {
        let cfg = default_light_falloff();
        let v = lfo_spot_attenuation(&cfg, 0.95);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn spot_attenuation_outside_outer() {
        let cfg = default_light_falloff();
        let v = lfo_spot_attenuation(&cfg, 0.5);
        assert!(v < 1e-5);
    }

    #[test]
    fn reset_restores() {
        let mut cfg = default_light_falloff();
        lfo_set_range(&mut cfg, 99.0);
        lfo_reset(&mut cfg);
        assert!((cfg.range - 10.0).abs() < 1e-5);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_light_falloff();
        let j = lfo_to_json(&cfg);
        assert!(j.contains("model"));
        assert!(j.contains("range"));
    }
}
