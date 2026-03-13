// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Grip force regulation model.

pub struct GripForce {
    pub target_force_n: f32,
    pub current_force_n: f32,
    pub slip_threshold: f32,
    pub friction_coeff: f32,
    pub safety_margin: f32,
}

pub fn new_grip_force() -> GripForce {
    GripForce {
        target_force_n: 10.0,
        current_force_n: 0.0,
        slip_threshold: 0.8,
        friction_coeff: 0.5,
        safety_margin: 1.2,
    }
}

pub fn grip_step(g: &mut GripForce, dt: f32) {
    /* exponential approach to target with time constant 0.1 s */
    let tau = 0.1_f32;
    let alpha = 1.0 - (-dt / tau).exp();
    g.current_force_n += alpha * (g.target_force_n - g.current_force_n);
}

pub fn grip_is_slipping(g: &GripForce, load_n: f32) -> bool {
    let max_friction = g.friction_coeff * g.current_force_n;
    load_n > max_friction * g.slip_threshold
}

pub fn grip_required_force(g: &GripForce, load_n: f32) -> f32 {
    let required = load_n / (g.friction_coeff * g.slip_threshold);
    required * g.safety_margin
}

pub fn grip_set_target(g: &mut GripForce, target: f32) {
    g.target_force_n = target.max(0.0);
}

pub fn grip_overshoot_factor(g: &GripForce) -> f32 {
    if g.target_force_n < 1e-6 {
        return 1.0;
    }
    g.current_force_n / g.target_force_n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grip_force() {
        /* new grip force starts at zero current force */
        let g = new_grip_force();
        assert_eq!(g.current_force_n, 0.0);
    }

    #[test]
    fn test_grip_step_approaches_target() {
        /* grip force approaches target after steps */
        let mut g = new_grip_force();
        for _ in 0..100 {
            grip_step(&mut g, 0.01);
        }
        assert!((g.current_force_n - g.target_force_n).abs() < 0.1);
    }

    #[test]
    fn test_grip_is_slipping_when_weak() {
        /* weak grip slips under high load */
        let mut g = new_grip_force();
        g.current_force_n = 1.0;
        assert!(grip_is_slipping(&g, 10.0));
    }

    #[test]
    fn test_grip_not_slipping_when_strong() {
        /* strong grip does not slip under low load */
        let mut g = new_grip_force();
        g.current_force_n = 100.0;
        assert!(!grip_is_slipping(&g, 1.0));
    }

    #[test]
    fn test_grip_required_force() {
        /* required force is positive */
        let g = new_grip_force();
        let r = grip_required_force(&g, 10.0);
        assert!(r > 0.0);
    }

    #[test]
    fn test_grip_set_target() {
        /* set_target changes target force */
        let mut g = new_grip_force();
        grip_set_target(&mut g, 50.0);
        assert!((g.target_force_n - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_grip_overshoot_factor() {
        /* overshoot factor is >= 0 */
        let g = new_grip_force();
        let f = grip_overshoot_factor(&g);
        assert!(f >= 0.0);
    }
}
