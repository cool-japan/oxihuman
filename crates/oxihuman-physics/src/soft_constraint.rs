// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Soft/compliant constraint wrapper with spring-based enforcement.

/// The type of a soft constraint.
#[allow(dead_code)]
#[derive(Clone)]
pub enum SoftConstraintType {
    /// Maintain a rest distance between two particles.
    Distance { rest: f32, stiffness: f32 },
    /// Maintain a rest angle between three particles.
    Angle { rest_rad: f32, stiffness: f32 },
    /// Pull a single particle toward a target point.
    Point { target: [f32; 3], stiffness: f32 },
    /// Keep a particle on the positive side of a plane.
    Plane {
        normal: [f32; 3],
        offset: f32,
        stiffness: f32,
    },
}

/// A single soft constraint.
#[allow(dead_code)]
pub struct SoftConstraint {
    pub id: u32,
    pub particles: Vec<usize>,
    pub constraint_type: SoftConstraintType,
    pub damping: f32,
    pub enabled: bool,
    /// Accumulated Lagrange multiplier (XPBD-style).
    pub lambda: f32,
}

/// World state for a soft-constraint simulation.
#[allow(dead_code)]
pub struct SoftConstraintWorld {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub inv_masses: Vec<f32>,
    pub constraints: Vec<SoftConstraint>,
    pub next_id: u32,
}

/// Create a new empty `SoftConstraintWorld`.
#[allow(dead_code)]
pub fn new_soft_world() -> SoftConstraintWorld {
    SoftConstraintWorld {
        positions: Vec::new(),
        velocities: Vec::new(),
        inv_masses: Vec::new(),
        constraints: Vec::new(),
        next_id: 0,
    }
}

/// Add a particle and return its index.
#[allow(dead_code)]
pub fn add_soft_particle(world: &mut SoftConstraintWorld, pos: [f32; 3], inv_mass: f32) -> usize {
    let idx = world.positions.len();
    world.positions.push(pos);
    world.velocities.push([0.0; 3]);
    world.inv_masses.push(inv_mass);
    idx
}

/// Add a soft constraint and return its id.
#[allow(dead_code)]
pub fn add_soft_constraint(world: &mut SoftConstraintWorld, c: SoftConstraint) -> u32 {
    let id = c.id;
    world.constraints.push(c);
    id
}

/// Dot product helper.
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Length of a 3-vector.
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

/// Solve a distance constraint (XPBD soft spring).
#[allow(dead_code)]
pub fn solve_soft_distance(world: &mut SoftConstraintWorld, c_idx: usize, dt: f32) {
    let c = &world.constraints[c_idx];
    if !c.enabled || c.particles.len() < 2 {
        return;
    }
    let (stiffness, rest) = match c.constraint_type {
        SoftConstraintType::Distance { rest, stiffness } => (stiffness, rest),
        _ => return,
    };
    let i0 = c.particles[0];
    let i1 = c.particles[1];
    let p0 = world.positions[i0];
    let p1 = world.positions[i1];
    let w0 = world.inv_masses[i0];
    let w1 = world.inv_masses[i1];

    let diff = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let dist = len3(diff);
    if dist < 1e-12 {
        return;
    }
    let constraint_val = dist - rest;
    let alpha = 1.0 / (stiffness * dt * dt);
    let lambda_old = world.constraints[c_idx].lambda;
    let d_lambda = (-constraint_val - alpha * lambda_old) / (w0 + w1 + alpha);
    world.constraints[c_idx].lambda += d_lambda;

    let n = [diff[0] / dist, diff[1] / dist, diff[2] / dist];
    let pos0 = world.positions[i0];
    let pos1 = world.positions[i1];
    world.positions[i0] = [
        pos0[0] - w0 * d_lambda * n[0],
        pos0[1] - w0 * d_lambda * n[1],
        pos0[2] - w0 * d_lambda * n[2],
    ];
    world.positions[i1] = [
        pos1[0] + w1 * d_lambda * n[0],
        pos1[1] + w1 * d_lambda * n[1],
        pos1[2] + w1 * d_lambda * n[2],
    ];
}

/// Solve a point constraint: pull a particle toward a fixed target.
#[allow(dead_code)]
pub fn solve_soft_point(world: &mut SoftConstraintWorld, c_idx: usize, dt: f32) {
    let c = &world.constraints[c_idx];
    if !c.enabled || c.particles.is_empty() {
        return;
    }
    let (target, stiffness) = match c.constraint_type {
        SoftConstraintType::Point { target, stiffness } => (target, stiffness),
        _ => return,
    };
    let i0 = c.particles[0];
    let p = world.positions[i0];
    let w = world.inv_masses[i0];
    if w < 1e-12 {
        return;
    }
    let diff = [target[0] - p[0], target[1] - p[1], target[2] - p[2]];
    let dist = len3(diff);
    if dist < 1e-12 {
        return;
    }
    let alpha = 1.0 / (stiffness * dt * dt);
    let lambda_old = world.constraints[c_idx].lambda;
    let d_lambda = (-dist - alpha * lambda_old) / (w + alpha);
    world.constraints[c_idx].lambda += d_lambda;
    let n = [diff[0] / dist, diff[1] / dist, diff[2] / dist];
    let pos = world.positions[i0];
    world.positions[i0] = [
        pos[0] - w * d_lambda * n[0],
        pos[1] - w * d_lambda * n[1],
        pos[2] - w * d_lambda * n[2],
    ];
}

/// Solve a plane constraint: keep a particle above the plane.
#[allow(dead_code)]
pub fn solve_soft_plane(world: &mut SoftConstraintWorld, c_idx: usize, dt: f32) {
    let c = &world.constraints[c_idx];
    if !c.enabled || c.particles.is_empty() {
        return;
    }
    let (normal, offset, stiffness) = match c.constraint_type {
        SoftConstraintType::Plane {
            normal,
            offset,
            stiffness,
        } => (normal, offset, stiffness),
        _ => return,
    };
    let i0 = c.particles[0];
    let p = world.positions[i0];
    let w = world.inv_masses[i0];
    if w < 1e-12 {
        return;
    }
    let pen = dot3(p, normal) - offset;
    if pen >= 0.0 {
        return; // not penetrating
    }
    let alpha = 1.0 / (stiffness * dt * dt);
    let lambda_old = world.constraints[c_idx].lambda;
    // d_lambda > 0 when pen < 0 (penetrating).
    // Correction pushes the particle in +normal direction to resolve penetration.
    let d_lambda = (-pen - alpha * lambda_old) / (w + alpha);
    world.constraints[c_idx].lambda += d_lambda;
    let pos = world.positions[i0];
    world.positions[i0] = [
        pos[0] + w * d_lambda * normal[0],
        pos[1] + w * d_lambda * normal[1],
        pos[2] + w * d_lambda * normal[2],
    ];
}

/// Step the whole world: apply gravity, solve all constraints, update velocities.
#[allow(dead_code)]
pub fn step_soft_world(world: &mut SoftConstraintWorld, dt: f32, gravity: [f32; 3]) {
    let n = world.positions.len();

    // Reset lambdas
    for c in &mut world.constraints {
        c.lambda = 0.0;
    }

    // Predict positions (semi-implicit Euler)
    for i in 0..n {
        let w = world.inv_masses[i];
        if w < 1e-12 {
            continue;
        }
        world.velocities[i][0] += gravity[0] * dt;
        world.velocities[i][1] += gravity[1] * dt;
        world.velocities[i][2] += gravity[2] * dt;
        world.positions[i][0] += world.velocities[i][0] * dt;
        world.positions[i][1] += world.velocities[i][1] * dt;
        world.positions[i][2] += world.velocities[i][2] * dt;
    }

    // Solve constraints
    let n_constraints = world.constraints.len();
    for ci in 0..n_constraints {
        match world.constraints[ci].constraint_type.clone() {
            SoftConstraintType::Distance { .. } => {
                solve_soft_distance(world, ci, dt);
            }
            SoftConstraintType::Point { .. } => {
                solve_soft_point(world, ci, dt);
            }
            SoftConstraintType::Plane { .. } => {
                solve_soft_plane(world, ci, dt);
            }
            SoftConstraintType::Angle { .. } => { /* not implemented in this step */ }
        }
    }
}

/// Return the number of particles in the world.
#[allow(dead_code)]
pub fn soft_particle_count(world: &SoftConstraintWorld) -> usize {
    world.positions.len()
}

/// Return the number of constraints in the world.
#[allow(dead_code)]
pub fn soft_constraint_count(world: &SoftConstraintWorld) -> usize {
    world.constraints.len()
}

/// Return the current constraint violation (scalar distance from satisfaction).
#[allow(dead_code)]
pub fn soft_constraint_violation(world: &SoftConstraintWorld, c_idx: usize) -> f32 {
    let c = &world.constraints[c_idx];
    match &c.constraint_type {
        SoftConstraintType::Distance { rest, .. } => {
            if c.particles.len() < 2 {
                return 0.0;
            }
            let p0 = world.positions[c.particles[0]];
            let p1 = world.positions[c.particles[1]];
            let diff = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            (len3(diff) - rest).abs()
        }
        SoftConstraintType::Point { target, .. } => {
            if c.particles.is_empty() {
                return 0.0;
            }
            let p = world.positions[c.particles[0]];
            let diff = [target[0] - p[0], target[1] - p[1], target[2] - p[2]];
            len3(diff)
        }
        SoftConstraintType::Plane { normal, offset, .. } => {
            if c.particles.is_empty() {
                return 0.0;
            }
            let p = world.positions[c.particles[0]];
            let pen = dot3(p, *normal) - offset;
            (-pen).max(0.0)
        }
        SoftConstraintType::Angle { .. } => 0.0,
    }
}

/// Enable the constraint with the given id.
#[allow(dead_code)]
pub fn enable_soft_constraint(world: &mut SoftConstraintWorld, id: u32) {
    for c in &mut world.constraints {
        if c.id == id {
            c.enabled = true;
        }
    }
}

/// Disable the constraint with the given id.
#[allow(dead_code)]
pub fn disable_soft_constraint(world: &mut SoftConstraintWorld, id: u32) {
    for c in &mut world.constraints {
        if c.id == id {
            c.enabled = false;
        }
    }
}

/// Return total spring potential energy (0.5 * k * violation^2) for all constraints.
#[allow(dead_code)]
pub fn soft_total_energy(world: &SoftConstraintWorld) -> f32 {
    let mut total = 0.0f32;
    for (ci, c) in world.constraints.iter().enumerate() {
        if !c.enabled {
            continue;
        }
        let stiffness = match &c.constraint_type {
            SoftConstraintType::Distance { stiffness, .. } => *stiffness,
            SoftConstraintType::Point { stiffness, .. } => *stiffness,
            SoftConstraintType::Plane { stiffness, .. } => *stiffness,
            SoftConstraintType::Angle { stiffness, .. } => *stiffness,
        };
        let v = soft_constraint_violation(world, ci);
        total += 0.5 * stiffness * v * v;
    }
    total
}

// ── spring-pair API (body_a / body_b / rest_length) ──────────────────────────

/// Configuration for the spring-pair soft constraint API.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SoftConstraintConfig {
    /// Spring stiffness coefficient (N/m).
    pub stiffness: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Enable the constraint on creation.
    pub enabled: bool,
}

/// Return a [`SoftConstraintConfig`] with sensible defaults.
#[allow(dead_code)]
pub fn default_soft_constraint_config() -> SoftConstraintConfig {
    SoftConstraintConfig {
        stiffness: 500.0,
        damping: 0.1,
        enabled: true,
    }
}

/// Create a new [`SoftConstraint`] that links two rigid bodies by a spring.
///
/// `body_a` and `body_b` are stored in `particles[0]` and `particles[1]`.
/// `rest_length` is stored inside the `Distance` variant of `constraint_type`.
#[allow(dead_code)]
pub fn new_soft_constraint(
    body_a: usize,
    body_b: usize,
    rest_length: f32,
    cfg: &SoftConstraintConfig,
) -> SoftConstraint {
    SoftConstraint {
        id: 0,
        particles: vec![body_a, body_b],
        constraint_type: SoftConstraintType::Distance {
            rest: rest_length,
            stiffness: cfg.stiffness,
        },
        damping: cfg.damping,
        enabled: cfg.enabled,
        lambda: 0.0,
    }
}

/// Helper: distance between two 3-D points.
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Extract the spring stiffness from a constraint (Distance variant; 0 otherwise).
fn constraint_stiffness(c: &SoftConstraint) -> f32 {
    match c.constraint_type {
        SoftConstraintType::Distance { stiffness, .. } => stiffness,
        SoftConstraintType::Point { stiffness, .. } => stiffness,
        SoftConstraintType::Plane { stiffness, .. } => stiffness,
        SoftConstraintType::Angle { stiffness, .. } => stiffness,
    }
}

/// Extract the rest length from a Distance constraint; returns 0 otherwise.
fn constraint_rest(c: &SoftConstraint) -> f32 {
    match c.constraint_type {
        SoftConstraintType::Distance { rest, .. } => rest,
        _ => 0.0,
    }
}

/// Compute the spring force exerted on body_a by body_b for a distance constraint.
///
/// Returns the force vector applied to body_a (body_b receives the opposite).
#[allow(dead_code)]
pub fn soft_constraint_force(
    constraint: &SoftConstraint,
    pos_a: [f32; 3],
    pos_b: [f32; 3],
) -> [f32; 3] {
    let d = dist3(pos_a, pos_b);
    let rest = constraint_rest(constraint);
    let k = constraint_stiffness(constraint);
    if d < 1e-12 {
        return [0.0; 3];
    }
    let stretch = d - rest;
    let magnitude = k * stretch;
    // Direction from a to b, normalised.
    let dir = [
        (pos_b[0] - pos_a[0]) / d,
        (pos_b[1] - pos_a[1]) / d,
        (pos_b[2] - pos_a[2]) / d,
    ];
    // Force on a points toward b when spring is stretched.
    [dir[0] * magnitude, dir[1] * magnitude, dir[2] * magnitude]
}

/// Return the scalar constraint error: `|distance - rest_length|`.
#[allow(dead_code)]
pub fn soft_constraint_error(constraint: &SoftConstraint, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
    let d = dist3(pos_a, pos_b);
    let rest = constraint_rest(constraint);
    (d - rest).abs()
}

/// Return the two body indices `(body_a, body_b)` stored in the constraint.
#[allow(dead_code)]
pub fn soft_constraint_bodies(constraint: &SoftConstraint) -> (usize, usize) {
    let a = constraint.particles.first().copied().unwrap_or(0);
    let b = constraint.particles.get(1).copied().unwrap_or(0);
    (a, b)
}

/// Set the spring stiffness on a Distance constraint.
#[allow(dead_code)]
pub fn set_soft_stiffness(constraint: &mut SoftConstraint, stiffness: f32) {
    if let SoftConstraintType::Distance { rest, .. } = constraint.constraint_type {
        constraint.constraint_type = SoftConstraintType::Distance { rest, stiffness };
    }
}

/// Set the damping coefficient on a constraint.
#[allow(dead_code)]
pub fn set_soft_damping(constraint: &mut SoftConstraint, damping: f32) {
    constraint.damping = damping;
}

/// Return `true` if the constraint error is within `tol`.
#[allow(dead_code)]
pub fn soft_constraint_is_satisfied(
    constraint: &SoftConstraint,
    pos_a: [f32; 3],
    pos_b: [f32; 3],
    tol: f32,
) -> bool {
    soft_constraint_error(constraint, pos_a, pos_b) <= tol
}

/// Return the spring potential energy: `0.5 * k * (d - rest)^2`.
#[allow(dead_code)]
pub fn soft_constraint_potential_energy(
    constraint: &SoftConstraint,
    pos_a: [f32; 3],
    pos_b: [f32; 3],
) -> f32 {
    let k = constraint_stiffness(constraint);
    let err = soft_constraint_error(constraint, pos_a, pos_b);
    0.5 * k * err * err
}

/// Return the rest length of the constraint.
#[allow(dead_code)]
pub fn soft_constraint_rest_length(constraint: &SoftConstraint) -> f32 {
    constraint_rest(constraint)
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world_two_particles(dist: f32) -> SoftConstraintWorld {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [0.0, 0.0, 0.0], 1.0);
        add_soft_particle(&mut w, [dist, 0.0, 0.0], 1.0);
        w
    }

    #[test]
    fn test_new_soft_world() {
        let w = new_soft_world();
        assert_eq!(soft_particle_count(&w), 0);
        assert_eq!(soft_constraint_count(&w), 0);
    }

    #[test]
    fn test_add_particle() {
        let mut w = new_soft_world();
        let idx = add_soft_particle(&mut w, [1.0, 2.0, 3.0], 0.5);
        assert_eq!(idx, 0);
        assert_eq!(soft_particle_count(&w), 1);
    }

    #[test]
    fn test_add_constraint() {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [0.0; 3], 1.0);
        add_soft_particle(&mut w, [1.0, 0.0, 0.0], 1.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0, 1],
            constraint_type: SoftConstraintType::Distance {
                rest: 1.0,
                stiffness: 100.0,
            },
            damping: 0.1,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        assert_eq!(soft_constraint_count(&w), 1);
    }

    #[test]
    fn test_particle_count_multiple() {
        let mut w = new_soft_world();
        for _ in 0..5 {
            add_soft_particle(&mut w, [0.0; 3], 1.0);
        }
        assert_eq!(soft_particle_count(&w), 5);
    }

    #[test]
    fn test_constraint_count_multiple() {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [0.0; 3], 1.0);
        add_soft_particle(&mut w, [1.0, 0.0, 0.0], 1.0);
        for i in 0..3 {
            let c = SoftConstraint {
                id: i,
                particles: vec![0, 1],
                constraint_type: SoftConstraintType::Distance {
                    rest: 1.0,
                    stiffness: 100.0,
                },
                damping: 0.0,
                enabled: true,
                lambda: 0.0,
            };
            add_soft_constraint(&mut w, c);
        }
        assert_eq!(soft_constraint_count(&w), 3);
    }

    #[test]
    fn test_solve_soft_distance_reduces_violation() {
        let mut w = make_world_two_particles(2.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0, 1],
            constraint_type: SoftConstraintType::Distance {
                rest: 1.0,
                stiffness: 1000.0,
            },
            damping: 0.0,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        let v_before = soft_constraint_violation(&w, 0);
        solve_soft_distance(&mut w, 0, 0.016);
        let v_after = soft_constraint_violation(&w, 0);
        assert!(v_after < v_before, "violation should decrease after solve");
    }

    #[test]
    fn test_step_world_applies_gravity() {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [0.0, 10.0, 0.0], 1.0);
        step_soft_world(&mut w, 0.1, [0.0, -9.8, 0.0]);
        assert!(
            w.positions[0][1] < 10.0,
            "particle should fall under gravity"
        );
    }

    #[test]
    fn test_enable_disable_constraint() {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [0.0; 3], 1.0);
        let c = SoftConstraint {
            id: 42,
            particles: vec![0],
            constraint_type: SoftConstraintType::Point {
                target: [0.0; 3],
                stiffness: 100.0,
            },
            damping: 0.0,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        disable_soft_constraint(&mut w, 42);
        assert!(!w.constraints[0].enabled);
        enable_soft_constraint(&mut w, 42);
        assert!(w.constraints[0].enabled);
    }

    #[test]
    fn test_total_energy_zero_at_rest() {
        let mut w = make_world_two_particles(1.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0, 1],
            constraint_type: SoftConstraintType::Distance {
                rest: 1.0,
                stiffness: 100.0,
            },
            damping: 0.0,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        let e = soft_total_energy(&w);
        assert!(e < 1e-6, "energy should be near zero at rest distance");
    }

    #[test]
    fn test_total_energy_nonzero_when_stretched() {
        let mut w = make_world_two_particles(2.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0, 1],
            constraint_type: SoftConstraintType::Distance {
                rest: 1.0,
                stiffness: 100.0,
            },
            damping: 0.0,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        let e = soft_total_energy(&w);
        assert!(e > 0.0, "energy should be positive when stretched");
    }

    #[test]
    fn test_solve_soft_point() {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [5.0, 0.0, 0.0], 1.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0],
            constraint_type: SoftConstraintType::Point {
                target: [0.0, 0.0, 0.0],
                stiffness: 1000.0,
            },
            damping: 0.0,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        let v_before = soft_constraint_violation(&w, 0);
        solve_soft_point(&mut w, 0, 0.016);
        let v_after = soft_constraint_violation(&w, 0);
        assert!(v_after < v_before);
    }

    #[test]
    fn test_solve_soft_plane() {
        let mut w = new_soft_world();
        add_soft_particle(&mut w, [0.0, -1.0, 0.0], 1.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0],
            constraint_type: SoftConstraintType::Plane {
                normal: [0.0, 1.0, 0.0],
                offset: 0.0,
                stiffness: 1000.0,
            },
            damping: 0.0,
            enabled: true,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        let v_before = soft_constraint_violation(&w, 0);
        assert!(v_before > 0.0);
        solve_soft_plane(&mut w, 0, 0.016);
        let v_after = soft_constraint_violation(&w, 0);
        assert!(v_after < v_before);
    }

    #[test]
    fn test_disabled_constraint_not_solved() {
        let mut w = make_world_two_particles(2.0);
        let c = SoftConstraint {
            id: 0,
            particles: vec![0, 1],
            constraint_type: SoftConstraintType::Distance {
                rest: 1.0,
                stiffness: 1000.0,
            },
            damping: 0.0,
            enabled: false,
            lambda: 0.0,
        };
        add_soft_constraint(&mut w, c);
        let pos_before = w.positions[0];
        solve_soft_distance(&mut w, 0, 0.016);
        assert_eq!(
            w.positions[0], pos_before,
            "disabled constraint should not move particle"
        );
    }

    // ── spring-pair API tests ────────────────────────────────────────────────

    #[test]
    fn test_default_soft_constraint_config() {
        let cfg = default_soft_constraint_config();
        assert!(cfg.stiffness > 0.0);
        assert!(cfg.damping >= 0.0);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_new_soft_constraint_bodies() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(3, 7, 2.0, &cfg);
        let (a, b) = soft_constraint_bodies(&c);
        assert_eq!(a, 3);
        assert_eq!(b, 7);
    }

    #[test]
    fn test_soft_constraint_rest_length() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.5, &cfg);
        assert!((soft_constraint_rest_length(&c) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_soft_constraint_error_at_rest() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        let err = soft_constraint_error(&c, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(err < 1e-6, "error should be ~0 at rest distance");
    }

    #[test]
    fn test_soft_constraint_error_stretched() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        let err = soft_constraint_error(&c, [0.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
        assert!((err - 2.0).abs() < 1e-5, "error should be 2.0 (3 - 1)");
    }

    #[test]
    fn test_soft_constraint_is_satisfied_true() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        assert!(soft_constraint_is_satisfied(
            &c,
            [0.0, 0.0, 0.0],
            [1.001, 0.0, 0.0],
            0.01
        ));
    }

    #[test]
    fn test_soft_constraint_is_satisfied_false() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        assert!(!soft_constraint_is_satisfied(
            &c,
            [0.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            0.01
        ));
    }

    #[test]
    fn test_soft_constraint_potential_energy_at_rest() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        let e = soft_constraint_potential_energy(&c, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(e < 1e-6, "energy should be ~0 at rest");
    }

    #[test]
    fn test_soft_constraint_potential_energy_stretched() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        let e = soft_constraint_potential_energy(&c, [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        // 0.5 * 500 * 1^2 = 250
        assert!((e - 250.0).abs() < 1e-3, "energy={e}");
    }

    #[test]
    fn test_soft_constraint_force_direction() {
        let cfg = default_soft_constraint_config();
        let c = new_soft_constraint(0, 1, 1.0, &cfg);
        // Stretched along x: force on a should point in +x direction.
        let force = soft_constraint_force(&c, [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(force[0] > 0.0, "force x should be positive (toward b)");
        assert!(force[1].abs() < 1e-6);
        assert!(force[2].abs() < 1e-6);
    }

    #[test]
    fn test_set_soft_stiffness() {
        let cfg = default_soft_constraint_config();
        let mut c = new_soft_constraint(0, 1, 1.0, &cfg);
        set_soft_stiffness(&mut c, 999.0);
        let e = soft_constraint_potential_energy(&c, [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        // 0.5 * 999 * 1^2 = 499.5
        assert!((e - 499.5).abs() < 0.1, "energy={e}");
    }

    #[test]
    fn test_set_soft_damping() {
        let cfg = default_soft_constraint_config();
        let mut c = new_soft_constraint(0, 1, 1.0, &cfg);
        set_soft_damping(&mut c, 5.0);
        assert!((c.damping - 5.0).abs() < 1e-6);
    }
}
