//! Position-based dynamics (PBD) constraint solver.
//!
//! Iteratively resolves distance and angle constraints on particles using the
//! position-based dynamics formulation (Müller et al. 2007).

/// A single PBD particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbdParticle {
    /// Current position.
    pub position: [f32; 3],
    /// Previous position (used for velocity integration).
    pub prev_position: [f32; 3],
    /// Inverse mass (0 = pinned/infinite mass).
    pub inv_mass: f32,
    /// Whether the particle is pinned (immovable).
    pub pinned: bool,
}

/// The type of PBD constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbdConstraintType {
    /// Maintains a fixed distance between two particles.
    Distance,
    /// Maintains an angle formed by three particles.
    Angle,
    /// Maintains the volume of a tetrahedral cell.
    Volume,
}

/// A single PBD constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbdConstraint {
    /// Constraint type.
    pub kind: PbdConstraintType,
    /// Particle indices involved (up to 4).
    pub indices: [usize; 4],
    /// Rest value (distance, angle in radians, or volume).
    pub rest_value: f32,
    /// Stiffness in [0, 1].
    pub stiffness: f32,
}

/// Configuration for the PBD solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbdSolverConfig {
    /// Number of constraint solver iterations per step.
    pub iterations: usize,
    /// Global damping factor applied to velocities.
    pub damping: f32,
}

/// The PBD solver state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbdSolver {
    /// All particles.
    pub particles: Vec<PbdParticle>,
    /// All constraints.
    pub constraints: Vec<PbdConstraint>,
    /// Solver configuration.
    pub config: PbdSolverConfig,
}

/// Returns the default PBD solver configuration.
#[allow(dead_code)]
pub fn default_pbd_solver_config() -> PbdSolverConfig {
    PbdSolverConfig {
        iterations: 10,
        damping: 0.01,
    }
}

/// Creates a new PBD solver with the given configuration.
#[allow(dead_code)]
pub fn new_pbd_solver(cfg: &PbdSolverConfig) -> PbdSolver {
    PbdSolver {
        particles: Vec::new(),
        constraints: Vec::new(),
        config: cfg.clone(),
    }
}

/// Adds a particle to the solver and returns its index.
#[allow(dead_code)]
pub fn pbd_add_particle(solver: &mut PbdSolver, pos: [f32; 3], mass: f32) -> usize {
    let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
    let idx = solver.particles.len();
    solver.particles.push(PbdParticle {
        position: pos,
        prev_position: pos,
        inv_mass,
        pinned: false,
    });
    idx
}

/// Adds a distance constraint between two particles.
#[allow(dead_code)]
pub fn pbd_add_distance_constraint(
    solver: &mut PbdSolver,
    a: usize,
    b: usize,
    rest_length: f32,
    stiffness: f32,
) {
    solver.constraints.push(PbdConstraint {
        kind: PbdConstraintType::Distance,
        indices: [a, b, 0, 0],
        rest_value: rest_length,
        stiffness: stiffness.clamp(0.0, 1.0),
    });
}

/// Pins a particle in place (sets inv_mass to 0).
#[allow(dead_code)]
pub fn pbd_pin_particle(solver: &mut PbdSolver, idx: usize) {
    if idx < solver.particles.len() {
        solver.particles[idx].pinned = true;
        solver.particles[idx].inv_mass = 0.0;
    }
}

/// Advances the simulation by one time step `dt` under uniform `gravity`.
#[allow(dead_code)]
pub fn pbd_step(solver: &mut PbdSolver, dt: f32, gravity: [f32; 3]) {
    // 1. Apply external forces (gravity) via Verlet integration.
    for p in solver.particles.iter_mut() {
        if p.pinned || p.inv_mass == 0.0 {
            continue;
        }
        let vel = [
            (p.position[0] - p.prev_position[0]) * (1.0 - solver.config.damping),
            (p.position[1] - p.prev_position[1]) * (1.0 - solver.config.damping),
            (p.position[2] - p.prev_position[2]) * (1.0 - solver.config.damping),
        ];
        let prev = p.position;
        p.position[0] += vel[0] + gravity[0] * dt * dt;
        p.position[1] += vel[1] + gravity[1] * dt * dt;
        p.position[2] += vel[2] + gravity[2] * dt * dt;
        p.prev_position = prev;
    }

    // 2. Solve constraints iteratively.
    for _ in 0..solver.config.iterations {
        // Collect constraint data first to avoid borrow conflicts.
        let constraints: Vec<PbdConstraint> = solver.constraints.clone();
        for c in &constraints {
            match c.kind {
                PbdConstraintType::Distance => {
                    solve_distance_constraint(solver, c);
                }
                PbdConstraintType::Angle | PbdConstraintType::Volume => {
                    // Stub: not implemented in this version.
                }
            }
        }
    }
}

/// Solves a single distance constraint by projecting particles.
fn solve_distance_constraint(solver: &mut PbdSolver, c: &PbdConstraint) {
    let a = c.indices[0];
    let b = c.indices[1];
    if a >= solver.particles.len() || b >= solver.particles.len() {
        return;
    }
    let pa = solver.particles[a].position;
    let pb = solver.particles[b].position;
    let dx = pb[0] - pa[0];
    let dy = pb[1] - pa[1];
    let dz = pb[2] - pa[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist < 1e-10 {
        return;
    }
    let w_a = solver.particles[a].inv_mass;
    let w_b = solver.particles[b].inv_mass;
    let w_sum = w_a + w_b;
    if w_sum < 1e-10 {
        return;
    }
    let correction = (dist - c.rest_value) / dist * c.stiffness;
    let cx = dx * correction;
    let cy = dy * correction;
    let cz = dz * correction;

    if !solver.particles[a].pinned {
        solver.particles[a].position[0] += cx * w_a / w_sum;
        solver.particles[a].position[1] += cy * w_a / w_sum;
        solver.particles[a].position[2] += cz * w_a / w_sum;
    }
    if !solver.particles[b].pinned {
        solver.particles[b].position[0] -= cx * w_b / w_sum;
        solver.particles[b].position[1] -= cy * w_b / w_sum;
        solver.particles[b].position[2] -= cz * w_b / w_sum;
    }
}

/// Returns the current position of particle `idx`.
#[allow(dead_code)]
pub fn pbd_particle_position(solver: &PbdSolver, idx: usize) -> [f32; 3] {
    solver.particles[idx].position
}

/// Returns the number of particles in the solver.
#[allow(dead_code)]
pub fn pbd_particle_count(solver: &PbdSolver) -> usize {
    solver.particles.len()
}

/// Returns the number of constraints in the solver.
#[allow(dead_code)]
pub fn pbd_constraint_count(solver: &PbdSolver) -> usize {
    solver.constraints.len()
}

/// Resets all particles to their initial positions (prev = pos) and clears
/// velocities by setting prev_position = position.
#[allow(dead_code)]
pub fn pbd_reset(solver: &mut PbdSolver) {
    for p in solver.particles.iter_mut() {
        p.prev_position = p.position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particle_solver() -> PbdSolver {
        let cfg = default_pbd_solver_config();
        let mut s = new_pbd_solver(&cfg);
        pbd_add_particle(&mut s, [0.0, 0.0, 0.0], 1.0);
        pbd_add_particle(&mut s, [2.0, 0.0, 0.0], 1.0);
        pbd_add_distance_constraint(&mut s, 0, 1, 1.0, 1.0);
        s
    }

    #[test]
    fn test_particle_count() {
        let s = two_particle_solver();
        assert_eq!(pbd_particle_count(&s), 2);
    }

    #[test]
    fn test_constraint_count() {
        let s = two_particle_solver();
        assert_eq!(pbd_constraint_count(&s), 1);
    }

    #[test]
    fn test_pin_particle() {
        let mut s = two_particle_solver();
        pbd_pin_particle(&mut s, 0);
        assert!(s.particles[0].pinned);
        assert_eq!(s.particles[0].inv_mass, 0.0);
    }

    #[test]
    fn test_distance_constraint_converges() {
        let mut s = two_particle_solver();
        // Step many times; distance should approach rest_length=1.
        for _ in 0..100 {
            pbd_step(&mut s, 0.016, [0.0, 0.0, 0.0]);
        }
        let pa = pbd_particle_position(&s, 0);
        let pb = pbd_particle_position(&s, 1);
        let dx = pb[0] - pa[0];
        let dy = pb[1] - pa[1];
        let dz = pb[2] - pa[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        assert!((dist - 1.0).abs() < 0.1, "distance={}", dist);
    }

    #[test]
    fn test_gravity_moves_particle() {
        let cfg = default_pbd_solver_config();
        let mut s = new_pbd_solver(&cfg);
        pbd_add_particle(&mut s, [0.0, 10.0, 0.0], 1.0);
        let y0 = pbd_particle_position(&s, 0)[1];
        pbd_step(&mut s, 0.1, [0.0, -9.8, 0.0]);
        let y1 = pbd_particle_position(&s, 0)[1];
        assert!(y1 < y0, "particle should fall: y0={} y1={}", y0, y1);
    }

    #[test]
    fn test_reset_clears_velocity() {
        let mut s = two_particle_solver();
        pbd_step(&mut s, 0.1, [0.0, -9.8, 0.0]);
        pbd_reset(&mut s);
        // After reset prev_position == position so velocity is zero.
        for p in &s.particles {
            let dx = p.position[0] - p.prev_position[0];
            let dy = p.position[1] - p.prev_position[1];
            let dz = p.position[2] - p.prev_position[2];
            assert!((dx * dx + dy * dy + dz * dz).sqrt() < 1e-6);
        }
    }

    #[test]
    fn test_pinned_particle_does_not_move() {
        let mut s = two_particle_solver();
        pbd_pin_particle(&mut s, 0);
        let pos0_before = pbd_particle_position(&s, 0);
        for _ in 0..50 {
            pbd_step(&mut s, 0.016, [0.0, -9.8, 0.0]);
        }
        let pos0_after = pbd_particle_position(&s, 0);
        assert_eq!(pos0_before, pos0_after, "pinned particle moved");
    }

    #[test]
    fn test_default_config_values() {
        let cfg = default_pbd_solver_config();
        assert!(cfg.iterations > 0);
        assert!(cfg.damping >= 0.0 && cfg.damping <= 1.0);
    }

    #[test]
    fn test_add_particle_returns_index() {
        let cfg = default_pbd_solver_config();
        let mut s = new_pbd_solver(&cfg);
        let i0 = pbd_add_particle(&mut s, [0.0; 3], 1.0);
        let i1 = pbd_add_particle(&mut s, [1.0; 3], 1.0);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
    }
}
