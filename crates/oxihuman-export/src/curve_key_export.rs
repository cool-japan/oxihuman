// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Curve keyframe export: animation curves with Bezier control points.

use std::f32::consts::FRAC_PI_2;

/// Interpolation mode for a keyframe.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum KeyInterpolation {
    Constant,
    Linear,
    Bezier,
}

/// A single keyframe on an animation curve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveKey {
    pub time: f32,
    pub value: f32,
    pub tangent_in: f32,
    pub tangent_out: f32,
    pub interpolation: KeyInterpolation,
}

/// An animation curve (collection of keyframes).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveKeyExport {
    pub name: String,
    pub keys: Vec<CurveKey>,
}

/// Create a curve.
#[allow(dead_code)]
pub fn new_curve_key_export(name: &str) -> CurveKeyExport {
    CurveKeyExport {
        name: name.to_string(),
        keys: Vec::new(),
    }
}

/// Add a linear keyframe.
#[allow(dead_code)]
pub fn add_linear_key(curve: &mut CurveKeyExport, time: f32, value: f32) {
    curve.keys.push(CurveKey {
        time,
        value,
        tangent_in: 0.0,
        tangent_out: 0.0,
        interpolation: KeyInterpolation::Linear,
    });
}

/// Add a bezier keyframe.
#[allow(dead_code)]
pub fn add_bezier_key(
    curve: &mut CurveKeyExport,
    time: f32,
    value: f32,
    tan_in: f32,
    tan_out: f32,
) {
    curve.keys.push(CurveKey {
        time,
        value,
        tangent_in: tan_in,
        tangent_out: tan_out,
        interpolation: KeyInterpolation::Bezier,
    });
}

/// Key count.
#[allow(dead_code)]
pub fn curve_key_count(curve: &CurveKeyExport) -> usize {
    curve.keys.len()
}

/// Duration of curve.
#[allow(dead_code)]
pub fn curve_key_duration(curve: &CurveKeyExport) -> f32 {
    if curve.keys.is_empty() {
        return 0.0;
    }
    let max_t = curve.keys.iter().map(|k| k.time).fold(0.0_f32, f32::max);
    let min_t = curve.keys.iter().map(|k| k.time).fold(f32::MAX, f32::min);
    max_t - min_t
}

/// Evaluate curve at time t (linear interpolation between keyframes).
#[allow(dead_code)]
pub fn evaluate_curve_key(curve: &CurveKeyExport, t: f32) -> Option<f32> {
    if curve.keys.is_empty() {
        return None;
    }
    let keys = &curve.keys;
    if t <= keys[0].time {
        return Some(keys[0].value);
    }
    if t >= keys[keys.len() - 1].time {
        return Some(keys[keys.len() - 1].value);
    }
    for i in 0..keys.len() - 1 {
        if t >= keys[i].time && t <= keys[i + 1].time {
            let dt = keys[i + 1].time - keys[i].time;
            let alpha = if dt < 1e-12 {
                0.0
            } else {
                (t - keys[i].time) / dt
            };
            return Some(keys[i].value + alpha * (keys[i + 1].value - keys[i].value));
        }
    }
    None
}

/// Value range (max - min) over all keys.
#[allow(dead_code)]
pub fn curve_value_range_ck(curve: &CurveKeyExport) -> f32 {
    if curve.keys.is_empty() {
        return 0.0;
    }
    let max_v = curve.keys.iter().map(|k| k.value).fold(f32::MIN, f32::max);
    let min_v = curve.keys.iter().map(|k| k.value).fold(f32::MAX, f32::min);
    max_v - min_v
}

/// Export to JSON.
#[allow(dead_code)]
pub fn curve_key_to_json(curve: &CurveKeyExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"key_count\":{},\"frac_pi_2\":{:.6}}}",
        curve.name,
        curve.keys.len(),
        FRAC_PI_2
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_curve_key_export() {
        let c = new_curve_key_export("rotation");
        assert_eq!(c.name, "rotation".to_string());
        assert_eq!(curve_key_count(&c), 0);
    }

    #[test]
    fn test_add_linear_key() {
        let mut c = new_curve_key_export("t");
        add_linear_key(&mut c, 0.0, 0.0);
        add_linear_key(&mut c, 1.0, 1.0);
        assert_eq!(curve_key_count(&c), 2);
    }

    #[test]
    fn test_add_bezier_key() {
        let mut c = new_curve_key_export("t");
        add_bezier_key(&mut c, 0.0, 0.0, 0.0, 0.5);
        assert!(c.keys[0].interpolation == KeyInterpolation::Bezier);
    }

    #[test]
    fn test_curve_key_duration() {
        let mut c = new_curve_key_export("t");
        add_linear_key(&mut c, 0.0, 0.0);
        add_linear_key(&mut c, 2.0, 1.0);
        assert!((curve_key_duration(&c) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_curve_key_midpoint() {
        let mut c = new_curve_key_export("t");
        add_linear_key(&mut c, 0.0, 0.0);
        add_linear_key(&mut c, 2.0, 2.0);
        assert!((evaluate_curve_key(&c, 1.0).unwrap() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_curve_key_empty() {
        let c = new_curve_key_export("t");
        assert!(evaluate_curve_key(&c, 0.5).is_none());
    }

    #[test]
    fn test_curve_value_range_ck() {
        let mut c = new_curve_key_export("t");
        add_linear_key(&mut c, 0.0, -1.0);
        add_linear_key(&mut c, 1.0, 1.0);
        assert!((curve_value_range_ck(&c) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_curve_key_to_json() {
        let c = new_curve_key_export("rot");
        let j = curve_key_to_json(&c);
        assert!(j.contains("\"name\":\"rot\""));
    }

    #[test]
    fn test_duration_empty() {
        let c = new_curve_key_export("t");
        assert!((curve_key_duration(&c)).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_at_boundary() {
        let mut c = new_curve_key_export("t");
        add_linear_key(&mut c, 1.0, 5.0);
        assert!((evaluate_curve_key(&c, 0.0).unwrap() - 5.0).abs() < 1e-5);
        assert!((evaluate_curve_key(&c, 2.0).unwrap() - 5.0).abs() < 1e-5);
    }
}
