// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Light attenuation models for point and spot lights.

use std::f32::consts::PI;

/// Attenuation model.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttenuationModel {
    InverseSquare,
    Linear,
    Quadratic,
    Custom,
}

/// Light attenuation parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightAttenuation {
    pub model: AttenuationModel,
    pub radius: f32,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

#[allow(dead_code)]
pub fn default_light_attenuation() -> LightAttenuation {
    LightAttenuation {
        model: AttenuationModel::InverseSquare,
        radius: 10.0,
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
    }
}

/// Compute attenuation factor at a given distance.
#[allow(dead_code)]
pub fn compute_attenuation(atten: &LightAttenuation, distance: f32) -> f32 {
    if distance < 0.0 || distance > atten.radius {
        return 0.0;
    }
    match atten.model {
        AttenuationModel::InverseSquare => 1.0 / (distance * distance + 1e-4),
        AttenuationModel::Linear => (1.0 - distance / atten.radius).max(0.0),
        AttenuationModel::Quadratic => {
            let t = 1.0 - distance / atten.radius;
            (t * t).max(0.0)
        }
        AttenuationModel::Custom => {
            1.0 / (atten.constant + atten.linear * distance + atten.quadratic * distance * distance)
        }
    }
}

/// Spot light angular attenuation.
#[allow(dead_code)]
pub fn spot_attenuation(cos_angle: f32, inner_cos: f32, outer_cos: f32) -> f32 {
    if cos_angle >= inner_cos {
        return 1.0;
    }
    if cos_angle <= outer_cos {
        return 0.0;
    }
    let t = (cos_angle - outer_cos) / (inner_cos - outer_cos);
    t * t
}

/// Luminous intensity from power in watts and solid angle.
#[allow(dead_code)]
pub fn luminous_intensity(power_watts: f32) -> f32 {
    power_watts / (4.0 * PI)
}

#[allow(dead_code)]
pub fn attenuation_model_name(model: AttenuationModel) -> &'static str {
    match model {
        AttenuationModel::InverseSquare => "inverse_square",
        AttenuationModel::Linear => "linear",
        AttenuationModel::Quadratic => "quadratic",
        AttenuationModel::Custom => "custom",
    }
}

#[allow(dead_code)]
pub fn set_radius(atten: &mut LightAttenuation, radius: f32) {
    atten.radius = radius.max(0.01);
}

#[allow(dead_code)]
pub fn attenuation_to_json(atten: &LightAttenuation) -> String {
    format!(
        r#"{{"model":"{}","radius":{:.4}}}"#,
        attenuation_model_name(atten.model),
        atten.radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_attenuation() {
        let a = default_light_attenuation();
        assert!((a.radius - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_inverse_square_at_zero() {
        let a = default_light_attenuation();
        let v = compute_attenuation(&a, 0.0);
        assert!(v > 100.0);
    }

    #[test]
    fn test_inverse_square_beyond_radius() {
        let a = default_light_attenuation();
        let v = compute_attenuation(&a, 20.0);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_linear_at_zero() {
        let mut a = default_light_attenuation();
        a.model = AttenuationModel::Linear;
        let v = compute_attenuation(&a, 0.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_at_radius() {
        let mut a = default_light_attenuation();
        a.model = AttenuationModel::Linear;
        let v = compute_attenuation(&a, 10.0);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_quadratic() {
        let mut a = default_light_attenuation();
        a.model = AttenuationModel::Quadratic;
        let v = compute_attenuation(&a, 5.0);
        assert!(v > 0.0 && v < 1.0);
    }

    #[test]
    fn test_spot_inside_cone() {
        let v = spot_attenuation(0.95, 0.9, 0.7);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_spot_outside_cone() {
        let v = spot_attenuation(0.5, 0.9, 0.7);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_luminous_intensity() {
        let li = luminous_intensity(100.0);
        assert!(li > 0.0);
    }

    #[test]
    fn test_attenuation_to_json() {
        let a = default_light_attenuation();
        let j = attenuation_to_json(&a);
        assert!(j.contains("inverse_square"));
    }
}
