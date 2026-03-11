// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Spring-based morph animation with damping.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringMorphV2 {
    pub target: f32,
    pub current: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
pub fn new_spring_morph_v2(stiffness: f32, damping: f32) -> SpringMorphV2 {
    SpringMorphV2 { target: 0.0, current: 0.0, velocity: 0.0, stiffness, damping }
}

#[allow(dead_code)]
pub fn smv2_set_target(spring: &mut SpringMorphV2, target: f32) {
    spring.target = target;
}

#[allow(dead_code)]
pub fn smv2_step(spring: &mut SpringMorphV2, dt: f32) {
    let force = spring.stiffness * (spring.target - spring.current)
        - spring.damping * spring.velocity;
    spring.velocity += force * dt;
    spring.current += spring.velocity * dt;
}

#[allow(dead_code)]
pub fn smv2_value(spring: &SpringMorphV2) -> f32 {
    spring.current
}

#[allow(dead_code)]
pub fn smv2_is_at_rest(spring: &SpringMorphV2, tol: f32) -> bool {
    (spring.current - spring.target).abs() < tol && spring.velocity.abs() < tol
}

#[allow(dead_code)]
pub fn smv2_energy(spring: &SpringMorphV2) -> f32 {
    0.5 * spring.stiffness * (spring.current - spring.target).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moves_toward_target() {
        let mut s = new_spring_morph_v2(10.0, 1.0);
        smv2_set_target(&mut s, 1.0);
        smv2_step(&mut s, 0.1);
        assert!(smv2_value(&s) > 0.0);
    }

    #[test]
    fn test_velocity_nonzero_after_step() {
        let mut s = new_spring_morph_v2(10.0, 0.5);
        smv2_set_target(&mut s, 1.0);
        smv2_step(&mut s, 0.1);
        assert!(s.velocity > 0.0);
    }

    #[test]
    fn test_is_at_rest_initially() {
        let s = new_spring_morph_v2(10.0, 1.0);
        assert!(smv2_is_at_rest(&s, 0.01));
    }

    #[test]
    fn test_is_not_at_rest_after_target_set() {
        let mut s = new_spring_morph_v2(10.0, 1.0);
        smv2_set_target(&mut s, 1.0);
        assert!(!smv2_is_at_rest(&s, 0.01));
    }

    #[test]
    fn test_energy_zero_at_rest() {
        let s = new_spring_morph_v2(10.0, 1.0);
        assert!((smv2_energy(&s)).abs() < 1e-6);
    }

    #[test]
    fn test_energy_nonzero_when_displaced() {
        let mut s = new_spring_morph_v2(10.0, 1.0);
        smv2_set_target(&mut s, 1.0);
        smv2_step(&mut s, 0.1);
        assert!(smv2_energy(&s) > 0.0);
    }

    #[test]
    fn test_multiple_steps_converge() {
        let mut s = new_spring_morph_v2(50.0, 10.0);
        smv2_set_target(&mut s, 1.0);
        for _ in 0..200 {
            smv2_step(&mut s, 0.05);
        }
        assert!((smv2_value(&s) - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_set_target_updates() {
        let mut s = new_spring_morph_v2(10.0, 1.0);
        smv2_set_target(&mut s, 0.5);
        assert!((s.target - 0.5).abs() < 1e-6);
    }
}
