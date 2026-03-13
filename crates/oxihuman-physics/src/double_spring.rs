// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Coupled oscillator: two masses connected by two springs.
//!
//! Layout: wall ---k1--- mass1 ---k2--- mass2
//! mass1 and mass2 are connected in series.

#![allow(dead_code)]

/// State of a coupled two-mass oscillator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DoubleSpring {
    /// Mass 1 (connected to wall by spring k1).
    pub m1: f32,
    /// Mass 2 (connected to mass1 by spring k2).
    pub m2: f32,
    /// Spring constant k1 (wall to mass1).
    pub k1: f32,
    /// Spring constant k2 (mass1 to mass2).
    pub k2: f32,
    /// Damping coefficient c1.
    pub c1: f32,
    /// Damping coefficient c2.
    pub c2: f32,
    /// Position of mass1 from equilibrium.
    pub x1: f32,
    /// Position of mass2 from equilibrium.
    pub x2: f32,
    /// Velocity of mass1.
    pub v1: f32,
    /// Velocity of mass2.
    pub v2: f32,
}

/// Create a new double spring oscillator.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn new_double_spring(m1: f32, m2: f32, k1: f32, k2: f32, c1: f32, c2: f32) -> DoubleSpring {
    DoubleSpring {
        m1: m1.max(1e-6),
        m2: m2.max(1e-6),
        k1,
        k2,
        c1,
        c2,
        x1: 0.0,
        x2: 0.0,
        v1: 0.0,
        v2: 0.0,
    }
}

/// Step the double spring by `dt` using semi-implicit Euler integration.
#[allow(dead_code)]
pub fn double_spring_step(ds: &mut DoubleSpring, f_ext1: f32, f_ext2: f32, dt: f32) {
    let spring1 = -ds.k1 * ds.x1 + ds.k2 * (ds.x2 - ds.x1);
    let spring2 = -ds.k2 * (ds.x2 - ds.x1);
    let damp1 = -ds.c1 * ds.v1 + ds.c2 * (ds.v2 - ds.v1);
    let damp2 = -ds.c2 * (ds.v2 - ds.v1);

    let a1 = (spring1 + damp1 + f_ext1) / ds.m1;
    let a2 = (spring2 + damp2 + f_ext2) / ds.m2;

    ds.v1 += a1 * dt;
    ds.v2 += a2 * dt;
    ds.x1 += ds.v1 * dt;
    ds.x2 += ds.v2 * dt;
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn double_spring_kinetic_energy(ds: &DoubleSpring) -> f32 {
    0.5 * ds.m1 * ds.v1 * ds.v1 + 0.5 * ds.m2 * ds.v2 * ds.v2
}

/// Total potential energy stored in springs.
#[allow(dead_code)]
pub fn double_spring_potential_energy(ds: &DoubleSpring) -> f32 {
    0.5 * ds.k1 * ds.x1 * ds.x1 + 0.5 * ds.k2 * (ds.x2 - ds.x1) * (ds.x2 - ds.x1)
}

/// Total mechanical energy.
#[allow(dead_code)]
pub fn double_spring_total_energy(ds: &DoubleSpring) -> f32 {
    double_spring_kinetic_energy(ds) + double_spring_potential_energy(ds)
}

/// Set initial displacement and velocity of both masses.
#[allow(dead_code)]
pub fn double_spring_set_state(ds: &mut DoubleSpring, x1: f32, x2: f32, v1: f32, v2: f32) {
    ds.x1 = x1;
    ds.x2 = x2;
    ds.v1 = v1;
    ds.v2 = v2;
}

/// Reset to equilibrium.
#[allow(dead_code)]
pub fn double_spring_reset(ds: &mut DoubleSpring) {
    ds.x1 = 0.0;
    ds.x2 = 0.0;
    ds.v1 = 0.0;
    ds.v2 = 0.0;
}

/// Natural frequency of mass1 ignoring coupling (sqrt(k1/m1)).
#[allow(dead_code)]
pub fn double_spring_omega1(ds: &DoubleSpring) -> f32 {
    (ds.k1 / ds.m1).sqrt()
}

/// Natural frequency of mass2 ignoring wall (sqrt(k2/m2)).
#[allow(dead_code)]
pub fn double_spring_omega2(ds: &DoubleSpring) -> f32 {
    (ds.k2 / ds.m2).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ds() -> DoubleSpring {
        new_double_spring(1.0, 1.0, 10.0, 10.0, 0.1, 0.1)
    }

    #[test]
    fn test_initial_state() {
        let ds = make_ds();
        assert_eq!(ds.x1, 0.0);
        assert_eq!(ds.x2, 0.0);
    }

    #[test]
    fn test_step_from_displaced() {
        let mut ds = make_ds();
        double_spring_set_state(&mut ds, 1.0, 0.0, 0.0, 0.0);
        double_spring_step(&mut ds, 0.0, 0.0, 0.01);
        assert!(ds.v1 != 0.0 || ds.x1 != 1.0);
    }

    #[test]
    fn test_potential_energy_at_displacement() {
        let mut ds = make_ds();
        double_spring_set_state(&mut ds, 1.0, 0.0, 0.0, 0.0);
        let pe = double_spring_potential_energy(&ds);
        assert!(pe > 0.0);
    }

    #[test]
    fn test_kinetic_energy_at_rest() {
        let ds = make_ds();
        assert_eq!(double_spring_kinetic_energy(&ds), 0.0);
    }

    #[test]
    fn test_total_energy_decreases_with_damping() {
        let mut ds = make_ds();
        double_spring_set_state(&mut ds, 1.0, 0.0, 0.0, 0.0);
        let e0 = double_spring_total_energy(&ds);
        for _ in 0..100 {
            double_spring_step(&mut ds, 0.0, 0.0, 0.01);
        }
        let e1 = double_spring_total_energy(&ds);
        assert!(e1 < e0);
    }

    #[test]
    fn test_omega1() {
        let ds = make_ds();
        let w = double_spring_omega1(&ds);
        assert!((w - 10.0_f32.sqrt()).abs() < 1e-4);
    }

    #[test]
    fn test_omega2() {
        let ds = make_ds();
        let w = double_spring_omega2(&ds);
        assert!((w - 10.0_f32.sqrt()).abs() < 1e-4);
    }

    #[test]
    fn test_reset() {
        let mut ds = make_ds();
        double_spring_set_state(&mut ds, 1.0, 2.0, 3.0, 4.0);
        double_spring_reset(&mut ds);
        assert_eq!(ds.x1, 0.0);
        assert_eq!(ds.v2, 0.0);
    }

    #[test]
    fn test_external_force() {
        let mut ds = make_ds();
        double_spring_step(&mut ds, 10.0, 0.0, 0.1);
        assert!(ds.v1 > 0.0);
    }

    #[test]
    fn test_both_masses_move() {
        let mut ds = make_ds();
        double_spring_set_state(&mut ds, 0.0, 0.0, 1.0, 0.0);
        double_spring_step(&mut ds, 0.0, 0.0, 0.1);
        assert!(ds.x1 != 0.0 || ds.x2 != 0.0);
    }
}
