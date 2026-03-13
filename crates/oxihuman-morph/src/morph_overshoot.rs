// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Overshoot animation: exceeds target then settles.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OvershootMorph {
    pub target: f32,
    pub current: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub overshoot_factor: f32,
}

#[allow(dead_code)]
pub fn new_overshoot_morph(stiffness: f32, overshoot_factor: f32) -> OvershootMorph {
    OvershootMorph { target: 0.0, current: 0.0, velocity: 0.0, stiffness, overshoot_factor }
}

#[allow(dead_code)]
pub fn om_set_target(m: &mut OvershootMorph, target: f32) {
    m.target = target;
    /* give initial velocity boost in direction of target */
    let dir = if target > m.current { 1.0 } else { -1.0 };
    m.velocity = dir * m.overshoot_factor;
}

#[allow(dead_code)]
pub fn om_step(m: &mut OvershootMorph, dt: f32) {
    /* low damping spring to allow overshoot */
    let damping = m.stiffness * 0.1;
    let force = m.stiffness * (m.target - m.current) - damping * m.velocity;
    m.velocity += force * dt;
    m.current += m.velocity * dt;
}

#[allow(dead_code)]
pub fn om_value(m: &OvershootMorph) -> f32 {
    m.current
}

#[allow(dead_code)]
pub fn om_overshoot_factor(m: &OvershootMorph) -> f32 {
    m.overshoot_factor
}

#[allow(dead_code)]
pub fn om_reset(m: &mut OvershootMorph) {
    m.current = 0.0;
    m.velocity = 0.0;
    m.target = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moves_toward_target() {
        let mut m = new_overshoot_morph(10.0, 0.5);
        om_set_target(&mut m, 1.0);
        om_step(&mut m, 0.1);
        assert!(om_value(&m) > 0.0);
    }

    #[test]
    fn test_overshoot_factor_getter() {
        let m = new_overshoot_morph(10.0, 0.75);
        assert!((om_overshoot_factor(&m) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut m = new_overshoot_morph(10.0, 0.5);
        om_set_target(&mut m, 1.0);
        om_step(&mut m, 0.2);
        om_reset(&mut m);
        assert_eq!(om_value(&m), 0.0);
        assert_eq!(m.velocity, 0.0);
        assert_eq!(m.target, 0.0);
    }

    #[test]
    fn test_value_after_step() {
        let mut m = new_overshoot_morph(5.0, 0.2);
        om_set_target(&mut m, 0.5);
        om_step(&mut m, 0.05);
        let v = om_value(&m);
        assert!(v.is_finite());
    }

    #[test]
    fn test_initial_velocity_boost() {
        let mut m = new_overshoot_morph(10.0, 2.0);
        om_set_target(&mut m, 1.0);
        assert!(m.velocity > 0.0);
    }

    #[test]
    fn test_negative_target_velocity_negative() {
        let mut m = new_overshoot_morph(10.0, 1.0);
        om_set_target(&mut m, -1.0);
        assert!(m.velocity < 0.0);
    }

    #[test]
    fn test_multiple_steps_finite() {
        let mut m = new_overshoot_morph(8.0, 0.3);
        om_set_target(&mut m, 0.8);
        for _ in 0..50 {
            om_step(&mut m, 0.02);
        }
        assert!(om_value(&m).is_finite());
    }

    #[test]
    fn test_new_initial_state() {
        let m = new_overshoot_morph(10.0, 0.5);
        assert_eq!(m.current, 0.0);
        assert_eq!(m.velocity, 0.0);
        assert_eq!(m.target, 0.0);
    }
}
