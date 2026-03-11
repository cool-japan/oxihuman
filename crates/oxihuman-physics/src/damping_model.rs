// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Various damping models for physics simulation.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DampingKind {
    Linear,
    Quadratic,
    Exponential,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DampingConfig {
    pub kind: DampingKind,
    pub coefficient: f32,
}

#[allow(dead_code)]
impl DampingConfig {
    pub fn linear(coefficient: f32) -> Self {
        Self { kind: DampingKind::Linear, coefficient }
    }

    pub fn quadratic(coefficient: f32) -> Self {
        Self { kind: DampingKind::Quadratic, coefficient }
    }

    pub fn exponential(coefficient: f32) -> Self {
        Self { kind: DampingKind::Exponential, coefficient }
    }
}

#[allow(dead_code)]
pub fn apply_linear_damping(velocity: f32, coefficient: f32, dt: f32) -> f32 {
    let factor = (1.0 - coefficient * dt).max(0.0);
    velocity * factor
}

#[allow(dead_code)]
pub fn apply_quadratic_damping(velocity: f32, coefficient: f32, dt: f32) -> f32 {
    let drag = coefficient * velocity * velocity.abs() * dt;
    if velocity.abs() < drag.abs() {
        0.0
    } else {
        velocity - drag.copysign(velocity)
    }
}

#[allow(dead_code)]
pub fn apply_exponential_damping(velocity: f32, coefficient: f32, dt: f32) -> f32 {
    velocity * (-coefficient * dt).exp()
}

#[allow(dead_code)]
pub fn apply_damping(velocity: f32, config: &DampingConfig, dt: f32) -> f32 {
    match config.kind {
        DampingKind::Linear => apply_linear_damping(velocity, config.coefficient, dt),
        DampingKind::Quadratic => apply_quadratic_damping(velocity, config.coefficient, dt),
        DampingKind::Exponential => apply_exponential_damping(velocity, config.coefficient, dt),
    }
}

#[allow(dead_code)]
pub fn apply_damping_vec3(vel: [f32; 3], config: &DampingConfig, dt: f32) -> [f32; 3] {
    [
        apply_damping(vel[0], config, dt),
        apply_damping(vel[1], config, dt),
        apply_damping(vel[2], config, dt),
    ]
}

#[allow(dead_code)]
pub fn damping_energy_loss(velocity: f32, config: &DampingConfig, dt: f32) -> f32 {
    let new_vel = apply_damping(velocity, config, dt);
    0.5 * (velocity * velocity - new_vel * new_vel)
}

#[allow(dead_code)]
pub fn critical_damping_coefficient(mass: f32, stiffness: f32) -> f32 {
    2.0 * (mass * stiffness).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_damping() {
        let v = apply_linear_damping(10.0, 0.1, 1.0);
        assert!((v - 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_linear_damping_clamp() {
        let v = apply_linear_damping(10.0, 2.0, 1.0);
        assert!((v).abs() < 1e-5);
    }

    #[test]
    fn test_exponential_damping() {
        let v = apply_exponential_damping(10.0, 1.0, 0.0);
        assert!((v - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_exponential_decay() {
        let v = apply_exponential_damping(10.0, 1.0, 1.0);
        assert!(v < 10.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_quadratic_damping() {
        let v = apply_quadratic_damping(10.0, 0.01, 1.0);
        assert!(v < 10.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_apply_damping_dispatch() {
        let cfg = DampingConfig::linear(0.5);
        let v = apply_damping(4.0, &cfg, 1.0);
        assert!((v - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_vec3_damping() {
        let cfg = DampingConfig::linear(0.1);
        let out = apply_damping_vec3([10.0, 20.0, 30.0], &cfg, 1.0);
        assert!((out[0] - 9.0).abs() < 1e-4);
    }

    #[test]
    fn test_energy_loss() {
        let cfg = DampingConfig::linear(0.1);
        let loss = damping_energy_loss(10.0, &cfg, 1.0);
        assert!(loss > 0.0);
    }

    #[test]
    fn test_critical_damping() {
        let c = critical_damping_coefficient(1.0, 4.0);
        assert!((c - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_damping_config_constructors() {
        let a = DampingConfig::linear(1.0);
        let b = DampingConfig::quadratic(2.0);
        let c = DampingConfig::exponential(3.0);
        assert_eq!(a.kind, DampingKind::Linear);
        assert_eq!(b.kind, DampingKind::Quadratic);
        assert_eq!(c.kind, DampingKind::Exponential);
    }
}
