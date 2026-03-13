// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Smooth-time morph: SmoothDamp-style interpolation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmoothTimeMorph {
    pub current: f32,
    pub target: f32,
    pub velocity: f32,
    pub smooth_time: f32,
}

#[allow(dead_code)]
pub fn new_smooth_time_morph(smooth_time: f32) -> SmoothTimeMorph {
    SmoothTimeMorph { current: 0.0, target: 0.0, velocity: 0.0, smooth_time }
}

#[allow(dead_code)]
pub fn stm_set_target(m: &mut SmoothTimeMorph, target: f32) {
    m.target = target;
}

#[allow(dead_code)]
pub fn stm_step(m: &mut SmoothTimeMorph, dt: f32) {
    let smooth_time = m.smooth_time.max(1e-5);
    let omega = 2.0 / smooth_time;
    let x = omega * dt;
    let exp_factor = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = m.current - m.target;
    let temp = (m.velocity + omega * change) * dt;
    m.velocity = (m.velocity - omega * temp) * exp_factor;
    m.current = m.target + (change + temp) * exp_factor;
}

#[allow(dead_code)]
pub fn stm_value(m: &SmoothTimeMorph) -> f32 {
    m.current
}

#[allow(dead_code)]
pub fn stm_smooth_time(m: &SmoothTimeMorph) -> f32 {
    m.smooth_time
}

#[allow(dead_code)]
pub fn stm_reset(m: &mut SmoothTimeMorph) {
    m.current = 0.0;
    m.velocity = 0.0;
    m.target = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moves_toward_target() {
        let mut m = new_smooth_time_morph(0.3);
        stm_set_target(&mut m, 1.0);
        stm_step(&mut m, 0.1);
        assert!(stm_value(&m) > 0.0);
    }

    #[test]
    fn test_smooth_time_getter() {
        let m = new_smooth_time_morph(0.5);
        assert!((stm_smooth_time(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears() {
        let mut m = new_smooth_time_morph(0.3);
        stm_set_target(&mut m, 1.0);
        stm_step(&mut m, 0.1);
        stm_reset(&mut m);
        assert_eq!(stm_value(&m), 0.0);
    }

    #[test]
    fn test_step_no_crash() {
        let mut m = new_smooth_time_morph(0.1);
        stm_step(&mut m, 0.016);
        assert!(stm_value(&m).is_finite());
    }

    #[test]
    fn test_converges_to_target() {
        let mut m = new_smooth_time_morph(0.1);
        stm_set_target(&mut m, 1.0);
        for _ in 0..100 {
            stm_step(&mut m, 0.05);
        }
        assert!((stm_value(&m) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_initial_value_zero() {
        let m = new_smooth_time_morph(0.3);
        assert_eq!(stm_value(&m), 0.0);
    }

    #[test]
    fn test_set_target_updates() {
        let mut m = new_smooth_time_morph(0.3);
        stm_set_target(&mut m, 0.75);
        assert!((m.target - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_no_overshoot_basic() {
        let mut m = new_smooth_time_morph(0.5);
        stm_set_target(&mut m, 1.0);
        for _ in 0..50 {
            stm_step(&mut m, 0.02);
        }
        /* value should be close to 1.0 and finite */
        assert!(stm_value(&m).is_finite());
    }
}
