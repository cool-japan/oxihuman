// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fascia connective tissue model (viscoelastic sheet).

/// A fascial tissue element.
#[derive(Debug, Clone)]
pub struct FasciaElement {
    /// Rest length (m).
    pub rest_len: f32,
    /// Current length (m).
    pub current_len: f32,
    /// Elastic stiffness (N/m).
    pub stiffness: f32,
    /// Viscous damping coefficient.
    pub viscosity: f32,
    /// Rate of length change (m/s).
    pub velocity: f32,
    /// Hydration level [0, 1] (affects stiffness).
    pub hydration: f32,
}

impl FasciaElement {
    pub fn new(rest_len: f32, stiffness: f32) -> Self {
        FasciaElement {
            rest_len,
            current_len: rest_len,
            stiffness,
            viscosity: 50.0,
            velocity: 0.0,
            hydration: 1.0,
        }
    }
}

/// Create a new fascia element.
pub fn new_fascia(rest_len: f32, stiffness: f32) -> FasciaElement {
    FasciaElement::new(rest_len, stiffness)
}

/// Compute the current elastic force.
pub fn fascia_elastic_force(f: &FasciaElement) -> f32 {
    let stretch = f.current_len - f.rest_len;
    f.stiffness * f.hydration * stretch.max(0.0)
}

/// Compute the viscous force.
pub fn fascia_viscous_force(f: &FasciaElement) -> f32 {
    f.viscosity * f.velocity
}

/// Total force (elastic + viscous).
pub fn fascia_total_force(f: &FasciaElement) -> f32 {
    fascia_elastic_force(f) + fascia_viscous_force(f)
}

/// Set the current length and velocity.
pub fn fascia_set_length(f: &mut FasciaElement, len: f32, vel: f32) {
    f.current_len = len.max(0.0);
    f.velocity = vel;
}

/// Compute strain = (current - rest) / rest.
pub fn fascia_strain(f: &FasciaElement) -> f32 {
    (f.current_len - f.rest_len) / f.rest_len.max(1e-10)
}

/// Return `true` if the fascia is under tension.
pub fn fascia_is_taut(f: &FasciaElement) -> bool {
    f.current_len > f.rest_len + 1e-6
}

/// Set hydration level [0, 1].
pub fn fascia_set_hydration(f: &mut FasciaElement, h: f32) {
    f.hydration = h.clamp(0.0, 1.0);
}

/// Compute stored elastic energy.
pub fn fascia_stored_energy(f: &FasciaElement) -> f32 {
    let stretch = (f.current_len - f.rest_len).max(0.0);
    0.5 * f.stiffness * f.hydration * stretch * stretch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fascia_at_rest() {
        let f = new_fascia(0.05, 500.0);
        assert!((fascia_elastic_force(&f)).abs() < 1e-5);
    }

    #[test]
    fn test_elastic_force_when_stretched() {
        let mut f = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f, 0.07, 0.0);
        assert!(fascia_elastic_force(&f) > 0.0);
    }

    #[test]
    fn test_no_compressive_force() {
        /* fascia does not push when compressed */
        let mut f = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f, 0.03, 0.0);
        assert!(fascia_elastic_force(&f).abs() < 1e-5);
    }

    #[test]
    fn test_viscous_force_nonzero() {
        let mut f = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f, 0.05, 0.1);
        assert!(fascia_viscous_force(&f).abs() > 0.0);
    }

    #[test]
    fn test_is_taut() {
        let mut f = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f, 0.07, 0.0);
        assert!(fascia_is_taut(&f));
    }

    #[test]
    fn test_hydration_reduces_stiffness() {
        let mut f1 = new_fascia(0.05, 500.0);
        let mut f2 = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f1, 0.07, 0.0);
        fascia_set_length(&mut f2, 0.07, 0.0);
        fascia_set_hydration(&mut f2, 0.5);
        assert!(fascia_elastic_force(&f2) < fascia_elastic_force(&f1));
    }

    #[test]
    fn test_strain_positive_when_stretched() {
        let mut f = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f, 0.06, 0.0);
        assert!(fascia_strain(&f) > 0.0);
    }

    #[test]
    fn test_stored_energy_positive_when_taut() {
        let mut f = new_fascia(0.05, 500.0);
        fascia_set_length(&mut f, 0.08, 0.0);
        assert!(fascia_stored_energy(&f) > 0.0);
    }

    #[test]
    fn test_stored_energy_zero_at_rest() {
        let f = new_fascia(0.05, 500.0);
        assert!(fascia_stored_energy(&f) < 1e-5);
    }
}
