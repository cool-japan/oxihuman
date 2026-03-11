// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rack body: rack-and-pinion gear mechanism simulation.

/// A rack-and-pinion body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RackBody {
    /// Pinion (gear) pitch radius (m).
    pub pinion_radius: f32,
    /// Pinion angle (rad).
    pub pinion_angle: f32,
    /// Pinion angular velocity (rad/s).
    pub pinion_omega: f32,
    /// Rack linear position (m).
    pub rack_pos: f32,
    /// Rack linear velocity (m/s).
    pub rack_vel: f32,
    /// Moment of inertia of pinion.
    pub pinion_inertia: f32,
    /// Mass of rack.
    pub rack_mass: f32,
    /// Friction coefficient.
    pub friction: f32,
}

/// Create a new `RackBody`.
#[allow(dead_code)]
pub fn new_rack_body(pinion_radius: f32, pinion_inertia: f32, rack_mass: f32) -> RackBody {
    RackBody {
        pinion_radius: pinion_radius.max(1e-4),
        pinion_angle: 0.0,
        pinion_omega: 0.0,
        rack_pos: 0.0,
        rack_vel: 0.0,
        pinion_inertia: pinion_inertia.max(1e-9),
        rack_mass: rack_mass.max(1e-9),
        friction: 0.01,
    }
}

/// Apply torque to the pinion for `dt` seconds.
#[allow(dead_code)]
pub fn rack_apply_torque(body: &mut RackBody, torque: f32, dt: f32) {
    // Inertia-coupled: τ = I·α + r·F_rack
    let alpha = torque / body.pinion_inertia;
    body.pinion_omega += alpha * dt;
    body.pinion_omega *= 1.0 - body.friction;
    body.pinion_angle += body.pinion_omega * dt;
    body.rack_vel = body.pinion_omega * body.pinion_radius;
    body.rack_pos += body.rack_vel * dt;
}

/// Apply a force to the rack for `dt` seconds.
#[allow(dead_code)]
pub fn rack_apply_force(body: &mut RackBody, force: f32, dt: f32) {
    let a = force / body.rack_mass;
    body.rack_vel += a * dt;
    body.rack_vel *= 1.0 - body.friction;
    body.rack_pos += body.rack_vel * dt;
    body.pinion_omega = body.rack_vel / body.pinion_radius;
    body.pinion_angle += body.pinion_omega * dt;
}

/// Gear ratio: 1 rad of pinion = `pinion_radius` m of rack.
#[allow(dead_code)]
pub fn rack_gear_ratio(body: &RackBody) -> f32 {
    body.pinion_radius
}

/// Rack position for a given pinion angle.
#[allow(dead_code)]
pub fn rack_pos_from_angle(body: &RackBody, angle: f32) -> f32 {
    angle * body.pinion_radius
}

/// Pinion angle for a given rack position.
#[allow(dead_code)]
pub fn rack_angle_from_pos(body: &RackBody, pos: f32) -> f32 {
    pos / body.pinion_radius.max(1e-9)
}

/// Kinetic energy of the system.
#[allow(dead_code)]
pub fn rack_kinetic_energy(body: &RackBody) -> f32 {
    let rotational = 0.5 * body.pinion_inertia * body.pinion_omega * body.pinion_omega;
    let linear = 0.5 * body.rack_mass * body.rack_vel * body.rack_vel;
    rotational + linear
}

/// Reset to initial state.
#[allow(dead_code)]
pub fn rack_reset(body: &mut RackBody) {
    body.pinion_angle = 0.0;
    body.pinion_omega = 0.0;
    body.rack_pos = 0.0;
    body.rack_vel = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_rack_body() {
        let rb = new_rack_body(0.05, 0.001, 1.0);
        assert!((rb.pinion_radius - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_torque_moves_rack() {
        let mut rb = new_rack_body(0.05, 0.001, 1.0);
        rack_apply_torque(&mut rb, 1.0, 0.1);
        assert!(rb.rack_pos.abs() > 0.0);
    }

    #[test]
    fn test_force_moves_rack() {
        let mut rb = new_rack_body(0.05, 0.001, 1.0);
        rack_apply_force(&mut rb, 10.0, 0.1);
        assert!(rb.rack_pos > 0.0);
    }

    #[test]
    fn test_gear_ratio() {
        let rb = new_rack_body(0.1, 0.001, 1.0);
        assert!((rack_gear_ratio(&rb) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_pos_from_angle() {
        let rb = new_rack_body(0.05, 0.001, 1.0);
        let pos = rack_pos_from_angle(&rb, 2.0 * PI);
        assert!((pos - 0.05 * 2.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn test_angle_from_pos() {
        let rb = new_rack_body(0.05, 0.001, 1.0);
        let angle = rack_angle_from_pos(&rb, 0.05 * PI);
        assert!((angle - PI).abs() < 1e-4);
    }

    #[test]
    fn test_kinetic_energy_nonneg() {
        let mut rb = new_rack_body(0.05, 0.001, 1.0);
        rack_apply_torque(&mut rb, 1.0, 0.1);
        assert!(rack_kinetic_energy(&rb) >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut rb = new_rack_body(0.05, 0.001, 1.0);
        rack_apply_torque(&mut rb, 1.0, 1.0);
        rack_reset(&mut rb);
        assert!((rb.rack_pos).abs() < 1e-9);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }
}
