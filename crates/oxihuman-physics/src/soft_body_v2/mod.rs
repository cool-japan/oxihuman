// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::needless_range_loop)]

//! Soft-body simulation v2 with Finite Element Method (FEM).
//!
//! This module provides a co-rotational FEM solver with Neo-Hookean
//! hyperelastic material model and implicit Euler integration.

pub mod fem_solver;
pub mod hyperelastic;
pub mod integrator;
pub mod tetrahedral;

pub use fem_solver::{FemSolver, FemState};
pub use hyperelastic::{NeoHookeanModel, SoftBodyMaterial};
pub use integrator::{ImplicitEulerIntegrator, IntegrationResult};
pub use tetrahedral::{TetMesh, TetrahedralElement};

use anyhow::{Context, Result};

/// 3D vector type alias for convenience.
pub type Vec3 = [f64; 3];

/// Configuration for the v2 soft-body solver.
#[derive(Debug, Clone)]
pub struct SoftBodyConfigV2 {
    /// Time step size.
    pub dt: f64,
    /// Gravity vector.
    pub gravity: Vec3,
    /// Number of solver iterations per step.
    pub iterations: usize,
    /// Material properties.
    pub material: SoftBodyMaterial,
    /// Velocity damping factor in [0, 1].
    pub damping: f64,
}

impl Default for SoftBodyConfigV2 {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            gravity: [0.0, -9.81, 0.0],
            iterations: 10,
            material: SoftBodyMaterial::default(),
            damping: 0.99,
        }
    }
}

/// Main soft-body v2 simulation object.
#[derive(Debug, Clone)]
pub struct SoftBodyV2 {
    /// Current node positions.
    pub positions: Vec<Vec3>,
    /// Current node velocities.
    pub velocities: Vec<Vec3>,
    /// Rest (reference) positions.
    pub rest_positions: Vec<Vec3>,
    /// Tetrahedra as index quadruples into `positions`.
    pub tetrahedra: Vec<[usize; 4]>,
    /// Inverse masses for each node (0 = fixed).
    pub inv_masses: Vec<f64>,
}

impl SoftBodyV2 {
    /// Create a new soft body from positions and tetrahedra.
    ///
    /// All nodes are given uniform mass by default.
    pub fn new(
        positions: Vec<Vec3>,
        tetrahedra: Vec<[usize; 4]>,
        mass_per_node: f64,
    ) -> Result<Self> {
        let n = positions.len();
        anyhow::ensure!(n > 0, "SoftBodyV2 requires at least one node");
        anyhow::ensure!(mass_per_node > 0.0, "mass_per_node must be positive");
        let inv_mass = 1.0 / mass_per_node;
        let rest_positions = positions.clone();
        let velocities = vec![[0.0; 3]; n];
        let inv_masses = vec![inv_mass; n];
        Ok(Self {
            positions,
            velocities,
            rest_positions,
            tetrahedra,
            inv_masses,
        })
    }

    /// Fix a node in place (set inverse mass to zero).
    pub fn fix_node(&mut self, idx: usize) -> Result<()> {
        let m = self
            .inv_masses
            .get_mut(idx)
            .context("node index out of bounds")?;
        *m = 0.0;
        Ok(())
    }

    /// Return the number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.positions.len()
    }

    /// Return the number of tetrahedra.
    pub fn num_tetrahedra(&self) -> usize {
        self.tetrahedra.len()
    }

    /// Advance the simulation by one time step using the given configuration.
    pub fn step(&mut self, config: &SoftBodyConfigV2) -> Result<()> {
        let tet_mesh = TetMesh::from_soft_body(self)?;
        let model = NeoHookeanModel::from_material(&config.material);
        let mut solver = FemSolver::new(&tet_mesh, &model);
        let forces = solver.compute_forces(&self.positions, &self.rest_positions, &tet_mesh)?;

        let mut integrator = ImplicitEulerIntegrator::new(config.dt, config.iterations);
        integrator.integrate(
            &mut self.positions,
            &mut self.velocities,
            &self.inv_masses,
            &forces,
            &config.gravity,
            config.damping,
        )?;

        Ok(())
    }
}

/// Helper: compute dot product of two 3-vectors.
#[inline]
pub fn dot3(a: &Vec3, b: &Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Helper: subtract two 3-vectors.
#[inline]
pub fn sub3(a: &Vec3, b: &Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Helper: add two 3-vectors.
#[inline]
pub fn add3(a: &Vec3, b: &Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Helper: scale a 3-vector.
#[inline]
pub fn scale3(v: &Vec3, s: f64) -> Vec3 {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Helper: squared length of a 3-vector.
#[inline]
pub fn len_sq3(v: &Vec3) -> f64 {
    dot3(v, v)
}

/// Helper: length of a 3-vector.
#[inline]
pub fn len3(v: &Vec3) -> f64 {
    len_sq3(v).sqrt()
}

/// 3x3 matrix stored as row-major `[[row0], [row1], [row2]]`.
pub type Mat3 = [[f64; 3]; 3];

/// Identity 3x3 matrix.
pub const IDENTITY3: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Matrix-vector multiply for 3x3.
#[inline]
pub fn mat3_mul_vec(m: &Mat3, v: &Vec3) -> Vec3 {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Matrix-matrix multiply for 3x3.
#[inline]
pub fn mat3_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    c
}

/// Transpose of a 3x3 matrix.
#[inline]
pub fn mat3_transpose(m: &Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Determinant of a 3x3 matrix.
#[inline]
pub fn mat3_det(m: &Mat3) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Inverse of a 3x3 matrix.  Returns `None` if determinant is near-zero.
pub fn mat3_inverse(m: &Mat3) -> Option<Mat3> {
    let det = mat3_det(m);
    if det.abs() < 1e-30 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
        ],
    ])
}

/// Add two 3x3 matrices.
#[inline]
pub fn mat3_add(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = a[i][j] + b[i][j];
        }
    }
    c
}

/// Subtract two 3x3 matrices: a - b.
#[inline]
pub fn mat3_sub(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = a[i][j] - b[i][j];
        }
    }
    c
}

/// Scale a 3x3 matrix by a scalar.
#[inline]
pub fn mat3_scale(m: &Mat3, s: f64) -> Mat3 {
    let mut r = *m;
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] *= s;
        }
    }
    r
}

/// Frobenius norm of a 3x3 matrix.
pub fn mat3_frobenius_norm(m: &Mat3) -> f64 {
    let mut sum = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            sum += m[i][j] * m[i][j];
        }
    }
    sum.sqrt()
}

/// Polar decomposition of F into R * S where R is rotation and S is symmetric.
///
/// Uses iterative method: R_{k+1} = 0.5 * (R_k + R_k^{-T}).
/// Converges quickly for well-conditioned matrices.
pub fn polar_decomposition(f: &Mat3, max_iter: usize, tol: f64) -> Result<(Mat3, Mat3)> {
    let mut r = *f;
    for _iter in 0..max_iter {
        let r_inv_t = match mat3_inverse(&r) {
            Some(inv) => mat3_transpose(&inv),
            None => {
                // If singular, perturb slightly
                let perturbed = mat3_add(&r, &mat3_scale(&IDENTITY3, 1e-10));
                let inv = mat3_inverse(&perturbed)
                    .context("polar decomposition: matrix still singular after perturbation")?;
                mat3_transpose(&inv)
            }
        };

        let r_new = mat3_scale(&mat3_add(&r, &r_inv_t), 0.5);
        let diff = mat3_frobenius_norm(&mat3_sub(&r_new, &r));
        r = r_new;
        if diff < tol {
            break;
        }
    }

    // S = R^T * F
    let s = mat3_mul(&mat3_transpose(&r), f);
    Ok((r, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_polar_decomposition() {
        let (r, s) = polar_decomposition(&IDENTITY3, 50, 1e-12).expect("should succeed");
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((r[i][j] - expected).abs() < 1e-10);
                assert!((s[i][j] - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_mat3_det() {
        assert!((mat3_det(&IDENTITY3) - 1.0).abs() < 1e-15);
        let m = [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]];
        assert!((mat3_det(&m) - 24.0).abs() < 1e-12);
    }

    #[test]
    fn test_mat3_inverse() {
        let m = [[2.0, 1.0, 0.0], [0.0, 3.0, 1.0], [1.0, 0.0, 2.0]];
        let inv = mat3_inverse(&m).expect("should succeed");
        let prod = mat3_mul(&m, &inv);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((prod[i][j] - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_soft_body_v2_creation() {
        let positions = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let tets = vec![[0, 1, 2, 3]];
        let body = SoftBodyV2::new(positions, tets, 1.0).expect("should succeed");
        assert_eq!(body.num_nodes(), 4);
        assert_eq!(body.num_tetrahedra(), 1);
    }

    #[test]
    fn test_fix_node() {
        let positions = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let tets = vec![[0, 1, 2, 3]];
        let mut body = SoftBodyV2::new(positions, tets, 1.0).expect("should succeed");
        body.fix_node(0).expect("should succeed");
        assert!((body.inv_masses[0]).abs() < 1e-15);
    }

    #[test]
    fn test_polar_decomposition_rotation() {
        // A pure rotation (90 degrees about Z axis)
        let r_in: Mat3 = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let (r_out, s_out) = polar_decomposition(&r_in, 50, 1e-12).expect("should succeed");
        // R should match input rotation
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (r_out[i][j] - r_in[i][j]).abs() < 1e-8,
                    "r_out[{i}][{j}] = {} vs expected {}",
                    r_out[i][j],
                    r_in[i][j]
                );
            }
        }
        // S should be identity
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (s_out[i][j] - expected).abs() < 1e-8,
                    "s_out[{i}][{j}] = {} vs expected {expected}",
                    s_out[i][j]
                );
            }
        }
    }
}
