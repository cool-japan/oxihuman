// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Balloon: pressure-volume elastic sphere body.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Balloon body state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BalloonBody {
    /// Current amount of gas (moles equivalent).
    pub gas_amount: f32,
    /// Elastic constant of the membrane.
    pub stiffness: f32,
    /// Rest radius at equilibrium pressure.
    pub rest_radius: f32,
    /// Current radius.
    pub radius: f32,
    /// Center position.
    pub position: [f32; 3],
    /// Velocity.
    pub velocity: [f32; 3],
    /// Mass of the membrane.
    pub mass: f32,
    /// External ambient pressure.
    pub ambient_pressure: f32,
}

/// Create a new balloon body.
#[allow(dead_code)]
pub fn new_balloon_body(
    gas_amount: f32,
    stiffness: f32,
    rest_radius: f32,
    mass: f32,
    ambient_pressure: f32,
) -> BalloonBody {
    BalloonBody {
        gas_amount,
        stiffness,
        rest_radius,
        radius: rest_radius,
        position: [0.0; 3],
        velocity: [0.0; 3],
        mass,
        ambient_pressure,
    }
}

/// Volume of the balloon (sphere).
#[allow(dead_code)]
pub fn balloon_volume(b: &BalloonBody) -> f32 {
    (4.0 / 3.0) * PI * b.radius * b.radius * b.radius
}

/// Surface area of the balloon.
#[allow(dead_code)]
pub fn balloon_surface_area(b: &BalloonBody) -> f32 {
    4.0 * PI * b.radius * b.radius
}

/// Internal pressure (ideal gas law proxy: P = gas_amount / V).
#[allow(dead_code)]
pub fn balloon_internal_pressure(b: &BalloonBody) -> f32 {
    let v = balloon_volume(b);
    if v > f32::EPSILON {
        b.gas_amount / v
    } else {
        0.0
    }
}

/// Net outward pressure (internal - ambient).
#[allow(dead_code)]
pub fn balloon_net_pressure(b: &BalloonBody) -> f32 {
    balloon_internal_pressure(b) - b.ambient_pressure
}

/// Elastic restoring force on radius (Hooke's law towards rest radius).
#[allow(dead_code)]
pub fn balloon_elastic_force(b: &BalloonBody) -> f32 {
    -b.stiffness * (b.radius - b.rest_radius)
}

/// Total radial force on the membrane: pressure * area + elastic restoring.
#[allow(dead_code)]
pub fn balloon_radial_force(b: &BalloonBody) -> f32 {
    let pressure_force = balloon_net_pressure(b) * balloon_surface_area(b);
    let elastic = balloon_elastic_force(b);
    pressure_force + elastic
}

/// Step the balloon: update radius based on radial force.
#[allow(dead_code)]
pub fn balloon_step(b: &mut BalloonBody, dt: f32) {
    let force = balloon_radial_force(b);
    // Use mass of membrane for radial acceleration
    let accel = if b.mass > f32::EPSILON {
        force / b.mass
    } else {
        0.0
    };
    // Simple velocity verlet
    b.radius = (b.radius + accel * dt * dt * 0.5).max(0.01);
}

/// Add gas to the balloon.
#[allow(dead_code)]
pub fn balloon_inflate(b: &mut BalloonBody, amount: f32) {
    b.gas_amount = (b.gas_amount + amount).max(0.0);
}

/// Remove gas from the balloon.
#[allow(dead_code)]
pub fn balloon_deflate(b: &mut BalloonBody, amount: f32) {
    b.gas_amount = (b.gas_amount - amount).max(0.0);
}

/// Check if the balloon is over-pressured (net pressure > threshold → burst).
#[allow(dead_code)]
pub fn balloon_is_burst(b: &BalloonBody, burst_pressure: f32) -> bool {
    balloon_net_pressure(b) > burst_pressure
}

/// Move the balloon by applying velocity for one time step.
#[allow(dead_code)]
pub fn balloon_move(b: &mut BalloonBody, dt: f32) {
    b.position[0] += b.velocity[0] * dt;
    b.position[1] += b.velocity[1] * dt;
    b.position[2] += b.velocity[2] * dt;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_balloon() -> BalloonBody {
        new_balloon_body(10.0, 5.0, 1.0, 0.1, 1.0)
    }

    #[test]
    fn volume_positive() {
        let b = default_balloon();
        assert!(balloon_volume(&b) > 0.0);
    }

    #[test]
    fn surface_area_positive() {
        let b = default_balloon();
        assert!(balloon_surface_area(&b) > 0.0);
    }

    #[test]
    fn internal_pressure_positive() {
        let b = default_balloon();
        assert!(balloon_internal_pressure(&b) > 0.0);
    }

    #[test]
    fn inflate_increases_gas() {
        let mut b = default_balloon();
        let before = b.gas_amount;
        balloon_inflate(&mut b, 2.0);
        assert!(b.gas_amount > before);
    }

    #[test]
    fn deflate_decreases_gas() {
        let mut b = default_balloon();
        balloon_deflate(&mut b, 5.0);
        assert!(b.gas_amount < 10.0);
    }

    #[test]
    fn deflate_clamps_to_zero() {
        let mut b = default_balloon();
        balloon_deflate(&mut b, 100.0);
        assert_eq!(b.gas_amount, 0.0);
    }

    #[test]
    fn elastic_force_at_rest_zero() {
        let mut b = default_balloon();
        b.radius = b.rest_radius;
        assert!((balloon_elastic_force(&b)).abs() < 1e-5);
    }

    #[test]
    fn burst_detection() {
        let mut b = default_balloon();
        b.gas_amount = 1000.0; // very high pressure
        assert!(balloon_is_burst(&b, 1.0));
    }

    #[test]
    fn move_updates_position() {
        let mut b = default_balloon();
        b.velocity = [1.0, 0.0, 0.0];
        balloon_move(&mut b, 1.0);
        assert!((b.position[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn step_radius_stays_positive() {
        let mut b = default_balloon();
        for _ in 0..10 {
            balloon_step(&mut b, 0.016);
        }
        assert!(b.radius > 0.0);
    }
}
