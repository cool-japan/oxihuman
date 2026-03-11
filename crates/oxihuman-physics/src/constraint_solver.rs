// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic iterative constraint solver (Gauss-Seidel style).
//!
//! Provides a configurable solver that handles distance, angle, position,
//! and volume constraints on a set of particles.

// ── math helpers ─────────────────────────────────────────────────────────────

#[inline]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn v3_len(v: [f32; 3]) -> f32 {
    v3_dot(v, v).sqrt()
}

// ── public types ─────────────────────────────────────────────────────────────

/// The type of constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Distance constraint between two particles.
    Distance,
    /// Volume preservation constraint over a tetrahedron.
    Volume,
    /// Angle constraint at a hinge of three particles.
    Angle,
    /// Pin/position constraint anchoring a particle to a target.
    Position,
}

/// A generic constraint that can represent any of the constraint types.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GenericConstraint {
    /// Type of constraint.
    pub kind: ConstraintType,
    /// Particle indices involved (2 for distance, 3 for angle, 4 for volume, 1 for position).
    pub particles: Vec<usize>,
    /// Rest value (rest length, rest angle, rest volume, or target position magnitude).
    pub rest_value: f32,
    /// Compliance (inverse stiffness): 0 = rigid.
    pub compliance: f32,
    /// Target position for Position constraints.
    pub target: [f32; 3],
    /// Whether this constraint is active.
    pub active: bool,
}

/// Configuration for the constraint solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintSolverConfig {
    /// Number of Gauss-Seidel iterations per solve.
    pub iterations: usize,
    /// Global damping factor in \[0, 1\].
    pub damping: f32,
    /// Time step for compliance computation (XPBD).
    pub dt: f32,
    /// Over-relaxation factor.  1.0 = standard, >1 = over-relax.
    pub omega: f32,
}

/// Mutable state of the constraint solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintSolverState {
    /// Particle positions.
    pub positions: Vec<[f32; 3]>,
    /// Inverse masses (0 = infinite mass / pinned).
    pub inv_masses: Vec<f32>,
    /// Active constraints.
    pub constraints: Vec<GenericConstraint>,
    /// Configuration.
    pub config: ConstraintSolverConfig,
}

// ── public functions ─────────────────────────────────────────────────────────

/// Create a default solver configuration.
#[allow(dead_code)]
pub fn default_solver_config() -> ConstraintSolverConfig {
    ConstraintSolverConfig {
        iterations: 10,
        damping: 0.99,
        dt: 1.0 / 60.0,
        omega: 1.0,
    }
}

/// Create a new solver state with given positions and uniform mass.
#[allow(dead_code)]
pub fn new_solver_state(positions: Vec<[f32; 3]>, mass: f32) -> ConstraintSolverState {
    let inv_mass = if mass > 1e-12 { 1.0 / mass } else { 0.0 };
    let n = positions.len();
    ConstraintSolverState {
        positions,
        inv_masses: vec![inv_mass; n],
        constraints: Vec::new(),
        config: default_solver_config(),
    }
}

/// Add a constraint to the solver state.
#[allow(dead_code)]
pub fn add_constraint(state: &mut ConstraintSolverState, c: GenericConstraint) {
    state.constraints.push(c);
}

/// Remove the constraint at the given index, if valid.
#[allow(dead_code)]
pub fn remove_constraint(state: &mut ConstraintSolverState, index: usize) -> bool {
    if index < state.constraints.len() {
        state.constraints.remove(index);
        true
    } else {
        false
    }
}

/// Perform a single Gauss-Seidel iteration over all constraints.
/// Returns the total constraint error for this iteration.
#[allow(dead_code)]
pub fn solve_iteration(state: &mut ConstraintSolverState) -> f32 {
    let mut total_error = 0.0f32;
    let dt = state.config.dt;
    let omega = state.config.omega;

    for ci in 0..state.constraints.len() {
        if !state.constraints[ci].active {
            continue;
        }
        let err = match state.constraints[ci].kind {
            ConstraintType::Distance => {
                let particles = state.constraints[ci].particles.clone();
                let rest = state.constraints[ci].rest_value;
                let compliance = state.constraints[ci].compliance;
                project_distance(
                    &mut state.positions,
                    &state.inv_masses,
                    particles[0],
                    particles[1],
                    rest,
                    compliance,
                    dt,
                    omega,
                )
            }
            ConstraintType::Angle => {
                let particles = state.constraints[ci].particles.clone();
                let rest = state.constraints[ci].rest_value;
                let compliance = state.constraints[ci].compliance;
                project_angle(
                    &mut state.positions,
                    &state.inv_masses,
                    particles[0],
                    particles[1],
                    particles[2],
                    rest,
                    compliance,
                    dt,
                    omega,
                )
            }
            ConstraintType::Position => {
                let particles = state.constraints[ci].particles.clone();
                let target = state.constraints[ci].target;
                let compliance = state.constraints[ci].compliance;
                project_position(
                    &mut state.positions,
                    &state.inv_masses,
                    particles[0],
                    target,
                    compliance,
                    dt,
                    omega,
                )
            }
            ConstraintType::Volume => {
                // Simplified volume constraint: just report error.
                let particles = state.constraints[ci].particles.clone();
                let rest = state.constraints[ci].rest_value;
                if particles.len() >= 4 {
                    let vol = tet_volume_4(
                        state.positions[particles[0]],
                        state.positions[particles[1]],
                        state.positions[particles[2]],
                        state.positions[particles[3]],
                    );
                    (vol - rest).abs()
                } else {
                    0.0
                }
            }
        };
        total_error += err;
    }

    total_error
}

/// Run N iterations of the solver.  Returns the final total error.
#[allow(dead_code)]
pub fn solve_n_iterations(state: &mut ConstraintSolverState, n: usize) -> f32 {
    let mut err = 0.0;
    for _ in 0..n {
        err = solve_iteration(state);
    }
    err
}

/// Project a distance constraint between two particles.
/// Returns the constraint error before projection.
#[allow(dead_code)]
pub fn distance_constraint_project(
    positions: &mut [[f32; 3]],
    inv_masses: &[f32],
    a: usize,
    b: usize,
    rest_length: f32,
    compliance: f32,
    dt: f32,
) -> f32 {
    project_distance(
        positions,
        inv_masses,
        a,
        b,
        rest_length,
        compliance,
        dt,
        1.0,
    )
}

/// Project an angle constraint at vertex `b` between edges a-b and b-c.
/// Returns the constraint error.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn angle_constraint_project(
    positions: &mut [[f32; 3]],
    inv_masses: &[f32],
    a: usize,
    b: usize,
    c: usize,
    rest_angle: f32,
    compliance: f32,
    dt: f32,
) -> f32 {
    project_angle(
        positions, inv_masses, a, b, c, rest_angle, compliance, dt, 1.0,
    )
}

/// Project a position constraint, pulling a particle toward a target.
/// Returns the constraint error.
#[allow(dead_code)]
pub fn position_constraint_project(
    positions: &mut [[f32; 3]],
    inv_masses: &[f32],
    particle: usize,
    target: [f32; 3],
    compliance: f32,
    dt: f32,
) -> f32 {
    project_position(positions, inv_masses, particle, target, compliance, dt, 1.0)
}

/// Return the number of constraints in the solver.
#[allow(dead_code)]
pub fn constraint_count(state: &ConstraintSolverState) -> usize {
    state.constraints.len()
}

/// Compute the total constraint error without modifying positions.
#[allow(dead_code)]
pub fn total_constraint_error(state: &ConstraintSolverState) -> f32 {
    let mut total = 0.0f32;
    for c in &state.constraints {
        if !c.active {
            continue;
        }
        let err = match c.kind {
            ConstraintType::Distance => {
                if c.particles.len() >= 2 {
                    let d = v3_len(v3_sub(
                        state.positions[c.particles[0]],
                        state.positions[c.particles[1]],
                    ));
                    (d - c.rest_value).abs()
                } else {
                    0.0
                }
            }
            ConstraintType::Angle => {
                if c.particles.len() >= 3 {
                    let angle = compute_angle(
                        state.positions[c.particles[0]],
                        state.positions[c.particles[1]],
                        state.positions[c.particles[2]],
                    );
                    (angle - c.rest_value).abs()
                } else {
                    0.0
                }
            }
            ConstraintType::Position => {
                if !c.particles.is_empty() {
                    v3_len(v3_sub(state.positions[c.particles[0]], c.target))
                } else {
                    0.0
                }
            }
            ConstraintType::Volume => {
                if c.particles.len() >= 4 {
                    let vol = tet_volume_4(
                        state.positions[c.particles[0]],
                        state.positions[c.particles[1]],
                        state.positions[c.particles[2]],
                        state.positions[c.particles[3]],
                    );
                    (vol - c.rest_value).abs()
                } else {
                    0.0
                }
            }
        };
        total += err;
    }
    total
}

/// Reset solver: move all particles back to provided positions.
#[allow(dead_code)]
pub fn reset_solver(state: &mut ConstraintSolverState, positions: &[[f32; 3]]) {
    let n = state.positions.len().min(positions.len());
    state.positions[..n].copy_from_slice(&positions[..n]);
}

/// Change the number of solver iterations.
#[allow(dead_code)]
pub fn set_solver_iterations(state: &mut ConstraintSolverState, iterations: usize) {
    state.config.iterations = iterations;
}

/// Compute constraint compliance: effective stiffness after considering
/// time step.  Returns alpha_tilde = compliance / dt^2.
#[allow(dead_code)]
pub fn constraint_compliance(compliance: f32, dt: f32) -> f32 {
    if dt.abs() < 1e-12 {
        return f32::INFINITY;
    }
    compliance / (dt * dt)
}

// ── internal helpers ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn project_distance(
    positions: &mut [[f32; 3]],
    inv_masses: &[f32],
    a: usize,
    b: usize,
    rest_length: f32,
    compliance: f32,
    dt: f32,
    omega: f32,
) -> f32 {
    let diff = v3_sub(positions[b], positions[a]);
    let dist = v3_len(diff);
    if dist < 1e-12 {
        return 0.0;
    }
    let c_val = dist - rest_length;
    let w = inv_masses[a] + inv_masses[b];
    let alpha = constraint_compliance(compliance, dt);
    let denom = w + alpha;
    if denom < 1e-12 {
        return c_val.abs();
    }
    let delta_lambda = -c_val / denom;
    let grad = v3_scale(diff, 1.0 / dist);
    let corr = v3_scale(grad, delta_lambda * omega);
    positions[a] = v3_sub(positions[a], v3_scale(corr, inv_masses[a]));
    positions[b] = v3_add(positions[b], v3_scale(corr, inv_masses[b]));
    c_val.abs()
}

#[allow(clippy::too_many_arguments)]
fn project_angle(
    positions: &mut [[f32; 3]],
    inv_masses: &[f32],
    a: usize,
    b: usize,
    c: usize,
    rest_angle: f32,
    compliance: f32,
    dt: f32,
    omega: f32,
) -> f32 {
    let angle = compute_angle(positions[a], positions[b], positions[c]);
    let error = angle - rest_angle;
    if error.abs() < 1e-7 {
        return 0.0;
    }

    // Simple penalty: push a and c along tangent directions.
    let ba = v3_sub(positions[a], positions[b]);
    let bc = v3_sub(positions[c], positions[b]);
    let la = v3_len(ba);
    let lc = v3_len(bc);
    if la < 1e-12 || lc < 1e-12 {
        return error.abs();
    }

    let alpha = constraint_compliance(compliance, dt);
    let w = inv_masses[a] + inv_masses[b] + inv_masses[c];
    let denom = w + alpha;
    if denom < 1e-12 {
        return error.abs();
    }

    let strength = -error / denom * omega * 0.1;

    // Move a perpendicular to ba in the plane of the triangle.
    let perp_a = v3_sub(bc, v3_scale(ba, v3_dot(bc, ba) / (la * la)));
    let lp = v3_len(perp_a);
    if lp > 1e-12 {
        let dir = v3_scale(perp_a, 1.0 / lp);
        positions[a] = v3_add(positions[a], v3_scale(dir, strength * inv_masses[a]));
    }

    let perp_c = v3_sub(ba, v3_scale(bc, v3_dot(ba, bc) / (lc * lc)));
    let lpc = v3_len(perp_c);
    if lpc > 1e-12 {
        let dir = v3_scale(perp_c, 1.0 / lpc);
        positions[c] = v3_add(positions[c], v3_scale(dir, strength * inv_masses[c]));
    }

    error.abs()
}

fn project_position(
    positions: &mut [[f32; 3]],
    inv_masses: &[f32],
    particle: usize,
    target: [f32; 3],
    compliance: f32,
    dt: f32,
    omega: f32,
) -> f32 {
    let diff = v3_sub(target, positions[particle]);
    let dist = v3_len(diff);
    if dist < 1e-12 {
        return 0.0;
    }
    let w = inv_masses[particle];
    let alpha = constraint_compliance(compliance, dt);
    let denom = w + alpha;
    if denom < 1e-12 {
        return dist;
    }
    let strength = dist / denom * omega;
    let dir = v3_scale(diff, 1.0 / dist);
    positions[particle] = v3_add(positions[particle], v3_scale(dir, strength * w));
    dist
}

fn compute_angle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ba = v3_sub(a, b);
    let bc = v3_sub(c, b);
    let la = v3_len(ba);
    let lc = v3_len(bc);
    if la < 1e-12 || lc < 1e-12 {
        return 0.0;
    }
    let cos_angle = (v3_dot(ba, bc) / (la * lc)).clamp(-1.0, 1.0);
    cos_angle.acos()
}

fn tet_volume_4(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    let ab = v3_sub(b, a);
    let ac = v3_sub(c, a);
    let ad = v3_sub(d, a);
    let cross = [
        ac[1] * ad[2] - ac[2] * ad[1],
        ac[2] * ad[0] - ac[0] * ad[2],
        ac[0] * ad[1] - ac[1] * ad[0],
    ];
    (v3_dot(ab, cross) / 6.0).abs()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particle_state() -> ConstraintSolverState {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        new_solver_state(positions, 1.0)
    }

    #[allow(dead_code)]
    fn three_particle_state() -> ConstraintSolverState {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        new_solver_state(positions, 1.0)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_solver_config();
        assert_eq!(cfg.iterations, 10);
        assert!((cfg.damping - 0.99).abs() < 1e-6);
    }

    #[test]
    fn test_new_solver_state() {
        let state = two_particle_state();
        assert_eq!(state.positions.len(), 2);
        assert_eq!(state.inv_masses.len(), 2);
        assert!(state.constraints.is_empty());
    }

    #[test]
    fn test_add_constraint() {
        let mut state = two_particle_state();
        let c = GenericConstraint {
            kind: ConstraintType::Distance,
            particles: vec![0, 1],
            rest_value: 1.0,
            compliance: 0.0,
            target: [0.0; 3],
            active: true,
        };
        add_constraint(&mut state, c);
        assert_eq!(constraint_count(&state), 1);
    }

    #[test]
    fn test_remove_constraint() {
        let mut state = two_particle_state();
        let c = GenericConstraint {
            kind: ConstraintType::Distance,
            particles: vec![0, 1],
            rest_value: 1.0,
            compliance: 0.0,
            target: [0.0; 3],
            active: true,
        };
        add_constraint(&mut state, c);
        assert!(remove_constraint(&mut state, 0));
        assert_eq!(constraint_count(&state), 0);
    }

    #[test]
    fn test_remove_constraint_invalid_index() {
        let mut state = two_particle_state();
        assert!(!remove_constraint(&mut state, 5));
    }

    #[test]
    fn test_distance_constraint_project() {
        let mut pos = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let inv = vec![1.0, 1.0];
        let err = distance_constraint_project(&mut pos, &inv, 0, 1, 1.0, 0.0, 1.0 / 60.0);
        assert!(err > 0.0);
        // After projection, distance should be closer to 1.0.
        let new_dist = v3_len(v3_sub(pos[1], pos[0]));
        assert!(new_dist < 2.0);
    }

    #[test]
    fn test_angle_constraint_project() {
        let mut pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let inv = vec![1.0, 1.0, 1.0];
        let rest = std::f32::consts::FRAC_PI_2;
        let err = angle_constraint_project(&mut pos, &inv, 0, 1, 2, rest, 0.0, 1.0 / 60.0);
        // The angle is already pi/2, so error should be small.
        assert!(err < 0.1);
    }

    #[test]
    fn test_position_constraint_project() {
        let mut pos = vec![[0.0f32, 0.0, 0.0]];
        let inv = vec![1.0];
        let err = position_constraint_project(&mut pos, &inv, 0, [1.0, 0.0, 0.0], 0.0, 1.0 / 60.0);
        assert!(err > 0.0);
        // Particle should have moved toward target.
        assert!(pos[0][0] > 0.0);
    }

    #[test]
    fn test_solve_iteration_distance() {
        let mut state = two_particle_state();
        let c = GenericConstraint {
            kind: ConstraintType::Distance,
            particles: vec![0, 1],
            rest_value: 1.0,
            compliance: 0.0,
            target: [0.0; 3],
            active: true,
        };
        add_constraint(&mut state, c);
        let err = solve_iteration(&mut state);
        assert!(err > 0.0);
    }

    #[test]
    fn test_solve_n_iterations() {
        let mut state = two_particle_state();
        let c = GenericConstraint {
            kind: ConstraintType::Distance,
            particles: vec![0, 1],
            rest_value: 1.0,
            compliance: 0.0,
            target: [0.0; 3],
            active: true,
        };
        add_constraint(&mut state, c);
        let err = solve_n_iterations(&mut state, 10);
        // After 10 iterations, error should be very small.
        assert!(err < 0.01);
    }

    #[test]
    fn test_total_constraint_error() {
        let mut state = two_particle_state();
        let c = GenericConstraint {
            kind: ConstraintType::Distance,
            particles: vec![0, 1],
            rest_value: 1.0,
            compliance: 0.0,
            target: [0.0; 3],
            active: true,
        };
        add_constraint(&mut state, c);
        let err = total_constraint_error(&state);
        assert!((err - 1.0).abs() < 1e-5); // distance is 2, rest is 1 → error = 1
    }

    #[test]
    fn test_reset_solver() {
        let mut state = two_particle_state();
        state.positions[0] = [5.0, 5.0, 5.0];
        let orig = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        reset_solver(&mut state, &orig);
        assert!((state.positions[0][0]).abs() < 1e-6);
    }

    #[test]
    fn test_set_solver_iterations() {
        let mut state = two_particle_state();
        set_solver_iterations(&mut state, 20);
        assert_eq!(state.config.iterations, 20);
    }

    #[test]
    fn test_constraint_compliance_value() {
        let alpha = constraint_compliance(0.001, 1.0 / 60.0);
        assert!(alpha > 0.0);
    }

    #[test]
    fn test_constraint_compliance_zero_dt() {
        let alpha = constraint_compliance(0.001, 0.0);
        assert!(alpha.is_infinite());
    }

    #[test]
    fn test_inactive_constraint_ignored() {
        let mut state = two_particle_state();
        let c = GenericConstraint {
            kind: ConstraintType::Distance,
            particles: vec![0, 1],
            rest_value: 1.0,
            compliance: 0.0,
            target: [0.0; 3],
            active: false,
        };
        add_constraint(&mut state, c);
        let orig = state.positions.clone();
        let _err = solve_iteration(&mut state);
        // Positions should not change since constraint is inactive.
        assert!((state.positions[0][0] - orig[0][0]).abs() < 1e-9);
    }
}
