// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ragdoll physics simulation.
//!
//! A ragdoll is a collection of rigid bodies (bones) connected by ball-socket
//! joint constraints.  This module provides a simplified verlet-integration
//! based simulation suitable for real-time character physics.

// ──────────────────────────────────────────────────────────────────────────────
// Config and data types
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration for ragdoll simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RagdollConfig {
    /// Number of constraint-solving iterations per step.
    pub solver_iterations: u32,
    /// Coefficient of restitution (bounciness) in [0, 1].
    pub restitution: f32,
    /// Linear damping factor applied each step (0 = no damping, 1 = full stop).
    pub linear_damping: f32,
    /// Floor Y coordinate (bones cannot fall below this).
    pub floor_y: f32,
}

/// A single bone in the ragdoll (an axis-aligned box rigid body).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RagdollBone {
    /// Mass of the bone (kg).
    pub mass: f32,
    /// Current world-space position of the bone's centre of mass.
    pub position: [f32; 3],
    /// Half-extents of the bounding box.
    pub half_extents: [f32; 3],
    /// Current linear velocity (m/s).
    pub velocity: [f32; 3],
    /// Accumulated forces for this time-step.
    pub force: [f32; 3],
    /// Whether this bone is static (pinned in place).
    pub is_static: bool,
}

/// A joint connecting two bones.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RagdollJoint {
    /// Index of the first (parent) bone.
    pub bone_a: usize,
    /// Index of the second (child) bone.
    pub bone_b: usize,
    /// World-space pivot point of the joint.
    pub pivot: [f32; 3],
    /// Maximum joint separation before the constraint is violated.
    pub max_distance: f32,
}

/// The top-level ragdoll simulation state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RagdollSystem {
    pub cfg: RagdollConfig,
    pub bones: Vec<RagdollBone>,
    pub joints: Vec<RagdollJoint>,
    /// Initial positions saved for `ragdoll_reset`.
    initial_positions: Vec<[f32; 3]>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public functions
// ──────────────────────────────────────────────────────────────────────────────

/// Returns a sensible default [`RagdollConfig`].
#[allow(dead_code)]
pub fn default_ragdoll_config() -> RagdollConfig {
    RagdollConfig {
        solver_iterations: 10,
        restitution: 0.3,
        linear_damping: 0.01,
        floor_y: 0.0,
    }
}

/// Creates a new empty [`RagdollSystem`] from the given config.
#[allow(dead_code)]
pub fn new_ragdoll(cfg: &RagdollConfig) -> RagdollSystem {
    RagdollSystem {
        cfg: cfg.clone(),
        bones: Vec::new(),
        joints: Vec::new(),
        initial_positions: Vec::new(),
    }
}

/// Adds a bone (rigid body) to the ragdoll and returns its index.
#[allow(dead_code)]
pub fn ragdoll_add_bone(
    sys: &mut RagdollSystem,
    mass: f32,
    pos: [f32; 3],
    half_extents: [f32; 3],
) -> usize {
    let idx = sys.bones.len();
    sys.initial_positions.push(pos);
    sys.bones.push(RagdollBone {
        mass,
        position: pos,
        half_extents,
        velocity: [0.0; 3],
        force: [0.0; 3],
        is_static: false,
    });
    idx
}

/// Connects two bones with a ball-socket joint at `pivot`.
#[allow(dead_code)]
pub fn ragdoll_add_joint(
    sys: &mut RagdollSystem,
    bone_a: usize,
    bone_b: usize,
    pivot: [f32; 3],
) {
    let max_dist = dist3(
        sys.bones[bone_a].position,
        sys.bones[bone_b].position,
    ) * 0.5 + 0.01;
    sys.joints.push(RagdollJoint {
        bone_a,
        bone_b,
        pivot,
        max_distance: max_dist,
    });
}

/// Advances the simulation by `dt` seconds under `gravity`.
#[allow(dead_code)]
pub fn step_ragdoll(sys: &mut RagdollSystem, dt: f32, gravity: [f32; 3]) {
    let damping = sys.cfg.linear_damping;
    let floor_y = sys.cfg.floor_y;

    // 1. Integrate forces → velocities → positions
    for bone in sys.bones.iter_mut() {
        if bone.is_static {
            continue;
        }
        // Gravity
        bone.force[0] += gravity[0] * bone.mass;
        bone.force[1] += gravity[1] * bone.mass;
        bone.force[2] += gravity[2] * bone.mass;

        // Velocity integration (semi-implicit Euler)
        bone.velocity[0] += (bone.force[0] / bone.mass) * dt;
        bone.velocity[1] += (bone.force[1] / bone.mass) * dt;
        bone.velocity[2] += (bone.force[2] / bone.mass) * dt;

        // Damping
        bone.velocity[0] *= 1.0 - damping;
        bone.velocity[1] *= 1.0 - damping;
        bone.velocity[2] *= 1.0 - damping;

        // Position update
        bone.position[0] += bone.velocity[0] * dt;
        bone.position[1] += bone.velocity[1] * dt;
        bone.position[2] += bone.velocity[2] * dt;

        // Floor collision
        let hy = bone.half_extents[1];
        if bone.position[1] - hy < floor_y {
            bone.position[1] = floor_y + hy;
            bone.velocity[1] = bone.velocity[1].abs() * sys.cfg.restitution;
        }

        // Clear forces
        bone.force = [0.0; 3];
    }

    // 2. Solve joint constraints
    for _ in 0..sys.cfg.solver_iterations {
        solve_joint_constraints(sys);
    }
}

/// Returns the world-space position of bone `bone_idx`.
#[allow(dead_code)]
pub fn ragdoll_bone_position(sys: &RagdollSystem, bone_idx: usize) -> [f32; 3] {
    sys.bones[bone_idx].position
}

/// Returns the number of bones in the ragdoll.
#[allow(dead_code)]
pub fn ragdoll_bone_count(sys: &RagdollSystem) -> usize {
    sys.bones.len()
}

/// Returns the number of joints in the ragdoll.
#[allow(dead_code)]
pub fn ragdoll_joint_count(sys: &RagdollSystem) -> usize {
    sys.joints.len()
}

/// Resets all bones to their initial positions with zero velocity.
#[allow(dead_code)]
pub fn ragdoll_reset(sys: &mut RagdollSystem) {
    for (bone, &init) in sys.bones.iter_mut().zip(sys.initial_positions.iter()) {
        bone.position = init;
        bone.velocity = [0.0; 3];
        bone.force = [0.0; 3];
    }
}

/// Applies an impulse (instantaneous velocity change) to a bone.
#[allow(dead_code)]
pub fn ragdoll_apply_impulse(sys: &mut RagdollSystem, bone_idx: usize, impulse: [f32; 3]) {
    let bone = &mut sys.bones[bone_idx];
    if bone.is_static || bone.mass <= 0.0 {
        return;
    }
    let inv_mass = 1.0 / bone.mass;
    bone.velocity[0] += impulse[0] * inv_mass;
    bone.velocity[1] += impulse[1] * inv_mass;
    bone.velocity[2] += impulse[2] * inv_mass;
}

// ──────────────────────────────────────────────────────────────────────────────
// Private helpers
// ──────────────────────────────────────────────────────────────────────────────

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Position-based distance constraint solver (one pass over all joints).
fn solve_joint_constraints(sys: &mut RagdollSystem) {
    // Collect joint data to avoid borrow issues
    let joints: Vec<RagdollJoint> = sys.joints.clone();
    for joint in &joints {
        let a = joint.bone_a;
        let b = joint.bone_b;
        let pa = sys.bones[a].position;
        let pb = sys.bones[b].position;
        let d = dist3(pa, pb);
        if d < 1e-9 {
            continue;
        }
        let rest = joint.max_distance;
        let diff = (d - rest) / d;
        if diff.abs() < 1e-9 {
            continue;
        }
        let ma = sys.bones[a].mass;
        let mb = sys.bones[b].mass;
        let inv_ma = if sys.bones[a].is_static { 0.0 } else { 1.0 / ma };
        let inv_mb = if sys.bones[b].is_static { 0.0 } else { 1.0 / mb };
        let total_inv = inv_ma + inv_mb;
        if total_inv < 1e-12 {
            continue;
        }
        let corr = diff / total_inv;
        let dx = (pb[0] - pa[0]) / d;
        let dy = (pb[1] - pa[1]) / d;
        let dz = (pb[2] - pa[2]) / d;
        if !sys.bones[a].is_static {
            sys.bones[a].position[0] += inv_ma * corr * dx;
            sys.bones[a].position[1] += inv_ma * corr * dy;
            sys.bones[a].position[2] += inv_ma * corr * dz;
        }
        if !sys.bones[b].is_static {
            sys.bones[b].position[0] -= inv_mb * corr * dx;
            sys.bones[b].position[1] -= inv_mb * corr * dy;
            sys.bones[b].position[2] -= inv_mb * corr * dz;
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_ragdoll() -> RagdollSystem {
        let cfg = default_ragdoll_config();
        let mut sys = new_ragdoll(&cfg);
        ragdoll_add_bone(&mut sys, 1.0, [0.0, 1.0, 0.0], [0.1, 0.25, 0.1]);
        ragdoll_add_bone(&mut sys, 1.0, [0.0, 0.5, 0.0], [0.1, 0.25, 0.1]);
        ragdoll_add_joint(&mut sys, 0, 1, [0.0, 0.75, 0.0]);
        sys
    }

    #[test]
    fn default_config_values() {
        let cfg = default_ragdoll_config();
        assert!(cfg.solver_iterations > 0);
        assert!(cfg.restitution >= 0.0 && cfg.restitution <= 1.0);
    }

    #[test]
    fn bone_count_correct() {
        let sys = simple_ragdoll();
        assert_eq!(ragdoll_bone_count(&sys), 2);
    }

    #[test]
    fn joint_count_correct() {
        let sys = simple_ragdoll();
        assert_eq!(ragdoll_joint_count(&sys), 1);
    }

    #[test]
    fn ragdoll_falls_under_gravity() {
        // Use a single-bone ragdoll (no joints) so gravity acts unopposed.
        let cfg = default_ragdoll_config();
        let mut sys = new_ragdoll(&cfg);
        ragdoll_add_bone(&mut sys, 1.0, [0.0, 5.0, 0.0], [0.1, 0.25, 0.1]);
        let initial_y = ragdoll_bone_position(&sys, 0)[1];
        let gravity = [0.0, -9.81, 0.0];
        // Several steps to accumulate meaningful displacement
        for _ in 0..10 {
            step_ragdoll(&mut sys, 0.016, gravity);
        }
        let new_y = ragdoll_bone_position(&sys, 0)[1];
        assert!(new_y < initial_y, "bone y={} should be below initial y={}", new_y, initial_y);
    }

    #[test]
    fn ragdoll_reset_restores_position() {
        let mut sys = simple_ragdoll();
        let orig = ragdoll_bone_position(&sys, 0);
        step_ragdoll(&mut sys, 0.1, [0.0, -9.81, 0.0]);
        ragdoll_reset(&mut sys);
        let restored = ragdoll_bone_position(&sys, 0);
        assert!((restored[0] - orig[0]).abs() < 1e-6);
        assert!((restored[1] - orig[1]).abs() < 1e-6);
        assert!((restored[2] - orig[2]).abs() < 1e-6);
    }

    #[test]
    fn impulse_changes_velocity() {
        let mut sys = simple_ragdoll();
        ragdoll_apply_impulse(&mut sys, 0, [5.0, 0.0, 0.0]);
        assert!(sys.bones[0].velocity[0].abs() > 0.0);
    }

    #[test]
    fn floor_collision_prevents_falling_through() {
        let cfg = RagdollConfig {
            floor_y: 0.0,
            linear_damping: 0.0,
            restitution: 0.0,
            solver_iterations: 1,
        };
        let mut sys = new_ragdoll(&cfg);
        // Place bone just above floor
        ragdoll_add_bone(&mut sys, 1.0, [0.0, 0.26, 0.0], [0.1, 0.25, 0.1]);
        // Give large downward velocity
        sys.bones[0].velocity[1] = -100.0;
        step_ragdoll(&mut sys, 0.1, [0.0, -9.81, 0.0]);
        let y = ragdoll_bone_position(&sys, 0)[1];
        // Should be at or above floor (floor_y + half_extent_y = 0.25)
        assert!(y >= 0.0, "bone y={} should be >= floor 0.0", y);
    }

    #[test]
    fn bone_position_accessor() {
        let sys = simple_ragdoll();
        let pos = ragdoll_bone_position(&sys, 1);
        assert!((pos[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn add_bone_returns_incrementing_indices() {
        let cfg = default_ragdoll_config();
        let mut sys = new_ragdoll(&cfg);
        let i0 = ragdoll_add_bone(&mut sys, 1.0, [0.0, 0.0, 0.0], [0.1; 3]);
        let i1 = ragdoll_add_bone(&mut sys, 1.0, [0.0, 1.0, 0.0], [0.1; 3]);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
    }
}
