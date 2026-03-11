// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Cardiac model (Frank-Starling).
pub struct Heart {
    pub heart_rate_bpm: f32,
    pub stroke_volume_ml: f32,
    pub preload: f32,
    pub afterload: f32,
    pub contractility: f32,
}

impl Heart {
    pub fn new() -> Self {
        Heart {
            heart_rate_bpm: 70.0,
            stroke_volume_ml: 70.0,
            preload: 1.0,
            afterload: 1.0,
            contractility: 1.0,
        }
    }
}

impl Default for Heart {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_heart() -> Heart {
    Heart::new()
}

/// CO (L/min) = HR * SV / 1000
pub fn heart_cardiac_output_l_per_min(h: &Heart) -> f32 {
    h.heart_rate_bpm * h.stroke_volume_ml / 1000.0
}

/// Frank-Starling: SV increases with preload (simplified linear).
pub fn heart_frank_starling_adjust(h: &mut Heart, preload: f32) {
    h.preload = preload;
    // SV scales from baseline 70 mL proportionally to preload
    h.stroke_volume_ml = (70.0 * preload * h.contractility).clamp(20.0, 200.0);
}

/// EF = SV / EDV, EDV ≈ 120 mL
pub fn heart_ejection_fraction(h: &Heart) -> f32 {
    h.stroke_volume_ml / 120.0
}

/// MAP = CO * TPR
pub fn heart_mean_arterial_pressure(h: &Heart, tpr: f32) -> f32 {
    heart_cardiac_output_l_per_min(h) * tpr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new heart has 70 bpm and 70 mL SV */
        let h = new_heart();
        assert!((h.heart_rate_bpm - 70.0).abs() < 1e-3);
        assert!((h.stroke_volume_ml - 70.0).abs() < 1e-3);
    }

    #[test]
    fn test_cardiac_output() {
        /* CO = 70 * 70 / 1000 = 4.9 L/min */
        let h = new_heart();
        assert!((heart_cardiac_output_l_per_min(&h) - 4.9).abs() < 0.01);
    }

    #[test]
    fn test_frank_starling_increase() {
        /* higher preload increases SV */
        let mut h = new_heart();
        let sv_before = h.stroke_volume_ml;
        heart_frank_starling_adjust(&mut h, 1.5);
        assert!(h.stroke_volume_ml > sv_before);
    }

    #[test]
    fn test_frank_starling_clamped() {
        /* SV is clamped to physiological range */
        let mut h = new_heart();
        heart_frank_starling_adjust(&mut h, 10.0);
        assert!(h.stroke_volume_ml <= 200.0);
    }

    #[test]
    fn test_ejection_fraction() {
        /* EF = SV/120 ≈ 0.583 at default */
        let h = new_heart();
        let ef = heart_ejection_fraction(&h);
        assert!((ef - 70.0 / 120.0).abs() < 1e-4);
    }

    #[test]
    fn test_mean_arterial_pressure() {
        /* MAP = CO * TPR */
        let h = new_heart();
        let map = heart_mean_arterial_pressure(&h, 20.0);
        assert!((map - 4.9 * 20.0).abs() < 0.1);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let h = Heart::default();
        assert!((h.heart_rate_bpm - 70.0).abs() < 1e-3);
    }
}
