#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Constraint limit (angular/linear) enforcement.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintLimit {
    pub lo: f32,
    pub hi: f32,
    pub enabled: bool,
    pub current: f32,
}

#[allow(dead_code)]
pub fn new_constraint_limit(lo: f32, hi: f32) -> ConstraintLimit {
    ConstraintLimit {
        lo,
        hi,
        enabled: true,
        current: 0.0,
    }
}

#[allow(dead_code)]
pub fn limit_clamp(limit: &ConstraintLimit, value: f32) -> f32 {
    if !limit.enabled {
        return value;
    }
    value.clamp(limit.lo, limit.hi)
}

#[allow(dead_code)]
pub fn limit_violated(limit: &ConstraintLimit) -> bool {
    if !limit.enabled {
        return false;
    }
    !(limit.lo..=limit.hi).contains(&limit.current)
}

#[allow(dead_code)]
pub fn limit_error(limit: &ConstraintLimit) -> f32 {
    if !limit.enabled {
        return 0.0;
    }
    if limit.current < limit.lo {
        limit.current - limit.lo
    } else if limit.current > limit.hi {
        limit.current - limit.hi
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn limit_is_enabled(limit: &ConstraintLimit) -> bool {
    limit.enabled
}

#[allow(dead_code)]
pub fn limit_set_current(limit: &mut ConstraintLimit, val: f32) {
    limit.current = val;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_enabled() {
        let l = new_constraint_limit(-1.0, 1.0);
        assert!(limit_is_enabled(&l));
    }

    #[test]
    fn test_clamp_in_range() {
        let l = new_constraint_limit(-1.0, 1.0);
        assert!((limit_clamp(&l, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_below() {
        let l = new_constraint_limit(-1.0, 1.0);
        assert!((limit_clamp(&l, -5.0) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_above() {
        let l = new_constraint_limit(-1.0, 1.0);
        assert!((limit_clamp(&l, 5.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_disabled() {
        let mut l = new_constraint_limit(-1.0, 1.0);
        l.enabled = false;
        assert!((limit_clamp(&l, 5.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_not_violated_inside() {
        let mut l = new_constraint_limit(-1.0, 1.0);
        limit_set_current(&mut l, 0.5);
        assert!(!limit_violated(&l));
    }

    #[test]
    fn test_violated_below() {
        let mut l = new_constraint_limit(-1.0, 1.0);
        limit_set_current(&mut l, -2.0);
        assert!(limit_violated(&l));
    }

    #[test]
    fn test_violated_above() {
        let mut l = new_constraint_limit(-1.0, 1.0);
        limit_set_current(&mut l, 2.0);
        assert!(limit_violated(&l));
    }

    #[test]
    fn test_error_below() {
        let mut l = new_constraint_limit(0.0, 1.0);
        limit_set_current(&mut l, -0.5);
        assert!((limit_error(&l) + 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_error_above() {
        let mut l = new_constraint_limit(0.0, 1.0);
        limit_set_current(&mut l, 1.5);
        assert!((limit_error(&l) - 0.5).abs() < 1e-6);
    }
}
