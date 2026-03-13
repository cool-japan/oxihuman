// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Additive morph layer (adds delta to base value).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdditiveMorphV2 {
    pub base: f32,
    pub delta: f32,
    pub scale: f32,
}

#[allow(dead_code)]
pub fn new_additive_morph_v2(scale: f32) -> AdditiveMorphV2 {
    AdditiveMorphV2 { base: 0.0, delta: 0.0, scale }
}

#[allow(dead_code)]
pub fn amv2_set_base(m: &mut AdditiveMorphV2, base: f32) {
    m.base = base;
}

#[allow(dead_code)]
pub fn amv2_set_delta(m: &mut AdditiveMorphV2, delta: f32) {
    m.delta = delta;
}

#[allow(dead_code)]
pub fn amv2_value(m: &AdditiveMorphV2) -> f32 {
    m.base + m.delta * m.scale
}

#[allow(dead_code)]
pub fn amv2_scale(m: &AdditiveMorphV2) -> f32 {
    m.scale
}

#[allow(dead_code)]
pub fn amv2_reset(m: &mut AdditiveMorphV2) {
    m.base = 0.0;
    m.delta = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_plus_delta_times_scale() {
        let mut m = new_additive_morph_v2(2.0);
        amv2_set_base(&mut m, 0.5);
        amv2_set_delta(&mut m, 0.25);
        assert!((amv2_value(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_getter() {
        let m = new_additive_morph_v2(3.0);
        assert!((amv2_scale(&m) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears() {
        let mut m = new_additive_morph_v2(1.0);
        amv2_set_base(&mut m, 1.0);
        amv2_set_delta(&mut m, 0.5);
        amv2_reset(&mut m);
        assert_eq!(amv2_value(&m), 0.0);
    }

    #[test]
    fn test_zero_delta_equals_base() {
        let mut m = new_additive_morph_v2(5.0);
        amv2_set_base(&mut m, 0.7);
        assert!((amv2_value(&m) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_negative_delta() {
        let mut m = new_additive_morph_v2(1.0);
        amv2_set_base(&mut m, 0.5);
        amv2_set_delta(&mut m, -0.2);
        assert!((amv2_value(&m) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_initial_value_zero() {
        let m = new_additive_morph_v2(1.0);
        assert_eq!(amv2_value(&m), 0.0);
    }

    #[test]
    fn test_scale_zero_returns_base() {
        let mut m = new_additive_morph_v2(0.0);
        amv2_set_base(&mut m, 0.8);
        amv2_set_delta(&mut m, 100.0);
        assert!((amv2_value(&m) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_delta_updates() {
        let mut m = new_additive_morph_v2(1.0);
        amv2_set_delta(&mut m, 0.4);
        assert!((m.delta - 0.4).abs() < 1e-6);
    }
}
