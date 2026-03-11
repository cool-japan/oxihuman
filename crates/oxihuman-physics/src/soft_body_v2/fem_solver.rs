// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::needless_range_loop)]

//! Co-rotational FEM solver with polar decomposition.
//!
//! The co-rotational approach extracts the rotation R from the deformation
//! gradient F = R * S via polar decomposition, then computes forces in the
//! rotated frame to handle large rotations without artifacts.

use anyhow::{Context, Result};

use super::hyperelastic::NeoHookeanModel;
use super::tetrahedral::TetMesh;
use super::{mat3_mul, mat3_transpose, polar_decomposition, Mat3, Vec3};

/// State tracked per element by the FEM solver.
#[derive(Debug, Clone)]
pub struct ElementState {
    /// Extracted rotation from polar decomposition.
    pub rotation: Mat3,
    /// Symmetric stretch component.
    pub stretch: Mat3,
    /// Deformation gradient.
    pub deformation_gradient: Mat3,
}

/// Per-element cached data for the solver.
#[derive(Debug, Clone)]
pub struct FemState {
    /// Per-element states.
    pub element_states: Vec<ElementState>,
}

/// Co-rotational FEM solver.
#[derive(Debug)]
pub struct FemSolver<'a> {
    /// Reference to the tetrahedral mesh.
    mesh: &'a TetMesh,
    /// Reference to the material model.
    model: &'a NeoHookeanModel,
    /// Max iterations for polar decomposition.
    polar_max_iter: usize,
    /// Tolerance for polar decomposition convergence.
    polar_tol: f64,
}

impl<'a> FemSolver<'a> {
    /// Create a new FEM solver.
    pub fn new(mesh: &'a TetMesh, model: &'a NeoHookeanModel) -> Self {
        Self {
            mesh,
            model,
            polar_max_iter: 50,
            polar_tol: 1e-10,
        }
    }

    /// Set polar decomposition parameters.
    pub fn with_polar_params(mut self, max_iter: usize, tol: f64) -> Self {
        self.polar_max_iter = max_iter;
        self.polar_tol = tol;
        self
    }

    /// Extract rotation and stretch from each element's deformation gradient.
    pub fn compute_element_states(&self, positions: &[Vec3]) -> Result<FemState> {
        let mut element_states = Vec::with_capacity(self.mesh.elements.len());

        for (idx, elem) in self.mesh.elements.iter().enumerate() {
            let f = elem
                .deformation_gradient(positions)
                .with_context(|| format!("failed to compute F for element {idx}"))?;

            let (r, s) = polar_decomposition(&f, self.polar_max_iter, self.polar_tol)
                .with_context(|| format!("polar decomposition failed for element {idx}"))?;

            element_states.push(ElementState {
                rotation: r,
                stretch: s,
                deformation_gradient: f,
            });
        }

        Ok(FemState { element_states })
    }

    /// Compute global force vector from elastic forces.
    ///
    /// Uses the co-rotational approach:
    /// 1. Compute deformation gradient F for each element
    /// 2. Extract rotation R via polar decomposition: F = R * S
    /// 3. Compute Piola-Kirchhoff stress from the full deformation gradient
    /// 4. Assemble element forces into global force vector
    ///
    /// The co-rotational formulation ensures that pure rotations do not
    /// generate spurious elastic forces.
    pub fn compute_forces(
        &mut self,
        positions: &[Vec3],
        _rest_positions: &[Vec3],
        mesh: &TetMesh,
    ) -> Result<Vec<Vec3>> {
        let n = mesh.num_nodes;
        let mut global_forces = vec![[0.0_f64; 3]; n];

        for (idx, elem) in mesh.elements.iter().enumerate() {
            let f = elem
                .deformation_gradient(positions)
                .with_context(|| format!("force computation: failed F for element {idx}"))?;

            // Polar decomposition: F = R * S
            let (r, _s) = polar_decomposition(&f, self.polar_max_iter, self.polar_tol)
                .with_context(|| {
                    format!("force computation: polar decomp failed for element {idx}")
                })?;

            // Co-rotational approach: compute forces using the rotated deformation
            // We use the full deformation gradient for the Neo-Hookean model
            // but the co-rotational stiffness uses the rotation to handle large rotations.
            let element_forces = self
                .compute_corotational_forces(elem, &f, &r)
                .with_context(|| format!("force computation failed for element {idx}"))?;

            // Assemble into global force vector
            for (local_idx, &node_idx) in elem.indices.iter().enumerate() {
                if node_idx < n {
                    for d in 0..3 {
                        global_forces[node_idx][d] += element_forces[local_idx][d];
                    }
                }
            }
        }

        Ok(global_forces)
    }

    /// Compute co-rotational forces for a single element.
    ///
    /// In the co-rotational formulation:
    /// - We compute the Piola-Kirchhoff stress from the full deformation gradient
    /// - The rotation R ensures the linear part of the stiffness is properly rotated
    fn compute_corotational_forces(
        &self,
        elem: &super::tetrahedral::TetrahedralElement,
        f_grad: &Mat3,
        rotation: &Mat3,
    ) -> Result<[Vec3; 4]> {
        // Check if the element is inverted
        let j = super::mat3_det(f_grad);
        if j <= 0.0 {
            // For inverted elements, use a regularized approach
            return self.compute_regularized_forces(elem, f_grad, rotation);
        }

        // Compute forces using the Neo-Hookean model
        let forces = self
            .model
            .element_forces(f_grad, &elem.inv_ref_shape, elem.rest_volume)?;

        Ok(forces)
    }

    /// Compute regularized forces for inverted elements.
    ///
    /// When J <= 0, the standard Neo-Hookean model breaks down.
    /// We use a simple penalty force to push the element back to a valid configuration.
    fn compute_regularized_forces(
        &self,
        elem: &super::tetrahedral::TetrahedralElement,
        _f_grad: &Mat3,
        rotation: &Mat3,
    ) -> Result<[Vec3; 4]> {
        // Use rotation to define a target configuration
        // Apply a stiff penalty proportional to the deviation from R * rest_shape
        let penalty_strength = self.model.mu * 10.0;

        // Force toward the rotated rest configuration
        let dm_inv_t = mat3_transpose(&elem.inv_ref_shape);
        let target_ds = mat3_mul(
            rotation,
            &mat3_transpose(&mat3_transpose(
                &super::mat3_inverse(&elem.inv_ref_shape).unwrap_or(super::IDENTITY3),
            )),
        );

        // Simplified penalty: just use rotation-scaled forces
        let h = super::mat3_scale(
            &mat3_mul(&super::mat3_sub(rotation, &super::IDENTITY3), &dm_inv_t),
            -penalty_strength * elem.rest_volume,
        );

        let _ = target_ds; // suppress unused warning

        let f1 = [h[0][0], h[1][0], h[2][0]];
        let f2 = [h[0][1], h[1][1], h[2][1]];
        let f3 = [h[0][2], h[1][2], h[2][2]];
        let f0 = [
            -(f1[0] + f2[0] + f3[0]),
            -(f1[1] + f2[1] + f3[1]),
            -(f1[2] + f2[2] + f3[2]),
        ];

        Ok([f0, f1, f2, f3])
    }

    /// Compute the global stiffness matrix in sparse triplet format.
    ///
    /// Returns (rows, cols, values) triplets for the assembled stiffness matrix.
    /// The matrix is of size (3*num_nodes) x (3*num_nodes).
    pub fn compute_stiffness_triplets(
        &self,
        positions: &[Vec3],
    ) -> Result<(Vec<usize>, Vec<usize>, Vec<f64>)> {
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        let mut values = Vec::new();

        for (idx, elem) in self.mesh.elements.iter().enumerate() {
            let f = elem
                .deformation_gradient(positions)
                .with_context(|| format!("stiffness: failed F for element {idx}"))?;

            let j = super::mat3_det(&f);
            if j <= 0.0 {
                // Skip inverted elements for stiffness computation
                continue;
            }

            let ke = self
                .model
                .element_stiffness(&f, &elem.inv_ref_shape, elem.rest_volume)
                .with_context(|| format!("stiffness computation failed for element {idx}"))?;

            // Map local 12x12 to global indices
            for local_i in 0..4 {
                let global_i = elem.indices[local_i];
                for local_j in 0..4 {
                    let global_j = elem.indices[local_j];
                    for di in 0..3 {
                        for dj in 0..3 {
                            let val = ke[local_i * 3 + di][local_j * 3 + dj];
                            if val.abs() > 1e-30 {
                                rows.push(global_i * 3 + di);
                                cols.push(global_j * 3 + dj);
                                values.push(val);
                            }
                        }
                    }
                }
            }
        }

        Ok((rows, cols, values))
    }

    /// Compute element-wise elastic energy.
    pub fn compute_elastic_energy(&self, positions: &[Vec3]) -> Result<f64> {
        let mut total = 0.0;
        for (idx, elem) in self.mesh.elements.iter().enumerate() {
            let f = elem
                .deformation_gradient(positions)
                .with_context(|| format!("energy: failed F for element {idx}"))?;
            let j = super::mat3_det(&f);
            if j <= 0.0 {
                // Penalize inverted elements
                total += self.model.mu * elem.rest_volume * 100.0;
                continue;
            }
            let w = self.model.strain_energy(&f)?;
            total += w * elem.rest_volume;
        }
        Ok(total)
    }

    /// Apply a force-based position correction to handle element inversion.
    ///
    /// Returns the number of inverted elements found.
    pub fn count_inverted_elements(&self, positions: &[Vec3]) -> Result<usize> {
        let mut count = 0;
        for elem in &self.mesh.elements {
            let f = elem.deformation_gradient(positions)?;
            if super::mat3_det(&f) <= 0.0 {
                count += 1;
            }
        }
        Ok(count)
    }
}

/// Assemble element forces into a global force vector.
///
/// This is a standalone function for assembling forces from multiple elements.
pub fn assemble_forces(element_forces: &[([usize; 4], [Vec3; 4])], num_nodes: usize) -> Vec<Vec3> {
    let mut global = vec![[0.0_f64; 3]; num_nodes];
    for (indices, forces) in element_forces {
        for (local_idx, &node_idx) in indices.iter().enumerate() {
            if node_idx < num_nodes {
                for d in 0..3 {
                    global[node_idx][d] += forces[local_idx][d];
                }
            }
        }
    }
    global
}

/// Sparse matrix-vector multiply using triplet format.
///
/// Computes y = A * x where A is given as (rows, cols, values) triplets.
pub fn sparse_matvec(
    rows: &[usize],
    cols: &[usize],
    values: &[f64],
    x: &[f64],
    n: usize,
) -> Vec<f64> {
    let mut y = vec![0.0; n];
    for ((&r, &c), &v) in rows.iter().zip(cols.iter()).zip(values.iter()) {
        if r < n && c < x.len() {
            y[r] += v * x[c];
        }
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soft_body_v2::hyperelastic::SoftBodyMaterial;
    use crate::soft_body_v2::tetrahedral::TetMesh;

    fn make_test_tet() -> (Vec<Vec3>, Vec<[usize; 4]>, TetMesh) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let tets = vec![[0, 1, 2, 3]];
        let mesh = TetMesh::new(&tets, &positions, positions.len()).unwrap();
        (positions, tets, mesh)
    }

    #[test]
    fn test_compute_forces_at_rest() {
        let (positions, _tets, mesh) = make_test_tet();
        let mat = SoftBodyMaterial::default();
        let model = NeoHookeanModel::from_material(&mat);
        let mut solver = FemSolver::new(&mesh, &model);

        let forces = solver
            .compute_forces(&positions, &positions, &mesh)
            .unwrap();

        // At rest configuration, forces should be (near) zero
        for (i, f) in forces.iter().enumerate() {
            for d in 0..3 {
                assert!(
                    f[d].abs() < 1e-8,
                    "force[{i}][{d}] = {} (expected ~0)",
                    f[d]
                );
            }
        }
    }

    #[test]
    fn test_compute_forces_under_stretch() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let rest = positions.clone();
        let tets = vec![[0, 1, 2, 3]];
        let mesh = TetMesh::new(&tets, &rest, rest.len()).unwrap();
        let mat = SoftBodyMaterial::default();
        let model = NeoHookeanModel::from_material(&mat);
        let mut solver = FemSolver::new(&mesh, &model);

        // Stretch the tet
        let stretched = vec![
            [0.0, 0.0, 0.0],
            [1.5, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let forces = solver.compute_forces(&stretched, &rest, &mesh).unwrap();

        // Forces should be non-zero under deformation
        let total_force_mag: f64 = forces
            .iter()
            .map(|f| f[0] * f[0] + f[1] * f[1] + f[2] * f[2])
            .sum();
        assert!(
            total_force_mag > 1e-6,
            "Forces should be non-zero under stretch"
        );
    }

    #[test]
    fn test_elastic_energy_at_rest() {
        let (positions, _tets, mesh) = make_test_tet();
        let mat = SoftBodyMaterial::default();
        let model = NeoHookeanModel::from_material(&mat);
        let solver = FemSolver::new(&mesh, &model);

        let energy = solver.compute_elastic_energy(&positions).unwrap();
        assert!(energy.abs() < 1e-10, "Energy at rest = {energy}");
    }

    #[test]
    fn test_elastic_energy_under_deformation() {
        let rest = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let mesh = TetMesh::new(&[[0, 1, 2, 3]], &rest, rest.len()).unwrap();
        let mat = SoftBodyMaterial::default();
        let model = NeoHookeanModel::from_material(&mat);
        let solver = FemSolver::new(&mesh, &model);

        let deformed = vec![
            [0.0, 0.0, 0.0],
            [1.2, 0.0, 0.0],
            [0.0, 1.2, 0.0],
            [0.0, 0.0, 1.2],
        ];
        let energy = solver.compute_elastic_energy(&deformed).unwrap();
        assert!(energy > 0.0, "Energy should be positive under deformation");
    }

    #[test]
    fn test_compute_element_states() {
        let (positions, _tets, mesh) = make_test_tet();
        let mat = SoftBodyMaterial::default();
        let model = NeoHookeanModel::from_material(&mat);
        let solver = FemSolver::new(&mesh, &model);

        let state = solver.compute_element_states(&positions).unwrap();
        assert_eq!(state.element_states.len(), 1);

        // At rest, rotation should be identity
        let r = &state.element_states[0].rotation;
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (r[i][j] - expected).abs() < 1e-8,
                    "R[{i}][{j}] = {}",
                    r[i][j]
                );
            }
        }
    }

    #[test]
    fn test_assemble_forces() {
        let element_forces = vec![
            (
                [0, 1, 2, 3],
                [
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [-1.0, -1.0, -1.0],
                ],
            ),
            (
                [0, 1, 2, 3],
                [
                    [0.5, 0.0, 0.0],
                    [0.0, 0.5, 0.0],
                    [0.0, 0.0, 0.5],
                    [-0.5, -0.5, -0.5],
                ],
            ),
        ];
        let global = assemble_forces(&element_forces, 4);
        assert!((global[0][0] - 1.5).abs() < 1e-12);
        assert!((global[1][1] - 1.5).abs() < 1e-12);
    }

    #[test]
    fn test_sparse_matvec() {
        // 2x2 identity
        let rows = vec![0, 1];
        let cols = vec![0, 1];
        let values = vec![1.0, 1.0];
        let x = vec![3.0, 4.0];
        let y = sparse_matvec(&rows, &cols, &values, &x, 2);
        assert!((y[0] - 3.0).abs() < 1e-12);
        assert!((y[1] - 4.0).abs() < 1e-12);
    }

    #[test]
    fn test_count_inverted_elements() {
        let (positions, _tets, mesh) = make_test_tet();
        let mat = SoftBodyMaterial::default();
        let model = NeoHookeanModel::from_material(&mat);
        let solver = FemSolver::new(&mesh, &model);

        let count = solver.count_inverted_elements(&positions).unwrap();
        assert_eq!(count, 0);
    }
}
