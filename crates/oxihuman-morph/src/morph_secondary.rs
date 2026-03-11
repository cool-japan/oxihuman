// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Secondary motion driven by primary morph.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SecondaryMorph {
    pub primary_weight: f32,
    pub lag: f32,
    pub current: f32,
    pub gain: f32,
}

#[allow(dead_code)]
pub fn new_secondary_morph(lag: f32, gain: f32) -> SecondaryMorph {
    SecondaryMorph { primary_weight: 0.0, lag, current: 0.0, gain }
}

#[allow(dead_code)]
pub fn sec_update(m: &mut SecondaryMorph, primary: f32, dt: f32) {
    m.primary_weight = primary;
    let target = primary * m.gain;
    let rate = (1.0 - m.lag) * dt;
    m.current += (target - m.current) * rate;
}

#[allow(dead_code)]
pub fn sec_value(m: &SecondaryMorph) -> f32 {
    m.current
}

#[allow(dead_code)]
pub fn sec_lag(m: &SecondaryMorph) -> f32 {
    m.lag
}

#[allow(dead_code)]
pub fn sec_gain(m: &SecondaryMorph) -> f32 {
    m.gain
}

#[allow(dead_code)]
pub fn sec_reset(m: &mut SecondaryMorph) {
    m.current = 0.0;
    m.primary_weight = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follows_primary() {
        let mut m = new_secondary_morph(0.5, 1.0);
        sec_update(&mut m, 1.0, 1.0);
        assert!(sec_value(&m) > 0.0);
    }

    #[test]
    fn test_lag_getter() {
        let m = new_secondary_morph(0.3, 1.0);
        assert!((sec_lag(&m) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_gain_getter() {
        let m = new_secondary_morph(0.5, 2.0);
        assert!((sec_gain(&m) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears() {
        let mut m = new_secondary_morph(0.5, 1.0);
        sec_update(&mut m, 1.0, 1.0);
        sec_reset(&mut m);
        assert_eq!(sec_value(&m), 0.0);
    }

    #[test]
    fn test_no_crash_zero_lag() {
        let mut m = new_secondary_morph(0.0, 1.0);
        sec_update(&mut m, 0.5, 0.016);
        assert!(sec_value(&m).is_finite());
    }

    #[test]
    fn test_gain_scales_output() {
        let mut m = new_secondary_morph(0.0, 2.0);
        sec_update(&mut m, 1.0, 1.0);
        assert!(sec_value(&m) > 1.0);
    }

    #[test]
    fn test_primary_stored() {
        let mut m = new_secondary_morph(0.5, 1.0);
        sec_update(&mut m, 0.7, 0.1);
        assert!((m.primary_weight - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_initial_value_zero() {
        let m = new_secondary_morph(0.5, 1.0);
        assert_eq!(sec_value(&m), 0.0);
    }
}
