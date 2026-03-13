#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Full rigid body state snapshot.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidBodyState {
    pub position: [f32; 3],
    /// Quaternion [x, y, z, w]
    pub rotation: [f32; 4],
    pub linear_vel: [f32; 3],
    pub angular_vel: [f32; 3],
    pub force_accum: [f32; 3],
    pub torque_accum: [f32; 3],
    pub inv_mass: f32,
    pub sleeping: bool,
}

#[allow(dead_code)]
pub fn new_rigid_body_state(pos: [f32; 3], inv_mass: f32) -> RigidBodyState {
    RigidBodyState {
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        linear_vel: [0.0; 3],
        angular_vel: [0.0; 3],
        force_accum: [0.0; 3],
        torque_accum: [0.0; 3],
        inv_mass,
        sleeping: false,
    }
}

#[allow(dead_code)]
pub fn rb_apply_force(state: &mut RigidBodyState, force: [f32; 3]) {
    state.force_accum[0] += force[0];
    state.force_accum[1] += force[1];
    state.force_accum[2] += force[2];
}

#[allow(dead_code)]
pub fn rb_integrate(state: &mut RigidBodyState, gravity: [f32; 3], dt: f32) {
    if state.sleeping {
        return;
    }
    // Apply gravity force
    let grav_force = [
        gravity[0] / state.inv_mass.max(1e-10),
        gravity[1] / state.inv_mass.max(1e-10),
        gravity[2] / state.inv_mass.max(1e-10),
    ];
    // Acceleration = (forces + gravity_force) * inv_mass
    let accel = [
        (state.force_accum[0] + grav_force[0]) * state.inv_mass,
        (state.force_accum[1] + grav_force[1]) * state.inv_mass,
        (state.force_accum[2] + grav_force[2]) * state.inv_mass,
    ];
    state.linear_vel[0] += accel[0] * dt;
    state.linear_vel[1] += accel[1] * dt;
    state.linear_vel[2] += accel[2] * dt;
    state.position[0] += state.linear_vel[0] * dt;
    state.position[1] += state.linear_vel[1] * dt;
    state.position[2] += state.linear_vel[2] * dt;
}

#[allow(dead_code)]
pub fn rb_clear_forces(state: &mut RigidBodyState) {
    state.force_accum = [0.0; 3];
    state.torque_accum = [0.0; 3];
}

#[allow(dead_code)]
pub fn rb_kinetic_energy(state: &RigidBodyState) -> f32 {
    if state.inv_mass < 1e-10 {
        return 0.0;
    }
    let mass = 1.0 / state.inv_mass;
    let v = &state.linear_vel;
    let v2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    0.5 * mass * v2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rb_position() {
        let rb = new_rigid_body_state([1.0, 2.0, 3.0], 1.0);
        assert_eq!(rb.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_new_rb_zero_vel() {
        let rb = new_rigid_body_state([0.0; 3], 1.0);
        assert_eq!(rb.linear_vel, [0.0; 3]);
    }

    #[test]
    fn test_apply_force() {
        let mut rb = new_rigid_body_state([0.0; 3], 1.0);
        rb_apply_force(&mut rb, [10.0, 0.0, 0.0]);
        assert!((rb.force_accum[0] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear_forces() {
        let mut rb = new_rigid_body_state([0.0; 3], 1.0);
        rb_apply_force(&mut rb, [5.0, 5.0, 5.0]);
        rb_clear_forces(&mut rb);
        assert_eq!(rb.force_accum, [0.0; 3]);
    }

    #[test]
    fn test_kinetic_energy_zero() {
        let rb = new_rigid_body_state([0.0; 3], 1.0);
        assert!(rb_kinetic_energy(&rb).abs() < 1e-6);
    }

    #[test]
    fn test_kinetic_energy_moving() {
        let mut rb = new_rigid_body_state([0.0; 3], 0.5); // mass = 2 kg
        rb.linear_vel = [2.0, 0.0, 0.0];
        let ke = rb_kinetic_energy(&rb);
        // 0.5 * 2 * 4 = 4
        assert!((ke - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_integrate_gravity() {
        let mut rb = new_rigid_body_state([0.0; 3], 1.0);
        rb_integrate(&mut rb, [0.0, -9.8, 0.0], 1.0);
        // gravity contribution: grav_force = [0, -9.8/1, 0], accel = [0, (-9.8)*1, 0]
        // vel after = [0, -9.8, 0], pos after = [0, -9.8, 0]
        assert!(rb.linear_vel[1] < 0.0);
    }

    #[test]
    fn test_sleeping_no_integrate() {
        let mut rb = new_rigid_body_state([0.0, 5.0, 0.0], 1.0);
        rb.sleeping = true;
        rb_integrate(&mut rb, [0.0, -9.8, 0.0], 1.0);
        assert_eq!(rb.position, [0.0, 5.0, 0.0]);
    }

    #[test]
    fn test_zero_inv_mass_kinetic_energy() {
        let mut rb = new_rigid_body_state([0.0; 3], 0.0);
        rb.linear_vel = [10.0, 0.0, 0.0];
        assert!(rb_kinetic_energy(&rb).abs() < 1e-6);
    }
}
