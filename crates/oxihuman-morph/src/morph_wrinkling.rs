// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Wrinkle morph driven by surface strain.

#[allow(dead_code)]
pub struct WrinkleDriver {
    pub strain: f32,
    pub threshold: f32,
    pub intensity: f32,
}

#[allow(dead_code)]
pub fn new_wrinkle_driver(threshold: f32) -> WrinkleDriver {
    WrinkleDriver { strain: 0.0, threshold, intensity: 1.0 }
}

#[allow(dead_code)]
pub fn wd_set_strain(d: &mut WrinkleDriver, strain: f32) {
    d.strain = strain;
}

#[allow(dead_code)]
pub fn wd_weight(d: &WrinkleDriver) -> f32 {
    if d.threshold < 1e-7 {
        return 0.0;
    }
    let excess = d.strain - d.threshold;
    (excess.max(0.0) / d.threshold).min(1.0)
}

#[allow(dead_code)]
pub fn wd_intensity(d: &WrinkleDriver) -> f32 {
    d.intensity
}

#[allow(dead_code)]
pub fn wd_set_intensity(d: &mut WrinkleDriver, v: f32) {
    d.intensity = v;
}

#[allow(dead_code)]
pub fn wd_is_active(d: &WrinkleDriver) -> bool {
    d.strain > d.threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_below_threshold_weight_zero() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_strain(&mut d, 0.2);
        assert_eq!(wd_weight(&d), 0.0);
    }

    #[test]
    fn test_above_threshold_weight_positive() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_strain(&mut d, 0.8);
        assert!(wd_weight(&d) > 0.0);
    }

    #[test]
    fn test_weight_clamped_to_one() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_strain(&mut d, 10.0);
        assert!((wd_weight(&d) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_intensity_getter() {
        let d = new_wrinkle_driver(0.5);
        assert!((wd_intensity(&d) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_intensity_setter() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_intensity(&mut d, 2.0);
        assert!((wd_intensity(&d) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_active_false() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_strain(&mut d, 0.3);
        assert!(!wd_is_active(&d));
    }

    #[test]
    fn test_is_active_true() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_strain(&mut d, 0.9);
        assert!(wd_is_active(&d));
    }

    #[test]
    fn test_weight_at_double_threshold() {
        let mut d = new_wrinkle_driver(0.5);
        wd_set_strain(&mut d, 1.0);
        let w = wd_weight(&d);
        assert!((w - 1.0).abs() < 1e-5);
    }
}
