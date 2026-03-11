// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wound closure dynamics model (exponential closure with treatment factors).

pub struct Wound {
    pub area_mm2: f32,
    pub healing_rate: f32,
    pub inflammation_level: f32,
}

pub fn new_wound(area_mm2: f32) -> Wound {
    Wound {
        area_mm2: area_mm2.max(0.0),
        healing_rate: 0.05,
        inflammation_level: 1.0,
    }
}

pub fn wound_step(w: &mut Wound, dt_hours: f32) {
    let rate = w.healing_rate / (1.0 + w.inflammation_level);
    w.area_mm2 = (w.area_mm2 * (-rate * dt_hours).exp()).max(0.0);
    w.inflammation_level = (w.inflammation_level - 0.01 * dt_hours).max(0.0);
}

pub fn wound_percent_closed(w: &Wound) -> f32 {
    0.0_f32.max(1.0 - w.area_mm2 / 100.0_f32.max(w.area_mm2)) * 100.0
}

pub fn wound_is_healed(w: &Wound) -> bool {
    w.area_mm2 < 1.0
}

pub fn wound_apply_treatment(w: &mut Wound, factor: f32) {
    w.healing_rate *= factor.max(0.0);
    w.inflammation_level = (w.inflammation_level * (1.0 - 0.2 * factor.clamp(0.0, 1.0))).max(0.0);
}

pub fn wound_healing_time_hours(w: &Wound) -> f32 {
    if w.area_mm2 < 1.0 {
        return 0.0;
    }
    let rate = w.healing_rate / (1.0 + w.inflammation_level).max(1e-6);
    if rate < 1e-9 {
        return f32::INFINITY;
    }
    (w.area_mm2.ln() / rate).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wound() {
        /* new wound has correct initial area */
        let w = new_wound(50.0);
        assert!((w.area_mm2 - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_wound_step_reduces_area() {
        /* wound area decreases over time */
        let mut w = new_wound(50.0);
        wound_step(&mut w, 24.0);
        assert!(w.area_mm2 < 50.0);
    }

    #[test]
    fn test_wound_percent_closed_initial() {
        /* initial wound is 0% closed */
        let w = new_wound(100.0);
        assert!(wound_percent_closed(&w) < 1.0);
    }

    #[test]
    fn test_wound_is_healed() {
        /* wound with area < 1 is healed */
        let w = new_wound(0.5);
        assert!(wound_is_healed(&w));
    }

    #[test]
    fn test_wound_not_healed() {
        /* wound with area >= 1 is not healed */
        let w = new_wound(10.0);
        assert!(!wound_is_healed(&w));
    }

    #[test]
    fn test_wound_apply_treatment() {
        /* treatment increases healing rate */
        let mut w = new_wound(50.0);
        let rate_before = w.healing_rate;
        wound_apply_treatment(&mut w, 2.0);
        assert!(w.healing_rate > rate_before);
    }

    #[test]
    fn test_wound_healing_time_positive() {
        /* healing time is positive for non-healed wounds */
        let w = new_wound(50.0);
        let t = wound_healing_time_hours(&w);
        assert!(t > 0.0);
    }
}
