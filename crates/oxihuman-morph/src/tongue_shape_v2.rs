// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tongue dorsum and tip shape morphs (v2).

use std::f32::consts::PI;

/// Tongue shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TongueShapeV2Params {
    /// Tongue tip pointedness 0..=1 (0 = wide/flat, 1 = pointed).
    pub tip_pointedness: f32,
    /// Tongue tip curl upward -1..=1.
    pub tip_curl: f32,
    /// Dorsum (back) arch height 0..=1.
    pub dorsum_arch: f32,
    /// Tongue width 0..=1.
    pub width: f32,
    /// Tongue thickness 0..=1.
    pub thickness: f32,
    /// Median groove depth 0..=1.
    pub median_groove: f32,
    /// Protrusion 0..=1 (how far tongue extends).
    pub protrusion: f32,
}

impl Default for TongueShapeV2Params {
    fn default() -> Self {
        Self {
            tip_pointedness: 0.3,
            tip_curl: 0.0,
            dorsum_arch: 0.4,
            width: 0.5,
            thickness: 0.5,
            median_groove: 0.3,
            protrusion: 0.0,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_tongue_shape_v2_params() -> TongueShapeV2Params {
    TongueShapeV2Params::default()
}

/// Set tip pointedness.
#[allow(dead_code)]
pub fn set_tip_pointedness(params: &mut TongueShapeV2Params, value: f32) {
    params.tip_pointedness = value.clamp(0.0, 1.0);
}

/// Set tip curl.
#[allow(dead_code)]
pub fn set_tip_curl(params: &mut TongueShapeV2Params, value: f32) {
    params.tip_curl = value.clamp(-1.0, 1.0);
}

/// Set dorsum arch height.
#[allow(dead_code)]
pub fn set_dorsum_arch(params: &mut TongueShapeV2Params, value: f32) {
    params.dorsum_arch = value.clamp(0.0, 1.0);
}

/// Set tongue protrusion.
#[allow(dead_code)]
pub fn set_tongue_protrusion(params: &mut TongueShapeV2Params, value: f32) {
    params.protrusion = value.clamp(0.0, 1.0);
}

/// Compute tongue surface height at a given length-normalized position.
///
/// `t` = 0 (base) to 1 (tip).
#[allow(dead_code)]
pub fn tongue_height_profile(t: f32, params: &TongueShapeV2Params) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let arch = params.dorsum_arch * (-((t - 0.4) / 0.3).powi(2)).exp();
    let tip_elevation = params.tip_curl * (t * PI * 0.5).sin() * t;
    arch + tip_elevation * 0.5
}

/// Compute tongue width at a given length position.
#[allow(dead_code)]
pub fn tongue_width_profile(t: f32, params: &TongueShapeV2Params) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let base_width = params.width;
    let tip_taper = 1.0 - params.tip_pointedness * t * t;
    base_width * tip_taper
}

/// Blend two tongue shape params.
#[allow(dead_code)]
pub fn blend_tongue_shape_v2(
    a: &TongueShapeV2Params,
    b: &TongueShapeV2Params,
    t: f32,
) -> TongueShapeV2Params {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    TongueShapeV2Params {
        tip_pointedness: a.tip_pointedness * inv + b.tip_pointedness * t,
        tip_curl: a.tip_curl * inv + b.tip_curl * t,
        dorsum_arch: a.dorsum_arch * inv + b.dorsum_arch * t,
        width: a.width * inv + b.width * t,
        thickness: a.thickness * inv + b.thickness * t,
        median_groove: a.median_groove * inv + b.median_groove * t,
        protrusion: a.protrusion * inv + b.protrusion * t,
    }
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_tongue_shape_v2(params: &mut TongueShapeV2Params) {
    *params = TongueShapeV2Params::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn tongue_shape_v2_to_json(params: &TongueShapeV2Params) -> String {
    format!(
        r#"{{"tip_pointedness":{:.4},"tip_curl":{:.4},"dorsum_arch":{:.4},"width":{:.4},"protrusion":{:.4}}}"#,
        params.tip_pointedness,
        params.tip_curl,
        params.dorsum_arch,
        params.width,
        params.protrusion
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default() {
        let p = TongueShapeV2Params::default();
        assert!((0.0..=1.0).contains(&p.tip_pointedness));
    }

    #[test]
    fn test_set_tip_pointedness_clamp() {
        let mut p = TongueShapeV2Params::default();
        set_tip_pointedness(&mut p, 5.0);
        assert!((p.tip_pointedness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_tip_curl_clamp() {
        let mut p = TongueShapeV2Params::default();
        set_tip_curl(&mut p, 5.0);
        assert!((p.tip_curl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_protrusion() {
        let mut p = TongueShapeV2Params::default();
        set_tongue_protrusion(&mut p, 0.7);
        assert!((p.protrusion - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_height_profile_base() {
        let p = TongueShapeV2Params::default();
        let h = tongue_height_profile(0.0, &p);
        assert!(h >= 0.0);
    }

    #[test]
    fn test_width_tip_narrower_than_base() {
        let p = TongueShapeV2Params {
            tip_pointedness: 1.0,
            width: 1.0,
            ..Default::default()
        };
        let w_base = tongue_width_profile(0.0, &p);
        let w_tip = tongue_width_profile(1.0, &p);
        assert!(w_tip < w_base);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = TongueShapeV2Params {
            protrusion: 0.0,
            ..Default::default()
        };
        let b = TongueShapeV2Params {
            protrusion: 1.0,
            ..Default::default()
        };
        let r = blend_tongue_shape_v2(&a, &b, 0.5);
        assert!((r.protrusion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = TongueShapeV2Params {
            protrusion: 0.9,
            ..Default::default()
        };
        reset_tongue_shape_v2(&mut p);
        assert!(p.protrusion.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = tongue_shape_v2_to_json(&TongueShapeV2Params::default());
        assert!(j.contains("tip_pointedness"));
        assert!(j.contains("protrusion"));
    }

    #[test]
    fn test_pi_used_in_profile() {
        let p = TongueShapeV2Params {
            tip_curl: 1.0,
            ..Default::default()
        };
        let _pi_ref = PI;
        let h = tongue_height_profile(1.0, &p);
        assert!(h.is_finite());
    }
}
