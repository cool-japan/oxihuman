// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Advanced cloth simulation v2 using Position Based Dynamics (PBD/XPBD).
//!
//! This module implements a full cloth simulation pipeline:
//! - Symplectic Euler time integration
//! - XPBD-style distance constraints with compliance
//! - Dihedral bend constraints for realistic folding
//! - Area conservation constraints
//! - Cloth-body collision response via signed distance fields

pub mod collision_response;
pub mod constraints;
pub mod integrator;
pub mod solver;

pub use collision_response::{
    resolve_cloth_body_collisions, resolve_cloth_self_collisions, CollisionBody, CollisionConfig,
};
pub use constraints::{
    AreaConservationConstraint, DihedralBendConstraint, DistanceConstraint,
};
pub use integrator::{apply_damping, integrate_positions, integrate_velocities};
pub use solver::{ClothConfigV2, ClothMeshV2, PbdSolver, SimulationStats};

/// Run a full simulation step: integrate, solve constraints, handle collisions.
///
/// This is the primary entry point for advancing the cloth simulation by one
/// time step. It performs:
/// 1. External force integration (gravity, wind, etc.)
/// 2. Position prediction via symplectic Euler
/// 3. Constraint projection (distance, bend, area) via Gauss-Seidel PBD
/// 4. Collision response against body proxies
/// 5. Velocity update from position deltas
///
/// # Errors
///
/// Returns an error if the solver encounters invalid mesh topology or
/// degenerate constraint configurations.
pub fn step_cloth_v2(
    mesh: &mut ClothMeshV2,
    config: &ClothConfigV2,
    solver: &mut PbdSolver,
    bodies: &[CollisionBody],
    collision_config: &CollisionConfig,
) -> anyhow::Result<SimulationStats> {
    solver.step(mesh, config, bodies, collision_config)
}

/// Build a rectangular cloth mesh for testing or basic usage.
///
/// Creates a grid of `cols x rows` vertices spanning `[0, width] x [0, height]`
/// in the XZ plane at y=0, with triangulation and all constraints initialized.
///
/// # Errors
///
/// Returns an error if `cols < 2` or `rows < 2`.
pub fn build_rectangular_cloth(
    cols: usize,
    rows: usize,
    width: f64,
    height: f64,
) -> anyhow::Result<(ClothMeshV2, PbdSolver)> {
    if cols < 2 || rows < 2 {
        anyhow::bail!("Cloth grid requires at least 2 columns and 2 rows");
    }

    let mut vertices = Vec::with_capacity(cols * rows);
    let mut velocities = Vec::with_capacity(cols * rows);
    let mut inv_masses = Vec::with_capacity(cols * rows);

    let dx = width / (cols - 1) as f64;
    let dz = height / (rows - 1) as f64;

    for r in 0..rows {
        for c in 0..cols {
            vertices.push([c as f64 * dx, 0.0, r as f64 * dz]);
            velocities.push([0.0, 0.0, 0.0]);
            inv_masses.push(1.0);
        }
    }

    let mut triangles = Vec::with_capacity((cols - 1) * (rows - 1) * 2);
    for r in 0..(rows - 1) {
        for c in 0..(cols - 1) {
            let i00 = r * cols + c;
            let i10 = r * cols + c + 1;
            let i01 = (r + 1) * cols + c;
            let i11 = (r + 1) * cols + c + 1;
            triangles.push([i00, i10, i01]);
            triangles.push([i10, i11, i01]);
        }
    }

    let mesh = ClothMeshV2::new(vertices, velocities, inv_masses, triangles);
    let solver = PbdSolver::from_mesh(&mesh)?;

    Ok((mesh, solver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rectangular_cloth() {
        let result = build_rectangular_cloth(5, 5, 1.0, 1.0);
        assert!(result.is_ok());
        let (mesh, solver) = result.expect("should succeed");
        assert_eq!(mesh.vertices().len(), 25);
        assert_eq!(mesh.triangles().len(), 32);
        assert!(!solver.distance_constraints().is_empty());
    }

    #[test]
    fn test_build_rectangular_cloth_too_small() {
        let result = build_rectangular_cloth(1, 5, 1.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_step_cloth_v2() {
        let (mut mesh, mut solver) = build_rectangular_cloth(4, 4, 1.0, 1.0).expect("should succeed");
        let config = ClothConfigV2::default();
        let collision_config = CollisionConfig::default();

        let result = step_cloth_v2(&mut mesh, &config, &mut solver, &[], &collision_config);
        assert!(result.is_ok());
        let stats = result.expect("should succeed");
        assert!(stats.constraint_iterations > 0);
    }

    #[test]
    fn test_gravity_pulls_down() {
        let (mut mesh, mut solver) = build_rectangular_cloth(3, 3, 1.0, 1.0).expect("should succeed");
        let config = ClothConfigV2 {
            gravity: [0.0, -9.81, 0.0],
            dt: 0.01,
            iterations: 5,
            ..ClothConfigV2::default()
        };
        let collision_config = CollisionConfig::default();

        // Pin corner so mesh doesn't just fall uniformly
        mesh.set_inv_mass(0, 0.0);

        let initial_y = mesh.vertices()[4][1];
        step_cloth_v2(&mut mesh, &config, &mut solver, &[], &collision_config).expect("should succeed");
        let final_y = mesh.vertices()[4][1];
        assert!(final_y < initial_y, "Gravity should pull vertices down");
    }
}
