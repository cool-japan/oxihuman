// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Force field primitives: radial, directional, vortex, and noise-based fields.

use std::f32::consts::PI;

/// The kind of force field.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForceFieldKind {
    Radial,
    Directional,
    Vortex,
    Drag,
}

/// A spatial force field that applies forces to bodies within range.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyForceField {
    pub kind: ForceFieldKind,
    pub center: [f32; 3],
    pub direction: [f32; 3],
    pub strength: f32,
    pub radius: f32,
    pub falloff: f32, // 1.0 = linear, 2.0 = quadratic
}

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = vec3_length(v);
    if l < 1e-10 { return [0.0, 0.0, 0.0]; }
    vec3_scale(v, 1.0 / l)
}

#[allow(dead_code)]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
impl BodyForceField {
    pub fn radial(center: [f32; 3], strength: f32, radius: f32) -> Self {
        Self {
            kind: ForceFieldKind::Radial,
            center,
            direction: [0.0, 0.0, 0.0],
            strength,
            radius,
            falloff: 2.0,
        }
    }

    pub fn directional(direction: [f32; 3], strength: f32) -> Self {
        Self {
            kind: ForceFieldKind::Directional,
            center: [0.0, 0.0, 0.0],
            direction: vec3_normalize(direction),
            strength,
            radius: f32::MAX,
            falloff: 0.0,
        }
    }

    pub fn vortex(center: [f32; 3], axis: [f32; 3], strength: f32, radius: f32) -> Self {
        Self {
            kind: ForceFieldKind::Vortex,
            center,
            direction: vec3_normalize(axis),
            strength,
            radius,
            falloff: 1.0,
        }
    }

    /// Compute force at a given position.
    pub fn force_at(&self, pos: [f32; 3]) -> [f32; 3] {
        match self.kind {
            ForceFieldKind::Radial => {
                let to_center = vec3_sub(self.center, pos);
                let dist = vec3_length(to_center);
                if dist > self.radius || dist < 1e-6 { return [0.0; 3]; }
                let atten = (1.0 - dist / self.radius).powf(self.falloff);
                let dir = vec3_normalize(to_center);
                vec3_scale(dir, self.strength * atten)
            }
            ForceFieldKind::Directional => {
                vec3_scale(self.direction, self.strength)
            }
            ForceFieldKind::Vortex => {
                let to_point = vec3_sub(pos, self.center);
                let dist = vec3_length(to_point);
                if dist > self.radius || dist < 1e-6 { return [0.0; 3]; }
                let tangent = vec3_cross(self.direction, vec3_normalize(to_point));
                let atten = (1.0 - dist / self.radius).powf(self.falloff);
                vec3_scale(tangent, self.strength * atten)
            }
            ForceFieldKind::Drag => {
                // Drag opposes velocity but we only have position, return zero
                [0.0; 3]
            }
        }
    }

    pub fn is_in_range(&self, pos: [f32; 3]) -> bool {
        let dist = vec3_length(vec3_sub(pos, self.center));
        dist <= self.radius
    }

    /// Potential energy at position (for radial fields).
    pub fn potential_at(&self, pos: [f32; 3]) -> f32 {
        let dist = vec3_length(vec3_sub(pos, self.center));
        if dist > self.radius { return 0.0; }
        -self.strength * (self.radius - dist) / self.radius
    }

    /// Volume of the sphere of influence.
    pub fn influence_volume(&self) -> f32 {
        (4.0 / 3.0) * PI * self.radius * self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radial_at_center() {
        let field = BodyForceField::radial([0.0, 0.0, 0.0], 10.0, 5.0);
        let f = field.force_at([0.0, 0.0, 0.0]);
        // At center, force is zero (no direction)
        assert!(vec3_length(f) < 1e-5);
    }

    #[test]
    fn test_radial_outside() {
        let field = BodyForceField::radial([0.0, 0.0, 0.0], 10.0, 1.0);
        let f = field.force_at([5.0, 0.0, 0.0]);
        assert!(vec3_length(f) < 1e-5);
    }

    #[test]
    fn test_radial_inside() {
        let field = BodyForceField::radial([0.0, 0.0, 0.0], 10.0, 5.0);
        let f = field.force_at([1.0, 0.0, 0.0]);
        assert!(f[0] < 0.0); // pulls toward center
    }

    #[test]
    fn test_directional() {
        let field = BodyForceField::directional([0.0, -1.0, 0.0], 9.8);
        let f = field.force_at([100.0, 200.0, 300.0]);
        assert!((f[1] - (-9.8)).abs() < 0.01);
    }

    #[test]
    fn test_vortex() {
        let field = BodyForceField::vortex([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0, 10.0);
        let f = field.force_at([1.0, 0.0, 0.0]);
        // Vortex should produce a tangential force (cross product with up axis)
        assert!(f[2].abs() > 0.1);
    }

    #[test]
    fn test_in_range() {
        let field = BodyForceField::radial([0.0, 0.0, 0.0], 10.0, 5.0);
        assert!(field.is_in_range([3.0, 0.0, 0.0]));
        assert!(!field.is_in_range([10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_potential() {
        let field = BodyForceField::radial([0.0, 0.0, 0.0], 10.0, 5.0);
        let p = field.potential_at([0.0, 0.0, 0.0]);
        assert!(p < 0.0); // negative potential at center
    }

    #[test]
    fn test_influence_volume() {
        let field = BodyForceField::radial([0.0, 0.0, 0.0], 1.0, 1.0);
        let v = field.influence_volume();
        assert!((v - 4.0 / 3.0 * PI).abs() < 0.01);
    }

    #[test]
    fn test_drag_returns_zero() {
        let field = BodyForceField {
            kind: ForceFieldKind::Drag,
            center: [0.0; 3], direction: [0.0; 3],
            strength: 1.0, radius: 10.0, falloff: 1.0,
        };
        let f = field.force_at([1.0, 1.0, 1.0]);
        assert!(vec3_length(f) < 1e-10);
    }

    #[test]
    fn test_vec3_ops() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let s = vec3_sub(a, b);
        assert!((s[0] - (-3.0)).abs() < 1e-6);
    }
}
