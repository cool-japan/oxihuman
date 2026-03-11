// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Physics material combination rules for friction and restitution between pairs.

/// How to combine two material values.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombineMode {
    Average,
    Minimum,
    Maximum,
    Multiply,
}

/// A physics material with friction and restitution.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PhysMaterial {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub friction_combine: CombineMode,
    pub restitution_combine: CombineMode,
}

#[allow(dead_code)]
impl PhysMaterial {
    pub fn new(static_f: f32, dynamic_f: f32, restitution: f32) -> Self {
        Self {
            static_friction: static_f,
            dynamic_friction: dynamic_f,
            restitution,
            friction_combine: CombineMode::Average,
            restitution_combine: CombineMode::Average,
        }
    }

    pub fn rubber() -> Self {
        Self::new(0.9, 0.8, 0.85).with_friction_combine(CombineMode::Maximum)
    }

    pub fn ice() -> Self {
        Self::new(0.05, 0.03, 0.1).with_friction_combine(CombineMode::Minimum)
    }

    pub fn steel() -> Self {
        Self::new(0.6, 0.5, 0.3)
    }

    pub fn wood() -> Self {
        Self::new(0.5, 0.4, 0.4)
    }

    pub fn with_friction_combine(mut self, mode: CombineMode) -> Self {
        self.friction_combine = mode;
        self
    }

    pub fn with_restitution_combine(mut self, mode: CombineMode) -> Self {
        self.restitution_combine = mode;
        self
    }
}

#[allow(dead_code)]
fn combine(a: f32, b: f32, mode: CombineMode) -> f32 {
    match mode {
        CombineMode::Average => (a + b) * 0.5,
        CombineMode::Minimum => a.min(b),
        CombineMode::Maximum => a.max(b),
        CombineMode::Multiply => a * b,
    }
}

/// Combined material properties for a contact pair.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CombinedMaterial {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

/// Combine two materials using the higher-priority combine mode.
#[allow(dead_code)]
pub fn combine_materials(a: &PhysMaterial, b: &PhysMaterial) -> CombinedMaterial {
    let friction_mode = higher_priority(a.friction_combine, b.friction_combine);
    let rest_mode = higher_priority(a.restitution_combine, b.restitution_combine);
    CombinedMaterial {
        static_friction: combine(a.static_friction, b.static_friction, friction_mode),
        dynamic_friction: combine(a.dynamic_friction, b.dynamic_friction, friction_mode),
        restitution: combine(a.restitution, b.restitution, rest_mode),
    }
}

#[allow(dead_code)]
fn priority(m: CombineMode) -> u8 {
    match m {
        CombineMode::Average => 0,
        CombineMode::Minimum => 1,
        CombineMode::Multiply => 2,
        CombineMode::Maximum => 3,
    }
}

#[allow(dead_code)]
fn higher_priority(a: CombineMode, b: CombineMode) -> CombineMode {
    if priority(a) >= priority(b) { a } else { b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average() {
        let a = PhysMaterial::new(0.6, 0.4, 0.5);
        let b = PhysMaterial::new(0.2, 0.1, 0.3);
        let c = combine_materials(&a, &b);
        assert!((c.static_friction - 0.4).abs() < 0.01);
        assert!((c.restitution - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_minimum() {
        let a = PhysMaterial::new(0.6, 0.4, 0.5).with_friction_combine(CombineMode::Minimum);
        let b = PhysMaterial::new(0.2, 0.1, 0.3);
        let c = combine_materials(&a, &b);
        assert!((c.static_friction - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_maximum() {
        let a = PhysMaterial::new(0.6, 0.4, 0.5).with_friction_combine(CombineMode::Maximum);
        let b = PhysMaterial::new(0.2, 0.1, 0.3);
        let c = combine_materials(&a, &b);
        assert!((c.static_friction - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_multiply() {
        let a = PhysMaterial::new(0.5, 0.4, 0.5).with_friction_combine(CombineMode::Multiply);
        let b = PhysMaterial::new(0.5, 0.4, 0.3);
        let c = combine_materials(&a, &b);
        assert!((c.static_friction - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_rubber() {
        let r = PhysMaterial::rubber();
        assert!(r.static_friction > 0.8);
        assert!(r.restitution > 0.8);
    }

    #[test]
    fn test_ice() {
        let i = PhysMaterial::ice();
        assert!(i.static_friction < 0.1);
    }

    #[test]
    fn test_steel() {
        let s = PhysMaterial::steel();
        assert!(s.static_friction > 0.4);
    }

    #[test]
    fn test_priority_max_wins() {
        let a = PhysMaterial::new(0.1, 0.1, 0.1).with_friction_combine(CombineMode::Maximum);
        let b = PhysMaterial::new(0.9, 0.9, 0.9).with_friction_combine(CombineMode::Minimum);
        let c = combine_materials(&a, &b);
        assert!((c.static_friction - 0.9).abs() < 0.01); // Maximum wins
    }

    #[test]
    fn test_restitution_combine_mode() {
        let a = PhysMaterial::new(0.5, 0.5, 0.8).with_restitution_combine(CombineMode::Minimum);
        let b = PhysMaterial::new(0.5, 0.5, 0.2);
        let c = combine_materials(&a, &b);
        assert!((c.restitution - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_combine_fn() {
        assert!((combine(0.4, 0.6, CombineMode::Average) - 0.5).abs() < 0.01);
        assert!((combine(0.4, 0.6, CombineMode::Minimum) - 0.4).abs() < 0.01);
        assert!((combine(0.4, 0.6, CombineMode::Maximum) - 0.6).abs() < 0.01);
        assert!((combine(0.4, 0.6, CombineMode::Multiply) - 0.24).abs() < 0.01);
    }
}
