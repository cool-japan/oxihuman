// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct GastricAcid {
    pub ph: f32,
    pub volume_ml: f32,
    pub secretion_rate_mmol_per_h: f32,
    pub buffer_capacity: f32,
}

pub fn new_gastric_acid() -> GastricAcid {
    GastricAcid {
        ph: 1.5,
        volume_ml: 50.0,
        secretion_rate_mmol_per_h: 10.0,
        buffer_capacity: 5.0,
    }
}

pub fn gastric_step(g: &mut GastricAcid, dt_hours: f32) {
    let delta_acid = g.secretion_rate_mmol_per_h * dt_hours;
    g.ph -= delta_acid / (g.buffer_capacity * g.volume_ml);
    g.ph = g.ph.clamp(0.1, 14.0);
}

pub fn gastric_is_acidic(g: &GastricAcid) -> bool {
    g.ph < 3.0
}

pub fn gastric_apply_antacid(g: &mut GastricAcid, buffer_mmol: f32) {
    g.ph += buffer_mmol / (g.buffer_capacity * g.volume_ml);
    g.ph = g.ph.min(14.0);
}

pub fn gastric_secretion_inhibit(g: &mut GastricAcid, factor: f32) {
    g.secretion_rate_mmol_per_h *= 1.0 - factor.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_ph() {
        /* initial pH is 1.5 */
        let g = new_gastric_acid();
        assert!((g.ph - 1.5).abs() < 1e-5);
    }

    #[test]
    fn acidic_initially() {
        /* initial pH < 3 → acidic */
        let g = new_gastric_acid();
        assert!(gastric_is_acidic(&g));
    }

    #[test]
    fn step_lowers_ph() {
        /* acid secretion lowers pH */
        let mut g = new_gastric_acid();
        g.ph = 3.5; /* start above threshold */
        gastric_step(&mut g, 0.1);
        /* pH decreases */
        assert!(g.ph < 3.5);
    }

    #[test]
    fn antacid_raises_ph() {
        /* antacid raises pH */
        let mut g = new_gastric_acid();
        let ph0 = g.ph;
        gastric_apply_antacid(&mut g, 50.0);
        assert!(g.ph > ph0);
    }

    #[test]
    fn inhibition_reduces_rate() {
        /* inhibition reduces secretion rate */
        let mut g = new_gastric_acid();
        let rate0 = g.secretion_rate_mmol_per_h;
        gastric_secretion_inhibit(&mut g, 0.5);
        assert!(g.secretion_rate_mmol_per_h < rate0);
    }

    #[test]
    fn ph_clamped_above_zero() {
        /* pH cannot go below 0.1 */
        let mut g = new_gastric_acid();
        g.ph = 0.5;
        gastric_step(&mut g, 10.0);
        assert!(g.ph >= 0.1);
    }

    #[test]
    fn ph_clamped_below_14() {
        /* antacid cannot push pH above 14 */
        let mut g = new_gastric_acid();
        gastric_apply_antacid(&mut g, 1e9);
        assert!(g.ph <= 14.0);
    }
}
