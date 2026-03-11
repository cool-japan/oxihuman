// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eyelash count and curve density parameters.

use std::f32::consts::PI;

/// Eyelash density parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EyelashDensityParams {
    /// Number of lashes per eye (upper lash line).
    pub upper_count: u32,
    /// Number of lashes per eye (lower lash line).
    pub lower_count: u32,
    /// Lash length 0..=1.
    pub length: f32,
    /// Lash curl angle in radians (0 = straight, PI/4 = strong curl).
    pub curl_angle: f32,
    /// Lash thickness 0..=1.
    pub thickness: f32,
    /// Darkness 0..=1.
    pub darkness: f32,
}

impl Default for EyelashDensityParams {
    fn default() -> Self {
        Self {
            upper_count: 80,
            lower_count: 40,
            length: 0.5,
            curl_angle: PI / 8.0,
            thickness: 0.4,
            darkness: 0.9,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_eyelash_density_params() -> EyelashDensityParams {
    EyelashDensityParams::default()
}

/// Set upper lash count.
#[allow(dead_code)]
pub fn set_upper_lash_count(params: &mut EyelashDensityParams, count: u32) {
    params.upper_count = count.clamp(0, 200);
}

/// Set lower lash count.
#[allow(dead_code)]
pub fn set_lower_lash_count(params: &mut EyelashDensityParams, count: u32) {
    params.lower_count = count.clamp(0, 100);
}

/// Set lash length.
#[allow(dead_code)]
pub fn set_lash_length(params: &mut EyelashDensityParams, value: f32) {
    params.length = value.clamp(0.0, 1.0);
}

/// Set lash curl angle.
#[allow(dead_code)]
pub fn set_lash_curl(params: &mut EyelashDensityParams, angle_rad: f32) {
    params.curl_angle = angle_rad.clamp(0.0, PI / 2.0);
}

/// Set lash thickness.
#[allow(dead_code)]
pub fn set_lash_thickness(params: &mut EyelashDensityParams, value: f32) {
    params.thickness = value.clamp(0.0, 1.0);
}

/// Set lash darkness.
#[allow(dead_code)]
pub fn set_lash_darkness(params: &mut EyelashDensityParams, value: f32) {
    params.darkness = value.clamp(0.0, 1.0);
}

/// Total lash count (upper + lower) for both eyes.
#[allow(dead_code)]
pub fn total_lash_count(params: &EyelashDensityParams) -> u32 {
    (params.upper_count + params.lower_count) * 2
}

/// Curl tip position relative to base (unit length).
#[allow(dead_code)]
pub fn curl_tip_offset(params: &EyelashDensityParams) -> [f32; 2] {
    let a = params.curl_angle;
    [a.sin() * params.length, (1.0 - a.cos()) * params.length]
}

/// Blend two eyelash density params.
#[allow(dead_code)]
pub fn blend_eyelash_density(
    a: &EyelashDensityParams,
    b: &EyelashDensityParams,
    t: f32,
) -> EyelashDensityParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let lerp_u32 = |x: u32, y: u32| -> u32 { (x as f32 * inv + y as f32 * t).round() as u32 };
    EyelashDensityParams {
        upper_count: lerp_u32(a.upper_count, b.upper_count),
        lower_count: lerp_u32(a.lower_count, b.lower_count),
        length: a.length * inv + b.length * t,
        curl_angle: a.curl_angle * inv + b.curl_angle * t,
        thickness: a.thickness * inv + b.thickness * t,
        darkness: a.darkness * inv + b.darkness * t,
    }
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_eyelash_density(params: &mut EyelashDensityParams) {
    *params = EyelashDensityParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn eyelash_density_to_json(params: &EyelashDensityParams) -> String {
    format!(
        r#"{{"upper_count":{},"lower_count":{},"length":{:.4},"curl_angle":{:.4},"thickness":{:.4},"darkness":{:.4}}}"#,
        params.upper_count,
        params.lower_count,
        params.length,
        params.curl_angle,
        params.thickness,
        params.darkness
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default() {
        let p = EyelashDensityParams::default();
        assert!(p.upper_count > 0);
        assert!(p.lower_count > 0);
    }

    #[test]
    fn test_set_upper_count_clamp() {
        let mut p = EyelashDensityParams::default();
        set_upper_lash_count(&mut p, 999);
        assert!(p.upper_count <= 200);
    }

    #[test]
    fn test_set_length_clamp() {
        let mut p = EyelashDensityParams::default();
        set_lash_length(&mut p, 5.0);
        assert!((p.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_curl_clamp() {
        let mut p = EyelashDensityParams::default();
        set_lash_curl(&mut p, PI * 2.0);
        assert!(p.curl_angle <= PI / 2.0 + 1e-5);
    }

    #[test]
    fn test_total_count() {
        let p = EyelashDensityParams::default();
        let total = total_lash_count(&p);
        assert_eq!(total, (p.upper_count + p.lower_count) * 2);
    }

    #[test]
    fn test_curl_tip_straight() {
        let mut p = EyelashDensityParams::default();
        set_lash_curl(&mut p, 0.0);
        let tip = curl_tip_offset(&p);
        assert!(tip[0].abs() < 1e-6);
        assert!(tip[1].abs() < 1e-6);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = EyelashDensityParams {
            length: 0.0,
            ..Default::default()
        };
        let b = EyelashDensityParams {
            length: 1.0,
            ..Default::default()
        };
        let r = blend_eyelash_density(&a, &b, 0.5);
        assert!((r.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = EyelashDensityParams {
            length: 0.0,
            ..Default::default()
        };
        reset_eyelash_density(&mut p);
        assert!((p.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = eyelash_density_to_json(&EyelashDensityParams::default());
        assert!(j.contains("upper_count"));
        assert!(j.contains("darkness"));
    }

    #[test]
    fn test_curl_angle_uses_pi() {
        let p = EyelashDensityParams::default();
        let _pi_ref = PI;
        assert!(p.curl_angle < PI);
    }
}
