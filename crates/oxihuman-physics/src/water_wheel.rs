// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Water wheel with flow-driven torque and angular velocity.

#![allow(dead_code)]

/// A water wheel driven by flowing water.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaterWheel {
    /// Wheel radius in meters.
    pub radius: f32,
    /// Moment of inertia in kg·m².
    pub inertia: f32,
    /// Angular velocity in rad/s.
    pub omega: f32,
    /// Viscous damping coefficient.
    pub damping: f32,
    /// Accumulated rotation in radians.
    pub angle: f32,
}

/// Create a new water wheel.
#[allow(dead_code)]
pub fn new_water_wheel(radius: f32, mass: f32, damping: f32) -> WaterWheel {
    let inertia = 0.5 * mass * radius * radius;
    WaterWheel {
        radius,
        inertia: inertia.max(1e-6),
        omega: 0.0,
        damping,
        angle: 0.0,
    }
}

/// Compute the torque exerted by water flow on the wheel.
/// torque = rho * flow_rate * radius * (flow_speed - radius * omega)
#[allow(dead_code)]
pub fn water_wheel_torque(wheel: &WaterWheel, flow_speed: f32, rho: f32, flow_rate: f32) -> f32 {
    let bucket_speed = wheel.radius * wheel.omega;
    let relative_speed = flow_speed - bucket_speed;
    rho * flow_rate * wheel.radius * relative_speed
}

/// Step the water wheel simulation by `dt` seconds.
#[allow(dead_code)]
pub fn water_wheel_step(
    wheel: &mut WaterWheel,
    flow_speed: f32,
    rho: f32,
    flow_rate: f32,
    dt: f32,
) {
    let torque = water_wheel_torque(wheel, flow_speed, rho, flow_rate);
    let drag_torque = -wheel.damping * wheel.omega;
    let alpha = (torque + drag_torque) / wheel.inertia;
    wheel.omega += alpha * dt;
    wheel.angle += wheel.omega * dt;
}

/// Angular velocity of the wheel in rad/s.
#[allow(dead_code)]
pub fn water_wheel_rpm(wheel: &WaterWheel) -> f32 {
    wheel.omega * 60.0 / (2.0 * std::f32::consts::PI)
}

/// Kinetic energy stored in the wheel: 0.5 * I * omega^2.
#[allow(dead_code)]
pub fn water_wheel_energy(wheel: &WaterWheel) -> f32 {
    0.5 * wheel.inertia * wheel.omega * wheel.omega
}

/// Apply braking torque to the wheel.
#[allow(dead_code)]
pub fn water_wheel_brake(wheel: &mut WaterWheel, brake_torque: f32, dt: f32) {
    if wheel.omega.abs() < 1e-8 {
        wheel.omega = 0.0;
        return;
    }
    let direction = if wheel.omega > 0.0 { -1.0 } else { 1.0 };
    let alpha = direction * brake_torque.abs() / wheel.inertia;
    let new_omega = wheel.omega + alpha * dt;
    if wheel.omega * new_omega <= 0.0 {
        wheel.omega = 0.0;
    } else {
        wheel.omega = new_omega;
    }
}

/// Reset the wheel state.
#[allow(dead_code)]
pub fn water_wheel_reset(wheel: &mut WaterWheel) {
    wheel.omega = 0.0;
    wheel.angle = 0.0;
}

/// Power output of the wheel (torque * omega).
#[allow(dead_code)]
pub fn water_wheel_power(wheel: &WaterWheel, flow_speed: f32, rho: f32, flow_rate: f32) -> f32 {
    water_wheel_torque(wheel, flow_speed, rho, flow_rate) * wheel.omega
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wheel() -> WaterWheel {
        new_water_wheel(0.5, 50.0, 0.1)
    }

    #[test]
    fn test_initial_state() {
        let w = make_wheel();
        assert_eq!(w.omega, 0.0);
        assert_eq!(w.angle, 0.0);
        assert!(w.inertia > 0.0);
    }

    #[test]
    fn test_torque_at_rest() {
        let w = make_wheel();
        let t = water_wheel_torque(&w, 2.0, 1000.0, 0.1);
        assert!(t > 0.0);
    }

    #[test]
    fn test_step_increases_omega() {
        let mut w = make_wheel();
        water_wheel_step(&mut w, 2.0, 1000.0, 0.1, 0.1);
        assert!(w.omega > 0.0);
    }

    #[test]
    fn test_angle_increases() {
        let mut w = make_wheel();
        water_wheel_step(&mut w, 2.0, 1000.0, 0.1, 1.0);
        assert!(w.angle > 0.0);
    }

    #[test]
    fn test_energy_positive() {
        let mut w = make_wheel();
        water_wheel_step(&mut w, 2.0, 1000.0, 0.1, 0.1);
        assert!(water_wheel_energy(&w) > 0.0);
    }

    #[test]
    fn test_rpm_conversion() {
        let mut w = make_wheel();
        w.omega = std::f32::consts::PI * 2.0;
        let rpm = water_wheel_rpm(&w);
        assert!((rpm - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_brake_stops_wheel() {
        let mut w = make_wheel();
        w.omega = 10.0;
        for _ in 0..1000 {
            water_wheel_brake(&mut w, 100.0, 0.1);
        }
        assert!(w.omega.abs() < 1e-4);
    }

    #[test]
    fn test_reset() {
        let mut w = make_wheel();
        w.omega = 5.0;
        w.angle = 3.0;
        water_wheel_reset(&mut w);
        assert_eq!(w.omega, 0.0);
        assert_eq!(w.angle, 0.0);
    }

    #[test]
    fn test_zero_flow_no_torque() {
        let w = make_wheel();
        let t = water_wheel_torque(&w, 0.0, 1000.0, 0.1);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn test_power_at_rest_is_zero() {
        let w = make_wheel();
        let p = water_wheel_power(&w, 2.0, 1000.0, 0.1);
        assert_eq!(p, 0.0);
    }
}
