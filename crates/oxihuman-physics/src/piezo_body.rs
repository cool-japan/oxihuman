// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Piezoelectric body (stress from electric field).

#![allow(dead_code)]

/// Piezoelectric coupling matrix d_ij (3x3 for simplified model).
/// Strain = d * E_field.
#[allow(dead_code)]
pub type PiezoCoupling = [[f64; 3]; 3];

/// Piezoelectric body (linear piezoelectricity).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PiezoBody {
    pub elastic_modulus: f64,
    pub coupling: PiezoCoupling,
    pub permittivity: [f64; 3],
    pub electric_field: [f64; 3],
    pub polarization: [f64; 3],
}

impl PiezoBody {
    #[allow(dead_code)]
    pub fn new(elastic_modulus: f64, coupling: PiezoCoupling, permittivity: [f64; 3]) -> Self {
        Self {
            elastic_modulus,
            coupling,
            permittivity,
            electric_field: [0.0; 3],
            polarization: [0.0; 3],
        }
    }

    #[allow(dead_code)]
    pub fn set_electric_field(&mut self, e: [f64; 3]) {
        self.electric_field = e;
    }

    /// Piezoelectric strain: epsilon_i = sum_j d_ij * E_j.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn piezo_strain(&self) -> [f64; 3] {
        let mut eps = [0.0f64; 3];
        for i in 0..3 {
            for j in 0..3 {
                eps[i] += self.coupling[i][j] * self.electric_field[j];
            }
        }
        eps
    }

    /// Piezoelectric stress (blocked): sigma = -E_mech * d * E_field.
    #[allow(dead_code)]
    pub fn blocked_stress(&self) -> [f64; 3] {
        let eps = self.piezo_strain();
        [-self.elastic_modulus * eps[0],
         -self.elastic_modulus * eps[1],
         -self.elastic_modulus * eps[2]]
    }

    /// Polarization: P_i = sum_j eps0 * chi_ij * E_j (simplified diagonal).
    #[allow(dead_code)]
    pub fn compute_polarization(&mut self) {
        for i in 0..3 {
            self.polarization[i] = self.permittivity[i] * self.electric_field[i];
        }
    }

    /// Electromechanical coupling coefficient k^2 = d^2 * E / (eps * S).
    /// (For first axis, simplified.)
    #[allow(dead_code)]
    pub fn coupling_coefficient_sq(&self) -> f64 {
        let d = self.coupling[0][0];
        let eps_33 = self.permittivity[0];
        let s_11 = 1.0 / self.elastic_modulus;
        if eps_33 > 0.0 && s_11 > 0.0 {
            d * d / (eps_33 * s_11)
        } else {
            0.0
        }
    }

    /// Blocking force (stress * cross-sectional area = 1).
    #[allow(dead_code)]
    pub fn blocking_force(&self) -> f64 {
        let s = self.blocked_stress();
        (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt()
    }

    /// Free strain magnitude.
    #[allow(dead_code)]
    pub fn free_strain_magnitude(&self) -> f64 {
        let eps = self.piezo_strain();
        (eps[0] * eps[0] + eps[1] * eps[1] + eps[2] * eps[2]).sqrt()
    }

    #[allow(dead_code)]
    pub fn reset_field(&mut self) {
        self.electric_field = [0.0; 3];
    }
}

/// Simple PZT-like piezoelectric material preset.
#[allow(dead_code)]
pub fn pzt_material() -> PiezoBody {
    let d: PiezoCoupling = [
        [0.0, 0.0, 200e-12],
        [0.0, 0.0, 200e-12],
        [0.0, 0.0, 450e-12],
    ];
    PiezoBody::new(60e9, d, [1.0e-8; 3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_strain_no_field() {
        let body = pzt_material();
        let eps = body.piezo_strain();
        assert!(eps.iter().all(|&x| x.abs() < 1e-12));
    }

    #[test]
    fn test_strain_with_field() {
        let mut body = pzt_material();
        body.set_electric_field([0.0, 0.0, 1e6]);
        let eps = body.piezo_strain();
        assert!(eps[2].abs() > 0.0);
    }

    #[test]
    fn test_blocked_stress_sign() {
        let mut body = pzt_material();
        body.set_electric_field([0.0, 0.0, 1e6]);
        let s = body.blocked_stress();
        assert!(s[2] != 0.0);
    }

    #[test]
    fn test_coupling_coefficient() {
        let mut d = [[0.0f64; 3]; 3];
        d[0][0] = 100e-12;
        let body = PiezoBody::new(100e9, d, [1e-9; 3]);
        let k2 = body.coupling_coefficient_sq();
        assert!(k2 >= 0.0);
    }

    #[test]
    fn test_blocking_force_zero_no_field() {
        let body = pzt_material();
        assert!((body.blocking_force()).abs() < 1e-12);
    }

    #[test]
    fn test_free_strain_magnitude() {
        let mut body = pzt_material();
        body.set_electric_field([0.0, 0.0, 1e6]);
        assert!(body.free_strain_magnitude() > 0.0);
    }

    #[test]
    fn test_polarization() {
        let mut body = pzt_material();
        body.set_electric_field([1e6, 0.0, 0.0]);
        body.compute_polarization();
        assert!(body.polarization[0] != 0.0);
    }

    #[test]
    fn test_reset_field() {
        let mut body = pzt_material();
        body.set_electric_field([1e6; 3]);
        body.reset_field();
        assert!(body.electric_field.iter().all(|&x| x.abs() < 1e-12));
    }

    #[test]
    fn test_strain_proportional_to_field() {
        let mut b1 = pzt_material();
        let mut b2 = pzt_material();
        b1.set_electric_field([0.0, 0.0, 1e6]);
        b2.set_electric_field([0.0, 0.0, 2e6]);
        let e1 = b1.piezo_strain();
        let e2 = b2.piezo_strain();
        assert!((e2[2] - 2.0 * e1[2]).abs() < 1e-20);
    }

    #[test]
    fn test_pzt_material_modulus() {
        let body = pzt_material();
        assert!((body.elastic_modulus - 60e9).abs() < 1.0);
    }
}
