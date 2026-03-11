// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A collision skin wraps around a body, providing a margin for contact generation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionSkin {
    pub margin: f32,
    pub friction: f32,
    pub restitution: f32,
}

#[allow(dead_code)]
impl CollisionSkin {
    pub fn new(margin: f32, friction: f32, restitution: f32) -> Self {
        Self {
            margin: margin.max(0.0),
            friction: friction.clamp(0.0, 1.0),
            restitution: restitution.clamp(0.0, 1.0),
        }
    }

    pub fn default_skin() -> Self {
        Self::new(0.01, 0.5, 0.3)
    }

    pub fn frictionless() -> Self {
        Self::new(0.01, 0.0, 0.3)
    }

    pub fn bouncy() -> Self {
        Self::new(0.01, 0.3, 0.9)
    }

    pub fn expand_aabb(&self, min: [f32; 3], max: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let m = self.margin;
        (
            [min[0] - m, min[1] - m, min[2] - m],
            [max[0] + m, max[1] + m, max[2] + m],
        )
    }

    pub fn combined_friction(&self, other: &CollisionSkin) -> f32 {
        (self.friction * other.friction).sqrt()
    }

    pub fn combined_restitution(&self, other: &CollisionSkin) -> f32 {
        self.restitution.max(other.restitution)
    }

    pub fn effective_radius(&self, base_radius: f32) -> f32 {
        base_radius + self.margin
    }

    pub fn is_within_skin(&self, distance: f32) -> bool {
        distance <= self.margin
    }

    pub fn penetration_depth(&self, distance: f32) -> f32 {
        if distance < 0.0 {
            -distance + self.margin
        } else if distance < self.margin {
            self.margin - distance
        } else {
            0.0
        }
    }
}

/// A composite skin with multiple layers.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CompositeSkin {
    pub layers: Vec<CollisionSkin>,
}

#[allow(dead_code)]
impl CompositeSkin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_layer(&mut self, skin: CollisionSkin) {
        self.layers.push(skin);
    }

    pub fn total_margin(&self) -> f32 {
        self.layers.iter().map(|s| s.margin).sum()
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_skin() {
        let s = CollisionSkin::default_skin();
        assert!((s.margin - 0.01).abs() < f32::EPSILON);
        assert!((s.friction - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_expand_aabb() {
        let s = CollisionSkin::new(0.1, 0.5, 0.5);
        let (mn, mx) = s.expand_aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((mn[0] - (-0.1)).abs() < 1e-6);
        assert!((mx[0] - 1.1).abs() < 1e-6);
    }

    #[test]
    fn test_combined_friction() {
        let a = CollisionSkin::new(0.01, 0.4, 0.5);
        let b = CollisionSkin::new(0.01, 0.9, 0.5);
        let cf = a.combined_friction(&b);
        assert!((cf - (0.4_f32 * 0.9).sqrt()).abs() < 1e-6);
    }

    #[test]
    fn test_combined_restitution() {
        let a = CollisionSkin::new(0.01, 0.5, 0.3);
        let b = CollisionSkin::new(0.01, 0.5, 0.8);
        assert!((a.combined_restitution(&b) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_radius() {
        let s = CollisionSkin::new(0.05, 0.5, 0.5);
        assert!((s.effective_radius(1.0) - 1.05).abs() < 1e-6);
    }

    #[test]
    fn test_is_within_skin() {
        let s = CollisionSkin::new(0.1, 0.5, 0.5);
        assert!(s.is_within_skin(0.05));
        assert!(!s.is_within_skin(0.2));
    }

    #[test]
    fn test_penetration_depth() {
        let s = CollisionSkin::new(0.1, 0.5, 0.5);
        assert!((s.penetration_depth(-0.05) - 0.15).abs() < 1e-6);
        assert!((s.penetration_depth(0.05) - 0.05).abs() < 1e-6);
        assert!((s.penetration_depth(0.2)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frictionless() {
        let s = CollisionSkin::frictionless();
        assert!((s.friction).abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_skin() {
        let mut c = CompositeSkin::new();
        c.add_layer(CollisionSkin::new(0.01, 0.5, 0.5));
        c.add_layer(CollisionSkin::new(0.02, 0.5, 0.5));
        assert_eq!(c.layer_count(), 2);
        assert!((c.total_margin() - 0.03).abs() < 1e-6);
    }

    #[test]
    fn test_clamped_values() {
        let s = CollisionSkin::new(-1.0, 2.0, -0.5);
        assert!((s.margin).abs() < f32::EPSILON);
        assert!((s.friction - 1.0).abs() < f32::EPSILON);
        assert!((s.restitution).abs() < f32::EPSILON);
    }
}
