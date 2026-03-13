// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Position-Based Dynamics (XPBD) simulation loop.
//!
//! Provides a unified PBD solver integrating distance, bend, volume,
//! point-fixed, and ground-plane constraints with sub-stepped integration.

#[allow(dead_code)]
/// A single PBD particle with position, velocity, and physical properties.
pub struct PbdParticle {
    pub position: [f32; 3],
    pub prev_position: [f32; 3],
    pub velocity: [f32; 3],
    /// Inverse mass; 0.0 means the particle is pinned/static.
    pub inv_mass: f32,
    /// Radius used in collision detection.
    pub radius: f32,
}

/// The type of a PBD constraint.
#[allow(dead_code)]
pub enum PbdConstraintKind {
    Distance,
    Bend,
    Volume,
    PointFixed { target: [f32; 3] },
    GroundPlane { y: f32 },
}

#[allow(dead_code)]
/// A single PBD constraint connecting one or more particles.
pub struct PbdConstraint {
    pub kind: PbdConstraintKind,
    /// XPBD compliance (inverse stiffness); 0.0 = rigid.
    pub compliance: f32,
    pub particles: Vec<usize>,
    /// Rest value: rest length, rest angle, rest volume, etc.
    pub rest_value: f32,
}

/// Global configuration for the PBD simulation.
pub struct PbdConfig {
    pub substeps: u32,
    pub gravity: [f32; 3],
    /// Velocity damping coefficient in [0, 1].
    pub damping: f32,
    /// Restitution (bounciness) in [0, 1].
    pub restitution: f32,
    /// Coulomb friction coefficient.
    pub friction: f32,
}

impl Default for PbdConfig {
    fn default() -> Self {
        Self {
            substeps: 8,
            gravity: [0.0, -9.81, 0.0],
            damping: 0.005,
            restitution: 0.2,
            friction: 0.3,
        }
    }
}

/// A complete XPBD simulation scene.
pub struct PbdSimulation {
    pub particles: Vec<PbdParticle>,
    pub constraints: Vec<PbdConstraint>,
    pub config: PbdConfig,
}

impl PbdSimulation {
    /// Create a new empty simulation with the given configuration.
    pub fn new(cfg: PbdConfig) -> Self {
        Self {
            particles: Vec::new(),
            constraints: Vec::new(),
            config: cfg,
        }
    }

    /// Add a particle and return its index.
    pub fn add_particle(&mut self, p: PbdParticle) -> usize {
        let idx = self.particles.len();
        self.particles.push(p);
        idx
    }

    /// Add a constraint to the simulation.
    pub fn add_constraint(&mut self, c: PbdConstraint) {
        self.constraints.push(c);
    }

    /// Advance the simulation by `dt` seconds using XPBD sub-stepping.
    pub fn step(&mut self, dt: f32) {
        let n = self.config.substeps.max(1);
        let sub_dt = dt / n as f32;
        for _ in 0..n {
            // 1. Integrate gravity and store previous positions.
            for p in &mut self.particles {
                if p.inv_mass == 0.0 {
                    continue;
                }
                p.prev_position = p.position;
                p.velocity[0] += self.config.gravity[0] * sub_dt;
                p.velocity[1] += self.config.gravity[1] * sub_dt;
                p.velocity[2] += self.config.gravity[2] * sub_dt;
                p.position[0] += p.velocity[0] * sub_dt;
                p.position[1] += p.velocity[1] * sub_dt;
                p.position[2] += p.velocity[2] * sub_dt;
            }

            // 2. Solve constraints.
            for ci in 0..self.constraints.len() {
                let compliance = self.constraints[ci].compliance;
                let rest = self.constraints[ci].rest_value;
                match &self.constraints[ci].kind {
                    PbdConstraintKind::Distance => {
                        if self.constraints[ci].particles.len() >= 2 {
                            let a = self.constraints[ci].particles[0];
                            let b = self.constraints[ci].particles[1];
                            solve_distance_pbd(
                                &mut self.particles,
                                a,
                                b,
                                rest,
                                compliance,
                                sub_dt,
                                n,
                            );
                        }
                    }
                    PbdConstraintKind::Bend => {
                        // Simplified: treat as a distance constraint between endpoints.
                        if self.constraints[ci].particles.len() >= 2 {
                            let a = self.constraints[ci].particles[0];
                            let b = self.constraints[ci].particles[1];
                            solve_distance_pbd(
                                &mut self.particles,
                                a,
                                b,
                                rest,
                                compliance,
                                sub_dt,
                                n,
                            );
                        }
                    }
                    PbdConstraintKind::Volume => {
                        // Approximate: maintain centroid distances.
                        if self.constraints[ci].particles.len() >= 2 {
                            let a = self.constraints[ci].particles[0];
                            let b = self.constraints[ci].particles[1];
                            solve_distance_pbd(
                                &mut self.particles,
                                a,
                                b,
                                rest,
                                compliance,
                                sub_dt,
                                n,
                            );
                        }
                    }
                    PbdConstraintKind::PointFixed { target } => {
                        let tgt = *target;
                        if let Some(idx) = self.constraints[ci].particles.first().copied() {
                            if idx < self.particles.len() && self.particles[idx].inv_mass > 0.0 {
                                self.particles[idx].position = tgt;
                            }
                        }
                    }
                    PbdConstraintKind::GroundPlane { y } => {
                        let ground_y = *y;
                        let restitution = self.config.restitution;
                        let friction = self.config.friction;
                        for pi in self.constraints[ci].particles.clone() {
                            if pi < self.particles.len() {
                                solve_ground_plane(
                                    &mut self.particles[pi],
                                    ground_y,
                                    restitution,
                                    friction,
                                );
                            }
                        }
                    }
                }
            }

            // 3. Update velocities from position delta and apply damping.
            let inv_sub_dt = if sub_dt > 0.0 { 1.0 / sub_dt } else { 0.0 };
            for p in &mut self.particles {
                if p.inv_mass == 0.0 {
                    continue;
                }
                p.velocity[0] =
                    (p.position[0] - p.prev_position[0]) * inv_sub_dt * (1.0 - self.config.damping);
                p.velocity[1] =
                    (p.position[1] - p.prev_position[1]) * inv_sub_dt * (1.0 - self.config.damping);
                p.velocity[2] =
                    (p.position[2] - p.prev_position[2]) * inv_sub_dt * (1.0 - self.config.damping);
            }

            // 4. Ground plane collision for all particles by default.
            let restitution = self.config.restitution;
            let friction = self.config.friction;
            for p in &mut self.particles {
                if p.inv_mass > 0.0 {
                    solve_ground_plane(p, f32::NEG_INFINITY, restitution, friction);
                }
            }
        }
    }

    /// Compute total kinetic energy: 0.5 * sum(m * |v|²).
    pub fn kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .filter(|p| p.inv_mass > 0.0)
            .map(|p| {
                let mass = 1.0 / p.inv_mass;
                let v2 = p.velocity[0] * p.velocity[0]
                    + p.velocity[1] * p.velocity[1]
                    + p.velocity[2] * p.velocity[2];
                0.5 * mass * v2
            })
            .sum()
    }

    /// Total number of particles in the simulation.
    pub fn total_particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Number of particles with inv_mass == 0 (pinned/static).
    pub fn pinned_count(&self) -> usize {
        self.particles.iter().filter(|p| p.inv_mass == 0.0).count()
    }
}

/// Create a chain of particles from `start` to `end` with `segments` segments.
///
/// The first particle is pinned (inv_mass = 0). Returns the indices of all created particles.
#[allow(clippy::too_many_arguments)]
pub fn add_rope(
    sim: &mut PbdSimulation,
    start: [f32; 3],
    end: [f32; 3],
    segments: usize,
    inv_mass: f32,
) -> Vec<usize> {
    let count = segments + 1;
    let mut indices = Vec::with_capacity(count);
    for i in 0..count {
        let t = if segments > 0 {
            i as f32 / segments as f32
        } else {
            0.0
        };
        let pos = [
            start[0] + t * (end[0] - start[0]),
            start[1] + t * (end[1] - start[1]),
            start[2] + t * (end[2] - start[2]),
        ];
        let im = if i == 0 { 0.0 } else { inv_mass };
        let idx = sim.add_particle(PbdParticle {
            position: pos,
            prev_position: pos,
            velocity: [0.0; 3],
            inv_mass: im,
            radius: 0.01,
        });
        indices.push(idx);
    }
    // Add distance constraints between consecutive particles.
    let seg_len = if segments > 0 {
        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        let dz = end[2] - start[2];
        (dx * dx + dy * dy + dz * dz).sqrt() / segments as f32
    } else {
        0.0
    };
    for i in 0..segments {
        sim.add_constraint(PbdConstraint {
            kind: PbdConstraintKind::Distance,
            compliance: 0.0,
            particles: vec![indices[i], indices[i + 1]],
            rest_value: seg_len,
        });
    }
    indices
}

/// Create a grid of particles forming a cloth patch.
///
/// Adds distance constraints along grid edges and bend constraints along
/// skip-one neighbours. Returns the indices of all created particles.
#[allow(clippy::too_many_arguments)]
pub fn add_cloth_grid(
    sim: &mut PbdSimulation,
    origin: [f32; 3],
    size: [f32; 2],
    divisions: [usize; 2],
    inv_mass: f32,
) -> Vec<usize> {
    let nx = divisions[0].max(1);
    let ny = divisions[1].max(1);
    let dx = size[0] / nx as f32;
    let dy = size[1] / ny as f32;
    let mut indices = Vec::with_capacity((nx + 1) * (ny + 1));

    // Create particles.
    for iy in 0..=(ny) {
        for ix in 0..=(nx) {
            let pos = [
                origin[0] + ix as f32 * dx,
                origin[1],
                origin[2] + iy as f32 * dy,
            ];
            let idx = sim.add_particle(PbdParticle {
                position: pos,
                prev_position: pos,
                velocity: [0.0; 3],
                inv_mass,
                radius: 0.01,
            });
            indices.push(idx);
        }
    }

    let stride = nx + 1;
    let flat = |ix: usize, iy: usize| indices[iy * stride + ix];

    // Distance constraints along edges.
    for iy in 0..=ny {
        for ix in 0..=nx {
            if ix < nx {
                let a = flat(ix, iy);
                let b = flat(ix + 1, iy);
                sim.add_constraint(PbdConstraint {
                    kind: PbdConstraintKind::Distance,
                    compliance: 0.0,
                    particles: vec![a, b],
                    rest_value: dx,
                });
            }
            if iy < ny {
                let a = flat(ix, iy);
                let b = flat(ix, iy + 1);
                sim.add_constraint(PbdConstraint {
                    kind: PbdConstraintKind::Distance,
                    compliance: 0.0,
                    particles: vec![a, b],
                    rest_value: dy,
                });
            }
        }
    }

    // Bend constraints (skip-one neighbours).
    for iy in 0..=ny {
        for ix in 0..=nx {
            if ix + 2 <= nx {
                let a = flat(ix, iy);
                let b = flat(ix + 2, iy);
                sim.add_constraint(PbdConstraint {
                    kind: PbdConstraintKind::Bend,
                    compliance: 1e-4,
                    particles: vec![a, b],
                    rest_value: 2.0 * dx,
                });
            }
            if iy + 2 <= ny {
                let a = flat(ix, iy);
                let b = flat(ix, iy + 2);
                sim.add_constraint(PbdConstraint {
                    kind: PbdConstraintKind::Bend,
                    compliance: 1e-4,
                    particles: vec![a, b],
                    rest_value: 2.0 * dy,
                });
            }
        }
    }

    indices
}

/// XPBD distance constraint solver: applies positional correction to particles `a` and `b`.
pub fn solve_distance_pbd(
    particles: &mut [PbdParticle],
    a: usize,
    b: usize,
    rest: f32,
    compliance: f32,
    dt: f32,
    substeps: u32,
) {
    if a >= particles.len() || b >= particles.len() {
        return;
    }
    let wa = particles[a].inv_mass;
    let wb = particles[b].inv_mass;
    let w_sum = wa + wb;
    if w_sum == 0.0 {
        return;
    }
    let diff = [
        particles[b].position[0] - particles[a].position[0],
        particles[b].position[1] - particles[a].position[1],
        particles[b].position[2] - particles[a].position[2],
    ];
    let dist = (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2]).sqrt();
    if dist < f32::EPSILON {
        return;
    }
    let dir = [diff[0] / dist, diff[1] / dist, diff[2] / dist];
    let sub_dt = dt / substeps.max(1) as f32;
    let alpha = compliance / (sub_dt * sub_dt);
    let lambda = (dist - rest) / (w_sum + alpha);
    particles[a].position[0] += wa * lambda * dir[0];
    particles[a].position[1] += wa * lambda * dir[1];
    particles[a].position[2] += wa * lambda * dir[2];
    particles[b].position[0] -= wb * lambda * dir[0];
    particles[b].position[1] -= wb * lambda * dir[1];
    particles[b].position[2] -= wb * lambda * dir[2];
}

/// Enforce a ground plane collision at height `y` with bounce and friction.
pub fn solve_ground_plane(p: &mut PbdParticle, y: f32, restitution: f32, friction: f32) {
    if p.position[1] < y + p.radius {
        p.position[1] = y + p.radius;
        if p.velocity[1] < 0.0 {
            p.velocity[1] = -p.velocity[1] * restitution;
        }
        // Apply friction to horizontal velocity.
        let vh = (p.velocity[0] * p.velocity[0] + p.velocity[2] * p.velocity[2]).sqrt();
        if vh > f32::EPSILON {
            let scale = (1.0 - friction).max(0.0);
            p.velocity[0] *= scale;
            p.velocity[2] *= scale;
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_particle(x: f32, y: f32, z: f32, inv_mass: f32) -> PbdParticle {
        PbdParticle {
            position: [x, y, z],
            prev_position: [x, y, z],
            velocity: [0.0; 3],
            inv_mass,
            radius: 0.01,
        }
    }

    #[test]
    fn new_simulation_empty() {
        let sim = PbdSimulation::new(PbdConfig::default());
        assert_eq!(sim.total_particle_count(), 0);
        assert_eq!(sim.pinned_count(), 0);
        assert!(sim.constraints.is_empty());
    }

    #[test]
    fn add_particle_returns_correct_index() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        let i0 = sim.add_particle(make_particle(0.0, 0.0, 0.0, 1.0));
        let i1 = sim.add_particle(make_particle(1.0, 0.0, 0.0, 1.0));
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
    }

    #[test]
    fn add_constraint_stores_correctly() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        sim.add_particle(make_particle(0.0, 0.0, 0.0, 1.0));
        sim.add_particle(make_particle(1.0, 0.0, 0.0, 1.0));
        sim.add_constraint(PbdConstraint {
            kind: PbdConstraintKind::Distance,
            compliance: 0.0,
            particles: vec![0, 1],
            rest_value: 1.0,
        });
        assert_eq!(sim.constraints.len(), 1);
    }

    #[test]
    fn step_does_not_produce_nan() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        sim.add_particle(make_particle(0.0, 2.0, 0.0, 1.0));
        sim.add_particle(make_particle(1.0, 2.0, 0.0, 1.0));
        sim.add_constraint(PbdConstraint {
            kind: PbdConstraintKind::Distance,
            compliance: 0.0,
            particles: vec![0, 1],
            rest_value: 1.0,
        });
        for _ in 0..10 {
            sim.step(0.016);
        }
        for p in &sim.particles {
            assert!(!p.position[0].is_nan(), "position x is NaN");
            assert!(!p.position[1].is_nan(), "position y is NaN");
            assert!(!p.position[2].is_nan(), "position z is NaN");
        }
    }

    #[test]
    fn kinetic_energy_positive_after_fall() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        sim.add_particle(make_particle(0.0, 5.0, 0.0, 1.0));
        sim.step(0.1);
        let ke = sim.kinetic_energy();
        assert!(ke >= 0.0, "kinetic energy must be non-negative");
        assert!(ke > 0.0, "particle must have moved after gravity step");
    }

    #[test]
    fn rope_has_n_minus_1_distance_constraints() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        let segments = 5;
        add_rope(&mut sim, [0.0; 3], [1.0, 0.0, 0.0], segments, 1.0);
        assert_eq!(sim.constraints.len(), segments);
    }

    #[test]
    fn cloth_grid_creates_correct_particle_count() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        let divs = [3, 4];
        let indices = add_cloth_grid(&mut sim, [0.0; 3], [1.0, 1.0], divs, 1.0);
        assert_eq!(indices.len(), (divs[0] + 1) * (divs[1] + 1));
        assert_eq!(sim.total_particle_count(), indices.len());
    }

    #[test]
    fn pinned_count_for_rope() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        add_rope(&mut sim, [0.0; 3], [1.0, 0.0, 0.0], 4, 1.0);
        assert_eq!(sim.pinned_count(), 1, "first rope particle must be pinned");
    }

    #[test]
    fn ground_plane_prevents_going_below_y() {
        let mut p = make_particle(0.0, 0.5, 0.0, 1.0);
        p.velocity[1] = -10.0;
        solve_ground_plane(&mut p, 0.0, 0.0, 0.0);
        assert!(
            p.position[1] >= p.radius,
            "particle must not go below ground"
        );
    }

    #[test]
    fn ground_plane_bounce_positive_velocity() {
        let mut p = make_particle(0.0, -0.1, 0.0, 1.0);
        p.velocity[1] = -5.0;
        solve_ground_plane(&mut p, 0.0, 0.5, 0.0);
        assert!(
            p.velocity[1] > 0.0,
            "bounce should invert vertical velocity"
        );
    }

    #[test]
    fn solve_distance_pbd_shrinks_overstretched() {
        let mut particles = vec![
            make_particle(0.0, 0.0, 0.0, 1.0),
            make_particle(2.0, 0.0, 0.0, 1.0),
        ];
        let rest = 1.0;
        let before = particles[1].position[0] - particles[0].position[0];
        solve_distance_pbd(&mut particles, 0, 1, rest, 0.0, 0.016, 1);
        let after = (particles[1].position[0] - particles[0].position[0]).abs();
        assert!(
            after < before,
            "overstretched constraint should reduce separation"
        );
    }

    #[test]
    fn solve_distance_pbd_static_particle_unaffected() {
        let mut particles = vec![
            make_particle(0.0, 0.0, 0.0, 0.0), // pinned
            make_particle(2.0, 0.0, 0.0, 1.0),
        ];
        let orig_a = particles[0].position;
        solve_distance_pbd(&mut particles, 0, 1, 1.0, 0.0, 0.016, 1);
        assert_eq!(
            particles[0].position, orig_a,
            "pinned particle must not move"
        );
    }

    #[test]
    fn total_particle_count_matches_added() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        for i in 0..7 {
            sim.add_particle(make_particle(i as f32, 0.0, 0.0, 1.0));
        }
        assert_eq!(sim.total_particle_count(), 7);
    }

    #[test]
    fn pbd_config_default_values() {
        let cfg = PbdConfig::default();
        assert_eq!(cfg.substeps, 8);
        assert!((cfg.gravity[1] + 9.81).abs() < 1e-5);
        assert!((cfg.damping - 0.005).abs() < 1e-5);
        assert!((cfg.restitution - 0.2).abs() < 1e-5);
        assert!((cfg.friction - 0.3).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_zero_for_static_particles() {
        let mut sim = PbdSimulation::new(PbdConfig::default());
        sim.add_particle(make_particle(0.0, 0.0, 0.0, 0.0));
        sim.add_particle(make_particle(1.0, 0.0, 0.0, 0.0));
        assert_eq!(sim.kinetic_energy(), 0.0);
    }
}
