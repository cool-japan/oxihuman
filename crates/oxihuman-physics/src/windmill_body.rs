// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Windmill body with wind-driven rotation.

#![allow(dead_code)]

/// A windmill with wind-driven rotor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindmillBody {
    /// Rotor radius in meters.
    pub radius: f32,
    /// Number of blades.
    pub num_blades: u32,
    /// Moment of inertia in kg·m².
    pub inertia: f32,
    /// Angular velocity in rad/s.
    pub omega: f32,
    /// Viscous damping coefficient.
    pub damping: f32,
    /// Accumulated angle in radians.
    pub angle: f32,
    /// Power coefficient (efficiency), 0.0..1.0.
    pub cp: f32,
}

/// Create a new windmill.
#[allow(dead_code)]
pub fn new_windmill(
    radius: f32,
    mass: f32,
    num_blades: u32,
    damping: f32,
    cp: f32,
) -> WindmillBody {
    let inertia = 0.5 * mass * radius * radius;
    WindmillBody {
        radius,
        num_blades,
        inertia: inertia.max(1e-6),
        omega: 0.0,
        damping,
        angle: 0.0,
        cp: cp.clamp(0.0, 0.593),
    }
}

/// Compute power extracted from wind (Betz theory with Cp): P = 0.5 * rho * A * v^3 * Cp.
#[allow(dead_code)]
pub fn windmill_power(wm: &WindmillBody, wind_speed: f32, rho: f32) -> f32 {
    let area = std::f32::consts::PI * wm.radius * wm.radius;
    0.5 * rho * area * wind_speed * wind_speed * wind_speed * wm.cp
}

/// Compute rotor torque from wind: T = P / omega (handle omega=0 case).
#[allow(dead_code)]
pub fn windmill_torque(wm: &WindmillBody, wind_speed: f32, rho: f32) -> f32 {
    if wm.omega.abs() < 1e-6 {
        let area = std::f32::consts::PI * wm.radius * wm.radius;
        return 0.5 * rho * area * wind_speed * wind_speed * wm.radius * wm.cp * 0.5;
    }
    windmill_power(wm, wind_speed, rho) / wm.omega
}

/// Step the windmill simulation.
#[allow(dead_code)]
pub fn windmill_step(wm: &mut WindmillBody, wind_speed: f32, rho: f32, dt: f32) {
    let torque = windmill_torque(wm, wind_speed, rho);
    let drag = -wm.damping * wm.omega;
    let alpha = (torque + drag) / wm.inertia;
    wm.omega += alpha * dt;
    if wm.omega < 0.0 {
        wm.omega = 0.0;
    }
    wm.angle += wm.omega * dt;
}

/// Tip-speed ratio: lambda = omega * R / v_wind.
#[allow(dead_code)]
pub fn windmill_tsr(wm: &WindmillBody, wind_speed: f32) -> f32 {
    if wind_speed.abs() < 1e-6 {
        return 0.0;
    }
    wm.omega * wm.radius / wind_speed
}

/// Kinetic energy of the rotor.
#[allow(dead_code)]
pub fn windmill_energy(wm: &WindmillBody) -> f32 {
    0.5 * wm.inertia * wm.omega * wm.omega
}

/// RPM of the rotor.
#[allow(dead_code)]
pub fn windmill_rpm(wm: &WindmillBody) -> f32 {
    wm.omega * 60.0 / (2.0 * std::f32::consts::PI)
}

/// Apply a load torque (e.g., from a generator).
#[allow(dead_code)]
pub fn windmill_apply_load(wm: &mut WindmillBody, load_torque: f32, dt: f32) {
    let alpha = -load_torque.abs() / wm.inertia;
    let new_omega = wm.omega + alpha * dt;
    wm.omega = new_omega.max(0.0);
}

/// Reset the windmill.
#[allow(dead_code)]
pub fn windmill_reset(wm: &mut WindmillBody) {
    wm.omega = 0.0;
    wm.angle = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wm() -> WindmillBody {
        new_windmill(5.0, 200.0, 3, 0.5, 0.4)
    }

    #[test]
    fn test_initial_state() {
        let wm = make_wm();
        assert_eq!(wm.omega, 0.0);
        assert!(wm.inertia > 0.0);
    }

    #[test]
    fn test_step_increases_omega() {
        let mut wm = make_wm();
        windmill_step(&mut wm, 10.0, 1.225, 1.0);
        assert!(wm.omega > 0.0);
    }

    #[test]
    fn test_angle_increases() {
        let mut wm = make_wm();
        windmill_step(&mut wm, 10.0, 1.225, 1.0);
        assert!(wm.angle >= 0.0);
    }

    #[test]
    fn test_power_positive() {
        let wm = make_wm();
        let p = windmill_power(&wm, 10.0, 1.225);
        assert!(p > 0.0);
    }

    #[test]
    fn test_energy_after_step() {
        let mut wm = make_wm();
        windmill_step(&mut wm, 10.0, 1.225, 1.0);
        assert!(windmill_energy(&wm) >= 0.0);
    }

    #[test]
    fn test_tsr_at_rest_is_zero() {
        let wm = make_wm();
        assert_eq!(windmill_tsr(&wm, 10.0), 0.0);
    }

    #[test]
    fn test_rpm_at_rest() {
        let wm = make_wm();
        assert_eq!(windmill_rpm(&wm), 0.0);
    }

    #[test]
    fn test_reset() {
        let mut wm = make_wm();
        wm.omega = 5.0;
        wm.angle = 10.0;
        windmill_reset(&mut wm);
        assert_eq!(wm.omega, 0.0);
        assert_eq!(wm.angle, 0.0);
    }

    #[test]
    fn test_no_omega_below_zero() {
        let mut wm = make_wm();
        windmill_step(&mut wm, 0.0, 1.225, 0.1);
        assert!(wm.omega >= 0.0);
    }

    #[test]
    fn test_cp_clamped() {
        let wm = new_windmill(5.0, 200.0, 3, 0.5, 1.0);
        assert!(wm.cp <= 0.593);
    }
}
