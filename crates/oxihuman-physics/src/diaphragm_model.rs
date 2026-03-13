// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Diaphragm {
    pub dome_height_cm: f32,
    pub activation: f32,
    pub max_force_n: f32,
    pub fatigue: f32,
}

pub fn new_diaphragm() -> Diaphragm {
    Diaphragm {
        dome_height_cm: 4.0,
        activation: 0.0,
        max_force_n: 250.0,
        fatigue: 0.0,
    }
}

pub fn diaphragm_force(d: &Diaphragm) -> f32 {
    d.activation * d.max_force_n * (1.0 - d.fatigue).max(0.0)
}

pub fn diaphragm_pressure_contribution(d: &Diaphragm, area_cm2: f32) -> f32 {
    if area_cm2 <= 0.0 {
        return 0.0;
    }
    diaphragm_force(d) / area_cm2
}

pub fn diaphragm_activate(d: &mut Diaphragm, level: f32) {
    d.activation = level.clamp(0.0, 1.0);
}

pub fn diaphragm_fatigue_step(d: &mut Diaphragm, dt: f32) {
    d.fatigue += d.activation * 0.01 * dt;
    d.fatigue = d.fatigue.clamp(0.0, 1.0);
}

pub fn diaphragm_is_paralyzed(d: &Diaphragm) -> bool {
    d.max_force_n < 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_force_zero() {
        /* no activation → no force */
        let d = new_diaphragm();
        assert_eq!(diaphragm_force(&d), 0.0);
    }

    #[test]
    fn force_positive_with_activation() {
        /* activation > 0 produces force */
        let mut d = new_diaphragm();
        diaphragm_activate(&mut d, 0.5);
        assert!(diaphragm_force(&d) > 0.0);
    }

    #[test]
    fn full_activation_max_force() {
        /* activation=1, fatigue=0 → max_force */
        let mut d = new_diaphragm();
        diaphragm_activate(&mut d, 1.0);
        assert!((diaphragm_force(&d) - d.max_force_n).abs() < 1e-4);
    }

    #[test]
    fn fatigue_accumulates() {
        /* fatigue increases with use */
        let mut d = new_diaphragm();
        diaphragm_activate(&mut d, 1.0);
        diaphragm_fatigue_step(&mut d, 1.0);
        assert!(d.fatigue > 0.0);
    }

    #[test]
    fn pressure_positive_for_positive_area() {
        /* positive area yields positive pressure */
        let mut d = new_diaphragm();
        diaphragm_activate(&mut d, 0.5);
        assert!(diaphragm_pressure_contribution(&d, 100.0) > 0.0);
    }

    #[test]
    fn not_paralyzed_normally() {
        /* healthy diaphragm is not paralyzed */
        let d = new_diaphragm();
        assert!(!diaphragm_is_paralyzed(&d));
    }

    #[test]
    fn paralyzed_when_max_force_small() {
        /* paralyzed when max_force < 1 */
        let mut d = new_diaphragm();
        d.max_force_n = 0.5;
        assert!(diaphragm_is_paralyzed(&d));
    }
}
