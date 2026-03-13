// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Orthotropic elastic material with 9 independent constants.

#![allow(dead_code)]

/// Orthotropic elastic constants (engineering notation).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrthoElastic {
    /// Young's moduli (E1, E2, E3).
    pub e: [f64; 3],
    /// Poisson's ratios (nu12, nu13, nu23).
    pub nu: [f64; 3],
    /// Shear moduli (G12, G13, G23).
    pub g: [f64; 3],
}

impl OrthoElastic {
    #[allow(dead_code)]
    pub fn new(e: [f64; 3], nu: [f64; 3], g: [f64; 3]) -> Self {
        Self { e, nu, g }
    }

    /// Isotropic special case.
    #[allow(dead_code)]
    pub fn isotropic(young: f64, poisson: f64) -> Self {
        let g = young / (2.0 * (1.0 + poisson));
        Self {
            e: [young; 3],
            nu: [poisson; 3],
            g: [g; 3],
        }
    }

    /// Compliance matrix row for normal stress (engineering notation).
    /// Returns [S11, S22, S33] for an axial strain given axial stress sigma_i.
    #[allow(dead_code)]
    pub fn compliance_diagonal(&self) -> [f64; 3] {
        [1.0 / self.e[0], 1.0 / self.e[1], 1.0 / self.e[2]]
    }

    /// Off-diagonal compliance: epsilon_j = -nu_ij/E_i * sigma_i.
    #[allow(dead_code)]
    pub fn poisson_compliance(&self) -> [[f64; 3]; 3] {
        let mut s = [[0.0f64; 3]; 3];
        s[0][1] = -self.nu[0] / self.e[0];
        s[0][2] = -self.nu[1] / self.e[0];
        s[1][0] = -self.nu[0] / self.e[1];
        s[1][2] = -self.nu[2] / self.e[1];
        s[2][0] = -self.nu[1] / self.e[2];
        s[2][1] = -self.nu[2] / self.e[2];
        s
    }

    /// Shear compliance: gamma_ij = tau_ij / G_ij.
    #[allow(dead_code)]
    pub fn shear_compliance(&self) -> [f64; 3] {
        [1.0 / self.g[0], 1.0 / self.g[1], 1.0 / self.g[2]]
    }

    /// Elastic strain energy density: W = 0.5 * sigma : epsilon.
    #[allow(dead_code)]
    pub fn strain_energy(&self, stress: &[f64; 6]) -> f64 {
        let s_comp = self.compliance_diagonal();
        let p_comp = self.poisson_compliance();
        let sh_comp = self.shear_compliance();

        let eps0 = s_comp[0] * stress[0]
            + p_comp[1][0] * stress[1]
            + p_comp[2][0] * stress[2];
        let eps1 = p_comp[0][1] * stress[0]
            + s_comp[1] * stress[1]
            + p_comp[2][1] * stress[2];
        let eps2 = p_comp[0][2] * stress[0]
            + p_comp[1][2] * stress[1]
            + s_comp[2] * stress[2];
        let eps3 = sh_comp[0] * stress[3];
        let eps4 = sh_comp[1] * stress[4];
        let eps5 = sh_comp[2] * stress[5];

        0.5 * (stress[0] * eps0
            + stress[1] * eps1
            + stress[2] * eps2
            + stress[3] * eps3
            + stress[4] * eps4
            + stress[5] * eps5)
    }

    /// Check symmetry: nu_ij/E_i == nu_ji/E_j (reciprocal relations).
    #[allow(dead_code)]
    pub fn is_symmetric(&self) -> bool {
        let tol = 1e-9;
        let r01 = (self.nu[0] / self.e[0] - self.nu[0] / self.e[1]).abs() < tol;
        let r02 = (self.nu[1] / self.e[0] - self.nu[1] / self.e[2]).abs() < tol;
        let r12 = (self.nu[2] / self.e[1] - self.nu[2] / self.e[2]).abs() < tol;
        r01 && r02 && r12
    }

    #[allow(dead_code)]
    pub fn avg_modulus(&self) -> f64 {
        (self.e[0] + self.e[1] + self.e[2]) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isotropic_compliance_diagonal() {
        let mat = OrthoElastic::isotropic(200e9, 0.3);
        let s = mat.compliance_diagonal();
        for &v in &s {
            assert!((v - 1.0 / 200e9).abs() < 1e-20);
        }
    }

    #[test]
    fn test_isotropic_shear_compliance() {
        let mat = OrthoElastic::isotropic(200e9, 0.3);
        let sc = mat.shear_compliance();
        let expected_g = 200e9 / (2.0 * 1.3);
        for &v in &sc {
            assert!((v - 1.0 / expected_g).abs() < 1e-10);
        }
    }

    #[test]
    fn test_strain_energy_positive_uniaxial() {
        let mat = OrthoElastic::isotropic(200e9, 0.3);
        let stress = [100.0, 0.0, 0.0, 0.0, 0.0, 0.0f64];
        let w = mat.strain_energy(&stress);
        assert!(w > 0.0, "W={w}");
    }

    #[test]
    fn test_strain_energy_zero_no_stress() {
        let mat = OrthoElastic::isotropic(200e9, 0.3);
        let stress = [0.0f64; 6];
        assert!((mat.strain_energy(&stress)).abs() < 1e-12);
    }

    #[test]
    fn test_poisson_compliance_off_diagonal() {
        let mat = OrthoElastic::isotropic(200e9, 0.3);
        let pc = mat.poisson_compliance();
        assert!(pc[0][1] < 0.0);
    }

    #[test]
    fn test_avg_modulus_isotropic() {
        let mat = OrthoElastic::isotropic(100.0, 0.25);
        assert!((mat.avg_modulus() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_custom_orthotropic() {
        let mat = OrthoElastic::new(
            [200e9, 100e9, 50e9],
            [0.3, 0.2, 0.25],
            [80e9, 40e9, 20e9],
        );
        assert!(mat.e[0] > mat.e[1]);
    }

    #[test]
    fn test_shear_compliance_reciprocal() {
        let mat = OrthoElastic::new([1.0, 1.0, 1.0], [0.3, 0.3, 0.3], [0.5, 0.5, 0.5]);
        let sc = mat.shear_compliance();
        assert!(sc.iter().all(|&x| (x - 2.0).abs() < 1e-9));
    }

    #[test]
    fn test_g_shear_isotropic_3d() {
        let mat = OrthoElastic::isotropic(200.0, 0.0);
        assert!((mat.g[0] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_strain_energy_shear_positive() {
        let mat = OrthoElastic::isotropic(200e9, 0.3);
        let stress = [0.0, 0.0, 0.0, 100.0, 0.0, 0.0f64];
        let w = mat.strain_energy(&stress);
        assert!(w > 0.0);
    }
}
