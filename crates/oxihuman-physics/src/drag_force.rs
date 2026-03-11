// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Aerodynamic drag model.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DragConfig {
    pub linear_coeff: f32,
    pub quadratic_coeff: f32,
    pub fluid_density: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DragResult {
    pub force: [f32; 3],
    pub power_dissipated: f32,
}

#[allow(dead_code)]
pub fn default_drag_config() -> DragConfig {
    DragConfig { linear_coeff: 0.1, quadratic_coeff: 0.5, fluid_density: 1.225 }
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
pub fn compute_drag_force(vel: [f32; 3], area: f32, config: &DragConfig) -> DragResult {
    let speed = vec3_len(vel);
    if speed < 1e-9 {
        return DragResult { force: [0.0; 3], power_dissipated: 0.0 };
    }

    let dir = [vel[0] / speed, vel[1] / speed, vel[2] / speed];
    let rho = config.fluid_density;

    // F = (linear_coeff * speed + quadratic_coeff * 0.5 * rho * area * speed^2)
    let f_mag = config.linear_coeff * speed
        + config.quadratic_coeff * 0.5 * rho * area * speed * speed;

    // Drag opposes motion
    let force = [-dir[0] * f_mag, -dir[1] * f_mag, -dir[2] * f_mag];
    let power_dissipated = f_mag * speed;

    DragResult { force, power_dissipated }
}

#[allow(dead_code)]
pub fn drag_coefficient_at_speed(config: &DragConfig, speed: f32) -> f32 {
    config.linear_coeff + config.quadratic_coeff * speed
}

#[allow(dead_code)]
pub fn terminal_velocity(mass: f32, area: f32, config: &DragConfig) -> f32 {
    // At terminal velocity: drag = weight (mg). Simplified: solve for v in quadratic drag
    let g = 9.81f32;
    let weight = mass * g;
    // quadratic part dominates: 0.5 * rho * Cd * A * v^2 = weight
    let denom = 0.5 * config.fluid_density * config.quadratic_coeff * area;
    if denom <= 1e-9 {
        return f32::INFINITY;
    }
    (weight / denom).sqrt()
}

#[allow(dead_code)]
pub fn drag_power(vel: [f32; 3], area: f32, config: &DragConfig) -> f32 {
    let res = compute_drag_force(vel, area, config);
    res.power_dissipated
}

#[allow(dead_code)]
pub fn is_in_terminal_velocity(
    vel: [f32; 3],
    mass: f32,
    area: f32,
    config: &DragConfig,
    tol: f32,
) -> bool {
    let speed = vec3_len(vel);
    let tv = terminal_velocity(mass, area, config);
    (speed - tv).abs() < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_velocity_no_drag() {
        let cfg = default_drag_config();
        let res = compute_drag_force([0.0, 0.0, 0.0], 1.0, &cfg);
        assert!((res.force[0]).abs() < 1e-9);
        assert!((res.power_dissipated).abs() < 1e-9);
    }

    #[test]
    fn test_drag_opposes_motion() {
        let cfg = default_drag_config();
        let vel = [1.0, 0.0, 0.0];
        let res = compute_drag_force(vel, 1.0, &cfg);
        assert!(res.force[0] < 0.0);
    }

    #[test]
    fn test_drag_increases_with_speed() {
        let cfg = default_drag_config();
        let res1 = compute_drag_force([1.0, 0.0, 0.0], 1.0, &cfg);
        let res2 = compute_drag_force([2.0, 0.0, 0.0], 1.0, &cfg);
        assert!(res2.power_dissipated > res1.power_dissipated);
    }

    #[test]
    fn test_drag_coefficient_increases_with_speed() {
        let cfg = default_drag_config();
        let c1 = drag_coefficient_at_speed(&cfg, 1.0);
        let c2 = drag_coefficient_at_speed(&cfg, 2.0);
        assert!(c2 > c1);
    }

    #[test]
    fn test_terminal_velocity_positive() {
        let cfg = default_drag_config();
        let tv = terminal_velocity(70.0, 0.5, &cfg);
        assert!(tv > 0.0);
    }

    #[test]
    fn test_drag_power_positive() {
        let cfg = default_drag_config();
        let p = drag_power([1.0, 0.0, 0.0], 1.0, &cfg);
        assert!(p > 0.0);
    }

    #[test]
    fn test_is_in_terminal_velocity_false() {
        let cfg = default_drag_config();
        assert!(!is_in_terminal_velocity([0.1, 0.0, 0.0], 70.0, 0.5, &cfg, 0.1));
    }

    #[test]
    fn test_is_in_terminal_velocity_true() {
        let cfg = default_drag_config();
        let tv = terminal_velocity(70.0, 0.5, &cfg);
        assert!(is_in_terminal_velocity([tv, 0.0, 0.0], 70.0, 0.5, &cfg, 1.0));
    }

    #[test]
    fn test_power_dissipated_nonneg() {
        let cfg = default_drag_config();
        let res = compute_drag_force([3.0, 4.0, 0.0], 0.5, &cfg);
        assert!(res.power_dissipated >= 0.0);
    }
}
