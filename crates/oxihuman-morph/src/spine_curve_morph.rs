// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Spine curvature morph — controls cervical, thoracic, and lumbar curves.

use std::f32::consts::PI;

/// Spine curvature morph configuration.
#[derive(Debug, Clone)]
pub struct SpineCurveMorph {
    pub cervical_lordosis: f32,
    pub thoracic_kyphosis: f32,
    pub lumbar_lordosis: f32,
    pub overall_flex: f32,
}

impl SpineCurveMorph {
    pub fn new() -> Self {
        Self {
            cervical_lordosis: 0.5,
            thoracic_kyphosis: 0.5,
            lumbar_lordosis: 0.5,
            overall_flex: 0.0,
        }
    }
}

impl Default for SpineCurveMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new spine curve morph with neutral values.
pub fn new_spine_curve_morph() -> SpineCurveMorph {
    SpineCurveMorph::new()
}

/// Set cervical lordosis degree (0 = flat, 1 = maximum lordosis).
pub fn scm_set_cervical_lordosis(m: &mut SpineCurveMorph, v: f32) {
    m.cervical_lordosis = v.clamp(0.0, 1.0);
}

/// Set thoracic kyphosis degree (0 = flat, 1 = maximum kyphosis).
pub fn scm_set_thoracic_kyphosis(m: &mut SpineCurveMorph, v: f32) {
    m.thoracic_kyphosis = v.clamp(0.0, 1.0);
}

/// Set lumbar lordosis degree (0 = flat, 1 = maximum lordosis).
pub fn scm_set_lumbar_lordosis(m: &mut SpineCurveMorph, v: f32) {
    m.lumbar_lordosis = v.clamp(0.0, 1.0);
}

/// Set overall spinal flexion offset (negative = extension, positive = flexion).
pub fn scm_set_overall_flex(m: &mut SpineCurveMorph, v: f32) {
    m.overall_flex = v.clamp(-1.0, 1.0);
}

/// Compute total angular deviation in radians as a simple heuristic.
pub fn scm_total_angle_rad(m: &SpineCurveMorph) -> f32 {
    let base = (m.cervical_lordosis + m.thoracic_kyphosis + m.lumbar_lordosis) / 3.0;
    base * PI * 0.5
}

/// Serialize to JSON-like string.
pub fn spine_curve_morph_to_json(m: &SpineCurveMorph) -> String {
    format!(
        r#"{{"cervical_lordosis":{:.4},"thoracic_kyphosis":{:.4},"lumbar_lordosis":{:.4},"overall_flex":{:.4}}}"#,
        m.cervical_lordosis, m.thoracic_kyphosis, m.lumbar_lordosis, m.overall_flex
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_spine_curve_morph();
        assert!((m.cervical_lordosis - 0.5).abs() < 1e-6);
        assert!((m.lumbar_lordosis - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cervical_clamp() {
        let mut m = new_spine_curve_morph();
        scm_set_cervical_lordosis(&mut m, 2.0);
        assert_eq!(m.cervical_lordosis, 1.0); /* clamped */
    }

    #[test]
    fn test_thoracic_clamp() {
        let mut m = new_spine_curve_morph();
        scm_set_thoracic_kyphosis(&mut m, -0.5);
        assert_eq!(m.thoracic_kyphosis, 0.0); /* clamped */
    }

    #[test]
    fn test_lumbar_set() {
        let mut m = new_spine_curve_morph();
        scm_set_lumbar_lordosis(&mut m, 0.8);
        assert!((m.lumbar_lordosis - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_overall_flex_clamp() {
        let mut m = new_spine_curve_morph();
        scm_set_overall_flex(&mut m, 3.0);
        assert_eq!(m.overall_flex, 1.0);
    }

    #[test]
    fn test_total_angle_positive() {
        let m = new_spine_curve_morph();
        assert!(scm_total_angle_rad(&m) > 0.0);
    }

    #[test]
    fn test_total_angle_max() {
        let mut m = new_spine_curve_morph();
        scm_set_cervical_lordosis(&mut m, 1.0);
        scm_set_thoracic_kyphosis(&mut m, 1.0);
        scm_set_lumbar_lordosis(&mut m, 1.0);
        let angle = scm_total_angle_rad(&m);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let m = new_spine_curve_morph();
        let s = spine_curve_morph_to_json(&m);
        assert!(s.contains("thoracic_kyphosis"));
    }

    #[test]
    fn test_clone() {
        let m = new_spine_curve_morph();
        let m2 = m.clone();
        assert!((m2.cervical_lordosis - m.cervical_lordosis).abs() < 1e-6);
    }
}
