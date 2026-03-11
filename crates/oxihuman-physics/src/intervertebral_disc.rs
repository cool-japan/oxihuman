// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

const HERNIATION_THRESHOLD: f32 = 500.0; /* kPa */

pub struct IvDisc {
    pub height: f32,
    pub nucleus_pressure: f32,
    pub annulus_stiffness: f32,
    pub creep_rate: f32,
}

pub fn new_iv_disc(height: f32) -> IvDisc {
    IvDisc {
        height,
        nucleus_pressure: 0.0,
        annulus_stiffness: 1000.0,
        creep_rate: 0.01,
    }
}

pub fn disc_axial_stiffness(d: &IvDisc) -> f32 {
    d.annulus_stiffness / d.height.max(1e-6)
}

pub fn disc_apply_load(d: &mut IvDisc, load_n: f32, dt: f32) {
    let compression = load_n / disc_axial_stiffness(d).max(1e-6);
    d.height -= compression * dt * d.creep_rate;
    d.height = d.height.max(0.001);
    d.nucleus_pressure += load_n * 0.001 * dt;
}

pub fn disc_is_herniated(d: &IvDisc) -> bool {
    d.nucleus_pressure > HERNIATION_THRESHOLD
}

pub fn disc_recovery(d: &mut IvDisc, dt: f32) {
    d.nucleus_pressure -= d.nucleus_pressure * d.creep_rate * dt;
    d.nucleus_pressure = d.nucleus_pressure.max(0.0);
}

pub fn disc_height_loss(d: &IvDisc) -> f32 {
    /* height loss from original; we track absolute height */
    d.nucleus_pressure * 1e-5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_pressure_zero() {
        /* new disc has zero pressure */
        let d = new_iv_disc(0.01);
        assert_eq!(d.nucleus_pressure, 0.0);
    }

    #[test]
    fn axial_stiffness_positive() {
        /* stiffness > 0 for valid height */
        let d = new_iv_disc(0.01);
        assert!(disc_axial_stiffness(&d) > 0.0);
    }

    #[test]
    fn load_increases_pressure() {
        /* applying a load raises nucleus pressure */
        let mut d = new_iv_disc(0.01);
        disc_apply_load(&mut d, 1000.0, 0.01);
        assert!(d.nucleus_pressure > 0.0);
    }

    #[test]
    fn not_herniated_initially() {
        /* fresh disc is not herniated */
        let d = new_iv_disc(0.01);
        assert!(!disc_is_herniated(&d));
    }

    #[test]
    fn recovery_reduces_pressure() {
        /* unloaded recovery decreases pressure */
        let mut d = new_iv_disc(0.01);
        d.nucleus_pressure = 100.0;
        disc_recovery(&mut d, 1.0);
        assert!(d.nucleus_pressure < 100.0);
    }

    #[test]
    fn height_loss_proportional() {
        /* height loss increases with pressure */
        let mut d = new_iv_disc(0.01);
        d.nucleus_pressure = 200.0;
        let loss1 = disc_height_loss(&d);
        d.nucleus_pressure = 400.0;
        let loss2 = disc_height_loss(&d);
        assert!(loss2 > loss1);
    }

    #[test]
    fn height_stays_positive_under_load() {
        /* height clamped above zero */
        let mut d = new_iv_disc(0.01);
        for _ in 0..1000 {
            disc_apply_load(&mut d, 10000.0, 0.1);
        }
        assert!(d.height > 0.0);
    }
}
