// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Multiplicative morph blending.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MultiplicativeMorph {
    pub value: f32,
    pub multiplier: f32,
}

#[allow(dead_code)]
pub fn new_multiplicative_morph(multiplier: f32) -> MultiplicativeMorph {
    MultiplicativeMorph { value: 0.0, multiplier }
}

#[allow(dead_code)]
pub fn mm_set_value(m: &mut MultiplicativeMorph, v: f32) {
    m.value = v;
}

#[allow(dead_code)]
pub fn mm_set_multiplier(m: &mut MultiplicativeMorph, mult: f32) {
    m.multiplier = mult;
}

#[allow(dead_code)]
pub fn mm_result(m: &MultiplicativeMorph) -> f32 {
    m.value * m.multiplier
}

#[allow(dead_code)]
pub fn mm_invert_multiplier(m: &mut MultiplicativeMorph) {
    if m.multiplier != 0.0 {
        m.multiplier = 1.0 / m.multiplier;
    }
}

#[allow(dead_code)]
pub fn mm_reset(m: &mut MultiplicativeMorph) {
    m.value = 0.0;
    m.multiplier = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_value_times_multiplier() {
        let mut m = new_multiplicative_morph(2.0);
        mm_set_value(&mut m, 3.0);
        assert!((mm_result(&m) - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_invert_multiplier() {
        let mut m = new_multiplicative_morph(4.0);
        mm_invert_multiplier(&mut m);
        assert!((m.multiplier - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_invert_zero_multiplier_noop() {
        let mut m = new_multiplicative_morph(0.0);
        mm_invert_multiplier(&mut m);
        assert_eq!(m.multiplier, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut m = new_multiplicative_morph(5.0);
        mm_set_value(&mut m, 3.0);
        mm_reset(&mut m);
        assert_eq!(m.value, 0.0);
        assert!((m.multiplier - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_multiplier_result_zero() {
        let mut m = new_multiplicative_morph(0.0);
        mm_set_value(&mut m, 100.0);
        assert_eq!(mm_result(&m), 0.0);
    }

    #[test]
    fn test_set_multiplier() {
        let mut m = new_multiplicative_morph(1.0);
        mm_set_multiplier(&mut m, 3.0);
        assert!((m.multiplier - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_negative_value() {
        let mut m = new_multiplicative_morph(2.0);
        mm_set_value(&mut m, -1.5);
        assert!((mm_result(&m) - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn test_initial_result_zero() {
        let m = new_multiplicative_morph(5.0);
        assert_eq!(mm_result(&m), 0.0);
    }
}
