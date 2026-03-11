// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct GutCompartment {
    pub amount_mg: f32,
    pub absorption_rate: f32, /* 1/h */
    pub transit_time_h: f32,
}

pub fn new_gut_compartment(dose_mg: f32) -> GutCompartment {
    GutCompartment {
        amount_mg: dose_mg,
        absorption_rate: 1.0,
        transit_time_h: 4.0,
    }
}

pub fn gut_step(g: &mut GutCompartment, dt_hours: f32) -> f32 {
    let absorbed = g.amount_mg * g.absorption_rate * dt_hours;
    let absorbed = absorbed.min(g.amount_mg);
    g.amount_mg -= absorbed;
    g.amount_mg = g.amount_mg.max(0.0);
    absorbed
}

pub fn gut_fraction_absorbed(g: &GutCompartment) -> f32 {
    /* fraction remaining determines fraction absorbed from original */
    1.0 - g.amount_mg.max(0.0) / (g.amount_mg + 1.0)
}

pub fn gut_peak_absorption_time(g: &GutCompartment) -> f32 {
    /* Tmax ≈ 1/ka for first-order model */
    1.0 / g.absorption_rate.max(1e-9)
}

pub fn gut_bioavailability(_g: &GutCompartment, remaining: f32, initial: f32) -> f32 {
    if initial <= 0.0 {
        return 0.0;
    }
    ((initial - remaining) / initial).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_dose() {
        /* amount equals dose at creation */
        let g = new_gut_compartment(100.0);
        assert!((g.amount_mg - 100.0).abs() < 1e-5);
    }

    #[test]
    fn gut_step_absorbs_amount() {
        /* step returns positive absorbed amount */
        let mut g = new_gut_compartment(100.0);
        let abs = gut_step(&mut g, 1.0);
        assert!(abs > 0.0);
    }

    #[test]
    fn amount_decreases_after_step() {
        /* remaining amount decreases after absorption */
        let mut g = new_gut_compartment(100.0);
        gut_step(&mut g, 1.0);
        assert!(g.amount_mg < 100.0);
    }

    #[test]
    fn amount_nonnegative() {
        /* amount never goes negative */
        let mut g = new_gut_compartment(10.0);
        for _ in 0..100 {
            gut_step(&mut g, 1.0);
        }
        assert!(g.amount_mg >= 0.0);
    }

    #[test]
    fn bioavailability_between_0_and_1() {
        /* bioavailability is in [0,1] */
        let g = new_gut_compartment(100.0);
        let f = gut_bioavailability(&g, 30.0, 100.0);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn peak_time_positive() {
        /* peak absorption time is positive */
        let g = new_gut_compartment(100.0);
        assert!(gut_peak_absorption_time(&g) > 0.0);
    }

    #[test]
    fn fraction_absorbed_bounded() {
        /* fraction absorbed in [0,1] */
        let g = new_gut_compartment(50.0);
        let f = gut_fraction_absorbed(&g);
        assert!((0.0..=1.0).contains(&f));
    }
}
