// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Finger spread (abduction/adduction) morph control.
//!
//! Models the lateral spreading of fingers independently for hand posing.

use std::f32::consts::PI;

/// Which finger.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

/// Per-finger spread parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FingerSpreadParams {
    /// Spread angles for each finger in radians (abduction from neutral).
    pub spreads: [f32; 5],
    /// Overall spread multiplier.
    pub global_scale: f32,
    /// Web skin stretch factor, 0..=1.
    pub web_stretch: f32,
}

impl Default for FingerSpreadParams {
    fn default() -> Self {
        Self {
            spreads: [0.0; 5],
            global_scale: 1.0,
            web_stretch: 0.3,
        }
    }
}

/// Result of finger spread evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerSpreadResult {
    /// Per-finger effective spread angle.
    pub effective_spreads: [f32; 5],
    /// Total hand width change estimate.
    pub width_change: f32,
    /// Web skin stretch weights per gap (4 gaps: thumb-index through ring-pinky).
    pub web_weights: [f32; 4],
}

/// Maximum spread angle per finger (anatomical limits).
#[allow(dead_code)]
pub fn max_spread(finger: Finger) -> f32 {
    match finger {
        Finger::Thumb => PI / 4.0,
        Finger::Index => PI / 8.0,
        Finger::Middle => PI / 12.0,
        Finger::Ring => PI / 8.0,
        Finger::Pinky => PI / 6.0,
    }
}

/// Clamp spread to anatomical limit.
#[allow(dead_code)]
pub fn clamp_spread(angle: f32, finger: Finger) -> f32 {
    let max = max_spread(finger);
    angle.clamp(-max * 0.5, max)
}

/// Compute the effective spread for one finger.
#[allow(dead_code)]
pub fn effective_spread(raw_angle: f32, finger: Finger, global_scale: f32) -> f32 {
    clamp_spread(raw_angle * global_scale, finger)
}

/// Web skin stretch between two adjacent fingers.
///
/// Returns a weight in 0..=1 representing how much the web skin is stretched.
#[allow(dead_code)]
pub fn web_stretch_weight(spread_a: f32, spread_b: f32, max_a: f32, max_b: f32) -> f32 {
    let diff = (spread_a - spread_b).abs();
    let max_diff = (max_a + max_b) * 0.5;
    if max_diff < 1e-6 {
        return 0.0;
    }
    (diff / max_diff).clamp(0.0, 1.0)
}

/// Finger index mapping.
#[allow(dead_code)]
pub fn finger_from_index(idx: usize) -> Option<Finger> {
    match idx {
        0 => Some(Finger::Thumb),
        1 => Some(Finger::Index),
        2 => Some(Finger::Middle),
        3 => Some(Finger::Ring),
        4 => Some(Finger::Pinky),
        _ => None,
    }
}

/// Evaluate finger spread morph.
#[allow(dead_code)]
pub fn evaluate_finger_spread(params: &FingerSpreadParams) -> FingerSpreadResult {
    let fingers = [
        Finger::Thumb,
        Finger::Index,
        Finger::Middle,
        Finger::Ring,
        Finger::Pinky,
    ];
    let mut effective_spreads = [0.0_f32; 5];

    for (i, &finger) in fingers.iter().enumerate() {
        effective_spreads[i] = effective_spread(params.spreads[i], finger, params.global_scale);
    }

    let mut web_weights = [0.0_f32; 4];
    for i in 0..4 {
        let max_a = max_spread(fingers[i]);
        let max_b = max_spread(fingers[i + 1]);
        web_weights[i] =
            web_stretch_weight(effective_spreads[i], effective_spreads[i + 1], max_a, max_b)
                * params.web_stretch;
    }

    let width_change: f32 = effective_spreads.iter().map(|s| s.sin()).sum::<f32>() * 0.01;

    FingerSpreadResult {
        effective_spreads,
        width_change,
        web_weights,
    }
}

/// Preset: relaxed spread.
#[allow(dead_code)]
pub fn preset_relaxed() -> FingerSpreadParams {
    FingerSpreadParams {
        spreads: [0.1, 0.03, 0.0, -0.02, -0.05],
        global_scale: 1.0,
        web_stretch: 0.3,
    }
}

/// Preset: wide spread.
#[allow(dead_code)]
pub fn preset_wide() -> FingerSpreadParams {
    FingerSpreadParams {
        spreads: [PI / 5.0, PI / 10.0, PI / 14.0, PI / 10.0, PI / 8.0],
        global_scale: 1.0,
        web_stretch: 0.8,
    }
}

/// Blend finger spread params.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn blend_finger_spread(
    a: &FingerSpreadParams,
    b: &FingerSpreadParams,
    t: f32,
) -> FingerSpreadParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let mut spreads = [0.0; 5];
    for i in 0..5 {
        spreads[i] = a.spreads[i] * inv + b.spreads[i] * t;
    }
    FingerSpreadParams {
        spreads,
        global_scale: a.global_scale * inv + b.global_scale * t,
        web_stretch: a.web_stretch * inv + b.web_stretch * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = FingerSpreadParams::default();
        assert_eq!(p.spreads, [0.0; 5]);
    }

    #[test]
    fn test_max_spread_thumb_largest() {
        let thumb = max_spread(Finger::Thumb);
        let middle = max_spread(Finger::Middle);
        assert!(thumb > middle);
    }

    #[test]
    fn test_clamp_spread() {
        let clamped = clamp_spread(PI, Finger::Index);
        assert!(clamped <= max_spread(Finger::Index));
    }

    #[test]
    fn test_effective_spread_zero_scale() {
        let e = effective_spread(0.5, Finger::Index, 0.0);
        assert!(e.abs() < 1e-6);
    }

    #[test]
    fn test_web_stretch_weight_same() {
        let w = web_stretch_weight(0.1, 0.1, 0.5, 0.5);
        assert!(w.abs() < 1e-6);
    }

    #[test]
    fn test_finger_from_index_valid() {
        assert_eq!(finger_from_index(0), Some(Finger::Thumb));
        assert_eq!(finger_from_index(4), Some(Finger::Pinky));
    }

    #[test]
    fn test_finger_from_index_invalid() {
        assert_eq!(finger_from_index(5), None);
    }

    #[test]
    fn test_evaluate_default() {
        let r = evaluate_finger_spread(&FingerSpreadParams::default());
        assert_eq!(r.effective_spreads, [0.0; 5]);
        assert!(r.width_change.abs() < 1e-6);
    }

    #[test]
    fn test_preset_wide_nonzero() {
        let p = preset_wide();
        let r = evaluate_finger_spread(&p);
        assert!(r.width_change > 0.0);
    }

    #[test]
    fn test_blend_finger_spread() {
        let a = FingerSpreadParams::default();
        let b = preset_wide();
        let r = blend_finger_spread(&a, &b, 0.5);
        assert!(r.spreads[0] > 0.0);
    }
}
