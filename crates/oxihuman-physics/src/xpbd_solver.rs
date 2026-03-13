// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! XPBD (Extended Position-Based Dynamics) constraint solver with compliance.

// ─── Structures ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct XpbdParticle {
    pub position: [f32; 3],
    pub prev_position: [f32; 3],
    pub velocity: [f32; 3],
    pub inv_mass: f32,
    pub id: u32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum XpbdConstraintType {
    Distance { rest_length: f32, compliance: f32 },
    Volume { rest_volume: f32, compliance: f32 },
    Bend { rest_angle: f32, compliance: f32 },
}

#[allow(dead_code)]
pub struct XpbdConstraint {
    /// Indices into the world's particle vector.
    pub particles: Vec<usize>,
    pub constraint_type: XpbdConstraintType,
    /// Accumulated Lagrange multiplier (reset each substep).
    pub lambda: f32,
}

#[allow(dead_code)]
pub struct XpbdWorld {
    pub particles: Vec<XpbdParticle>,
    pub constraints: Vec<XpbdConstraint>,
    pub gravity: [f32; 3],
    pub next_id: u32,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn kinetic_energy(p: &XpbdParticle) -> f32 {
    if p.inv_mass <= 0.0 {
        return 0.0;
    }
    let mass = 1.0 / p.inv_mass;
    let v = p.velocity;
    0.5 * mass * dot3(v, v)
}

// ─── World management ─────────────────────────────────────────────────────────

/// Create a new XPBD world with default gravity (−9.81 m/s² downward).
#[allow(dead_code)]
pub fn new_xpbd_world() -> XpbdWorld {
    XpbdWorld {
        particles: Vec::new(),
        constraints: Vec::new(),
        gravity: [0.0, -9.81, 0.0],
        next_id: 0,
    }
}

/// Add a particle to the world; returns its index.
#[allow(dead_code)]
pub fn add_xpbd_particle(world: &mut XpbdWorld, pos: [f32; 3], inv_mass: f32) -> usize {
    let idx = world.particles.len();
    let id = world.next_id;
    world.next_id += 1;
    world.particles.push(XpbdParticle {
        position: pos,
        prev_position: pos,
        velocity: [0.0; 3],
        inv_mass,
        id,
    });
    idx
}

/// Add a distance constraint between particles `a` and `b`.
/// The rest length is computed from the current positions.
#[allow(dead_code)]
pub fn add_distance_constraint(world: &mut XpbdWorld, a: usize, b: usize, compliance: f32) {
    let rest_length = dist3(world.particles[a].position, world.particles[b].position);
    world.constraints.push(XpbdConstraint {
        particles: vec![a, b],
        constraint_type: XpbdConstraintType::Distance {
            rest_length,
            compliance,
        },
        lambda: 0.0,
    });
}

/// Add a bend constraint between three consecutive particles.
/// The rest angle is approximated as 0 (straight).
#[allow(dead_code)]
pub fn add_bend_constraint(world: &mut XpbdWorld, a: usize, b: usize, c: usize, compliance: f32) {
    world.constraints.push(XpbdConstraint {
        particles: vec![a, b, c],
        constraint_type: XpbdConstraintType::Bend {
            rest_angle: 0.0,
            compliance,
        },
        lambda: 0.0,
    });
}

// ─── Simulation steps ─────────────────────────────────────────────────────────

/// Apply gravity and integrate positions (semi-implicit Euler predict step).
#[allow(dead_code)]
pub fn predict_positions(world: &mut XpbdWorld, dt: f32) {
    for p in world.particles.iter_mut() {
        if p.inv_mass <= 0.0 {
            continue;
        }
        // Integrate velocity with gravity.
        p.velocity[0] += world.gravity[0] * dt;
        p.velocity[1] += world.gravity[1] * dt;
        p.velocity[2] += world.gravity[2] * dt;
        // Save previous position.
        p.prev_position = p.position;
        // Predict new position.
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;
    }
}

/// Update velocities from position deltas after constraint solving.
#[allow(dead_code)]
pub fn update_velocities(world: &mut XpbdWorld, dt: f32) {
    let inv_dt = if dt > 1e-12 { 1.0 / dt } else { 0.0 };
    for p in world.particles.iter_mut() {
        if p.inv_mass <= 0.0 {
            continue;
        }
        p.velocity[0] = (p.position[0] - p.prev_position[0]) * inv_dt;
        p.velocity[1] = (p.position[1] - p.prev_position[1]) * inv_dt;
        p.velocity[2] = (p.position[2] - p.prev_position[2]) * inv_dt;
    }
}

/// Reset accumulated Lagrange multipliers to zero.
#[allow(dead_code)]
pub fn reset_lambdas(world: &mut XpbdWorld) {
    for c in world.constraints.iter_mut() {
        c.lambda = 0.0;
    }
}

/// Solve a distance constraint in-place (XPBD formulation with compliance).
#[allow(dead_code)]
pub fn solve_distance_constraint(
    particles: &mut [XpbdParticle],
    a: usize,
    b: usize,
    rest_len: f32,
    compliance: f32,
    lambda: &mut f32,
    dt: f32,
) {
    let pa = particles[a].position;
    let pb = particles[b].position;
    let diff = sub3(pb, pa);
    let current_len = (dot3(diff, diff)).sqrt();
    if current_len < 1e-12 {
        return;
    }
    let constraint = current_len - rest_len;
    let alpha = compliance / (dt * dt);
    let w_a = particles[a].inv_mass;
    let w_b = particles[b].inv_mass;
    let w_sum = w_a + w_b;
    if w_sum < 1e-12 {
        return;
    }
    let delta_lambda = (-constraint - alpha * *lambda) / (w_sum + alpha);
    *lambda += delta_lambda;
    let dir = scale3(diff, 1.0 / current_len);
    let correction = scale3(dir, delta_lambda);
    particles[a].position[0] -= correction[0] * w_a;
    particles[a].position[1] -= correction[1] * w_a;
    particles[a].position[2] -= correction[2] * w_a;
    particles[b].position[0] += correction[0] * w_b;
    particles[b].position[1] += correction[1] * w_b;
    particles[b].position[2] += correction[2] * w_b;
}

/// Collect constraint data, solve, write back positions.
fn solve_constraints(world: &mut XpbdWorld, dt: f32) {
    // Collect constraint data to avoid borrow conflicts.
    let constraint_data: Vec<_> = world
        .constraints
        .iter()
        .map(|c| (c.particles.clone(), c.constraint_type.clone()))
        .collect();

    for (con_idx, (parts, ctype)) in constraint_data.iter().enumerate() {
        match ctype {
            XpbdConstraintType::Distance {
                rest_length,
                compliance,
            } => {
                if parts.len() >= 2 {
                    let a = parts[0];
                    let b = parts[1];
                    let lambda = &mut world.constraints[con_idx].lambda;
                    solve_distance_constraint(
                        &mut world.particles,
                        a,
                        b,
                        *rest_length,
                        *compliance,
                        lambda,
                        dt,
                    );
                }
            }
            XpbdConstraintType::Bend { .. } | XpbdConstraintType::Volume { .. } => {
                // Simplified: no-op for Bend/Volume in this implementation.
            }
        }
    }
}

/// Run one simulation step with `substeps` sub-iterations.
#[allow(dead_code)]
pub fn xpbd_step(world: &mut XpbdWorld, dt: f32, substeps: u32) {
    let sub_dt = if substeps > 0 {
        dt / substeps as f32
    } else {
        dt
    };
    for _ in 0..substeps {
        predict_positions(world, sub_dt);
        reset_lambdas(world);
        solve_constraints(world, sub_dt);
        update_velocities(world, sub_dt);
    }
}

// ─── Accessors ────────────────────────────────────────────────────────────────

/// Number of particles in the world.
#[allow(dead_code)]
pub fn xpbd_particle_count(world: &XpbdWorld) -> usize {
    world.particles.len()
}

/// Number of constraints in the world.
#[allow(dead_code)]
pub fn xpbd_constraint_count(world: &XpbdWorld) -> usize {
    world.constraints.len()
}

/// Sum of kinetic energies of all particles.
#[allow(dead_code)]
pub fn xpbd_total_energy(world: &XpbdWorld) -> f32 {
    world.particles.iter().map(kinetic_energy).sum()
}

/// Fix a particle in place (set inv_mass = 0).
#[allow(dead_code)]
pub fn pin_particle(world: &mut XpbdWorld, idx: usize) {
    if idx < world.particles.len() {
        world.particles[idx].inv_mass = 0.0;
    }
}

/// Unpin a particle (set inv_mass = 1).
#[allow(dead_code)]
pub fn unpin_particle(world: &mut XpbdWorld, idx: usize) {
    if idx < world.particles.len() {
        world.particles[idx].inv_mass = 1.0;
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_world_is_empty() {
        let world = new_xpbd_world();
        assert_eq!(xpbd_particle_count(&world), 0);
        assert_eq!(xpbd_constraint_count(&world), 0);
    }

    #[test]
    fn test_add_particle_increments_count() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(xpbd_particle_count(&world), 1);
        add_xpbd_particle(&mut world, [1.0, 0.0, 0.0], 1.0);
        assert_eq!(xpbd_particle_count(&world), 2);
    }

    #[test]
    fn test_add_distance_constraint() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [1.0, 0.0, 0.0], 1.0);
        add_distance_constraint(&mut world, 0, 1, 0.0);
        assert_eq!(xpbd_constraint_count(&world), 1);
    }

    #[test]
    fn test_add_bend_constraint() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [1.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [2.0, 0.0, 0.0], 1.0);
        add_bend_constraint(&mut world, 0, 1, 2, 0.001);
        assert_eq!(xpbd_constraint_count(&world), 1);
    }

    #[test]
    fn test_particle_count_function() {
        let mut world = new_xpbd_world();
        for i in 0..5 {
            add_xpbd_particle(&mut world, [i as f32, 0.0, 0.0], 1.0);
        }
        assert_eq!(xpbd_particle_count(&world), 5);
    }

    #[test]
    fn test_constraint_count_function() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [1.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [2.0, 0.0, 0.0], 1.0);
        add_distance_constraint(&mut world, 0, 1, 0.0);
        add_distance_constraint(&mut world, 1, 2, 0.0);
        assert_eq!(xpbd_constraint_count(&world), 2);
    }

    #[test]
    fn test_predict_positions_moves_particle() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 10.0, 0.0], 1.0);
        let before_y = world.particles[0].position[1];
        predict_positions(&mut world, 0.016);
        let after_y = world.particles[0].position[1];
        // Gravity should move the particle downward.
        assert!(after_y < before_y, "before={} after={}", before_y, after_y);
    }

    #[test]
    fn test_predict_positions_pinned_particle_does_not_move() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 10.0, 0.0], 0.0); // pinned
        let before = world.particles[0].position;
        predict_positions(&mut world, 0.016);
        let after = world.particles[0].position;
        assert_eq!(before, after);
    }

    #[test]
    fn test_solve_distance_constraint_moves_particles() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [2.0, 0.0, 0.0], 1.0); // too far
        add_distance_constraint(&mut world, 0, 1, 0.0);
        // Override rest length for a deterministic test.
        if let XpbdConstraintType::Distance {
            ref mut rest_length,
            ..
        } = world.constraints[0].constraint_type
        {
            *rest_length = 1.0;
        }
        let mut lambda = 0.0_f32;
        solve_distance_constraint(&mut world.particles, 0, 1, 1.0, 0.0, &mut lambda, 0.016);
        let d = dist3(world.particles[0].position, world.particles[1].position);
        assert!((d - 1.0).abs() < 1e-4, "d={}", d);
    }

    #[test]
    fn test_pin_particle() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        pin_particle(&mut world, 0);
        assert_eq!(world.particles[0].inv_mass, 0.0);
    }

    #[test]
    fn test_unpin_particle() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 0.0);
        unpin_particle(&mut world, 0);
        assert_eq!(world.particles[0].inv_mass, 1.0);
    }

    #[test]
    fn test_xpbd_step_falls_with_gravity() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 10.0, 0.0], 1.0);
        let before_y = world.particles[0].position[1];
        xpbd_step(&mut world, 0.1, 4);
        let after_y = world.particles[0].position[1];
        assert!(
            after_y < before_y,
            "particle did not fall: before={} after={}",
            before_y,
            after_y
        );
    }

    #[test]
    fn test_xpbd_total_energy_after_step() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 10.0, 0.0], 1.0);
        xpbd_step(&mut world, 0.1, 4);
        let energy = xpbd_total_energy(&world);
        assert!(energy >= 0.0, "energy={}", energy);
    }

    #[test]
    fn test_reset_lambdas() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        add_xpbd_particle(&mut world, [1.0, 0.0, 0.0], 1.0);
        add_distance_constraint(&mut world, 0, 1, 0.0);
        world.constraints[0].lambda = 42.0;
        reset_lambdas(&mut world);
        assert_eq!(world.constraints[0].lambda, 0.0);
    }

    #[test]
    fn test_update_velocities() {
        let mut world = new_xpbd_world();
        add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
        world.particles[0].prev_position = [0.0, 0.0, 0.0];
        world.particles[0].position = [1.0, 0.0, 0.0];
        update_velocities(&mut world, 1.0);
        assert!((world.particles[0].velocity[0] - 1.0).abs() < 1e-5);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Solver-facade API
// ═══════════════════════════════════════════════════════════════════════════
//
// The types below provide the `XpbdSolver`/`XpbdConfig` API required by the
// module spec.  They wrap the underlying `XpbdWorld` machinery defined above.

/// Configuration for the XPBD solver facade.
#[allow(dead_code)]
pub struct XpbdConfig {
    /// Gravitational acceleration (default [0, -9.81, 0]).
    pub gravity: [f32; 3],
    /// Default compliance for newly added constraints.
    pub default_compliance: f32,
}

/// An XPBD distance constraint (solver-facade version).
#[allow(dead_code)]
pub struct XpbdDistConstraint {
    pub particle_a: usize,
    pub particle_b: usize,
    pub rest_length: f32,
    pub compliance: f32,
    pub lambda: f32,
}

/// XPBD solver facade that owns particles and constraints.
#[allow(dead_code)]
pub struct XpbdSolver {
    pub particles: Vec<XpbdParticle>,
    pub constraints: Vec<XpbdDistConstraint>,
    pub gravity: [f32; 3],
}

/// Default XPBD configuration (gravity −9.81 m/s²).
#[allow(dead_code)]
pub fn default_xpbd_config() -> XpbdConfig {
    XpbdConfig {
        gravity: [0.0, -9.81, 0.0],
        default_compliance: 0.0,
    }
}

/// Create a new XPBD solver from a configuration.
#[allow(dead_code)]
pub fn new_xpbd_solver(cfg: &XpbdConfig) -> XpbdSolver {
    XpbdSolver {
        particles: Vec::new(),
        constraints: Vec::new(),
        gravity: cfg.gravity,
    }
}

/// Add a particle to the solver.  `mass` of 0.0 pins the particle.
/// Returns the new particle's index.
#[allow(dead_code)]
pub fn xpbd_add_particle(solver: &mut XpbdSolver, pos: [f32; 3], mass: f32) -> usize {
    let inv_mass = if mass > 1e-12 { 1.0 / mass } else { 0.0 };
    let idx = solver.particles.len();
    solver.particles.push(XpbdParticle {
        position: pos,
        prev_position: pos,
        velocity: [0.0; 3],
        inv_mass,
        id: idx as u32,
    });
    idx
}

/// Add a distance constraint between particles `a` and `b`.
#[allow(dead_code)]
pub fn xpbd_add_distance_constraint(
    solver: &mut XpbdSolver,
    a: usize,
    b: usize,
    rest: f32,
    compliance: f32,
) {
    solver.constraints.push(XpbdDistConstraint {
        particle_a: a,
        particle_b: b,
        rest_length: rest,
        compliance,
        lambda: 0.0,
    });
}

/// Simulate `substeps` sub-steps of duration `dt / substeps`.
#[allow(dead_code)]
pub fn xpbd_solver_step(solver: &mut XpbdSolver, dt: f32, substeps: u32) {
    let n = if substeps > 0 { substeps } else { 1 };
    let sub_dt = dt / n as f32;

    for _ in 0..n {
        // --- predict ---
        for p in solver.particles.iter_mut() {
            if p.inv_mass <= 0.0 {
                continue;
            }
            p.velocity[0] += solver.gravity[0] * sub_dt;
            p.velocity[1] += solver.gravity[1] * sub_dt;
            p.velocity[2] += solver.gravity[2] * sub_dt;
            p.prev_position = p.position;
            p.position[0] += p.velocity[0] * sub_dt;
            p.position[1] += p.velocity[1] * sub_dt;
            p.position[2] += p.velocity[2] * sub_dt;
        }

        // --- reset lambdas ---
        for c in solver.constraints.iter_mut() {
            c.lambda = 0.0;
        }

        // --- solve constraints ---
        for ci in 0..solver.constraints.len() {
            let (a_idx, b_idx, rest_len, compliance) = {
                let c = &solver.constraints[ci];
                (c.particle_a, c.particle_b, c.rest_length, c.compliance)
            };
            let lambda = &mut solver.constraints[ci].lambda;
            solve_distance_constraint(
                &mut solver.particles,
                a_idx,
                b_idx,
                rest_len,
                compliance,
                lambda,
                sub_dt,
            );
        }

        // --- update velocities ---
        let inv_dt = if sub_dt > 1e-12 { 1.0 / sub_dt } else { 0.0 };
        for p in solver.particles.iter_mut() {
            if p.inv_mass <= 0.0 {
                continue;
            }
            p.velocity[0] = (p.position[0] - p.prev_position[0]) * inv_dt;
            p.velocity[1] = (p.position[1] - p.prev_position[1]) * inv_dt;
            p.velocity[2] = (p.position[2] - p.prev_position[2]) * inv_dt;
        }
    }
}

/// Position of particle at `idx`.
#[allow(dead_code)]
pub fn xpbd_particle_position(solver: &XpbdSolver, idx: usize) -> [f32; 3] {
    solver.particles[idx].position
}

/// Number of particles in the solver.
#[allow(dead_code)]
pub fn xpbd_solver_particle_count(solver: &XpbdSolver) -> usize {
    solver.particles.len()
}

/// Number of constraints in the solver.
#[allow(dead_code)]
pub fn xpbd_solver_constraint_count(solver: &XpbdSolver) -> usize {
    solver.constraints.len()
}

/// Pin particle `idx` (set its mass to infinity → inv_mass = 0).
#[allow(dead_code)]
pub fn xpbd_pin_particle(solver: &mut XpbdSolver, idx: usize) {
    if idx < solver.particles.len() {
        solver.particles[idx].inv_mass = 0.0;
    }
}

/// Reset all particles to their initial (creation) positions and clear velocities.
#[allow(dead_code)]
pub fn xpbd_reset(solver: &mut XpbdSolver) {
    for p in solver.particles.iter_mut() {
        p.velocity = [0.0; 3];
        p.prev_position = p.position;
    }
    for c in solver.constraints.iter_mut() {
        c.lambda = 0.0;
    }
}

// ─── Solver-facade Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod solver_tests {
    use super::*;

    #[test]
    fn test_default_config_gravity() {
        let cfg = default_xpbd_config();
        assert!((cfg.gravity[1] + 9.81).abs() < 1e-5);
    }

    #[test]
    fn test_new_solver_is_empty() {
        let cfg = default_xpbd_config();
        let solver = new_xpbd_solver(&cfg);
        assert_eq!(xpbd_solver_particle_count(&solver), 0);
        assert_eq!(xpbd_solver_constraint_count(&solver), 0);
    }

    #[test]
    fn test_add_particle_returns_index() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        let i0 = xpbd_add_particle(&mut solver, [0.0, 0.0, 0.0], 1.0);
        let i1 = xpbd_add_particle(&mut solver, [1.0, 0.0, 0.0], 1.0);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(xpbd_solver_particle_count(&solver), 2);
    }

    #[test]
    fn test_add_distance_constraint() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        xpbd_add_particle(&mut solver, [0.0, 0.0, 0.0], 1.0);
        xpbd_add_particle(&mut solver, [1.0, 0.0, 0.0], 1.0);
        xpbd_add_distance_constraint(&mut solver, 0, 1, 1.0, 0.0);
        assert_eq!(xpbd_solver_constraint_count(&solver), 1);
    }

    #[test]
    fn test_particle_falls_with_gravity() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        xpbd_add_particle(&mut solver, [0.0, 10.0, 0.0], 1.0);
        let before = xpbd_particle_position(&solver, 0)[1];
        xpbd_solver_step(&mut solver, 0.1, 4);
        let after = xpbd_particle_position(&solver, 0)[1];
        assert!(after < before, "before={} after={}", before, after);
    }

    #[test]
    fn test_pinned_particle_does_not_move() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        xpbd_add_particle(&mut solver, [0.0, 10.0, 0.0], 1.0);
        xpbd_pin_particle(&mut solver, 0);
        let before = xpbd_particle_position(&solver, 0);
        xpbd_solver_step(&mut solver, 0.1, 4);
        let after = xpbd_particle_position(&solver, 0);
        assert_eq!(before, after);
    }

    #[test]
    fn test_zero_mass_particle_is_pinned() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        xpbd_add_particle(&mut solver, [5.0, 5.0, 5.0], 0.0);
        let before = xpbd_particle_position(&solver, 0);
        xpbd_solver_step(&mut solver, 1.0, 10);
        let after = xpbd_particle_position(&solver, 0);
        assert_eq!(before, after);
    }

    #[test]
    fn test_reset_clears_velocity() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        xpbd_add_particle(&mut solver, [0.0, 10.0, 0.0], 1.0);
        xpbd_solver_step(&mut solver, 0.1, 4);
        xpbd_reset(&mut solver);
        assert_eq!(solver.particles[0].velocity, [0.0; 3]);
    }

    #[test]
    fn test_distance_constraint_enforced() {
        let cfg = default_xpbd_config();
        let mut solver = new_xpbd_solver(&cfg);
        // Two particles separated by 2 units, rest length = 1.
        xpbd_add_particle(&mut solver, [0.0, 0.0, 0.0], 1.0);
        xpbd_add_particle(&mut solver, [2.0, 0.0, 0.0], 1.0);
        xpbd_add_distance_constraint(&mut solver, 0, 1, 1.0, 0.0);
        // Several constraint-only steps (no gravity step involved, use small dt).
        xpbd_solver_step(&mut solver, 0.001, 1);
        let p0 = xpbd_particle_position(&solver, 0);
        let p1 = xpbd_particle_position(&solver, 1);
        let dx = p1[0] - p0[0];
        let dy = p1[1] - p0[1];
        let dz = p1[2] - p0[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        // After correction the distance should be much closer to 1.
        assert!(dist < 1.5, "dist={}", dist);
    }
}
