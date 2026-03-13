// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Magnetic dipole torque and force in an external field.

/// A magnetic dipole.
#[derive(Debug, Clone)]
pub struct MagneticDipole {
    /// Magnetic moment vector m (A·m²).
    pub moment: [f64; 3],
}

impl MagneticDipole {
    pub fn new(moment: [f64; 3]) -> Self {
        MagneticDipole { moment }
    }

    /// Torque τ = m × B.
    pub fn torque(&self, b_field: [f64; 3]) -> [f64; 3] {
        cross(self.moment, b_field)
    }

    /// Potential energy U = -m · B.
    pub fn potential_energy(&self, b_field: [f64; 3]) -> f64 {
        -dot(self.moment, b_field)
    }

    /// Magnitude of the magnetic moment.
    pub fn moment_magnitude(&self) -> f64 {
        mag3(self.moment)
    }

    /// Alignment angle between m and B (radians).
    pub fn alignment_angle(&self, b_field: [f64; 3]) -> f64 {
        let mm = mag3(self.moment);
        let bm = mag3(b_field);
        if mm < 1e-30 || bm < 1e-30 { return 0.0; }
        let cos_a = (dot(self.moment, b_field) / (mm * bm)).clamp(-1.0, 1.0);
        cos_a.acos()
    }

    /// Force on a dipole in a non-uniform field (gradient approximation).
    ///
    /// F ≈ ∇(m · B) — approximated here as m_z * dBz/dz along z.
    pub fn force_in_gradient(&self, db_dz: f64) -> f64 {
        self.moment[2] * db_dz
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn mag3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Create a new magnetic dipole.
pub fn new_magnetic_dipole(moment: [f64; 3]) -> MagneticDipole {
    MagneticDipole::new(moment)
}

/// Torque.
pub fn md_torque(d: &MagneticDipole, b: [f64; 3]) -> [f64; 3] {
    d.torque(b)
}

/// Potential energy.
pub fn md_potential_energy(d: &MagneticDipole, b: [f64; 3]) -> f64 {
    d.potential_energy(b)
}

/// Moment magnitude.
pub fn md_moment_magnitude(d: &MagneticDipole) -> f64 {
    d.moment_magnitude()
}

/// Alignment angle.
pub fn md_alignment_angle(d: &MagneticDipole, b: [f64; 3]) -> f64 {
    d.alignment_angle(b)
}

/// Force in gradient.
pub fn md_force_in_gradient(d: &MagneticDipole, db_dz: f64) -> f64 {
    d.force_in_gradient(db_dz)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_torque_aligned_zero() {
        /* m and B parallel → τ = 0 */
        let d = new_magnetic_dipole([0.0, 0.0, 1.0]);
        let tau = md_torque(&d, [0.0, 0.0, 1.0]);
        assert!(tau.iter().all(|&x| x.abs() < 1e-12) /* aligned → zero torque */);
    }

    #[test]
    fn test_torque_perpendicular_nonzero() {
        let d = new_magnetic_dipole([1.0, 0.0, 0.0]);
        let tau = md_torque(&d, [0.0, 1.0, 0.0]);
        assert!(mag3(tau) > 0.0 /* perpendicular → nonzero torque */);
    }

    #[test]
    fn test_potential_energy_aligned_negative() {
        /* aligned with field: minimum energy (negative) */
        let d = new_magnetic_dipole([0.0, 0.0, 1.0]);
        assert!(md_potential_energy(&d, [0.0, 0.0, 1.0]) < 0.0 /* negative PE when aligned */);
    }

    #[test]
    fn test_potential_energy_antiparallel_positive() {
        let d = new_magnetic_dipole([0.0, 0.0, 1.0]);
        assert!(md_potential_energy(&d, [0.0, 0.0, -1.0]) > 0.0 /* antiparallel: positive PE */);
    }

    #[test]
    fn test_moment_magnitude() {
        let d = new_magnetic_dipole([3.0, 4.0, 0.0]);
        assert!((md_moment_magnitude(&d) - 5.0).abs() < 1e-9 /* 3-4-5 */);
    }

    #[test]
    fn test_alignment_angle_zero_when_parallel() {
        let d = new_magnetic_dipole([1.0, 0.0, 0.0]);
        let angle = md_alignment_angle(&d, [1.0, 0.0, 0.0]);
        assert!(angle.abs() < 1e-9 /* zero angle */);
    }

    #[test]
    fn test_alignment_angle_ninety_perpendicular() {
        let d = new_magnetic_dipole([1.0, 0.0, 0.0]);
        let angle = md_alignment_angle(&d, [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-9 /* 90° */);
    }

    #[test]
    fn test_force_in_gradient() {
        let d = new_magnetic_dipole([0.0, 0.0, 2.0]);
        let f = md_force_in_gradient(&d, 5.0);
        assert!((f - 10.0).abs() < 1e-9 /* m_z * dBz/dz = 2*5 */);
    }

    #[test]
    fn test_zero_moment_zero_torque() {
        let d = new_magnetic_dipole([0.0; 3]);
        let tau = md_torque(&d, [1.0, 0.0, 0.0]);
        assert!(tau.iter().all(|&x| x == 0.0) /* zero moment → zero torque */);
    }

    #[test]
    fn test_torque_magnitude() {
        /* |τ| = |m||B|sin(θ) — perpendicular: sin=1 */
        let d = new_magnetic_dipole([2.0, 0.0, 0.0]);
        let b = [0.0, 3.0, 0.0];
        let tau = md_torque(&d, b);
        assert!((mag3(tau) - 6.0).abs() < 1e-9 /* 2*3*1 */);
    }
}

