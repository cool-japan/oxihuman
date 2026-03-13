// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Angular velocity integration and damping utilities.
//!
//! This module provides a simple rigid-body angular integrator.  The body's
//! orientation is represented as a quaternion `[x, y, z, w]` and is updated
//! each step by integrating the angular velocity vector.

#![allow(dead_code)]

/// Configuration for angular velocity simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngularVelocityConfig {
    /// Linear damping coefficient applied per second (0 = no damping).
    pub damping: f32,
    /// Maximum angular speed (rad/s) that can be applied in one step.
    pub max_angular_speed: f32,
}

/// Returns a sensible default [`AngularVelocityConfig`].
#[allow(dead_code)]
pub fn default_angular_config() -> AngularVelocityConfig {
    AngularVelocityConfig {
        damping: 0.1,
        max_angular_speed: 100.0,
    }
}

/// A rigid body with angular state only.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AngularBody {
    /// Orientation quaternion `[x, y, z, w]` (unit quaternion).
    pub orientation: [f32; 4],
    /// Angular velocity vector (rad/s) in world space.
    pub angular_velocity: [f32; 3],
    /// Moment of inertia (scalar, kg·m²; assumed isotropic for simplicity).
    pub inertia: f32,
    /// Configuration.
    pub config: AngularVelocityConfig,
}

/// Create a new [`AngularBody`] at identity orientation.
#[allow(dead_code)]
pub fn new_angular_body(inertia: f32, config: AngularVelocityConfig) -> AngularBody {
    AngularBody {
        orientation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
        angular_velocity: [0.0; 3],
        inertia,
        config,
    }
}

/// Apply a torque impulse `tau` (N·m) for duration `dt`.
#[allow(dead_code)]
pub fn angular_apply_torque(body: &mut AngularBody, torque: [f32; 3], dt: f32) {
    let inv_i = 1.0 / body.inertia.max(1e-12);
    body.angular_velocity[0] += torque[0] * inv_i * dt;
    body.angular_velocity[1] += torque[1] * inv_i * dt;
    body.angular_velocity[2] += torque[2] * inv_i * dt;
    // Clamp to max speed.
    let speed = angular_velocity_magnitude(body);
    if speed > body.config.max_angular_speed {
        let scale = body.config.max_angular_speed / speed;
        body.angular_velocity[0] *= scale;
        body.angular_velocity[1] *= scale;
        body.angular_velocity[2] *= scale;
    }
}

/// Apply angular damping for duration `dt`.
#[allow(dead_code)]
pub fn angular_damp(body: &mut AngularBody, dt: f32) {
    let factor = (1.0 - body.config.damping * dt).max(0.0);
    body.angular_velocity[0] *= factor;
    body.angular_velocity[1] *= factor;
    body.angular_velocity[2] *= factor;
}

/// Integrate orientation by the current angular velocity over `dt` seconds.
#[allow(dead_code)]
pub fn angular_step(body: &mut AngularBody, dt: f32) {
    angular_damp(body, dt);
    let w = body.angular_velocity;
    // dq/dt = 0.5 * [wx,wy,wz,0] * q
    let q = body.orientation;
    let dqx = 0.5 * ( w[0]*q[3] + w[1]*q[2] - w[2]*q[1]);
    let dqy = 0.5 * (-w[0]*q[2] + w[1]*q[3] + w[2]*q[0]);
    let dqz = 0.5 * ( w[0]*q[1] - w[1]*q[0] + w[2]*q[3]);
    let dqw = 0.5 * (-w[0]*q[0] - w[1]*q[1] - w[2]*q[2]);
    let nx = q[0] + dqx * dt;
    let ny = q[1] + dqy * dt;
    let nz = q[2] + dqz * dt;
    let nw = q[3] + dqw * dt;
    // Re-normalise.
    let len = (nx*nx + ny*ny + nz*nz + nw*nw).sqrt().max(1e-12);
    body.orientation = [nx/len, ny/len, nz/len, nw/len];
}

/// Rotational kinetic energy: ½ I ω².
#[allow(dead_code)]
pub fn angular_kinetic_energy(body: &AngularBody) -> f32 {
    let w2 = body.angular_velocity.iter().map(|v| v*v).sum::<f32>();
    0.5 * body.inertia * w2
}

/// Angular momentum vector: L = I * ω.
#[allow(dead_code)]
pub fn angular_momentum(body: &AngularBody) -> [f32; 3] {
    [
        body.inertia * body.angular_velocity[0],
        body.inertia * body.angular_velocity[1],
        body.inertia * body.angular_velocity[2],
    ]
}

/// Magnitude of the angular velocity vector (rad/s).
#[allow(dead_code)]
pub fn angular_velocity_magnitude(body: &AngularBody) -> f32 {
    let v = body.angular_velocity;
    (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt()
}

/// Serialize the body state to a simple JSON string.
#[allow(dead_code)]
pub fn angular_body_to_json(body: &AngularBody) -> String {
    let q = body.orientation;
    let w = body.angular_velocity;
    format!(
        "{{\"orientation\":[{},{},{},{}],\"angular_velocity\":[{},{},{}],\"inertia\":{}}}",
        q[0], q[1], q[2], q[3],
        w[0], w[1], w[2],
        body.inertia
    )
}

/// Reset orientation to identity and angular velocity to zero.
#[allow(dead_code)]
pub fn angular_reset(body: &mut AngularBody) {
    body.orientation = [0.0, 0.0, 0.0, 1.0];
    body.angular_velocity = [0.0; 3];
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_body() -> AngularBody {
        new_angular_body(1.0, default_angular_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_angular_config();
        assert!(cfg.damping >= 0.0);
        assert!(cfg.max_angular_speed > 0.0);
    }

    #[test]
    fn test_initial_state() {
        let body = make_body();
        assert_eq!(body.orientation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(body.angular_velocity, [0.0; 3]);
        assert_eq!(angular_kinetic_energy(&body), 0.0);
    }

    #[test]
    fn test_apply_torque_increases_velocity() {
        let mut body = make_body();
        angular_apply_torque(&mut body, [0.0, 0.0, 1.0], 0.1);
        assert!(body.angular_velocity[2] > 0.0);
    }

    #[test]
    fn test_kinetic_energy_after_torque() {
        let mut body = make_body();
        angular_apply_torque(&mut body, [1.0, 0.0, 0.0], 1.0);
        assert!(angular_kinetic_energy(&body) > 0.0);
    }

    #[test]
    fn test_momentum_proportional_to_inertia() {
        let mut body = new_angular_body(2.0, default_angular_config());
        angular_apply_torque(&mut body, [1.0, 0.0, 0.0], 1.0);
        let mom = angular_momentum(&body);
        // L = I*ω; ω = torque / I * dt = 1/2
        assert!((mom[0] - 2.0 * body.angular_velocity[0]).abs() < 1e-5);
    }

    #[test]
    fn test_orientation_normalised_after_step() {
        let mut body = make_body();
        body.angular_velocity = [1.0, 2.0, 3.0];
        angular_step(&mut body, 0.05);
        let q = body.orientation;
        let len = (q[0]*q[0]+q[1]*q[1]+q[2]*q[2]+q[3]*q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-5, "quaternion must remain unit: len={len}");
    }

    #[test]
    fn test_damping_reduces_velocity() {
        let mut body = make_body();
        body.angular_velocity = [1.0, 0.0, 0.0];
        angular_damp(&mut body, 1.0);
        assert!(body.angular_velocity[0] < 1.0);
    }

    #[test]
    fn test_json_output() {
        let body = make_body();
        let json = angular_body_to_json(&body);
        assert!(json.contains("orientation"));
        assert!(json.contains("angular_velocity"));
        assert!(json.contains("inertia"));
    }

    #[test]
    fn test_reset() {
        let mut body = make_body();
        body.angular_velocity = [5.0, 5.0, 5.0];
        body.orientation = [0.5, 0.5, 0.5, 0.5];
        angular_reset(&mut body);
        assert_eq!(body.orientation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(body.angular_velocity, [0.0; 3]);
    }

    #[test]
    fn test_velocity_magnitude() {
        let mut body = make_body();
        body.angular_velocity = [3.0, 4.0, 0.0];
        assert!((angular_velocity_magnitude(&body) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_speed_clamp() {
        let mut body = make_body();
        // Apply enormous torque.
        angular_apply_torque(&mut body, [0.0, 0.0, 1e6], 1.0);
        assert!(angular_velocity_magnitude(&body) <= body.config.max_angular_speed + 1e-4);
    }
}
