// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::needless_range_loop)]

//! Neo-Hookean hyperelastic material model.
//!
//! Strain energy density:
//!   W = (mu/2)(I1 - 3) - mu * ln(J) + (lambda/2)(ln J)^2
//!
//! where:
//!   F = deformation gradient
//!   J = det(F)
//!   I1 = tr(F^T F)
//!   mu, lambda = Lame parameters derived from Young's modulus E and Poisson ratio nu
//!
//! First Piola-Kirchhoff stress:
//!   P = mu * (F - F^{-T}) + lambda * ln(J) * F^{-T}

use anyhow::{Context, Result};

use super::{mat3_det, mat3_inverse, mat3_mul, mat3_scale, mat3_sub, mat3_transpose, Mat3, Vec3};

/// Material properties for the soft body.
#[derive(Debug, Clone, Copy)]
pub struct SoftBodyMaterial {
    /// Young's modulus (stiffness), in Pascals.
    pub youngs_modulus: f64,
    /// Poisson's ratio, in range (0, 0.5).
    pub poisson_ratio: f64,
}

impl Default for SoftBodyMaterial {
    fn default() -> Self {
        Self {
            youngs_modulus: 1000.0,
            poisson_ratio: 0.3,
        }
    }
}

impl SoftBodyMaterial {
    /// Create a new material with given parameters.
    pub fn new(youngs_modulus: f64, poisson_ratio: f64) -> Result<Self> {
        anyhow::ensure!(
            youngs_modulus > 0.0,
            "Young's modulus must be positive, got {youngs_modulus}"
        );
        anyhow::ensure!(
            (0.0..0.5).contains(&poisson_ratio),
            "Poisson ratio must be in (0, 0.5), got {poisson_ratio}"
        );
        Ok(Self {
            youngs_modulus,
            poisson_ratio,
        })
    }

    /// Compute first Lame parameter lambda.
    pub fn lambda(&self) -> f64 {
        let e = self.youngs_modulus;
        let nu = self.poisson_ratio;
        e * nu / ((1.0 + nu) * (1.0 - 2.0 * nu))
    }

    /// Compute second Lame parameter mu (shear modulus).
    pub fn mu(&self) -> f64 {
        let e = self.youngs_modulus;
        let nu = self.poisson_ratio;
        e / (2.0 * (1.0 + nu))
    }
}

/// Neo-Hookean hyperelastic material model.
#[derive(Debug, Clone)]
pub struct NeoHookeanModel {
    /// First Lame parameter.
    pub lambda: f64,
    /// Second Lame parameter (shear modulus).
    pub mu: f64,
}

impl NeoHookeanModel {
    /// Create from Lame parameters directly.
    pub fn new(lambda: f64, mu: f64) -> Self {
        Self { lambda, mu }
    }

    /// Create from a `SoftBodyMaterial`.
    pub fn from_material(mat: &SoftBodyMaterial) -> Self {
        Self {
            lambda: mat.lambda(),
            mu: mat.mu(),
        }
    }

    /// Compute strain energy density W for a given deformation gradient F.
    ///
    /// W = (mu/2)(I1 - 3) - mu * ln(J) + (lambda/2)(ln J)^2
    pub fn strain_energy(&self, f: &Mat3) -> Result<f64> {
        let j = mat3_det(f);
        anyhow::ensure!(
            j > 0.0,
            "deformation gradient has non-positive determinant J={j}, element is inverted"
        );

        let i1 = first_invariant(f);
        let ln_j = j.ln();

        let w = 0.5 * self.mu * (i1 - 3.0) - self.mu * ln_j + 0.5 * self.lambda * ln_j * ln_j;
        Ok(w)
    }

    /// Compute the first Piola-Kirchhoff stress tensor P.
    ///
    /// P = mu * (F - F^{-T}) + lambda * ln(J) * F^{-T}
    pub fn piola_kirchhoff_stress(&self, f: &Mat3) -> Result<Mat3> {
        let j = mat3_det(f);
        anyhow::ensure!(
            j > 0.0,
            "PK stress: non-positive determinant J={j}, element is inverted"
        );

        let f_inv = mat3_inverse(f).context("PK stress: failed to invert deformation gradient")?;
        let f_inv_t = mat3_transpose(&f_inv);
        let ln_j = j.ln();

        // P = mu * (F - F^{-T}) + lambda * ln(J) * F^{-T}
        let term1 = mat3_sub(f, &f_inv_t);
        let p = mat3_add_scaled(&term1, self.mu, &f_inv_t, self.lambda * ln_j);
        Ok(p)
    }

    /// Compute element nodal forces for a single tetrahedron.
    ///
    /// The force on each node is derived from the first Piola-Kirchhoff stress:
    ///   H = -V_0 * P * Dm^{-T}
    ///
    /// where V_0 is the rest volume and Dm^{-T} is the transpose of Dm^{-1}.
    ///
    /// Returns forces for the 4 nodes: [f0, f1, f2, f3].
    /// f1, f2, f3 are the columns of H; f0 = -(f1 + f2 + f3).
    pub fn element_forces(
        &self,
        f_grad: &Mat3,
        inv_ref_shape: &Mat3,
        rest_volume: f64,
    ) -> Result<[Vec3; 4]> {
        let p = self.piola_kirchhoff_stress(f_grad)?;

        let dm_inv_t = mat3_transpose(inv_ref_shape);
        let h = mat3_scale(&mat3_mul(&p, &dm_inv_t), -rest_volume);

        // Forces on nodes 1, 2, 3 are columns of H
        let f1 = [h[0][0], h[1][0], h[2][0]];
        let f2 = [h[0][1], h[1][1], h[2][1]];
        let f3 = [h[0][2], h[1][2], h[2][2]];
        // Force on node 0 is reaction: f0 = -(f1 + f2 + f3)
        let f0 = [
            -(f1[0] + f2[0] + f3[0]),
            -(f1[1] + f2[1] + f3[1]),
            -(f1[2] + f2[2] + f3[2]),
        ];

        Ok([f0, f1, f2, f3])
    }

    /// Compute the tangent stiffness contribution for one element.
    ///
    /// This returns a simplified approximation using the co-rotational approach:
    /// For implicit integration we need dP/dF. We approximate using finite differences
    /// around the current configuration for each of the 9 components of F.
    ///
    /// Returns a 12x12 stiffness matrix stored as `[[f64; 12]; 12]`.
    pub fn element_stiffness(
        &self,
        f_grad: &Mat3,
        inv_ref_shape: &Mat3,
        rest_volume: f64,
    ) -> Result<[[f64; 12]; 12]> {
        let epsilon = 1e-7;
        let base_forces = self.element_forces(f_grad, inv_ref_shape, rest_volume)?;

        let mut k = [[0.0_f64; 12]; 12];

        // We perturb each of the 12 DOFs (4 nodes x 3 dimensions)
        // and compute force difference via finite differences.
        // This gives us the stiffness matrix K = -df/dx.
        for node_idx in 0..4 {
            for dim in 0..3 {
                let col = node_idx * 3 + dim;

                // Create a perturbed deformation gradient
                let f_perturbed =
                    perturb_deformation_gradient(f_grad, inv_ref_shape, node_idx, dim, epsilon);

                let perturbed_forces =
                    self.element_forces(&f_perturbed, inv_ref_shape, rest_volume)?;

                // K[:, col] = -(perturbed_forces - base_forces) / epsilon
                for n in 0..4 {
                    for d in 0..3 {
                        let row = n * 3 + d;
                        k[row][col] = -(perturbed_forces[n][d] - base_forces[n][d]) / epsilon;
                    }
                }
            }
        }

        Ok(k)
    }
}

/// Compute first invariant I1 = tr(F^T F) = sum of squared entries of F.
fn first_invariant(f: &Mat3) -> f64 {
    let mut sum = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            sum += f[i][j] * f[i][j];
        }
    }
    sum
}

/// Add two matrices with individual scalars: result = a*s_a + b*s_b.
fn mat3_add_scaled(a: &Mat3, s_a: f64, b: &Mat3, s_b: f64) -> Mat3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][j] * s_a + b[i][j] * s_b;
        }
    }
    r
}

/// Perturb the deformation gradient by adding epsilon to one node's position component.
///
/// F = Ds * Dm^{-1}, so if we perturb node k's component d by epsilon,
/// the change in Ds depends on which node it is:
/// - Node 0: affects all edge vectors (subtracted from all)
/// - Nodes 1,2,3: affects only edge k (column k-1 of Ds)
fn perturb_deformation_gradient(
    f: &Mat3,
    inv_ref_shape: &Mat3,
    node_idx: usize,
    dim: usize,
    epsilon: f64,
) -> Mat3 {
    // delta_Ds depends on which node is perturbed
    let mut delta_ds = [[0.0; 3]; 3];

    if node_idx == 0 {
        // Perturbing node 0 affects all three edge vectors negatively
        for col in 0..3 {
            delta_ds[dim][col] = -epsilon;
        }
    } else {
        // Perturbing node k (1,2,3) affects column (k-1)
        delta_ds[dim][node_idx - 1] = epsilon;
    }

    // delta_F = delta_Ds * Dm^{-1}
    let delta_f = super::mat3_mul(&delta_ds, inv_ref_shape);
    super::mat3_add(f, &delta_f)
}

/// Compute total elastic potential energy for the mesh.
pub fn total_elastic_energy(
    model: &NeoHookeanModel,
    positions: &[Vec3],
    elements: &[super::tetrahedral::TetrahedralElement],
) -> Result<f64> {
    let mut total = 0.0;
    for elem in elements {
        let f = elem.deformation_gradient(positions)?;
        let w = model.strain_energy(&f)?;
        total += w * elem.rest_volume;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lame_parameters() {
        let mat = SoftBodyMaterial::new(1000.0, 0.3).unwrap();
        let lambda = mat.lambda();
        let mu = mat.mu();
        // E = mu * (3*lambda + 2*mu) / (lambda + mu)
        let e_check = mu * (3.0 * lambda + 2.0 * mu) / (lambda + mu);
        assert!((e_check - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_strain_energy_at_rest() {
        let model = NeoHookeanModel::new(577.0, 384.6);
        let identity = super::super::IDENTITY3;
        let w = model.strain_energy(&identity).unwrap();
        // At rest (F=I), I1=3, J=1, ln(J)=0 => W = 0
        assert!(w.abs() < 1e-12, "W at rest = {w}");
    }

    #[test]
    fn test_pk_stress_at_rest() {
        let model = NeoHookeanModel::new(577.0, 384.6);
        let identity = super::super::IDENTITY3;
        let p = model.piola_kirchhoff_stress(&identity).unwrap();
        // At rest P should be zero
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    p[i][j].abs() < 1e-10,
                    "P[{i}][{j}] = {} (expected 0)",
                    p[i][j]
                );
            }
        }
    }

    #[test]
    fn test_element_forces_at_rest() {
        let model = NeoHookeanModel::new(577.0, 384.6);
        let identity = super::super::IDENTITY3;
        let rest_volume = 1.0 / 6.0;
        let forces = model
            .element_forces(&identity, &identity, rest_volume)
            .unwrap();
        for n in 0..4 {
            for d in 0..3 {
                assert!(
                    forces[n][d].abs() < 1e-10,
                    "f[{n}][{d}] = {} (expected 0)",
                    forces[n][d]
                );
            }
        }
    }

    #[test]
    fn test_strain_energy_uniform_stretch() {
        let model = NeoHookeanModel::new(100.0, 50.0);
        // Uniform stretch by factor s
        let s = 1.5;
        let f_stretch: Mat3 = [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]];
        let w = model.strain_energy(&f_stretch).unwrap();
        // Should be positive for any deformation away from rest
        assert!(w > 0.0, "W = {w} should be positive for stretch");
    }

    #[test]
    fn test_pk_stress_uniaxial_stretch() {
        let mat = SoftBodyMaterial::new(1000.0, 0.3).unwrap();
        let model = NeoHookeanModel::from_material(&mat);
        let s = 1.1;
        let f_stretch: Mat3 = [[s, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let p = model.piola_kirchhoff_stress(&f_stretch).unwrap();
        // P[0][0] should be positive (tensile stress in stretch direction)
        assert!(p[0][0] > 0.0, "P[0][0] = {} should be positive", p[0][0]);
        // Off-diagonal should be zero for diagonal F
        assert!(p[0][1].abs() < 1e-10);
        assert!(p[1][0].abs() < 1e-10);
    }

    #[test]
    fn test_material_validation() {
        assert!(SoftBodyMaterial::new(-100.0, 0.3).is_err());
        assert!(SoftBodyMaterial::new(1000.0, 0.5).is_err());
        assert!(SoftBodyMaterial::new(1000.0, -0.1).is_err());
    }

    #[test]
    fn test_element_stiffness_symmetry() {
        let model = NeoHookeanModel::new(577.0, 384.6);
        let identity = super::super::IDENTITY3;
        // Slightly stretched state to avoid singularity at identity
        let f: Mat3 = [[1.01, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let rest_volume = 1.0 / 6.0;
        let k = model.element_stiffness(&f, &identity, rest_volume).unwrap();
        // Stiffness matrix should be approximately symmetric
        for i in 0..12 {
            for j in i + 1..12 {
                let diff = (k[i][j] - k[j][i]).abs();
                let scale = k[i][j].abs().max(k[j][i].abs()).max(1e-10);
                assert!(
                    diff / scale < 0.01,
                    "K[{i}][{j}]={} vs K[{j}][{i}]={} diff={}",
                    k[i][j],
                    k[j][i],
                    diff
                );
            }
        }
    }

    #[test]
    fn test_first_invariant() {
        let identity = super::super::IDENTITY3;
        assert!((first_invariant(&identity) - 3.0).abs() < 1e-15);
    }
}
