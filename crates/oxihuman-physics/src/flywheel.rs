// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Flywheel with moment of inertia, angular velocity, and braking.

#![allow(dead_code)]

/// A flywheel energy storage/delivery device.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Flywheel {
    /// Moment of inertia in kg·m².
    pub inertia: f32,
    /// Angular velocity in rad/s.
    pub omega: f32,
    /// Viscous drag coefficient.
    pub drag: f32,
    /// Maximum angular velocity (rad/s).
    pub omega_max: f32,
}

/// Create a new flywheel.
#[allow(dead_code)]
pub fn new_flywheel(inertia: f32, drag: f32, omega_max: f32) -> Flywheel {
    Flywheel {
        inertia: inertia.max(1e-6),
        omega: 0.0,
        drag,
        omega_max: omega_max.abs(),
    }
}

/// Apply a driving torque to the flywheel and step by dt.
#[allow(dead_code)]
pub fn flywheel_step(fw: &mut Flywheel, torque: f32, dt: f32) {
    let drag_torque = -fw.drag * fw.omega;
    let alpha = (torque + drag_torque) / fw.inertia;
    fw.omega += alpha * dt;
    fw.omega = fw.omega.clamp(-fw.omega_max, fw.omega_max);
}

/// Kinetic energy stored: 0.5 * I * omega^2.
#[allow(dead_code)]
pub fn flywheel_energy(fw: &Flywheel) -> f32 {
    0.5 * fw.inertia * fw.omega * fw.omega
}

/// Apply regenerative braking: extract power and decelerate.
/// Returns the extracted torque (always positive).
#[allow(dead_code)]
pub fn flywheel_brake(fw: &mut Flywheel, brake_torque: f32, dt: f32) -> f32 {
    if fw.omega.abs() < 1e-8 {
        return 0.0;
    }
    let sign = if fw.omega > 0.0 { -1.0 } else { 1.0 };
    let alpha = sign * brake_torque.abs() / fw.inertia;
    let new_omega = fw.omega + alpha * dt;
    if fw.omega * new_omega < 0.0 {
        let extracted = fw.omega;
        fw.omega = 0.0;
        extracted.abs() * fw.inertia / dt
    } else {
        fw.omega = new_omega;
        brake_torque.abs()
    }
}

/// RPM of the flywheel.
#[allow(dead_code)]
pub fn flywheel_rpm(fw: &Flywheel) -> f32 {
    fw.omega * 60.0 / (2.0 * std::f32::consts::PI)
}

/// Power delivered by the flywheel to a load torque.
#[allow(dead_code)]
pub fn flywheel_power(fw: &Flywheel, load_torque: f32) -> f32 {
    fw.omega * load_torque
}

/// Check if the flywheel is at maximum speed.
#[allow(dead_code)]
pub fn flywheel_at_max(fw: &Flywheel) -> bool {
    fw.omega.abs() >= fw.omega_max - 1e-4
}

/// Reset the flywheel.
#[allow(dead_code)]
pub fn flywheel_reset(fw: &mut Flywheel) {
    fw.omega = 0.0;
}

/// Angular momentum: L = I * omega.
#[allow(dead_code)]
pub fn flywheel_angular_momentum(fw: &Flywheel) -> f32 {
    fw.inertia * fw.omega
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fw() -> Flywheel {
        new_flywheel(10.0, 0.1, 1000.0)
    }

    #[test]
    fn test_initial_state() {
        let fw = make_fw();
        assert_eq!(fw.omega, 0.0);
        assert!(fw.inertia > 0.0);
    }

    #[test]
    fn test_step_positive_torque() {
        let mut fw = make_fw();
        flywheel_step(&mut fw, 100.0, 1.0);
        assert!(fw.omega > 0.0);
    }

    #[test]
    fn test_energy_increases() {
        let mut fw = make_fw();
        flywheel_step(&mut fw, 100.0, 1.0);
        assert!(flywheel_energy(&fw) > 0.0);
    }

    #[test]
    fn test_omega_clamped_to_max() {
        let mut fw = new_flywheel(1.0, 0.0, 10.0);
        fw.omega = 9.9;
        flywheel_step(&mut fw, 1000.0, 1.0);
        assert!(fw.omega <= fw.omega_max + 1e-4);
    }

    #[test]
    fn test_brake_decelerates() {
        let mut fw = make_fw();
        fw.omega = 100.0;
        flywheel_brake(&mut fw, 50.0, 0.1);
        assert!(fw.omega < 100.0);
    }

    #[test]
    fn test_rpm_conversion() {
        let mut fw = make_fw();
        fw.omega = 2.0 * std::f32::consts::PI;
        assert!((flywheel_rpm(&fw) - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_at_max_speed() {
        let mut fw = make_fw();
        fw.omega = fw.omega_max;
        assert!(flywheel_at_max(&fw));
    }

    #[test]
    fn test_reset() {
        let mut fw = make_fw();
        fw.omega = 50.0;
        flywheel_reset(&mut fw);
        assert_eq!(fw.omega, 0.0);
    }

    #[test]
    fn test_angular_momentum() {
        let mut fw = make_fw();
        fw.omega = 10.0;
        let l = flywheel_angular_momentum(&fw);
        assert!((l - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_power_output() {
        let mut fw = make_fw();
        fw.omega = 10.0;
        let p = flywheel_power(&fw, 5.0);
        assert!((p - 50.0).abs() < 1e-4);
    }
}
