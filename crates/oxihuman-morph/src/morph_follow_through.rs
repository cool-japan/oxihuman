// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Follow-through: continuation past target after main action.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FollowThroughMorph {
    pub target: f32,
    pub current: f32,
    pub velocity: f32,
    pub follow_factor: f32,
}

#[allow(dead_code)]
pub fn new_follow_through_morph(follow_factor: f32) -> FollowThroughMorph {
    FollowThroughMorph { target: 0.0, current: 0.0, velocity: 0.0, follow_factor }
}

#[allow(dead_code)]
pub fn ft_set_target(m: &mut FollowThroughMorph, target: f32) {
    let dir = if target > m.current { 1.0 } else { -1.0 };
    m.target = target;
    /* initial velocity overshoots past target */
    m.velocity = dir * m.follow_factor;
}

#[allow(dead_code)]
pub fn ft_step(m: &mut FollowThroughMorph, dt: f32) {
    /* spring pulling back to target */
    let stiffness: f32 = 8.0;
    let damping = 2.0 * stiffness.sqrt();
    let force = stiffness * (m.target - m.current) - damping * m.velocity;
    m.velocity += force * dt;
    m.current += m.velocity * dt;
}

#[allow(dead_code)]
pub fn ft_value(m: &FollowThroughMorph) -> f32 {
    m.current
}

#[allow(dead_code)]
pub fn ft_follow_factor(m: &FollowThroughMorph) -> f32 {
    m.follow_factor
}

#[allow(dead_code)]
pub fn ft_reset(m: &mut FollowThroughMorph) {
    m.current = 0.0;
    m.velocity = 0.0;
    m.target = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moves_past_target_initially() {
        let mut m = new_follow_through_morph(2.0);
        ft_set_target(&mut m, 1.0);
        ft_step(&mut m, 0.1);
        /* positive velocity from follow_factor means current > 0 */
        assert!(ft_value(&m) > 0.0);
    }

    #[test]
    fn test_follow_factor_getter() {
        let m = new_follow_through_morph(0.6);
        assert!((ft_follow_factor(&m) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut m = new_follow_through_morph(1.0);
        ft_set_target(&mut m, 1.0);
        ft_step(&mut m, 0.2);
        ft_reset(&mut m);
        assert_eq!(ft_value(&m), 0.0);
        assert_eq!(m.velocity, 0.0);
        assert_eq!(m.target, 0.0);
    }

    #[test]
    fn test_step_no_crash() {
        let mut m = new_follow_through_morph(0.5);
        ft_step(&mut m, 0.016);
        assert!(ft_value(&m).is_finite());
    }

    #[test]
    fn test_initial_velocity_nonzero_after_set_target() {
        let mut m = new_follow_through_morph(1.5);
        ft_set_target(&mut m, 1.0);
        assert!(m.velocity.abs() > 0.0);
    }

    #[test]
    fn test_converges_over_time() {
        let mut m = new_follow_through_morph(0.1);
        ft_set_target(&mut m, 1.0);
        for _ in 0..200 {
            ft_step(&mut m, 0.05);
        }
        assert!((ft_value(&m) - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_new_initial_state() {
        let m = new_follow_through_morph(1.0);
        assert_eq!(m.current, 0.0);
        assert_eq!(m.velocity, 0.0);
    }

    #[test]
    fn test_negative_target() {
        let mut m = new_follow_through_morph(1.0);
        ft_set_target(&mut m, -1.0);
        assert!(m.velocity < 0.0);
    }
}
