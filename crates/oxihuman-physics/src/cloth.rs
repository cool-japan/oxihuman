// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cloth spring-mass simulation using Verlet integration.
//!
//! Simulates cloth behavior for clothing meshes with structural, shear,
//! and bending springs so that clothing can drape realistically over a body.

use std::collections::HashSet;

// ── ClothParticle ─────────────────────────────────────────────────────────────

/// A particle in the cloth simulation.
#[derive(Debug, Clone)]
pub struct ClothParticle {
    /// Current world-space position.
    pub position: [f32; 3],
    /// Previous position (used by Verlet integration).
    pub prev_position: [f32; 3],
    /// Particle mass.
    pub mass: f32,
    /// When `true` the particle is anchored and never moves.
    pub pinned: bool,
}

impl ClothParticle {
    /// Create a new particle at `position` with the given `mass`.
    pub fn new(position: [f32; 3], mass: f32) -> Self {
        Self {
            position,
            prev_position: position,
            mass,
            pinned: false,
        }
    }

    /// Builder helper: mark this particle as pinned.
    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
}

// ── SpringKind ────────────────────────────────────────────────────────────────

/// The structural role of a spring in the cloth lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpringKind {
    /// Direct edge connections between adjacent vertices.
    Structural,
    /// Diagonal connections within a triangle face.
    Shear,
    /// Skip-one connections for resistance to out-of-plane bending.
    Bending,
}

// ── Spring ────────────────────────────────────────────────────────────────────

/// A spring connecting two particles.
#[derive(Debug, Clone)]
pub struct Spring {
    /// Index of the first particle.
    pub a: usize,
    /// Index of the second particle.
    pub b: usize,
    /// Natural (rest) length of the spring.
    pub rest_length: f32,
    /// Stiffness coefficient in `[0, 1]`; higher values are stiffer.
    pub stiffness: f32,
    /// Structural role of this spring.
    pub kind: SpringKind,
}

// ── ClothSim ──────────────────────────────────────────────────────────────────

/// Cloth simulation state.
pub struct ClothSim {
    /// All particles in the cloth.
    pub particles: Vec<ClothParticle>,
    /// All springs connecting particles.
    pub springs: Vec<Spring>,
    /// Gravitational acceleration vector (world units / s²).
    pub gravity: [f32; 3],
    /// Velocity damping factor in `[0, 1]` applied each step; e.g. `0.99`.
    pub damping: f32,
}

// ── helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Canonical (smaller, larger) index pair for deduplication.
#[inline]
fn edge_key(i: usize, j: usize) -> (usize, usize) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

// ── ClothSim impl ─────────────────────────────────────────────────────────────

impl ClothSim {
    /// Build a cloth simulation from a mesh.
    ///
    /// * `positions` — vertex positions.
    /// * `indices`   — triangle index list (every 3 indices form one triangle).
    /// * `stiffness` — spring stiffness coefficient applied to all springs.
    pub fn from_mesh(positions: &[[f32; 3]], indices: &[u32], stiffness: f32) -> Self {
        // Build particles ──────────────────────────────────────────────────────
        let particles: Vec<ClothParticle> = positions
            .iter()
            .map(|&p| ClothParticle::new(p, 1.0))
            .collect();

        let n_tris = indices.len() / 3;

        // Track which edges/springs have been added so we don't duplicate ─────
        let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
        let mut springs: Vec<Spring> = Vec::new();

        /// Add a spring if we haven't seen this edge before.
        macro_rules! add_spring {
            ($a:expr, $b:expr, $kind:expr) => {{
                let key = edge_key($a, $b);
                if edge_set.insert(key) {
                    let rest_length = dist3(positions[$a], positions[$b]);
                    if rest_length > 1e-10 {
                        springs.push(Spring {
                            a: $a,
                            b: $b,
                            rest_length,
                            stiffness,
                            kind: $kind,
                        });
                    }
                }
            }};
        }

        // ── Structural springs (triangle edges) ──────────────────────────────
        for tri in 0..n_tris {
            let i0 = indices[tri * 3] as usize;
            let i1 = indices[tri * 3 + 1] as usize;
            let i2 = indices[tri * 3 + 2] as usize;
            add_spring!(i0, i1, SpringKind::Structural);
            add_spring!(i1, i2, SpringKind::Structural);
            add_spring!(i2, i0, SpringKind::Structural);
        }

        // ── Bending springs (opposite-vertex pairs sharing an edge) ──────────
        // Build edge → list of triangles that contain the edge.
        use std::collections::HashMap;
        let mut edge_tris: HashMap<(usize, usize), Vec<[usize; 3]>> = HashMap::new();
        for tri in 0..n_tris {
            let i0 = indices[tri * 3] as usize;
            let i1 = indices[tri * 3 + 1] as usize;
            let i2 = indices[tri * 3 + 2] as usize;
            let face = [i0, i1, i2];
            for &(ea, eb) in &[(i0, i1), (i1, i2), (i2, i0)] {
                edge_tris.entry(edge_key(ea, eb)).or_default().push(face);
            }
        }

        for tris_for_edge in edge_tris.values() {
            if tris_for_edge.len() == 2 {
                let t0 = tris_for_edge[0];
                let t1 = tris_for_edge[1];
                // Find the vertex in t0 not shared with t1, and vice-versa.
                let opp0 = t0.iter().find(|&&v| !t1.contains(&v)).copied();
                let opp1 = t1.iter().find(|&&v| !t0.contains(&v)).copied();
                if let (Some(a), Some(b)) = (opp0, opp1) {
                    add_spring!(a, b, SpringKind::Bending);
                }
            }
        }

        Self {
            particles,
            springs,
            gravity: [0.0, -9.81, 0.0],
            damping: 0.99,
        }
    }

    /// Pin all particles whose Y coordinate is at or above `y_threshold`.
    pub fn pin_by_y(mut self, y_threshold: f32) -> Self {
        for p in &mut self.particles {
            if p.position[1] >= y_threshold {
                p.pinned = true;
            }
        }
        self
    }

    /// Advance the simulation by `dt` seconds with `sub_steps` Verlet sub-iterations.
    pub fn step(&mut self, dt: f32, sub_steps: usize) {
        let sub_dt = dt / sub_steps.max(1) as f32;

        for _ in 0..sub_steps {
            // ── Verlet integration ────────────────────────────────────────────
            for p in &mut self.particles {
                if p.pinned {
                    continue;
                }
                let vx = p.position[0] - p.prev_position[0];
                let vy = p.position[1] - p.prev_position[1];
                let vz = p.position[2] - p.prev_position[2];

                let new_x = p.position[0] + vx * self.damping + self.gravity[0] * sub_dt * sub_dt;
                let new_y = p.position[1] + vy * self.damping + self.gravity[1] * sub_dt * sub_dt;
                let new_z = p.position[2] + vz * self.damping + self.gravity[2] * sub_dt * sub_dt;

                p.prev_position = p.position;
                p.position = [new_x, new_y, new_z];
            }

            // ── Constraints ───────────────────────────────────────────────────
            self.satisfy_springs();
            self.apply_floor_constraint();
        }
    }

    /// Return the current position of every particle.
    pub fn positions(&self) -> Vec<[f32; 3]> {
        self.particles.iter().map(|p| p.position).collect()
    }

    /// Prevent particles from falling below `y = 0`.
    fn apply_floor_constraint(&mut self) {
        for p in &mut self.particles {
            if p.position[1] < 0.0 {
                p.position[1] = 0.0;
            }
        }
    }

    /// One pass of Position-Based Dynamics spring constraint satisfaction.
    fn satisfy_springs(&mut self) {
        // We need split borrows; copy spring data to avoid borrow conflicts.
        let springs: Vec<Spring> = self.springs.clone();

        for s in &springs {
            let pa = self.particles[s.a].position;
            let pb = self.particles[s.b].position;

            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dz = pb[2] - pa[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist < 1e-10 {
                continue;
            }

            let factor = (1.0 - s.rest_length / dist) * s.stiffness;
            let cx = dx * factor;
            let cy = dy * factor;
            let cz = dz * factor;

            if !self.particles[s.a].pinned {
                self.particles[s.a].position[0] += cx * 0.5;
                self.particles[s.a].position[1] += cy * 0.5;
                self.particles[s.a].position[2] += cz * 0.5;
            }
            if !self.particles[s.b].pinned {
                self.particles[s.b].position[0] -= cx * 0.5;
                self.particles[s.b].position[1] -= cy * 0.5;
                self.particles[s.b].position[2] -= cz * 0.5;
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_cloth() -> ClothSim {
        // 4-vertex quad split into 2 triangles, no pins.
        let positions = vec![
            [0.0f32, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 1, 3, 2];
        ClothSim::from_mesh(&positions, &indices, 0.8)
    }

    #[test]
    fn cloth_from_mesh_particle_count() {
        assert_eq!(quad_cloth().particles.len(), 4);
    }

    #[test]
    fn cloth_from_mesh_has_springs() {
        assert!(!quad_cloth().springs.is_empty());
    }

    #[test]
    fn cloth_step_moves_particles() {
        let mut sim = quad_cloth();
        let before: Vec<[f32; 3]> = sim.positions();
        sim.step(0.016, 4);
        let after: Vec<[f32; 3]> = sim.positions();
        let moved = before.iter().zip(after.iter()).any(|(b, a)| b != a);
        assert!(moved, "at least one particle should move under gravity");
    }

    #[test]
    fn cloth_pinned_particles_dont_move() {
        // Pin the two top vertices (y == 1.0).
        let positions = vec![
            [0.0f32, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 1, 3, 2];
        let mut sim = ClothSim::from_mesh(&positions, &indices, 0.8).pin_by_y(1.0);

        let pinned_before: Vec<[f32; 3]> = sim
            .particles
            .iter()
            .filter(|p| p.pinned)
            .map(|p| p.position)
            .collect();

        sim.step(0.016, 8);

        let pinned_after: Vec<[f32; 3]> = sim
            .particles
            .iter()
            .filter(|p| p.pinned)
            .map(|p| p.position)
            .collect();

        assert_eq!(
            pinned_before, pinned_after,
            "pinned particles must not move"
        );
    }

    #[test]
    fn cloth_positions_returns_vec() {
        let sim = quad_cloth();
        assert_eq!(sim.positions().len(), sim.particles.len());
    }

    #[test]
    fn cloth_spring_rest_length_positive() {
        let sim = quad_cloth();
        for s in &sim.springs {
            assert!(
                s.rest_length > 0.0,
                "spring ({},{}) has rest_length <= 0",
                s.a,
                s.b
            );
        }
    }
}
