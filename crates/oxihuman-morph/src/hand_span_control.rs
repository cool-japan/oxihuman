// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Hand span control for character hand morphing.

/// Hand span parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandSpanParams {
    pub span: f32,
    pub finger_length: f32,
    pub palm_width: f32,
    pub thickness: f32,
}

/// Hand span result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandSpanResult {
    pub span_weight: f32,
    pub finger_weight: f32,
    pub palm_weight: f32,
    pub overall_weight: f32,
}

/// Default hand span parameters.
#[allow(dead_code)]
pub fn default_hand_span() -> HandSpanParams {
    HandSpanParams {
        span: 0.5,
        finger_length: 0.5,
        palm_width: 0.5,
        thickness: 0.5,
    }
}

/// Evaluate hand span morph weights.
#[allow(dead_code)]
pub fn evaluate_hand_span(params: &HandSpanParams) -> HandSpanResult {
    let s = params.span.clamp(0.0, 1.0);
    let f = params.finger_length.clamp(0.0, 1.0);
    let p = params.palm_width.clamp(0.0, 1.0);
    HandSpanResult {
        span_weight: s,
        finger_weight: f,
        palm_weight: p,
        overall_weight: s * 0.4 + f * 0.3 + p * 0.3,
    }
}

/// Blend hand span params.
#[allow(dead_code)]
pub fn blend_hand_span(a: &HandSpanParams, b: &HandSpanParams, t: f32) -> HandSpanParams {
    let t = t.clamp(0.0, 1.0);
    HandSpanParams {
        span: a.span + (b.span - a.span) * t,
        finger_length: a.finger_length + (b.finger_length - a.finger_length) * t,
        palm_width: a.palm_width + (b.palm_width - a.palm_width) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
    }
}

/// Set span.
#[allow(dead_code)]
pub fn set_hand_span(params: &mut HandSpanParams, value: f32) {
    params.span = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_hand_span(params: &HandSpanParams) -> bool {
    (0.0..=1.0).contains(&params.span)
        && (0.0..=1.0).contains(&params.finger_length)
        && (0.0..=1.0).contains(&params.palm_width)
        && (0.0..=1.0).contains(&params.thickness)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_hand_span(params: &mut HandSpanParams) {
    *params = default_hand_span();
}

/// Compute finger-to-palm ratio.
#[allow(dead_code)]
pub fn finger_palm_ratio(params: &HandSpanParams) -> f32 {
    if params.palm_width > 0.001 {
        params.finger_length / params.palm_width
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_hand_span();
        assert!((p.span - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_hand_span();
        let r = evaluate_hand_span(&p);
        assert!((0.0..=1.0).contains(&r.overall_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_hand_span();
        let mut b = default_hand_span();
        b.span = 1.0;
        let c = blend_hand_span(&a, &b, 0.5);
        assert!((c.span - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set() {
        let mut p = default_hand_span();
        set_hand_span(&mut p, 0.8);
        assert!((p.span - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_hand_span(&default_hand_span()));
    }

    #[test]
    fn test_invalid() {
        let p = HandSpanParams { span: 2.0, finger_length: 0.5, palm_width: 0.5, thickness: 0.5 };
        assert!(!is_valid_hand_span(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = HandSpanParams { span: 0.9, finger_length: 0.1, palm_width: 0.2, thickness: 0.3 };
        reset_hand_span(&mut p);
        assert!((p.span - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ratio() {
        let p = default_hand_span();
        assert!((finger_palm_ratio(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_full() {
        let p = HandSpanParams { span: 1.0, finger_length: 1.0, palm_width: 1.0, thickness: 1.0 };
        let r = evaluate_hand_span(&p);
        assert!((r.overall_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero() {
        let p = HandSpanParams { span: 0.0, finger_length: 0.0, palm_width: 0.0, thickness: 0.0 };
        let r = evaluate_hand_span(&p);
        assert!(r.overall_weight.abs() < 1e-6);
    }
}
