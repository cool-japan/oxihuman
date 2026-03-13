// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Logistic tumor growth model.

pub struct Tumor {
    pub volume_mm3: f32,
    pub carrying_capacity: f32,
    pub growth_rate: f32,
    pub necrotic_fraction: f32,
}

pub fn new_tumor(initial_volume: f32) -> Tumor {
    Tumor {
        volume_mm3: initial_volume.max(0.001),
        carrying_capacity: 10_000.0,
        growth_rate: 0.1,
        necrotic_fraction: 0.0,
    }
}

pub fn tumor_step(t: &mut Tumor, dt_days: f32) {
    /* logistic growth: dV/dt = r * V * (1 - V/K) */
    let dv = t.growth_rate * t.volume_mm3 * (1.0 - t.volume_mm3 / t.carrying_capacity);
    t.volume_mm3 = (t.volume_mm3 + dv * dt_days).max(0.0);
    /* necrotic fraction grows as tumor grows large */
    t.necrotic_fraction = (t.volume_mm3 / t.carrying_capacity * 0.3).min(1.0);
}

pub fn tumor_doubling_time_days(t: &Tumor) -> f32 {
    if t.growth_rate < 1e-9 {
        return f32::INFINITY;
    }
    2.0_f32.ln() / t.growth_rate
}

pub fn tumor_is_detectable(t: &Tumor) -> bool {
    t.volume_mm3 > 64.0
}

pub fn tumor_apply_treatment(t: &mut Tumor, kill_fraction: f32) {
    let kf = kill_fraction.clamp(0.0, 1.0);
    t.volume_mm3 = (t.volume_mm3 * (1.0 - kf)).max(0.0);
    t.necrotic_fraction = (t.necrotic_fraction + kf * 0.5).min(1.0);
}

pub fn tumor_viable_volume(t: &Tumor) -> f32 {
    t.volume_mm3 * (1.0 - t.necrotic_fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tumor() {
        /* new tumor has correct initial volume */
        let t = new_tumor(10.0);
        assert!((t.volume_mm3 - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_tumor_step_grows() {
        /* tumor volume increases with each step */
        let mut t = new_tumor(10.0);
        tumor_step(&mut t, 1.0);
        assert!(t.volume_mm3 > 10.0);
    }

    #[test]
    fn test_tumor_doubling_time() {
        /* doubling time is positive */
        let t = new_tumor(10.0);
        let dt = tumor_doubling_time_days(&t);
        assert!(dt > 0.0);
    }

    #[test]
    fn test_tumor_is_detectable_small() {
        /* small tumor is not detectable */
        let t = new_tumor(10.0);
        assert!(!tumor_is_detectable(&t));
    }

    #[test]
    fn test_tumor_is_detectable_large() {
        /* large tumor is detectable */
        let t = new_tumor(100.0);
        assert!(tumor_is_detectable(&t));
    }

    #[test]
    fn test_tumor_apply_treatment_reduces() {
        /* treatment reduces tumor volume */
        let mut t = new_tumor(1000.0);
        let vol_before = t.volume_mm3;
        tumor_apply_treatment(&mut t, 0.5);
        assert!(t.volume_mm3 < vol_before);
    }

    #[test]
    fn test_tumor_viable_volume() {
        /* viable volume <= total volume */
        let t = new_tumor(1000.0);
        assert!(tumor_viable_volume(&t) <= t.volume_mm3);
    }
}
