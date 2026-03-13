// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Position-based cloth simulation (PBD).
//!
//! Implements a standalone cloth solver using position-based dynamics.
//! Distinct from `rope_cloth` which is a hybrid rope/cloth body; this module
//! focuses on a grid-based cloth mesh with stretch and shear constraints.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for the cloth simulation.
pub struct ClothSimConfig {
    /// Gravity acceleration vector.
    pub gravity: [f32; 3],
    /// Default mass per particle (kg).
    pub particle_mass: f32,
    /// Number of PBD constraint-projection iterations per step.
    pub solver_iterations: u32,
    /// Stiffness for stretch constraints in [0, 1].
    pub stretch_stiffness: f32,
    /// Stiffness for shear constraints in [0, 1].
    pub shear_stiffness: f32,
    /// Velocity damping factor applied each step.
    pub damping: f32,
}

/// A single cloth particle.
pub struct ClothParticle {
    /// Current position.
    pub position: [f32; 3],
    /// Predicted position used during the PBD solve.
    pub predicted: [f32; 3],
    /// Current velocity.
    pub velocity: [f32; 3],
    /// Inverse mass (0 = pinned/infinite mass).
    pub inv_mass: f32,
}

/// A distance constraint between two particles.
struct ClothConstraint {
    /// Index of particle A.
    a: usize,
    /// Index of particle B.
    b: usize,
    /// Rest length.
    rest_length: f32,
    /// Stiffness in [0, 1].
    stiffness: f32,
}

/// The full cloth simulation mesh.
pub struct ClothSimMesh {
    /// All particles.
    pub particles: Vec<ClothParticle>,
    /// Distance constraints (stretch + shear).
    constraints: Vec<ClothConstraint>,
    /// Gravity.
    pub gravity: [f32; 3],
    /// Solver iterations.
    pub solver_iterations: u32,
    /// Damping.
    pub damping: f32,
    /// Count of stretch constraints (stored first).
    stretch_count: usize,
    /// Count of shear constraints (stored after stretch).
    shear_count: usize,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return a `ClothSimConfig` with sensible defaults.
pub fn default_cloth_sim_config() -> ClothSimConfig {
    ClothSimConfig {
        gravity: [0.0, -9.81, 0.0],
        particle_mass: 0.1,
        solver_iterations: 10,
        stretch_stiffness: 0.9,
        shear_stiffness: 0.5,
        damping: 0.01,
    }
}

/// Create a new empty cloth simulation mesh from a config.
pub fn new_cloth_sim_mesh(cfg: &ClothSimConfig) -> ClothSimMesh {
    ClothSimMesh {
        particles: vec![],
        constraints: vec![],
        gravity: cfg.gravity,
        solver_iterations: cfg.solver_iterations,
        damping: cfg.damping,
        stretch_count: 0,
        shear_count: 0,
    }
}

/// Add a particle at `position` with the given mass.
///
/// Returns the index of the newly added particle.
pub fn add_cloth_particle(cloth: &mut ClothSimMesh, position: [f32; 3], mass: f32) -> usize {
    let inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    let idx = cloth.particles.len();
    cloth.particles.push(ClothParticle {
        position,
        predicted: position,
        velocity: [0.0; 3],
        inv_mass,
    });
    idx
}

/// Add a stretch (structural) constraint between particles `a` and `b`.
///
/// Rest length defaults to the current distance between the particles.
pub fn add_stretch_constraint(cloth: &mut ClothSimMesh, a: usize, b: usize, stiffness: f32) {
    let rest_length = if a < cloth.particles.len() && b < cloth.particles.len() {
        len3(sub3(cloth.particles[b].position, cloth.particles[a].position))
    } else {
        1.0
    };
    cloth.constraints.insert(
        cloth.stretch_count,
        ClothConstraint {
            a,
            b,
            rest_length,
            stiffness: stiffness.clamp(0.0, 1.0),
        },
    );
    cloth.stretch_count += 1;
}

/// Add a shear constraint between particles `a` and `b` (diagonal of a quad).
///
/// Rest length defaults to the current distance between the particles.
pub fn add_shear_constraint(cloth: &mut ClothSimMesh, a: usize, b: usize, stiffness: f32) {
    let rest_length = if a < cloth.particles.len() && b < cloth.particles.len() {
        len3(sub3(cloth.particles[b].position, cloth.particles[a].position))
    } else {
        1.0
    };
    cloth.constraints.push(ClothConstraint {
        a,
        b,
        rest_length,
        stiffness: stiffness.clamp(0.0, 1.0),
    });
    cloth.shear_count += 1;
}

/// Return the number of particles in the cloth.
pub fn cloth_particle_count(cloth: &ClothSimMesh) -> usize {
    cloth.particles.len()
}

/// Return the total number of constraints (stretch + shear).
pub fn cloth_constraint_count(cloth: &ClothSimMesh) -> usize {
    cloth.constraints.len()
}

/// Pin particle `idx` so that it cannot move (set inv_mass to 0).
pub fn pin_cloth_particle(cloth: &mut ClothSimMesh, idx: usize) {
    if let Some(p) = cloth.particles.get_mut(idx) {
        p.inv_mass = 0.0;
        p.velocity = [0.0; 3];
    }
}

/// Unpin particle `idx`, restoring its inv_mass from `mass`.
pub fn unpin_cloth_particle(cloth: &mut ClothSimMesh, idx: usize, mass: f32) {
    if let Some(p) = cloth.particles.get_mut(idx) {
        p.inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    }
}

/// Apply gravity to all free particles by updating their velocities.
pub fn apply_gravity_cloth(cloth: &mut ClothSimMesh, dt: f32) {
    let g = cloth.gravity;
    for p in cloth.particles.iter_mut() {
        if p.inv_mass < 1e-10 {
            continue;
        }
        p.velocity = add3(p.velocity, scale3(g, dt));
    }
}

/// Perform one full PBD update step of duration `dt`.
///
/// Steps:
/// 1. Apply gravity to velocities.
/// 2. Predict positions: x* = x + v * dt.
/// 3. Project constraints (iterative).
/// 4. Update velocities from position delta.
/// 5. Apply damping.
#[allow(clippy::too_many_arguments)]
pub fn update_cloth_sim(cloth: &mut ClothSimMesh, dt: f32) {
    let n = cloth.particles.len();
    if n == 0 {
        return;
    }

    let g = cloth.gravity;
    let damp = 1.0 - cloth.damping.clamp(0.0, 1.0);

    // 1. Apply external forces (gravity) to velocity
    for p in cloth.particles.iter_mut() {
        if p.inv_mass < 1e-10 {
            p.velocity = [0.0; 3];
            continue;
        }
        p.velocity = add3(p.velocity, scale3(g, dt));
        p.velocity = scale3(p.velocity, damp);
    }

    // 2. Predict positions
    for p in cloth.particles.iter_mut() {
        if p.inv_mass < 1e-10 {
            p.predicted = p.position;
        } else {
            p.predicted = add3(p.position, scale3(p.velocity, dt));
        }
    }

    // 3. Project constraints
    for _ in 0..cloth.solver_iterations {
        for c in cloth.constraints.iter() {
            let (a, b) = (c.a, c.b);
            if a >= n || b >= n {
                continue;
            }
            let pa = cloth.particles[a].predicted;
            let pb = cloth.particles[b].predicted;
            let diff = sub3(pb, pa);
            let dist = len3(diff);
            if dist < 1e-10 {
                continue;
            }
            let err = dist - c.rest_length;
            let w_a = cloth.particles[a].inv_mass;
            let w_b = cloth.particles[b].inv_mass;
            let w_total = w_a + w_b;
            if w_total < 1e-10 {
                continue;
            }
            let correction = scale3(diff, (c.stiffness * err) / (w_total * dist));
            if w_a > 1e-10 {
                cloth.particles[a].predicted =
                    add3(pa, scale3(correction, w_a));
            }
            if w_b > 1e-10 {
                cloth.particles[b].predicted =
                    sub3(pb, scale3(correction, w_b));
            }
        }
    }

    // 4. Update velocities and commit positions
    let inv_dt = 1.0 / dt.max(1e-6);
    for p in cloth.particles.iter_mut() {
        if p.inv_mass < 1e-10 {
            continue;
        }
        p.velocity = scale3(sub3(p.predicted, p.position), inv_dt);
        p.position = p.predicted;
    }
}

/// Compute the total mechanical energy (kinetic + gravitational potential)
/// of the cloth.
pub fn cloth_sim_energy(cloth: &ClothSimMesh) -> f32 {
    let g_mag = len3(cloth.gravity);
    let mut total = 0.0f32;
    for p in &cloth.particles {
        if p.inv_mass < 1e-10 {
            continue;
        }
        let mass = 1.0 / p.inv_mass;
        let v2 = dot3(p.velocity, p.velocity);
        total += 0.5 * mass * v2;
        // Potential relative to y = 0 (height component)
        total += mass * g_mag * p.position[1];
    }
    total
}

/// Compute the axis-aligned bounding box of all cloth particles.
///
/// Returns `[min_x, min_y, min_z, max_x, max_y, max_z]`.
pub fn cloth_sim_aabb(cloth: &ClothSimMesh) -> [f32; 6] {
    if cloth.particles.is_empty() {
        return [0.0; 6];
    }
    let mut mn = cloth.particles[0].position;
    let mut mx = cloth.particles[0].position;
    for p in cloth.particles.iter().skip(1) {
        for k in 0..3 {
            if p.position[k] < mn[k] {
                mn[k] = p.position[k];
            }
            if p.position[k] > mx[k] {
                mx[k] = p.position[k];
            }
        }
    }
    [mn[0], mn[1], mn[2], mx[0], mx[1], mx[2]]
}

/// Reset all particles to zero velocity; pinned particles keep their positions.
pub fn reset_cloth_sim(cloth: &mut ClothSimMesh) {
    for p in cloth.particles.iter_mut() {
        p.velocity = [0.0; 3];
        p.predicted = p.position;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a 2x2 cloth patch (4 particles, 4 stretch + 2 shear constraints).
    fn patch() -> ClothSimMesh {
        let cfg = default_cloth_sim_config();
        let mut cloth = new_cloth_sim_mesh(&cfg);
        // Grid:
        //  3 - 2
        //  |   |
        //  0 - 1
        add_cloth_particle(&mut cloth, [0.0, 0.0, 0.0], 0.1);
        add_cloth_particle(&mut cloth, [1.0, 0.0, 0.0], 0.1);
        add_cloth_particle(&mut cloth, [1.0, 1.0, 0.0], 0.1);
        add_cloth_particle(&mut cloth, [0.0, 1.0, 0.0], 0.1);
        // Stretch edges
        add_stretch_constraint(&mut cloth, 0, 1, 0.9);
        add_stretch_constraint(&mut cloth, 1, 2, 0.9);
        add_stretch_constraint(&mut cloth, 2, 3, 0.9);
        add_stretch_constraint(&mut cloth, 3, 0, 0.9);
        // Shear diagonals
        add_shear_constraint(&mut cloth, 0, 2, 0.5);
        add_shear_constraint(&mut cloth, 1, 3, 0.5);
        cloth
    }

    // -----------------------------------------------------------------------
    // default_cloth_sim_config
    // -----------------------------------------------------------------------

    #[test]
    fn default_config_gravity_downward() {
        let cfg = default_cloth_sim_config();
        assert!(cfg.gravity[1] < 0.0);
    }

    #[test]
    fn default_config_stiffness_in_range() {
        let cfg = default_cloth_sim_config();
        assert!(cfg.stretch_stiffness >= 0.0 && cfg.stretch_stiffness <= 1.0);
        assert!(cfg.shear_stiffness >= 0.0 && cfg.shear_stiffness <= 1.0);
    }

    // -----------------------------------------------------------------------
    // new_cloth_sim_mesh
    // -----------------------------------------------------------------------

    #[test]
    fn new_mesh_starts_empty() {
        let cfg = default_cloth_sim_config();
        let cloth = new_cloth_sim_mesh(&cfg);
        assert_eq!(cloth_particle_count(&cloth), 0);
        assert_eq!(cloth_constraint_count(&cloth), 0);
    }

    // -----------------------------------------------------------------------
    // add_cloth_particle
    // -----------------------------------------------------------------------

    #[test]
    fn add_particle_increments_count() {
        let mut cloth = patch();
        let before = cloth_particle_count(&cloth);
        add_cloth_particle(&mut cloth, [5.0, 0.0, 0.0], 0.1);
        assert_eq!(cloth_particle_count(&cloth), before + 1);
    }

    #[test]
    fn added_particle_has_correct_position() {
        let cfg = default_cloth_sim_config();
        let mut cloth = new_cloth_sim_mesh(&cfg);
        let idx = add_cloth_particle(&mut cloth, [3.0, 4.0, 5.0], 1.0);
        let pos = cloth.particles[idx].position;
        assert!((pos[0] - 3.0).abs() < 1e-6);
        assert!((pos[1] - 4.0).abs() < 1e-6);
        assert!((pos[2] - 5.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // add_stretch_constraint / add_shear_constraint / cloth_constraint_count
    // -----------------------------------------------------------------------

    #[test]
    fn patch_has_correct_constraint_count() {
        let cloth = patch();
        assert_eq!(cloth_constraint_count(&cloth), 6); // 4 stretch + 2 shear
    }

    #[test]
    fn stretch_count_tracked_separately() {
        let cloth = patch();
        assert_eq!(cloth.stretch_count, 4);
        assert_eq!(cloth.shear_count, 2);
    }

    // -----------------------------------------------------------------------
    // pin / unpin
    // -----------------------------------------------------------------------

    #[test]
    fn pin_particle_zeros_inv_mass() {
        let mut cloth = patch();
        pin_cloth_particle(&mut cloth, 0);
        assert!(cloth.particles[0].inv_mass < 1e-10);
    }

    #[test]
    fn unpin_particle_restores_inv_mass() {
        let mut cloth = patch();
        pin_cloth_particle(&mut cloth, 0);
        unpin_cloth_particle(&mut cloth, 0, 0.1);
        assert!(cloth.particles[0].inv_mass > 1.0, "inv_mass={}", cloth.particles[0].inv_mass);
    }

    #[test]
    fn pin_out_of_range_does_not_panic() {
        let mut cloth = patch();
        pin_cloth_particle(&mut cloth, 999);
    }

    // -----------------------------------------------------------------------
    // apply_gravity_cloth
    // -----------------------------------------------------------------------

    #[test]
    fn apply_gravity_changes_velocity_of_free_particles() {
        let mut cloth = patch();
        let v0_before = cloth.particles[0].velocity[1];
        apply_gravity_cloth(&mut cloth, 0.01);
        let v0_after = cloth.particles[0].velocity[1];
        assert!(v0_after < v0_before, "gravity should reduce y-velocity");
    }

    #[test]
    fn apply_gravity_does_not_move_pinned_particles() {
        let mut cloth = patch();
        pin_cloth_particle(&mut cloth, 0);
        apply_gravity_cloth(&mut cloth, 0.01);
        let v = cloth.particles[0].velocity;
        let v2 = dot3(v, v);
        assert!(v2 < 1e-10, "pinned particle should have zero velocity");
    }

    // -----------------------------------------------------------------------
    // update_cloth_sim
    // -----------------------------------------------------------------------

    #[test]
    fn update_preserves_particle_count() {
        let mut cloth = patch();
        update_cloth_sim(&mut cloth, 0.01);
        assert_eq!(cloth_particle_count(&cloth), 4);
    }

    #[test]
    fn update_moves_free_particles() {
        let mut cloth = patch();
        let pos_before = cloth.particles[0].position;
        update_cloth_sim(&mut cloth, 0.01);
        let pos_after = cloth.particles[0].position;
        // Under gravity, particles should move
        let delta = len3(sub3(pos_after, pos_before));
        assert!(delta > 0.0, "particles should move under gravity");
    }

    #[test]
    fn update_keeps_pinned_particle_fixed() {
        let mut cloth = patch();
        pin_cloth_particle(&mut cloth, 3);
        let pos_before = cloth.particles[3].position;
        for _ in 0..10 {
            update_cloth_sim(&mut cloth, 0.01);
        }
        let pos_after = cloth.particles[3].position;
        let delta = len3(sub3(pos_after, pos_before));
        assert!(delta < 1e-6, "pinned particle moved: delta={}", delta);
    }

    #[test]
    fn update_empty_cloth_does_not_panic() {
        let cfg = default_cloth_sim_config();
        let mut cloth = new_cloth_sim_mesh(&cfg);
        update_cloth_sim(&mut cloth, 0.01);
    }

    // -----------------------------------------------------------------------
    // cloth_sim_energy
    // -----------------------------------------------------------------------

    #[test]
    fn energy_is_finite() {
        let cloth = patch();
        let e = cloth_sim_energy(&cloth);
        assert!(e.is_finite());
    }

    #[test]
    fn energy_increases_after_velocity_added() {
        let mut cloth = patch();
        let e0 = cloth_sim_energy(&cloth);
        cloth.particles[0].velocity = [1.0, 0.0, 0.0];
        let e1 = cloth_sim_energy(&cloth);
        assert!(e1 > e0, "e0={} e1={}", e0, e1);
    }

    // -----------------------------------------------------------------------
    // cloth_sim_aabb
    // -----------------------------------------------------------------------

    #[test]
    fn aabb_covers_all_particles() {
        let cloth = patch();
        let aabb = cloth_sim_aabb(&cloth);
        // Particles span [0,1] x [0,1] x [0,0]
        assert!((aabb[0] - 0.0).abs() < 1e-6); // min_x
        assert!((aabb[3] - 1.0).abs() < 1e-6); // max_x
        assert!((aabb[1] - 0.0).abs() < 1e-6); // min_y
        assert!((aabb[4] - 1.0).abs() < 1e-6); // max_y
    }

    #[test]
    fn aabb_empty_is_zero() {
        let cfg = default_cloth_sim_config();
        let cloth = new_cloth_sim_mesh(&cfg);
        let aabb = cloth_sim_aabb(&cloth);
        assert_eq!(aabb, [0.0; 6]);
    }

    // -----------------------------------------------------------------------
    // reset_cloth_sim
    // -----------------------------------------------------------------------

    #[test]
    fn reset_zeros_velocities() {
        let mut cloth = patch();
        cloth.particles[0].velocity = [1.0, 2.0, 3.0];
        reset_cloth_sim(&mut cloth);
        for p in &cloth.particles {
            let v2 = dot3(p.velocity, p.velocity);
            assert!(v2 < 1e-10);
        }
    }

    #[test]
    fn reset_preserves_positions() {
        let mut cloth = patch();
        let pos_before: Vec<[f32; 3]> = cloth.particles.iter().map(|p| p.position).collect();
        reset_cloth_sim(&mut cloth);
        for (a, b) in pos_before.iter().zip(cloth.particles.iter()) {
            assert!((a[0] - b.position[0]).abs() < 1e-6);
        }
    }
}
