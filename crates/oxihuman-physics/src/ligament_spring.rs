// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Ligament with nonlinear spring (toe + linear region).
pub struct Ligament {
    pub toe_strain: f32,
    pub linear_modulus: f32,
    pub cross_section: f32,
    pub rest_length: f32,
    pub current_length: f32,
}

impl Ligament {
    pub fn new() -> Self {
        Ligament {
            toe_strain: 0.03,
            linear_modulus: 800.0, // MPa-ish
            cross_section: 1e-4,   // m²
            rest_length: 0.05,
            current_length: 0.05,
        }
    }
}

impl Default for Ligament {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_ligament() -> Ligament {
    Ligament::new()
}

pub fn ligament_strain(l: &Ligament) -> f32 {
    (l.current_length - l.rest_length) / l.rest_length
}

/// Toe region: quadratic; linear region: E * A * strain.
pub fn ligament_force(l: &Ligament) -> f32 {
    let strain = ligament_strain(l);
    if strain <= 0.0 {
        return 0.0;
    }
    if strain < l.toe_strain {
        // quadratic toe region
        let k_toe = l.linear_modulus * l.cross_section / (2.0 * l.toe_strain);
        k_toe * strain * strain / l.toe_strain
    } else {
        // linear region with toe offset
        let f_toe = l.linear_modulus * l.cross_section * l.toe_strain / 2.0;
        f_toe + l.linear_modulus * l.cross_section * (strain - l.toe_strain)
    }
}

pub fn ligament_elongate(l: &mut Ligament, delta: f32) {
    l.current_length += delta;
    if l.current_length < 0.0 {
        l.current_length = 0.0;
    }
}

pub fn ligament_is_taut(l: &Ligament) -> bool {
    ligament_strain(l) > 0.0
}

/// Secant stiffness: force / (strain * rest_length).
pub fn ligament_stiffness(l: &Ligament) -> f32 {
    let strain = ligament_strain(l);
    if strain.abs() < 1e-9 {
        return 0.0;
    }
    ligament_force(l) / (strain * l.rest_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new ligament at rest */
        let l = new_ligament();
        assert!((ligament_strain(&l)).abs() < 1e-7);
    }

    #[test]
    fn test_slack() {
        /* compressed ligament is slack */
        let mut l = new_ligament();
        ligament_elongate(&mut l, -0.001);
        assert!(!ligament_is_taut(&l));
        assert!((ligament_force(&l)).abs() < 1e-9);
    }

    #[test]
    fn test_toe_region() {
        /* small elongation falls in toe region */
        let mut l = new_ligament();
        let delta = l.rest_length * 0.01;
        ligament_elongate(&mut l, delta);
        assert!(ligament_force(&l) > 0.0);
    }

    #[test]
    fn test_linear_region() {
        /* large elongation falls in linear region */
        let mut l = new_ligament();
        let delta = l.rest_length * 0.1;
        ligament_elongate(&mut l, delta);
        assert!(ligament_force(&l) > 0.0);
    }

    #[test]
    fn test_is_taut() {
        /* taut when positive strain */
        let mut l = new_ligament();
        ligament_elongate(&mut l, 0.001);
        assert!(ligament_is_taut(&l));
    }

    #[test]
    fn test_force_increases_with_elongation() {
        /* force is monotonically increasing */
        let mut l1 = new_ligament();
        let mut l2 = new_ligament();
        ligament_elongate(&mut l1, 0.002);
        ligament_elongate(&mut l2, 0.004);
        assert!(ligament_force(&l2) > ligament_force(&l1));
    }

    #[test]
    fn test_stiffness_positive_when_taut() {
        /* stiffness is positive when ligament is taut */
        let mut l = new_ligament();
        ligament_elongate(&mut l, 0.005);
        assert!(ligament_stiffness(&l) > 0.0);
    }
}
