#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Linear (translational) constraint between bodies.

/// A linear constraint with limits.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LinearConstraint {
    pub min_limit: f32,
    pub max_limit: f32,
    pub current: f32,
    pub stiffness: f32,
    pub limited: bool,
}

#[allow(dead_code)]
pub fn new_linear_constraint(min_limit: f32, max_limit: f32, stiffness: f32) -> LinearConstraint {
    LinearConstraint {
        min_limit,
        max_limit,
        current: 0.0,
        stiffness,
        limited: true,
    }
}

#[allow(dead_code)]
pub fn linear_error(c: &LinearConstraint) -> f32 {
    if !c.limited {
        return 0.0;
    }
    if c.current < c.min_limit {
        c.current - c.min_limit
    } else if c.current > c.max_limit {
        c.current - c.max_limit
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn solve_linear(c: &mut LinearConstraint) -> f32 {
    let err = linear_error(c);
    let correction = -err * c.stiffness;
    c.current += correction;
    correction
}

#[allow(dead_code)]
pub fn linear_limit_min(c: &LinearConstraint) -> f32 {
    c.min_limit
}

#[allow(dead_code)]
pub fn linear_limit_max(c: &LinearConstraint) -> f32 {
    c.max_limit
}

#[allow(dead_code)]
pub fn linear_is_limited(c: &LinearConstraint) -> bool {
    c.limited
}

#[allow(dead_code)]
pub fn linear_force(c: &LinearConstraint) -> f32 {
    linear_error(c).abs() * c.stiffness
}

#[allow(dead_code)]
pub fn linear_reset(c: &mut LinearConstraint) {
    c.current = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = new_linear_constraint(-1.0, 1.0, 0.5);
        assert_eq!(c.min_limit, -1.0);
        assert_eq!(c.max_limit, 1.0);
    }

    #[test]
    fn test_no_error_in_range() {
        let c = new_linear_constraint(-1.0, 1.0, 1.0);
        assert_eq!(linear_error(&c), 0.0);
    }

    #[test]
    fn test_error_below_min() {
        let mut c = new_linear_constraint(-1.0, 1.0, 1.0);
        c.current = -2.0;
        assert!((linear_error(&c) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_error_above_max() {
        let mut c = new_linear_constraint(-1.0, 1.0, 1.0);
        c.current = 3.0;
        assert!((linear_error(&c) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_solve() {
        let mut c = new_linear_constraint(-1.0, 1.0, 1.0);
        c.current = 2.0;
        let correction = solve_linear(&mut c);
        assert!(correction < 0.0); // should push back
    }

    #[test]
    fn test_limits() {
        let c = new_linear_constraint(-5.0, 5.0, 1.0);
        assert_eq!(linear_limit_min(&c), -5.0);
        assert_eq!(linear_limit_max(&c), 5.0);
    }

    #[test]
    fn test_is_limited() {
        let c = new_linear_constraint(0.0, 1.0, 1.0);
        assert!(linear_is_limited(&c));
    }

    #[test]
    fn test_force() {
        let mut c = new_linear_constraint(-1.0, 1.0, 0.5);
        c.current = 3.0;
        assert!(linear_force(&c) > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut c = new_linear_constraint(-1.0, 1.0, 1.0);
        c.current = 5.0;
        linear_reset(&mut c);
        assert_eq!(c.current, 0.0);
    }

    #[test]
    fn test_unlimited() {
        let mut c = new_linear_constraint(-1.0, 1.0, 1.0);
        c.limited = false;
        c.current = 100.0;
        assert_eq!(linear_error(&c), 0.0);
    }
}
