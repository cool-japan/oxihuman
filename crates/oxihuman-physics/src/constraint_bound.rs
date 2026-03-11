// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Defines bounds/limits for a physics constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ConstraintBound {
    lower: f32,
    upper: f32,
    stiffness: f32,
    damping: f32,
    active: bool,
}

#[allow(dead_code)]
impl ConstraintBound {
    pub fn new(lower: f32, upper: f32) -> Self {
        Self {
            lower: lower.min(upper),
            upper: lower.max(upper),
            stiffness: 1000.0,
            damping: 50.0,
            active: true,
        }
    }

    pub fn angular(lower_deg: f32, upper_deg: f32) -> Self {
        let lower_rad = lower_deg * PI / 180.0;
        let upper_rad = upper_deg * PI / 180.0;
        Self::new(lower_rad, upper_rad)
    }

    pub fn symmetric(limit: f32) -> Self {
        Self::new(-limit, limit)
    }

    pub fn free() -> Self {
        Self {
            lower: f32::NEG_INFINITY,
            upper: f32::INFINITY,
            stiffness: 0.0,
            damping: 0.0,
            active: false,
        }
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn lower(&self) -> f32 {
        self.lower
    }

    pub fn upper(&self) -> f32 {
        self.upper
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn clamp(&self, value: f32) -> f32 {
        value.clamp(self.lower, self.upper)
    }

    pub fn is_within(&self, value: f32) -> bool {
        (self.lower..=self.upper).contains(&value)
    }

    pub fn violation(&self, value: f32) -> f32 {
        if value < self.lower {
            self.lower - value
        } else if value > self.upper {
            value - self.upper
        } else {
            0.0
        }
    }

    pub fn correction_force(&self, value: f32, velocity: f32) -> f32 {
        if !self.active {
            return 0.0;
        }
        if value < self.lower {
            let penetration = self.lower - value;
            penetration * self.stiffness - velocity * self.damping
        } else if value > self.upper {
            let penetration = value - self.upper;
            -penetration * self.stiffness - velocity * self.damping
        } else {
            0.0
        }
    }

    pub fn range(&self) -> f32 {
        self.upper - self.lower
    }

    pub fn center(&self) -> f32 {
        (self.lower + self.upper) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cb = ConstraintBound::new(-1.0, 1.0);
        assert!((cb.lower() - (-1.0)).abs() < 1e-6);
        assert!((cb.upper() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_swapped_bounds() {
        let cb = ConstraintBound::new(5.0, -5.0);
        assert!(cb.lower() <= cb.upper());
    }

    #[test]
    fn test_symmetric() {
        let cb = ConstraintBound::symmetric(2.0);
        assert!((cb.lower() - (-2.0)).abs() < 1e-6);
        assert!((cb.upper() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_angular() {
        let cb = ConstraintBound::angular(-90.0, 90.0);
        assert!((cb.lower() - (-PI / 2.0)).abs() < 1e-4);
        assert!((cb.upper() - (PI / 2.0)).abs() < 1e-4);
    }

    #[test]
    fn test_clamp() {
        let cb = ConstraintBound::new(0.0, 10.0);
        assert!((cb.clamp(-5.0)).abs() < 1e-6);
        assert!((cb.clamp(15.0) - 10.0).abs() < 1e-6);
        assert!((cb.clamp(5.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_within() {
        let cb = ConstraintBound::new(0.0, 1.0);
        assert!(cb.is_within(0.5));
        assert!(!cb.is_within(2.0));
    }

    #[test]
    fn test_violation() {
        let cb = ConstraintBound::new(0.0, 1.0);
        assert!((cb.violation(0.5)).abs() < 1e-6);
        assert!((cb.violation(-0.5) - 0.5).abs() < 1e-6);
        assert!((cb.violation(1.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_correction_force() {
        let cb = ConstraintBound::new(0.0, 1.0);
        let f = cb.correction_force(-0.1, 0.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_free() {
        let cb = ConstraintBound::free();
        assert!(!cb.is_active());
        assert!((cb.correction_force(-1000.0, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn test_range_center() {
        let cb = ConstraintBound::new(2.0, 8.0);
        assert!((cb.range() - 6.0).abs() < 1e-6);
        assert!((cb.center() - 5.0).abs() < 1e-6);
    }
}
