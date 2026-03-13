// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hyperelastic (Neo-Hookean) material model stub.
//!
//! Implements the compressible Neo-Hookean strain energy density and its
//! derivatives (Cauchy stress, tangent moduli) for finite-strain elasticity.

/// Neo-Hookean material parameters.
#[derive(Debug, Clone)]
pub struct NeoHookeanParams {
    /// Shear modulus μ `[Pa]`.
    pub mu: f64,
    /// First Lamé parameter λ `[Pa]`.
    pub lambda: f64,
}

impl NeoHookeanParams {
    /// Construct from Young's modulus and Poisson ratio.
    pub fn from_e_nu(e: f64, nu: f64) -> Self {
        let mu = e / (2.0 * (1.0 + nu));
        let lambda = e * nu / ((1.0 + nu) * (1.0 - 2.0 * nu));
        Self { mu, lambda }
    }

    pub fn bulk_modulus(&self) -> f64 {
        self.lambda + 2.0 * self.mu / 3.0
    }
}

impl Default for NeoHookeanParams {
    fn default() -> Self {
        Self::from_e_nu(1e6, 0.4)
    }
}

/// A 3×3 deformation gradient F stored column-major (9 entries).
#[derive(Debug, Clone)]
pub struct DeformGrad {
    pub f: [f64; 9],
}

impl DeformGrad {
    pub fn identity() -> Self {
        Self {
            f: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Compute J = det(F).
    pub fn jacobian(&self) -> f64 {
        let f = &self.f;
        f[0] * (f[4] * f[8] - f[5] * f[7]) - f[1] * (f[3] * f[8] - f[5] * f[6])
            + f[2] * (f[3] * f[7] - f[4] * f[6])
    }

    /// Compute I1 = tr(F^T F).
    pub fn i1(&self) -> f64 {
        self.f.iter().map(|x| x * x).sum()
    }
}

/// Compute the Neo-Hookean strain energy density W(F).
///
/// W = (μ/2)(I1 - 3) - μ*ln(J) + (λ/2)*ln(J)^2
pub fn strain_energy_density(fg: &DeformGrad, params: &NeoHookeanParams) -> f64 {
    let j = fg.jacobian();
    if j <= 0.0 {
        return f64::INFINITY;
    }
    let i1 = fg.i1();
    let ln_j = j.ln();
    0.5 * params.mu * (i1 - 3.0) - params.mu * ln_j + 0.5 * params.lambda * ln_j * ln_j
}

/// Compute the Cauchy stress P (1st PK stress) — simplified diagonal version for stretch-only.
///
/// For an isotropic stretch λ_i = diag(λ1, λ2, λ3), σ_i = μ(λ_i² - 1/J) / J + λ*ln(J)/J
pub fn cauchy_stress_isotropic(stretch: [f64; 3], params: &NeoHookeanParams) -> [f64; 3] {
    let j = stretch[0] * stretch[1] * stretch[2];
    if j <= 0.0 {
        return [0.0; 3];
    }
    let ln_j = j.ln();
    let mut sigma = [0.0f64; 3];
    for i in 0..3 {
        sigma[i] = (params.mu * (stretch[i] * stretch[i] - 1.0) + params.lambda * ln_j) / j;
    }
    sigma
}

/// Compute volumetric strain energy contribution.
pub fn volumetric_energy(j: f64, params: &NeoHookeanParams) -> f64 {
    if j <= 0.0 {
        return f64::INFINITY;
    }
    let ln_j = j.ln();
    0.5 * params.lambda * ln_j * ln_j
}

/// Compute isochoric (deviatoric) strain energy contribution.
pub fn isochoric_energy(fg: &DeformGrad, params: &NeoHookeanParams) -> f64 {
    let j = fg.jacobian().max(1e-12);
    let i1 = fg.i1();
    let i1_bar = i1 / j.powf(2.0 / 3.0);
    0.5 * params.mu * (i1_bar - 3.0)
}

/// Compute the hydrostatic pressure p = -tr(σ)/3.
pub fn hydrostatic_pressure(stress: &[f64; 3]) -> f64 {
    -(stress[0] + stress[1] + stress[2]) / 3.0
}

/// Check material stability (Drucker stability: W > 0 for F ≠ I).
pub fn is_stable(fg: &DeformGrad, params: &NeoHookeanParams) -> bool {
    let w = strain_energy_density(fg, params);
    w.is_finite() && w >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jacobian_identity() {
        let fg = DeformGrad::identity();
        assert!((fg.jacobian() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_i1_identity() {
        let fg = DeformGrad::identity();
        assert!((fg.i1() - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_strain_energy_zero_at_identity() {
        let fg = DeformGrad::identity();
        let p = NeoHookeanParams::default();
        let w = strain_energy_density(&fg, &p);
        assert!(w.abs() < 1e-10);
    }

    #[test]
    fn test_strain_energy_positive_under_tension() {
        let mut fg = DeformGrad::identity();
        fg.f[0] = 1.2; /* stretch in x */
        let p = NeoHookeanParams::default();
        assert!(strain_energy_density(&fg, &p) > 0.0);
    }

    #[test]
    fn test_cauchy_stress_identity_near_zero() {
        let p = NeoHookeanParams::default();
        let sigma = cauchy_stress_isotropic([1.0, 1.0, 1.0], &p);
        assert!(sigma.iter().all(|s| s.abs() < 1e-9));
    }

    #[test]
    fn test_from_e_nu_shear_modulus() {
        let p = NeoHookeanParams::from_e_nu(1e6, 0.3);
        let expected_mu = 1e6 / (2.0 * 1.3);
        assert!((p.mu - expected_mu).abs() < 1.0);
    }

    #[test]
    fn test_bulk_modulus_positive() {
        let p = NeoHookeanParams::default();
        assert!(p.bulk_modulus() > 0.0);
    }

    #[test]
    fn test_is_stable_identity() {
        let fg = DeformGrad::identity();
        let p = NeoHookeanParams::default();
        assert!(is_stable(&fg, &p));
    }

    #[test]
    fn test_hydrostatic_pressure() {
        let sigma = [-10.0f64, -10.0, -10.0];
        assert!((hydrostatic_pressure(&sigma) - 10.0).abs() < 1e-9);
    }
}
