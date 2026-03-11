// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A named blend weight for expression mixing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExprBlendWeight {
    pub name: String,
    pub value: f32,
}

/// Create a new blend weight with name and initial value 0.
#[allow(dead_code)]
pub fn new_blend_weight(name: &str) -> ExprBlendWeight {
    ExprBlendWeight {
        name: name.to_string(),
        value: 0.0,
    }
}

/// Set the blend weight value (clamped 0..=1).
#[allow(dead_code)]
pub fn set_blend_weight_value(bw: &mut ExprBlendWeight, v: f32) {
    bw.value = v.clamp(0.0, 1.0);
}

/// Get the blend weight value.
#[allow(dead_code)]
pub fn get_blend_weight_value(bw: &ExprBlendWeight) -> f32 {
    bw.value
}

/// Check if the blend weight is active (> epsilon).
#[allow(dead_code)]
pub fn blend_weight_is_active(bw: &ExprBlendWeight) -> bool {
    bw.value > 1e-6
}

/// Normalize a slice of blend weights so they sum to 1.
#[allow(dead_code)]
pub fn blend_weight_normalize(weights: &mut [ExprBlendWeight]) {
    let sum: f32 = weights.iter().map(|w| w.value).sum();
    if sum > 1e-9 {
        for w in weights.iter_mut() {
            w.value /= sum;
        }
    }
}

/// Return the sum of a slice of blend weights.
#[allow(dead_code)]
pub fn blend_weights_sum(weights: &[ExprBlendWeight]) -> f32 {
    weights.iter().map(|w| w.value).sum()
}

/// Serialize a blend weight to JSON.
#[allow(dead_code)]
pub fn blend_weight_to_json(bw: &ExprBlendWeight) -> String {
    format!("{{\"name\":\"{}\",\"value\":{:.4}}}", bw.name, bw.value)
}

/// Reset the blend weight to 0.
#[allow(dead_code)]
pub fn blend_weight_reset(bw: &mut ExprBlendWeight) {
    bw.value = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_zero() {
        let bw = new_blend_weight("test");
        assert!(get_blend_weight_value(&bw).abs() < 1e-6);
    }

    #[test]
    fn set_and_get() {
        let mut bw = new_blend_weight("x");
        set_blend_weight_value(&mut bw, 0.75);
        assert!((get_blend_weight_value(&bw) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn clamped_high() {
        let mut bw = new_blend_weight("x");
        set_blend_weight_value(&mut bw, 1.5);
        assert!((get_blend_weight_value(&bw) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamped_low() {
        let mut bw = new_blend_weight("x");
        set_blend_weight_value(&mut bw, -0.5);
        assert!(get_blend_weight_value(&bw).abs() < 1e-6);
    }

    #[test]
    fn is_active() {
        let mut bw = new_blend_weight("x");
        assert!(!blend_weight_is_active(&bw));
        set_blend_weight_value(&mut bw, 0.5);
        assert!(blend_weight_is_active(&bw));
    }

    #[test]
    fn normalize() {
        let mut ws = vec![new_blend_weight("a"), new_blend_weight("b")];
        set_blend_weight_value(&mut ws[0], 0.6);
        set_blend_weight_value(&mut ws[1], 0.4);
        blend_weight_normalize(&mut ws);
        let sum = blend_weights_sum(&ws);
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_zeros() {
        let mut ws = vec![new_blend_weight("a")];
        blend_weight_normalize(&mut ws);
        assert!(get_blend_weight_value(&ws[0]).abs() < 1e-6);
    }

    #[test]
    fn sum_of_weights() {
        let mut ws = vec![new_blend_weight("a"), new_blend_weight("b")];
        set_blend_weight_value(&mut ws[0], 0.3);
        set_blend_weight_value(&mut ws[1], 0.5);
        assert!((blend_weights_sum(&ws) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn to_json() {
        let bw = new_blend_weight("test");
        let j = blend_weight_to_json(&bw);
        assert!(j.contains("\"test\""));
    }

    #[test]
    fn reset() {
        let mut bw = new_blend_weight("x");
        set_blend_weight_value(&mut bw, 0.9);
        blend_weight_reset(&mut bw);
        assert!(get_blend_weight_value(&bw).abs() < 1e-6);
    }
}
