// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// A vehicle suspension spring.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSpring {
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_travel: f32,
}

/// Create a new suspension spring.
#[allow(dead_code)]
pub fn new_suspension(rest: f32, k: f32, d: f32, max: f32) -> SuspensionSpring {
    SuspensionSpring {
        rest_length: rest,
        stiffness: k,
        damping: d,
        max_travel: max,
    }
}

/// Compute the spring force given current length and compression velocity.
/// Positive velocity means compressing, negative means extending.
/// Force is positive (upward/outward) when compressed.
#[allow(dead_code)]
pub fn suspension_force(spring: &SuspensionSpring, current_len: f32, velocity: f32) -> f32 {
    let travel = spring_travel(spring, current_len).clamp(-spring.max_travel, spring.max_travel);
    let spring_f = spring.stiffness * travel;
    let damp_f = spring.damping * velocity;
    (spring_f + damp_f).max(0.0)
}

/// Check whether the suspension is compressed (current_len < rest_length).
#[allow(dead_code)]
pub fn suspension_compressed(spring: &SuspensionSpring, current_len: f32) -> bool {
    current_len < spring.rest_length
}

/// Get the compression travel (positive = compressed, negative = extended).
#[allow(dead_code)]
pub fn spring_travel(spring: &SuspensionSpring, current_len: f32) -> f32 {
    spring.rest_length - current_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_at_rest_length_zero() {
        let s = new_suspension(1.0, 100.0, 10.0, 0.5);
        // At rest length, no velocity → force = 0
        let f = suspension_force(&s, 1.0, 0.0);
        assert!((f).abs() < 1e-5);
    }

    #[test]
    fn force_when_compressed_positive() {
        let s = new_suspension(1.0, 100.0, 5.0, 0.5);
        // Compressed by 0.1, no velocity
        let f = suspension_force(&s, 0.9, 0.0);
        assert!(f > 0.0);
    }

    #[test]
    fn force_when_fully_extended_zero() {
        let s = new_suspension(1.0, 100.0, 5.0, 0.5);
        // Extended beyond rest: spring tries to pull, but clamped to 0
        let f = suspension_force(&s, 1.5, 0.0);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn suspension_compressed_true() {
        let s = new_suspension(1.0, 100.0, 5.0, 0.5);
        assert!(suspension_compressed(&s, 0.8));
    }

    #[test]
    fn suspension_compressed_false() {
        let s = new_suspension(1.0, 100.0, 5.0, 0.5);
        assert!(!suspension_compressed(&s, 1.2));
    }

    #[test]
    fn spring_travel_positive_when_compressed() {
        let s = new_suspension(1.0, 100.0, 5.0, 0.5);
        assert!(spring_travel(&s, 0.7) > 0.0);
    }

    #[test]
    fn spring_travel_negative_when_extended() {
        let s = new_suspension(1.0, 100.0, 5.0, 0.5);
        assert!(spring_travel(&s, 1.3) < 0.0);
    }

    #[test]
    fn damping_increases_force() {
        let s_no_damp = new_suspension(1.0, 100.0, 0.0, 0.5);
        let s_damp = new_suspension(1.0, 100.0, 20.0, 0.5);
        let f_no = suspension_force(&s_no_damp, 0.9, 1.0);
        let f_damp = suspension_force(&s_damp, 0.9, 1.0);
        assert!(f_damp > f_no);
    }

    #[test]
    fn max_travel_clamps_force() {
        let s = new_suspension(1.0, 100.0, 0.0, 0.1);
        // Compressed by 0.5 but max_travel = 0.1
        let f = suspension_force(&s, 0.5, 0.0);
        // Max force = stiffness * max_travel = 10.0
        assert!((f - 10.0).abs() < 1e-4);
    }

    #[test]
    fn new_suspension_fields() {
        let s = new_suspension(0.5, 200.0, 15.0, 0.3);
        assert!((s.rest_length - 0.5).abs() < 1e-6);
        assert!((s.stiffness - 200.0).abs() < 1e-6);
        assert!((s.damping - 15.0).abs() < 1e-6);
        assert!((s.max_travel - 0.3).abs() < 1e-6);
    }
}
