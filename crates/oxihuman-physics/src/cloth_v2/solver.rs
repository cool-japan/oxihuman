// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! PBD (Position Based Dynamics) solver with Gauss-Seidel iteration.
//!
//! Implements the full XPBD simulation loop:
//! 1. Symplectic Euler velocity integration (external forces)
//! 2. Position prediction
//! 3. Gauss-Seidel constraint projection (distance, bend, area)
//! 4. Collision response
//! 5. Velocity derivation from position changes

use super::collision_response::{
    resolve_cloth_body_collisions, resolve_cloth_self_collisions, CollisionBody, CollisionConfig,
};
use super::constraints::{
    build_area_constraints, build_bend_constraints, build_distance_constraints,
    AreaConservationConstraint, DihedralBendConstraint, DistanceConstraint,
};
use super::integrator::{
    apply_damping, integrate_positions, integrate_velocities,
    update_velocities_from_positions,
};

/// Configuration for the cloth simulation.
#[derive(Debug, Clone)]
pub struct ClothConfigV2 {
    /// Time step size in seconds.
    pub dt: f64,
    /// Gravity acceleration vector.
    pub gravity: [f64; 3],
    /// Number of constraint solver iterations per step.
    pub iterations: usize,
    /// Compliance for distance (stretch) constraints. 0 = infinitely stiff.
    pub stretch_compliance: f64,
    /// Compliance for dihedral bend constraints.
    pub bend_compliance: f64,
    /// Compliance for area conservation constraints.
    pub area_compliance: f64,
    /// Velocity damping factor (0..1). 0 = no damping.
    pub damping: f64,
    /// Number of substeps for more stable simulation.
    pub substeps: usize,
    /// Whether to solve bend constraints.
    pub enable_bend: bool,
    /// Whether to solve area conservation constraints.
    pub enable_area_conservation: bool,
}

impl Default for ClothConfigV2 {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            gravity: [0.0, -9.81, 0.0],
            iterations: 10,
            stretch_compliance: 0.0,
            bend_compliance: 0.001,
            area_compliance: 0.0001,
            damping: 0.01,
            substeps: 1,
            enable_bend: true,
            enable_area_conservation: true,
        }
    }
}

/// The cloth mesh data structure.
///
/// Stores vertex positions, velocities, inverse masses, and triangle topology.
#[derive(Debug, Clone)]
pub struct ClothMeshV2 {
    /// Vertex positions in 3D space.
    vertices: Vec<[f64; 3]>,
    /// Vertex velocities.
    velocities: Vec<[f64; 3]>,
    /// Inverse masses (0 = pinned/fixed vertex).
    inv_masses: Vec<f64>,
    /// Triangle indices (each triangle is 3 vertex indices).
    triangles: Vec<[usize; 3]>,
}

impl ClothMeshV2 {
    /// Create a new cloth mesh.
    pub fn new(
        vertices: Vec<[f64; 3]>,
        velocities: Vec<[f64; 3]>,
        inv_masses: Vec<f64>,
        triangles: Vec<[usize; 3]>,
    ) -> Self {
        Self {
            vertices,
            velocities,
            inv_masses,
            triangles,
        }
    }

    /// Get vertex positions.
    pub fn vertices(&self) -> &[[f64; 3]] {
        &self.vertices
    }

    /// Get mutable vertex positions.
    pub fn vertices_mut(&mut self) -> &mut Vec<[f64; 3]> {
        &mut self.vertices
    }

    /// Get vertex velocities.
    pub fn velocities(&self) -> &[[f64; 3]] {
        &self.velocities
    }

    /// Get mutable vertex velocities.
    pub fn velocities_mut(&mut self) -> &mut Vec<[f64; 3]> {
        &mut self.velocities
    }

    /// Get inverse masses.
    pub fn inv_masses(&self) -> &[f64] {
        &self.inv_masses
    }

    /// Get triangle indices.
    pub fn triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }

    /// Set the inverse mass for a specific vertex.
    /// Use 0.0 to pin/fix a vertex.
    pub fn set_inv_mass(&mut self, index: usize, inv_mass: f64) {
        if let Some(m) = self.inv_masses.get_mut(index) {
            *m = inv_mass;
        }
    }

    /// Pin a vertex (set inverse mass to 0).
    pub fn pin_vertex(&mut self, index: usize) {
        self.set_inv_mass(index, 0.0);
    }

    /// Unpin a vertex with a given mass.
    pub fn unpin_vertex(&mut self, index: usize, mass: f64) {
        if mass > 1e-30 {
            self.set_inv_mass(index, 1.0 / mass);
        }
    }

    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Apply an impulse to a vertex.
    pub fn apply_impulse(&mut self, index: usize, impulse: &[f64; 3]) {
        if let (Some(v), Some(&w)) = (self.velocities.get_mut(index), self.inv_masses.get(index)) {
            if w > 0.0 {
                v[0] += impulse[0] * w;
                v[1] += impulse[1] * w;
                v[2] += impulse[2] * w;
            }
        }
    }

    /// Compute the axis-aligned bounding box of the cloth.
    pub fn aabb(&self) -> Option<([f64; 3], [f64; 3])> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut min = self.vertices[0];
        let mut max = self.vertices[0];
        for v in &self.vertices[1..] {
            for k in 0..3 {
                if v[k] < min[k] {
                    min[k] = v[k];
                }
                if v[k] > max[k] {
                    max[k] = v[k];
                }
            }
        }
        Some((min, max))
    }
}

/// Statistics from a simulation step.
#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    /// Number of constraint iterations performed.
    pub constraint_iterations: usize,
    /// Maximum constraint error after solving.
    pub max_constraint_error: f64,
    /// Number of body collisions resolved.
    pub body_collisions: usize,
    /// Number of self-collisions resolved.
    pub self_collisions: usize,
    /// Total number of distance constraints.
    pub distance_constraint_count: usize,
    /// Total number of bend constraints.
    pub bend_constraint_count: usize,
    /// Total number of area constraints.
    pub area_constraint_count: usize,
}

/// The PBD solver holding all constraints and scratch buffers.
#[derive(Debug, Clone)]
pub struct PbdSolver {
    /// Distance (stretch/shear) constraints.
    distance_constraints: Vec<DistanceConstraint>,
    /// Dihedral bend constraints.
    bend_constraints: Vec<DihedralBendConstraint>,
    /// Area conservation constraints.
    area_constraints: Vec<AreaConservationConstraint>,
    /// Scratch buffer for previous positions.
    prev_positions: Vec<[f64; 3]>,
}

impl PbdSolver {
    /// Create a new empty solver.
    pub fn new() -> Self {
        Self {
            distance_constraints: Vec::new(),
            bend_constraints: Vec::new(),
            area_constraints: Vec::new(),
            prev_positions: Vec::new(),
        }
    }

    /// Build all constraints from a cloth mesh.
    ///
    /// # Errors
    ///
    /// Returns an error if the mesh has no triangles.
    pub fn from_mesh(mesh: &ClothMeshV2) -> anyhow::Result<Self> {
        if mesh.triangles.is_empty() {
            anyhow::bail!("Cannot build solver from mesh with no triangles");
        }

        let distance_constraints =
            build_distance_constraints(&mesh.triangles, &mesh.vertices);
        let bend_constraints =
            build_bend_constraints(&mesh.triangles, &mesh.vertices);
        let area_constraints =
            build_area_constraints(&mesh.triangles, &mesh.vertices);

        Ok(Self {
            distance_constraints,
            bend_constraints,
            area_constraints,
            prev_positions: Vec::with_capacity(mesh.vertices.len()),
        })
    }

    /// Get distance constraints (read-only).
    pub fn distance_constraints(&self) -> &[DistanceConstraint] {
        &self.distance_constraints
    }

    /// Get bend constraints (read-only).
    pub fn bend_constraints(&self) -> &[DihedralBendConstraint] {
        &self.bend_constraints
    }

    /// Get area constraints (read-only).
    pub fn area_constraints(&self) -> &[AreaConservationConstraint] {
        &self.area_constraints
    }

    /// Add a custom distance constraint.
    pub fn add_distance_constraint(&mut self, constraint: DistanceConstraint) {
        self.distance_constraints.push(constraint);
    }

    /// Add a custom bend constraint.
    pub fn add_bend_constraint(&mut self, constraint: DihedralBendConstraint) {
        self.bend_constraints.push(constraint);
    }

    /// Add a custom area constraint.
    pub fn add_area_constraint(&mut self, constraint: AreaConservationConstraint) {
        self.area_constraints.push(constraint);
    }

    /// Perform one full simulation step.
    ///
    /// # Errors
    ///
    /// Returns an error if the mesh data is inconsistent.
    pub fn step(
        &mut self,
        mesh: &mut ClothMeshV2,
        config: &ClothConfigV2,
        bodies: &[CollisionBody],
        collision_config: &CollisionConfig,
    ) -> anyhow::Result<SimulationStats> {
        let substeps = config.substeps.max(1);
        let sub_dt = config.dt / substeps as f64;

        let mut stats = SimulationStats {
            distance_constraint_count: self.distance_constraints.len(),
            bend_constraint_count: self.bend_constraints.len(),
            area_constraint_count: self.area_constraints.len(),
            ..SimulationStats::default()
        };

        for _ in 0..substeps {
            let sub_stats = self.substep(mesh, config, sub_dt, bodies, collision_config)?;
            stats.constraint_iterations += sub_stats.constraint_iterations;
            stats.body_collisions += sub_stats.body_collisions;
            stats.self_collisions += sub_stats.self_collisions;
            if sub_stats.max_constraint_error > stats.max_constraint_error {
                stats.max_constraint_error = sub_stats.max_constraint_error;
            }
        }

        Ok(stats)
    }

    /// Perform a single substep.
    fn substep(
        &mut self,
        mesh: &mut ClothMeshV2,
        config: &ClothConfigV2,
        dt: f64,
        bodies: &[CollisionBody],
        collision_config: &CollisionConfig,
    ) -> anyhow::Result<SimulationStats> {
        // 1. Integrate velocities (apply gravity and external forces)
        integrate_velocities(
            &mut mesh.velocities,
            &mesh.inv_masses,
            &config.gravity,
            None,
            dt,
        );

        // 2. Apply damping
        apply_damping(&mut mesh.velocities, &mesh.inv_masses, config.damping);

        // 3. Predict positions (symplectic Euler)
        integrate_positions(
            &mut mesh.vertices,
            &mesh.velocities,
            &mut self.prev_positions,
            &mesh.inv_masses,
            dt,
        );

        // 4. Reset Lagrange multipliers
        for c in &mut self.distance_constraints {
            c.reset_lambda();
        }
        for c in &mut self.bend_constraints {
            c.reset_lambda();
        }
        for c in &mut self.area_constraints {
            c.reset_lambda();
        }

        // 5. Gauss-Seidel constraint projection
        let mut max_error = 0.0_f64;
        let iterations = config.iterations.max(1);

        for _ in 0..iterations {
            let mut iter_max_error = 0.0_f64;

            // Distance constraints
            for c in &mut self.distance_constraints {
                let err = c.project(
                    &mut mesh.vertices,
                    &mesh.inv_masses,
                    config.stretch_compliance,
                    dt,
                );
                if err > iter_max_error {
                    iter_max_error = err;
                }
            }

            // Bend constraints
            if config.enable_bend {
                for c in &mut self.bend_constraints {
                    let err = c.project(
                        &mut mesh.vertices,
                        &mesh.inv_masses,
                        config.bend_compliance,
                        dt,
                    );
                    if err > iter_max_error {
                        iter_max_error = err;
                    }
                }
            }

            // Area conservation constraints
            if config.enable_area_conservation {
                for c in &mut self.area_constraints {
                    let err = c.project(
                        &mut mesh.vertices,
                        &mesh.inv_masses,
                        config.area_compliance,
                        dt,
                    );
                    if err > iter_max_error {
                        iter_max_error = err;
                    }
                }
            }

            max_error = iter_max_error;
        }

        // 6. Collision response
        let body_collisions = resolve_cloth_body_collisions(
            &mut mesh.vertices,
            &mut mesh.velocities,
            &mesh.inv_masses,
            bodies,
            collision_config,
            dt,
        );

        let self_collisions = resolve_cloth_self_collisions(
            &mut mesh.vertices,
            &mut mesh.velocities,
            &mesh.inv_masses,
            &mesh.triangles,
            collision_config,
        );

        // 7. Update velocities from position changes
        update_velocities_from_positions(
            &mesh.vertices,
            &self.prev_positions,
            &mut mesh.velocities,
            &mesh.inv_masses,
            dt,
        );

        Ok(SimulationStats {
            constraint_iterations: iterations,
            max_constraint_error: max_error,
            body_collisions,
            self_collisions,
            distance_constraint_count: self.distance_constraints.len(),
            bend_constraint_count: self.bend_constraints.len(),
            area_constraint_count: self.area_constraints.len(),
        })
    }
}

impl Default for PbdSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_mesh() -> ClothMeshV2 {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
        ];
        let velocities = vec![[0.0; 3]; 4];
        let inv_masses = vec![1.0; 4];
        let triangles = vec![[0, 1, 2], [1, 3, 2]];
        ClothMeshV2::new(vertices, velocities, inv_masses, triangles)
    }

    #[test]
    fn test_solver_from_mesh() {
        let mesh = make_simple_mesh();
        let solver = PbdSolver::from_mesh(&mesh).expect("should succeed");
        assert_eq!(solver.distance_constraints().len(), 5);
        assert_eq!(solver.bend_constraints().len(), 1);
        assert_eq!(solver.area_constraints().len(), 2);
    }

    #[test]
    fn test_solver_empty_mesh() {
        let mesh = ClothMeshV2::new(vec![], vec![], vec![], vec![]);
        let result = PbdSolver::from_mesh(&mesh);
        assert!(result.is_err());
    }

    #[test]
    fn test_step_returns_stats() {
        let mut mesh = make_simple_mesh();
        let mut solver = PbdSolver::from_mesh(&mesh).expect("should succeed");
        let config = ClothConfigV2::default();
        let collision_config = CollisionConfig::default();

        let stats = solver.step(&mut mesh, &config, &[], &collision_config).expect("should succeed");
        assert_eq!(stats.constraint_iterations, config.iterations);
        assert_eq!(stats.distance_constraint_count, 5);
    }

    #[test]
    fn test_pinned_vertex_stays() {
        let mut mesh = make_simple_mesh();
        mesh.pin_vertex(0);
        let original_pos = mesh.vertices()[0];

        let mut solver = PbdSolver::from_mesh(&mesh).expect("should succeed");
        let config = ClothConfigV2 {
            dt: 0.01,
            gravity: [0.0, -9.81, 0.0],
            iterations: 5,
            ..ClothConfigV2::default()
        };
        let collision_config = CollisionConfig::default();

        solver.step(&mut mesh, &config, &[], &collision_config).expect("should succeed");

        assert_eq!(mesh.vertices()[0], original_pos, "Pinned vertex should not move");
    }

    #[test]
    fn test_cloth_with_ground_plane() {
        let mut mesh = make_simple_mesh();
        // Start cloth above ground, let it fall
        for v in mesh.vertices_mut().iter_mut() {
            v[1] = 2.0;
        }
        let mut solver = PbdSolver::from_mesh(&mesh).expect("should succeed");

        let config = ClothConfigV2 {
            dt: 0.01,
            iterations: 5,
            ..ClothConfigV2::default()
        };
        let bodies = vec![CollisionBody::Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 0.0,
        }];
        let collision_config = CollisionConfig {
            margin: 0.01,
            ..CollisionConfig::default()
        };

        // Run many steps
        for _ in 0..200 {
            solver.step(&mut mesh, &config, &bodies, &collision_config).expect("should succeed");
        }

        // All vertices should be at or above ground
        for (i, v) in mesh.vertices().iter().enumerate() {
            assert!(
                v[1] >= -0.05,
                "Vertex {} at y={} should be above ground",
                i,
                v[1]
            );
        }
    }

    #[test]
    fn test_substeps() {
        let mut mesh = make_simple_mesh();
        let mut solver = PbdSolver::from_mesh(&mesh).expect("should succeed");
        let config = ClothConfigV2 {
            substeps: 4,
            iterations: 3,
            ..ClothConfigV2::default()
        };
        let collision_config = CollisionConfig::default();

        let stats = solver.step(&mut mesh, &config, &[], &collision_config).expect("should succeed");
        // 4 substeps * 3 iterations each
        assert_eq!(stats.constraint_iterations, 12);
    }

    #[test]
    fn test_mesh_aabb() {
        let mesh = make_simple_mesh();
        let aabb = mesh.aabb();
        assert!(aabb.is_some());
        let (min, max) = aabb.expect("should succeed");
        assert_eq!(min, [0.0, 0.0, 0.0]);
        assert_eq!(max, [1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_apply_impulse() {
        let mut mesh = make_simple_mesh();
        mesh.apply_impulse(0, &[10.0, 0.0, 0.0]);
        assert!((mesh.velocities()[0][0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_impulse_pinned() {
        let mut mesh = make_simple_mesh();
        mesh.pin_vertex(0);
        mesh.apply_impulse(0, &[10.0, 0.0, 0.0]);
        assert_eq!(mesh.velocities()[0], [0.0, 0.0, 0.0]);
    }
}
