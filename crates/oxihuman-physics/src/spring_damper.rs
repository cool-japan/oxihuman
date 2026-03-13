// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Spring-damper system for physics simulation.
//!
//! Implements classical Hooke's law springs with viscous damping for
//! point-mass systems. Supports creating individual spring-damper pairs
//! and managing collections of them in a unified system.

// ── structs ──────────────────────────────────────────────────────────────────

/// A single spring-damper connection between two particle indices.
#[allow(dead_code)]
pub struct SpringDamper {
    /// Index of the first particle.
    pub a: usize,
    /// Index of the second particle.
    pub b: usize,
    /// Spring stiffness coefficient (N/m).
    pub stiffness: f32,
    /// Damping coefficient (Ns/m).
    pub damping: f32,
    /// Rest length of the spring.
    pub rest_length: f32,
}

/// Configuration for creating spring-damper elements.
#[allow(dead_code)]
pub struct SpringDamperConfig {
    /// Default stiffness.
    pub stiffness: f32,
    /// Default damping.
    pub damping: f32,
    /// Default rest length (0 means auto-compute from initial positions).
    pub rest_length: f32,
    /// Integration time step.
    pub dt: f32,
}

/// A system of particles connected by spring-damper elements.
#[allow(dead_code)]
pub struct SpringDamperSystem {
    /// Particle positions.
    pub positions: Vec<[f32; 3]>,
    /// Particle velocities.
    pub velocities: Vec<[f32; 3]>,
    /// Particle masses.
    pub masses: Vec<f32>,
    /// Whether each particle is pinned (immovable).
    pub pinned: Vec<bool>,
    /// All springs in the system.
    pub springs: Vec<SpringDamper>,
    /// Integration time step.
    pub dt: f32,
}

// ── math helpers ─────────────────────────────────────────────────────────────

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

// ── public API ───────────────────────────────────────────────────────────────

/// Returns a default spring-damper configuration.
#[allow(dead_code)]
pub fn default_spring_damper_config() -> SpringDamperConfig {
    SpringDamperConfig {
        stiffness: 100.0,
        damping: 1.0,
        rest_length: 0.0,
        dt: 1.0 / 60.0,
    }
}

/// Create a new spring-damper element.
#[allow(dead_code)]
pub fn new_spring_damper(
    a: usize,
    b: usize,
    stiffness: f32,
    damping: f32,
    rest_length: f32,
) -> SpringDamper {
    SpringDamper {
        a,
        b,
        stiffness,
        damping,
        rest_length,
    }
}

/// Create a new spring-damper system with the given particle data.
#[allow(dead_code)]
pub fn new_spring_damper_system(
    positions: Vec<[f32; 3]>,
    masses: Vec<f32>,
    dt: f32,
) -> SpringDamperSystem {
    let n = positions.len();
    SpringDamperSystem {
        positions,
        velocities: vec![[0.0, 0.0, 0.0]; n],
        masses,
        pinned: vec![false; n],
        springs: Vec::new(),
        dt,
    }
}

/// Add a spring to the system. If `rest_length` is 0, auto-compute from current positions.
#[allow(dead_code)]
pub fn add_spring(
    system: &mut SpringDamperSystem,
    a: usize,
    b: usize,
    stiffness: f32,
    damping: f32,
    rest_length: f32,
) {
    let rl = if rest_length <= 0.0 {
        len3(sub3(system.positions[a], system.positions[b]))
    } else {
        rest_length
    };
    system.springs.push(SpringDamper {
        a,
        b,
        stiffness,
        damping,
        rest_length: rl,
    });
}

/// Remove a spring by index. Returns `true` if removed.
#[allow(dead_code)]
pub fn remove_spring(system: &mut SpringDamperSystem, index: usize) -> bool {
    if index < system.springs.len() {
        system.springs.remove(index);
        true
    } else {
        false
    }
}

/// Compute the spring force (Hooke's law + damping) for a single spring.
/// Returns the force acting on particle `a` (negate for particle `b`).
#[allow(dead_code)]
pub fn compute_spring_force(
    spring: &SpringDamper,
    pos_a: [f32; 3],
    pos_b: [f32; 3],
    vel_a: [f32; 3],
    vel_b: [f32; 3],
) -> [f32; 3] {
    let diff = sub3(pos_b, pos_a);
    let dist = len3(diff);
    if dist < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    let dir = scale3(diff, 1.0 / dist);
    let stretch = dist - spring.rest_length;

    // Hooke: F = k * stretch
    let spring_f = spring.stiffness * stretch;

    // Damping: F = c * relative_velocity_along_spring
    let rel_vel = sub3(vel_b, vel_a);
    let damp_f = spring.damping * dot3(rel_vel, dir);

    scale3(dir, spring_f + damp_f)
}

/// Update the entire spring system: compute forces, integrate velocities and positions.
#[allow(dead_code)]
pub fn update_spring_system(system: &mut SpringDamperSystem) {
    let n = system.positions.len();
    let mut forces = vec![[0.0f32; 3]; n];

    // Accumulate spring forces
    for spring in &system.springs {
        let f = compute_spring_force(
            spring,
            system.positions[spring.a],
            system.positions[spring.b],
            system.velocities[spring.a],
            system.velocities[spring.b],
        );
        forces[spring.a] = add3(forces[spring.a], f);
        forces[spring.b] = sub3(forces[spring.b], f);
    }

    // Symplectic Euler integration
    let dt = system.dt;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        if system.pinned[i] {
            continue;
        }
        let inv_mass = if system.masses[i] > 0.0 {
            1.0 / system.masses[i]
        } else {
            0.0
        };
        let acc = scale3(forces[i], inv_mass);
        system.velocities[i] = add3(system.velocities[i], scale3(acc, dt));
        system.positions[i] = add3(system.positions[i], scale3(system.velocities[i], dt));
    }
}

/// Return the number of springs in the system.
#[allow(dead_code)]
pub fn spring_count(system: &SpringDamperSystem) -> usize {
    system.springs.len()
}

/// Compute the potential energy stored in a single spring: 0.5 * k * dx^2.
#[allow(dead_code)]
pub fn spring_energy(spring: &SpringDamper, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
    let dist = len3(sub3(pos_b, pos_a));
    let dx = dist - spring.rest_length;
    0.5 * spring.stiffness * dx * dx
}

/// Compute the total potential energy of all springs in the system.
#[allow(dead_code)]
pub fn total_system_energy(system: &SpringDamperSystem) -> f32 {
    system
        .springs
        .iter()
        .map(|s| spring_energy(s, system.positions[s.a], system.positions[s.b]))
        .sum()
}

/// Set the stiffness of a spring by index.
#[allow(dead_code)]
pub fn set_spring_stiffness(system: &mut SpringDamperSystem, index: usize, stiffness: f32) {
    if let Some(s) = system.springs.get_mut(index) {
        s.stiffness = stiffness;
    }
}

/// Set the damping of a spring by index.
#[allow(dead_code)]
pub fn set_spring_damping(system: &mut SpringDamperSystem, index: usize, damping: f32) {
    if let Some(s) = system.springs.get_mut(index) {
        s.damping = damping;
    }
}

/// Compute the current extension (stretch) of a spring in the system.
#[allow(dead_code)]
pub fn spring_extension(system: &SpringDamperSystem, index: usize) -> f32 {
    if let Some(s) = system.springs.get(index) {
        let dist = len3(sub3(system.positions[s.b], system.positions[s.a]));
        dist - s.rest_length
    } else {
        0.0
    }
}

/// Reset all velocities to zero and positions to their initial state is not feasible
/// without storing initial positions, so this resets velocities only.
#[allow(dead_code)]
pub fn reset_spring_system(system: &mut SpringDamperSystem) {
    for v in &mut system.velocities {
        *v = [0.0, 0.0, 0.0];
    }
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particle_system() -> SpringDamperSystem {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let masses = vec![1.0, 1.0];
        let mut sys = new_spring_damper_system(positions, masses, 1.0 / 60.0);
        add_spring(&mut sys, 0, 1, 100.0, 1.0, 1.0);
        sys
    }

    #[test]
    fn test_default_config() {
        let cfg = default_spring_damper_config();
        assert!(cfg.stiffness > 0.0);
        assert!(cfg.damping > 0.0);
        assert!(cfg.dt > 0.0);
    }

    #[test]
    fn test_new_spring_damper() {
        let sd = new_spring_damper(0, 1, 50.0, 2.0, 1.5);
        assert_eq!(sd.a, 0);
        assert_eq!(sd.b, 1);
        assert!((sd.stiffness - 50.0).abs() < 1e-6);
        assert!((sd.rest_length - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_system() {
        let sys = new_spring_damper_system(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![1.0, 1.0],
            1.0 / 60.0,
        );
        assert_eq!(sys.positions.len(), 2);
        assert_eq!(sys.velocities.len(), 2);
        assert!(sys.springs.is_empty());
    }

    #[test]
    fn test_add_spring() {
        let mut sys = two_particle_system();
        assert_eq!(spring_count(&sys), 1);
        add_spring(&mut sys, 0, 1, 50.0, 0.5, 0.5);
        assert_eq!(spring_count(&sys), 2);
    }

    #[test]
    fn test_remove_spring() {
        let mut sys = two_particle_system();
        assert!(remove_spring(&mut sys, 0));
        assert_eq!(spring_count(&sys), 0);
        assert!(!remove_spring(&mut sys, 0));
    }

    #[test]
    fn test_spring_force_at_rest() {
        let sd = new_spring_damper(0, 1, 100.0, 1.0, 1.0);
        let f = compute_spring_force(
            &sd,
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        // At rest length, force should be near zero
        assert!(f[0].abs() < 1e-6);
        assert!(f[1].abs() < 1e-6);
        assert!(f[2].abs() < 1e-6);
    }

    #[test]
    fn test_spring_force_stretched() {
        let sd = new_spring_damper(0, 1, 100.0, 0.0, 1.0);
        let f = compute_spring_force(
            &sd,
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        // Stretched by 1.0: force = 100 * 1.0 = 100 in +x direction on particle a
        assert!((f[0] - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_spring_force_compressed() {
        let sd = new_spring_damper(0, 1, 100.0, 0.0, 1.0);
        let f = compute_spring_force(
            &sd,
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        // Compressed by 0.5: force = 100 * -0.5 = -50 in +x direction on particle a
        assert!((f[0] - (-50.0)).abs() < 1e-3);
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_update_system_at_rest() {
        let mut sys = two_particle_system();
        let pos_before = sys.positions.clone();
        update_spring_system(&mut sys);
        // At rest length, positions should barely change
        #[allow(clippy::needless_range_loop)]
        for k in 0..3 {
            assert!((sys.positions[0][k] - pos_before[0][k]).abs() < 0.01);
            assert!((sys.positions[1][k] - pos_before[1][k]).abs() < 0.01);
        }
    }

    #[test]
    fn test_update_system_stretched() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let masses = vec![1.0, 1.0];
        let mut sys = new_spring_damper_system(positions, masses, 1.0 / 60.0);
        add_spring(&mut sys, 0, 1, 100.0, 1.0, 1.0);
        update_spring_system(&mut sys);
        // Particles should move toward each other
        assert!(sys.positions[0][0] > 0.0);
        assert!(sys.positions[1][0] < 2.0);
    }

    #[test]
    fn test_spring_energy_at_rest() {
        let sd = new_spring_damper(0, 1, 100.0, 1.0, 1.0);
        let e = spring_energy(&sd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(e.abs() < 1e-6);
    }

    #[test]
    fn test_spring_energy_stretched() {
        let sd = new_spring_damper(0, 1, 100.0, 0.0, 1.0);
        let e = spring_energy(&sd, [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        // E = 0.5 * 100 * 1^2 = 50
        assert!((e - 50.0).abs() < 1e-3);
    }

    #[test]
    fn test_total_system_energy() {
        let sys = two_particle_system();
        let e = total_system_energy(&sys);
        // At rest, energy should be zero
        assert!(e.abs() < 1e-6);
    }

    #[test]
    fn test_set_spring_stiffness() {
        let mut sys = two_particle_system();
        set_spring_stiffness(&mut sys, 0, 200.0);
        assert!((sys.springs[0].stiffness - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_spring_damping() {
        let mut sys = two_particle_system();
        set_spring_damping(&mut sys, 0, 5.0);
        assert!((sys.springs[0].damping - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_spring_extension_at_rest() {
        let sys = two_particle_system();
        let ext = spring_extension(&sys, 0);
        assert!(ext.abs() < 1e-6);
    }

    #[test]
    fn test_reset_spring_system() {
        let mut sys = two_particle_system();
        sys.velocities[0] = [1.0, 2.0, 3.0];
        reset_spring_system(&mut sys);
        assert!(sys.velocities[0].iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn test_pinned_particle_stays_put() {
        let positions = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let masses = vec![1.0, 1.0];
        let mut sys = new_spring_damper_system(positions, masses, 1.0 / 60.0);
        add_spring(&mut sys, 0, 1, 100.0, 1.0, 1.0);
        sys.pinned[0] = true;
        update_spring_system(&mut sys);
        assert!((sys.positions[0][0]).abs() < 1e-6);
    }
}
