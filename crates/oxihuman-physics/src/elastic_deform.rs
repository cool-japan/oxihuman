// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Elastic deformation energy and force computations.

#![allow(dead_code)]

/// Compute the Neo-Hookean elastic energy for a deformation gradient F.
/// mu: shear modulus, lambda: first Lame parameter.
#[allow(dead_code)]
#[allow(non_snake_case)]
pub fn neo_hookean_energy(F: [[f32; 3]; 3], mu: f32, lambda: f32) -> f32 {
    // I1 = trace(F^T F)
    let mut i1 = 0.0f32;
    for row in &F {
        for &val in row {
            i1 += val * val;
        }
    }
    // J = det(F) (simplified 3x3 determinant)
    let j = F[0][0] * (F[1][1] * F[2][2] - F[1][2] * F[2][1])
        - F[0][1] * (F[1][0] * F[2][2] - F[1][2] * F[2][0])
        + F[0][2] * (F[1][0] * F[2][1] - F[1][1] * F[2][0]);
    let log_j = if j > 1e-10 { j.ln() } else { -100.0 };
    0.5 * mu * (i1 - 3.0) - mu * log_j + 0.5 * lambda * log_j * log_j
}

/// Compute a simple linear strain measure between two displacement vectors.
#[allow(dead_code)]
pub fn linear_strain(u: [f32; 3], v: [f32; 3]) -> f32 {
    let diff = [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
    (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2]).sqrt()
}

/// Compute the bulk modulus K from Young's modulus E and Poisson's ratio nu.
#[allow(dead_code)]
pub fn bulk_modulus(youngs: f32, poisson: f32) -> f32 {
    youngs / (3.0 * (1.0 - 2.0 * poisson))
}

/// Compute the shear modulus G from Young's modulus E and Poisson's ratio nu.
#[allow(dead_code)]
pub fn shear_modulus(youngs: f32, poisson: f32) -> f32 {
    youngs / (2.0 * (1.0 + poisson))
}

/// Compute the elastic potential energy for a simple spring-like stretch.
/// E = 0.5 * k * (stretch - 1)^2
#[allow(dead_code)]
pub fn elastic_potential_simple(stretch: f32, k: f32) -> f32 {
    0.5 * k * (stretch - 1.0) * (stretch - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_f() -> [[f32; 3]; 3] {
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    }

    #[test]
    fn neo_hookean_identity_is_zero() {
        let e = neo_hookean_energy(identity_f(), 1.0, 1.0);
        // At identity, I1=3, J=1, logJ=0, so energy = 0.5*mu*(3-3) - mu*0 + 0 = 0
        assert!(e.abs() < 1e-5, "expected ~0 got {e}");
    }

    #[test]
    fn neo_hookean_energy_nonnegative() {
        let mut f = identity_f();
        f[0][0] = 1.5; // stretch in x
        let e = neo_hookean_energy(f, 1.0, 1.0);
        assert!(e >= 0.0);
    }

    #[test]
    fn linear_strain_zero_for_equal() {
        let s = linear_strain([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
        assert!(s.abs() < 1e-6);
    }

    #[test]
    fn linear_strain_positive() {
        let s = linear_strain([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn bulk_modulus_positive() {
        let k = bulk_modulus(210_000.0, 0.3);
        assert!(k > 0.0);
    }

    #[test]
    fn shear_modulus_positive() {
        let g = shear_modulus(210_000.0, 0.3);
        assert!(g > 0.0);
    }

    #[test]
    fn elastic_potential_at_rest_is_zero() {
        let e = elastic_potential_simple(1.0, 100.0);
        assert!(e.abs() < 1e-6);
    }

    #[test]
    fn elastic_potential_increases_with_stretch() {
        let e1 = elastic_potential_simple(1.1, 100.0);
        let e2 = elastic_potential_simple(1.5, 100.0);
        assert!(e2 > e1);
    }

    #[test]
    fn bulk_shear_ratio_consistent() {
        let e = 100.0f32;
        let nu = 0.25f32;
        let k = bulk_modulus(e, nu);
        let g = shear_modulus(e, nu);
        // For nu=0.25, K/G = 5/3
        let ratio = k / g;
        assert!((ratio - 5.0 / 3.0).abs() < 1e-3, "got ratio {ratio}");
    }
}
