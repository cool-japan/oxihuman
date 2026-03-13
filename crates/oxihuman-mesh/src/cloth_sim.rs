// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::{compute_normals, MeshBuffers};

// ---------------------------------------------------------------------------
// Parameter structs
// ---------------------------------------------------------------------------

pub struct ClothSimParams {
    pub gravity: [f32; 3],
    pub wind: [f32; 3],
    pub damping: f32,
    pub substeps: usize,
    pub constraint_iterations: usize,
    pub dt: f32,
}

impl Default for ClothSimParams {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.8, 0.0],
            wind: [0.0, 0.0, 0.0],
            damping: 0.98,
            substeps: 5,
            constraint_iterations: 3,
            dt: 0.016,
        }
    }
}

// ---------------------------------------------------------------------------
// Core structs
// ---------------------------------------------------------------------------

pub struct ClothParticle {
    pub position: [f32; 3],
    pub prev_position: [f32; 3],
    pub inv_mass: f32,
}

pub struct DistanceConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

pub struct ClothSim {
    pub particles: Vec<ClothParticle>,
    pub constraints: Vec<DistanceConstraint>,
    pub params: ClothSimParams,
}

pub struct ClothSimResult {
    pub final_positions: Vec<[f32; 3]>,
    pub frames: Vec<Vec<[f32; 3]>>,
    pub total_kinetic_energy: f32,
}

// ---------------------------------------------------------------------------
// Helper: 3-vector arithmetic
// ---------------------------------------------------------------------------

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_len(a: [f32; 3]) -> f32 {
    vec3_dot(a, a).sqrt()
}

// ---------------------------------------------------------------------------
// ClothSim implementation
// ---------------------------------------------------------------------------

impl ClothSim {
    pub fn new(params: ClothSimParams) -> Self {
        Self {
            particles: Vec::new(),
            constraints: Vec::new(),
            params,
        }
    }

    /// Add a particle and return its index.
    pub fn add_particle(&mut self, pos: [f32; 3], inv_mass: f32) -> usize {
        let idx = self.particles.len();
        self.particles.push(ClothParticle {
            position: pos,
            prev_position: pos,
            inv_mass,
        });
        idx
    }

    /// Pin a particle (set inv_mass to 0.0).
    pub fn pin(&mut self, idx: usize) {
        if let Some(p) = self.particles.get_mut(idx) {
            p.inv_mass = 0.0;
        }
    }

    /// Add a distance constraint; rest_length is computed from current positions.
    pub fn add_constraint(&mut self, a: usize, b: usize, stiffness: f32) {
        let pa = self.particles[a].position;
        let pb = self.particles[b].position;
        let rest_length = vec3_len(vec3_sub(pb, pa));
        self.constraints.push(DistanceConstraint {
            a,
            b,
            rest_length,
            stiffness,
        });
    }

    /// Advance simulation by one time step using Position-Based Dynamics.
    pub fn simulate(&mut self) {
        let dt = self.params.dt;
        let substeps = self.params.substeps;
        let constraint_iterations = self.params.constraint_iterations;
        let damping = self.params.damping;
        let gravity = self.params.gravity;
        let wind = self.params.wind;

        let sub_dt = dt / substeps as f32;
        let sub_dt2 = sub_dt * sub_dt;

        // gravity + wind combined acceleration scaled by sub_dt^2
        let gw = [
            (gravity[0] + wind[0]) * sub_dt2,
            (gravity[1] + wind[1]) * sub_dt2,
            (gravity[2] + wind[2]) * sub_dt2,
        ];

        for _ in 0..substeps {
            // --- integrate ---
            for p in self.particles.iter_mut() {
                if p.inv_mass < 1e-10 {
                    continue;
                }
                let velocity = vec3_scale(vec3_sub(p.position, p.prev_position), damping);
                let new_pos = vec3_add(vec3_add(p.position, velocity), gw);
                p.prev_position = p.position;
                p.position = new_pos;
            }

            // --- solve constraints ---
            for _ in 0..constraint_iterations {
                // Collect constraint data to avoid borrow conflicts
                let constraints: Vec<(usize, usize, f32, f32)> = self
                    .constraints
                    .iter()
                    .map(|c| (c.a, c.b, c.rest_length, c.stiffness))
                    .collect();

                for (a, b, rest, stiffness) in constraints {
                    let pa = self.particles[a].position;
                    let pb = self.particles[b].position;
                    let ima = self.particles[a].inv_mass;
                    let imb = self.particles[b].inv_mass;

                    let delta = vec3_sub(pb, pa);
                    let dist = vec3_len(delta);
                    if dist < 1e-10 {
                        continue;
                    }
                    let total_inv = ima + imb;
                    if total_inv < 1e-10 {
                        continue;
                    }

                    // correction vector
                    let corr = vec3_scale(delta, (dist - rest) * stiffness / dist);

                    self.particles[a].position = vec3_add(pa, vec3_scale(corr, ima / total_inv));
                    self.particles[b].position = vec3_sub(pb, vec3_scale(corr, imb / total_inv));
                }
            }
        }
    }

    /// Return a snapshot of all particle positions.
    pub fn positions(&self) -> Vec<[f32; 3]> {
        self.particles.iter().map(|p| p.position).collect()
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Build a rows×cols cloth grid with structural and shear constraints.
pub fn build_cloth_grid(
    rows: usize,
    cols: usize,
    spacing: f32,
    params: ClothSimParams,
) -> ClothSim {
    let mut sim = ClothSim::new(params);

    // Create particles: row = y-axis, col = x-axis, z = 0
    for r in 0..rows {
        for c in 0..cols {
            let pos = [c as f32 * spacing, r as f32 * spacing, 0.0];
            sim.add_particle(pos, 1.0);
        }
    }

    // Pin top row (last row = highest y)
    let top_row = rows - 1;
    for c in 0..cols {
        let idx = top_row * cols + c;
        sim.pin(idx);
    }

    let stiffness = 1.0_f32;

    // Structural constraints: horizontal (same row) and vertical (same col)
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            // horizontal
            if c + 1 < cols {
                sim.add_constraint(idx, idx + 1, stiffness);
            }
            // vertical
            if r + 1 < rows {
                sim.add_constraint(idx, idx + cols, stiffness);
            }
        }
    }

    // Shear constraints: diagonals
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            // diagonal: down-right
            if r + 1 < rows && c + 1 < cols {
                sim.add_constraint(idx, idx + cols + 1, stiffness);
            }
            // diagonal: down-left
            if r + 1 < rows && c > 0 {
                sim.add_constraint(idx, idx + cols - 1, stiffness);
            }
        }
    }

    sim
}

/// Run the simulation for n steps, capturing a position snapshot each step.
pub fn simulate_n_steps(sim: &mut ClothSim, n: usize) -> ClothSimResult {
    let mut frames: Vec<Vec<[f32; 3]>> = Vec::with_capacity(n);

    for _ in 0..n {
        sim.simulate();
        frames.push(sim.positions());
    }

    let final_positions = sim.positions();

    // Kinetic energy: 0.5 * |pos - prev_pos|^2 for non-pinned particles
    let total_kinetic_energy: f32 = sim
        .particles
        .iter()
        .filter(|p| p.inv_mass > 1e-10)
        .map(|p| {
            let v = vec3_sub(p.position, p.prev_position);
            0.5 * vec3_dot(v, v)
        })
        .sum();

    ClothSimResult {
        final_positions,
        frames,
        total_kinetic_energy,
    }
}

/// Copy simulation particle positions into a mesh and recompute normals.
pub fn apply_sim_to_mesh(mesh: &mut MeshBuffers, sim: &ClothSim) -> MeshBuffers {
    let n = mesh.positions.len().min(sim.particles.len());
    for i in 0..n {
        mesh.positions[i] = sim.particles[i].position;
    }
    compute_normals(mesh);
    mesh.clone()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> ClothSimParams {
        ClothSimParams::default()
    }

    fn simple_mesh(n_verts: usize) -> MeshBuffers {
        use oxihuman_morph::engine::MeshBuffers as MB;
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0f32; 3]; n_verts],
            normals: vec![[0.0, 0.0, 1.0]; n_verts],
            uvs: vec![[0.0, 0.0]; n_verts],
            indices: (0..n_verts as u32).collect(),
            has_suit: false,
        })
    }

    #[test]
    fn test_cloth_sim_params_default() {
        let p = ClothSimParams::default();
        assert!((p.gravity[1] - (-9.8)).abs() < 1e-5);
        assert_eq!(p.substeps, 5);
        assert_eq!(p.constraint_iterations, 3);
        assert!((p.damping - 0.98).abs() < 1e-5);
        assert!((p.dt - 0.016).abs() < 1e-5);
        assert_eq!(p.wind, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_add_particle_returns_index() {
        let mut sim = ClothSim::new(default_params());
        let i0 = sim.add_particle([0.0, 0.0, 0.0], 1.0);
        let i1 = sim.add_particle([1.0, 0.0, 0.0], 1.0);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(sim.particles.len(), 2);
    }

    #[test]
    fn test_pin_particle_zeroes_inv_mass() {
        let mut sim = ClothSim::new(default_params());
        let idx = sim.add_particle([0.0, 0.0, 0.0], 1.0);
        sim.pin(idx);
        assert_eq!(sim.particles[idx].inv_mass, 0.0);
    }

    #[test]
    fn test_add_constraint_sets_rest_length() {
        let mut sim = ClothSim::new(default_params());
        sim.add_particle([0.0, 0.0, 0.0], 1.0);
        sim.add_particle([1.0, 0.0, 0.0], 1.0);
        sim.add_constraint(0, 1, 1.0);
        assert!((sim.constraints[0].rest_length - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_simulate_does_not_panic() {
        let mut sim = ClothSim::new(default_params());
        sim.add_particle([0.0, 0.0, 0.0], 1.0);
        sim.add_particle([1.0, 0.0, 0.0], 1.0);
        sim.add_constraint(0, 1, 1.0);
        // Should not panic
        for _ in 0..10 {
            sim.simulate();
        }
    }

    #[test]
    fn test_pinned_particle_does_not_move() {
        let mut sim = ClothSim::new(default_params());
        let pinned = sim.add_particle([0.0, 5.0, 0.0], 0.0); // pinned via inv_mass=0
        sim.add_particle([0.0, 4.0, 0.0], 1.0);

        let initial_pos = sim.particles[pinned].position;
        for _ in 0..20 {
            sim.simulate();
        }
        let final_pos = sim.particles[pinned].position;
        assert!((final_pos[0] - initial_pos[0]).abs() < 1e-6);
        assert!((final_pos[1] - initial_pos[1]).abs() < 1e-6);
        assert!((final_pos[2] - initial_pos[2]).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_pulls_free_particle_down() {
        let params = ClothSimParams {
            damping: 1.0,
            ..Default::default()
        }; // no energy loss so we can see movement
        let mut sim = ClothSim::new(params);
        sim.add_particle([0.0, 0.0, 0.0], 1.0);

        let initial_y = sim.particles[0].position[1];
        for _ in 0..10 {
            sim.simulate();
        }
        let final_y = sim.particles[0].position[1];
        assert!(
            final_y < initial_y,
            "particle should fall: {} < {}",
            final_y,
            initial_y
        );
    }

    #[test]
    fn test_constraint_keeps_particles_connected() {
        let params = ClothSimParams {
            substeps: 10,
            constraint_iterations: 5,
            ..Default::default()
        };
        let mut sim = ClothSim::new(params);

        // Pin one particle at top, hang one below
        let top = sim.add_particle([0.0, 1.0, 0.0], 0.0);
        let bot = sim.add_particle([0.0, 0.0, 0.0], 1.0);
        sim.add_constraint(top, bot, 1.0);

        for _ in 0..30 {
            sim.simulate();
        }

        // Distance should remain close to rest_length (1.0)
        let pa = sim.particles[top].position;
        let pb = sim.particles[bot].position;
        let dist = vec3_len(vec3_sub(pb, pa));
        assert!(dist < 2.0, "constraint should limit distance, got {}", dist);
    }

    #[test]
    fn test_build_cloth_grid_particle_count() {
        let rows = 4;
        let cols = 5;
        let sim = build_cloth_grid(rows, cols, 0.1, default_params());
        assert_eq!(sim.particles.len(), rows * cols);
    }

    #[test]
    fn test_build_cloth_grid_top_row_pinned() {
        let rows = 3;
        let cols = 4;
        let sim = build_cloth_grid(rows, cols, 0.1, default_params());
        let top_row = rows - 1;
        for c in 0..cols {
            let idx = top_row * cols + c;
            assert_eq!(
                sim.particles[idx].inv_mass, 0.0,
                "top row particle {} should be pinned",
                idx
            );
        }
        // Bottom row should not be pinned
        assert!(sim.particles[0].inv_mass > 0.0);
    }

    #[test]
    fn test_simulate_n_steps_frame_count() {
        let mut sim = build_cloth_grid(3, 3, 0.1, default_params());
        let n = 7;
        let result = simulate_n_steps(&mut sim, n);
        assert_eq!(result.frames.len(), n);
        assert_eq!(result.final_positions.len(), 9);
    }

    #[test]
    fn test_simulate_n_steps_kinetic_energy_nonneg() {
        let mut sim = build_cloth_grid(3, 3, 0.1, default_params());
        let result = simulate_n_steps(&mut sim, 5);
        assert!(
            result.total_kinetic_energy >= 0.0,
            "kinetic energy must be non-negative"
        );
    }

    #[test]
    fn test_positions_count_matches_particles() {
        let mut sim = ClothSim::new(default_params());
        sim.add_particle([0.0, 0.0, 0.0], 1.0);
        sim.add_particle([1.0, 0.0, 0.0], 1.0);
        sim.add_particle([0.0, 1.0, 0.0], 0.0);
        let pos = sim.positions();
        assert_eq!(pos.len(), sim.particles.len());
    }

    #[test]
    fn test_apply_sim_to_mesh() {
        let n = 4;
        let mut mesh = simple_mesh(n);
        let mut sim = ClothSim::new(default_params());
        for i in 0..n {
            sim.add_particle([i as f32, 0.0, 0.0], 1.0);
        }
        // Run a few steps so positions change
        for _ in 0..5 {
            sim.simulate();
        }
        let result = apply_sim_to_mesh(&mut mesh, &sim);
        // Check positions were copied
        for i in 0..n {
            let mp = result.positions[i];
            let sp = sim.particles[i].position;
            assert!((mp[0] - sp[0]).abs() < 1e-6);
            assert!((mp[1] - sp[1]).abs() < 1e-6);
            assert!((mp[2] - sp[2]).abs() < 1e-6);
        }
        // Normals should be recomputed (non-trivially present)
        assert_eq!(result.normals.len(), n);
    }
}
