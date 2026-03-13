// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Anisotropic stiffness tensor stub.
//!
//! Stores and manipulates the 6×6 Voigt-notation stiffness matrix for
//! general anisotropic (triclinic) linear elastic materials.

/// 6×6 stiffness matrix in Voigt notation (row-major).
#[derive(Debug, Clone)]
pub struct StiffnessTensor {
    pub c: [[f64; 6]; 6],
}

impl StiffnessTensor {
    /// Construct a zero stiffness tensor.
    pub fn zero() -> Self {
        Self { c: [[0.0; 6]; 6] }
    }

    /// Construct an isotropic stiffness tensor from λ and μ.
    pub fn isotropic(lambda: f64, mu: f64) -> Self {
        let mut t = Self::zero();
        /* C_11 = C_22 = C_33 = λ + 2μ */
        let c11 = lambda + 2.0 * mu;
        for i in 0..3 {
            t.c[i][i] = c11;
        }
        /* Off-diagonal: C_12 = C_13 = C_23 = λ */
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    t.c[i][j] = lambda;
                }
            }
        }
        /* Shear: C_44 = C_55 = C_66 = μ */
        for i in 3..6 {
            t.c[i][i] = mu;
        }
        t
    }

    /// Construct a transversely isotropic tensor (fiber direction = z).
    pub fn transversely_isotropic(e_t: f64, e_l: f64, nu_tt: f64, nu_lt: f64, g_lt: f64) -> Self {
        let mut t = Self::zero();
        let nu_tl = nu_lt * e_t / e_l;
        let d = 1.0 - nu_tt * nu_tt - 2.0 * nu_tl * nu_lt - 2.0 * nu_tl * nu_tt * nu_lt;
        if d.abs() < 1e-30 {
            return t;
        }
        t.c[0][0] = e_t * (1.0 - nu_tl * nu_lt) / d;
        t.c[1][1] = t.c[0][0];
        t.c[2][2] = e_l * (1.0 - nu_tt * nu_tt) / d;
        t.c[0][1] = e_t * (nu_tt + nu_tl * nu_lt) / d;
        t.c[1][0] = t.c[0][1];
        t.c[0][2] = e_t * (nu_lt + nu_tt * nu_lt) / d;
        t.c[2][0] = t.c[0][2];
        t.c[1][2] = t.c[0][2];
        t.c[2][1] = t.c[0][2];
        t.c[3][3] = g_lt;
        t.c[4][4] = g_lt;
        t.c[5][5] = 0.5 * e_t / (1.0 + nu_tt);
        t
    }

    /// Apply Voigt-notation: σ = C * ε, where ε is a 6-vector.
    #[allow(clippy::needless_range_loop)]
    pub fn apply(&self, strain: &[f64; 6]) -> [f64; 6] {
        let mut stress = [0.0f64; 6];
        for i in 0..6 {
            for j in 0..6 {
                stress[i] += self.c[i][j] * strain[j];
            }
        }
        stress
    }

    /// Check symmetry: `C[i][j]` == `C[j][i]` (within tolerance).
    pub fn is_symmetric(&self, tol: f64) -> bool {
        for i in 0..6 {
            for j in 0..6 {
                if (self.c[i][j] - self.c[j][i]).abs() > tol {
                    return false;
                }
            }
        }
        true
    }

    /// Compute the bulk modulus K from the tensor (isotropic average).
    pub fn bulk_modulus_voigt(&self) -> f64 {
        (self.c[0][0]
            + self.c[1][1]
            + self.c[2][2]
            + 2.0 * (self.c[0][1] + self.c[0][2] + self.c[1][2]))
            / 9.0
    }

    /// Compute the shear modulus G from the tensor (Voigt average).
    pub fn shear_modulus_voigt(&self) -> f64 {
        (self.c[0][0] + self.c[1][1] + self.c[2][2] - self.c[0][1] - self.c[0][2] - self.c[1][2]
            + 3.0 * (self.c[3][3] + self.c[4][4] + self.c[5][5]))
            / 15.0
    }
}

/// Rotate a stiffness tensor by angle `theta` about the z-axis.
pub fn rotate_stiffness_z(c: &StiffnessTensor, theta: f64) -> StiffnessTensor {
    /* Stub: only applies c² and s² terms for the C_11, C_22 swap */
    let cs = theta.cos();
    let sn = theta.sin();
    let mut out = c.clone();
    out.c[0][0] = c.c[0][0] * cs * cs + c.c[1][1] * sn * sn;
    out.c[1][1] = c.c[0][0] * sn * sn + c.c[1][1] * cs * cs;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isotropic_is_symmetric() {
        let t = StiffnessTensor::isotropic(1e9, 5e8);
        assert!(t.is_symmetric(1e-6));
    }

    #[test]
    fn test_isotropic_apply_identity_strain() {
        let t = StiffnessTensor::isotropic(1e9, 5e8);
        let strain = [0.001, 0.0, 0.0, 0.0, 0.0, 0.0];
        let stress = t.apply(&strain);
        assert!(stress[0] > 0.0);
    }

    #[test]
    fn test_zero_tensor_apply_zero() {
        let t = StiffnessTensor::zero();
        let strain = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let stress = t.apply(&strain);
        assert_eq!(stress, [0.0; 6]);
    }

    #[test]
    fn test_bulk_modulus_positive() {
        let t = StiffnessTensor::isotropic(1e9, 5e8);
        assert!(t.bulk_modulus_voigt() > 0.0);
    }

    #[test]
    fn test_shear_modulus_positive() {
        let t = StiffnessTensor::isotropic(1e9, 5e8);
        assert!(t.shear_modulus_voigt() > 0.0);
    }

    #[test]
    fn test_transversely_isotropic_nonzero() {
        let t = StiffnessTensor::transversely_isotropic(1e9, 2e9, 0.3, 0.2, 4e8);
        assert!(t.c[0][0] > 0.0);
    }

    #[test]
    fn test_rotate_z_identity() {
        let t = StiffnessTensor::isotropic(1e9, 5e8);
        let r = rotate_stiffness_z(&t, 0.0);
        assert!((r.c[0][0] - t.c[0][0]).abs() < 1e-3);
    }

    #[test]
    fn test_isotropic_c11_c44_relation() {
        /* For isotropic: C11 - C12 = 2*C44 (approximately) */
        let lambda = 1e9;
        let mu = 5e8;
        let t = StiffnessTensor::isotropic(lambda, mu);
        let lhs = t.c[0][0] - t.c[0][1];
        let rhs = 2.0 * t.c[3][3];
        assert!((lhs - rhs).abs() < 1.0);
    }

    #[test]
    fn test_zero_is_symmetric() {
        let t = StiffnessTensor::zero();
        assert!(t.is_symmetric(1e-30));
    }
}
