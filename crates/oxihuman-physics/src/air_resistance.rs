// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Compute aerodynamic drag force vector opposing velocity.
/// F_drag = -0.5 * rho * cd * area * |v|^2 * v_hat
#[allow(dead_code)]
pub fn drag_force(vel: [f32; 3], cd: f32, area: f32, rho: f32) -> [f32; 3] {
    let speed_sq = vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2];
    if speed_sq < 1e-12 {
        return [0.0; 3];
    }
    let speed = speed_sq.sqrt();
    let mag = 0.5 * rho * cd * area * speed_sq;
    let scale = -mag / speed;
    [vel[0] * scale, vel[1] * scale, vel[2] * scale]
}

/// Terminal velocity: v_t = sqrt(2 * mass * g / (rho * cd * area))
#[allow(dead_code)]
pub fn terminal_velocity(mass: f32, cd: f32, area: f32, rho: f32, g: f32) -> f32 {
    let denom = rho * cd * area;
    if denom < 1e-12 {
        return f32::INFINITY;
    }
    (2.0 * mass * g / denom).sqrt()
}

/// Deceleration due to drag: a = 0.5 * rho * cd * area * speed^2 / mass
#[allow(dead_code)]
pub fn drag_decel(speed: f32, cd: f32, area: f32, rho: f32, mass: f32) -> f32 {
    if mass < 1e-12 {
        return 0.0;
    }
    0.5 * rho * cd * area * speed * speed / mass
}

/// Reynolds number: Re = speed * length / viscosity
#[allow(dead_code)]
pub fn reynolds_number(speed: f32, length: f32, viscosity: f32) -> f32 {
    if viscosity < 1e-12 {
        return f32::INFINITY;
    }
    speed * length / viscosity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_force_zero_velocity() {
        let f = drag_force([0.0; 3], 1.0, 1.0, 1.0);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn drag_force_opposes_velocity() {
        let vel = [1.0, 0.0, 0.0];
        let f = drag_force(vel, 1.0, 1.0, 1.0);
        assert!(f[0] < 0.0, "drag must oppose positive x velocity");
    }

    #[test]
    fn drag_force_magnitude_scales_with_speed_sq() {
        let f1 = drag_force([1.0, 0.0, 0.0], 1.0, 1.0, 1.0);
        let f2 = drag_force([2.0, 0.0, 0.0], 1.0, 1.0, 1.0);
        // drag ~ v^2, so doubling speed quadruples force
        assert!((f2[0].abs() / f1[0].abs() - 4.0).abs() < 1e-4);
    }

    #[test]
    fn terminal_velocity_positive() {
        let vt = terminal_velocity(1.0, 0.5, 1.0, 1.2, 9.81);
        assert!(vt > 0.0);
    }

    #[test]
    fn terminal_velocity_zero_area_gives_infinity() {
        let vt = terminal_velocity(1.0, 0.5, 0.0, 1.2, 9.81);
        assert!(vt.is_infinite());
    }

    #[test]
    fn drag_decel_positive() {
        let a = drag_decel(10.0, 0.5, 1.0, 1.2, 1.0);
        assert!(a > 0.0);
    }

    #[test]
    fn drag_decel_zero_mass_returns_zero() {
        let a = drag_decel(10.0, 0.5, 1.0, 1.2, 0.0);
        assert_eq!(a, 0.0);
    }

    #[test]
    fn reynolds_number_basic() {
        // Re = 10.0 * 0.1 / 1e-5 = 100000
        let re = reynolds_number(10.0, 0.1, 1e-5);
        assert!((re - 100_000.0).abs() < 1.0);
    }

    #[test]
    fn reynolds_zero_viscosity_gives_infinity() {
        let re = reynolds_number(1.0, 1.0, 0.0);
        assert!(re.is_infinite());
    }

    #[test]
    fn drag_force_3d() {
        let f = drag_force([0.0, 1.0, 0.0], 1.0, 1.0, 1.0);
        assert!(f[1] < 0.0);
        assert!((f[0]).abs() < 1e-6);
    }
}
