// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Represents a contact normal with penetration depth information.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContactNormal {
    pub normal: [f32; 3],
    pub depth: f32,
    pub point: [f32; 3],
}

#[allow(dead_code)]
impl ContactNormal {
    pub fn new(normal: [f32; 3], depth: f32, point: [f32; 3]) -> Self {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let normalized = if len > 1e-9 {
            [normal[0] / len, normal[1] / len, normal[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        };
        Self {
            normal: normalized,
            depth,
            point,
        }
    }

    pub fn up_contact(depth: f32, point: [f32; 3]) -> Self {
        Self::new([0.0, 1.0, 0.0], depth, point)
    }

    pub fn is_separating(&self) -> bool {
        self.depth <= 0.0
    }

    pub fn is_penetrating(&self) -> bool {
        self.depth > 0.0
    }

    pub fn separation_vector(&self) -> [f32; 3] {
        [
            self.normal[0] * self.depth,
            self.normal[1] * self.depth,
            self.normal[2] * self.depth,
        ]
    }

    pub fn flip(&self) -> Self {
        Self {
            normal: [-self.normal[0], -self.normal[1], -self.normal[2]],
            depth: self.depth,
            point: self.point,
        }
    }

    pub fn dot_velocity(&self, velocity: [f32; 3]) -> f32 {
        self.normal[0] * velocity[0] + self.normal[1] * velocity[1] + self.normal[2] * velocity[2]
    }

    pub fn is_approaching(&self, velocity: [f32; 3]) -> bool {
        self.dot_velocity(velocity) < 0.0
    }

    pub fn reflect_velocity(&self, velocity: [f32; 3]) -> [f32; 3] {
        let dot = self.dot_velocity(velocity);
        [
            velocity[0] - 2.0 * dot * self.normal[0],
            velocity[1] - 2.0 * dot * self.normal[1],
            velocity[2] - 2.0 * dot * self.normal[2],
        ]
    }

    pub fn project_velocity(&self, velocity: [f32; 3]) -> [f32; 3] {
        let dot = self.dot_velocity(velocity);
        [
            velocity[0] - dot * self.normal[0],
            velocity[1] - dot * self.normal[1],
            velocity[2] - dot * self.normal[2],
        ]
    }

    pub fn angle_with(&self, other: &ContactNormal) -> f32 {
        let dot: f32 = (0..3).map(|i| self.normal[i] * other.normal[i]).sum();
        dot.clamp(-1.0, 1.0).acos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_normalizes() {
        let cn = ContactNormal::new([0.0, 2.0, 0.0], 1.0, [0.0; 3]);
        assert!((cn.normal[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_up_contact() {
        let cn = ContactNormal::up_contact(0.5, [1.0, 0.0, 1.0]);
        assert_eq!(cn.normal, [0.0, 1.0, 0.0]);
        assert!((cn.depth - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_is_penetrating() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], 0.1, [0.0; 3]);
        assert!(cn.is_penetrating());
        assert!(!cn.is_separating());
    }

    #[test]
    fn test_is_separating() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], -0.1, [0.0; 3]);
        assert!(cn.is_separating());
    }

    #[test]
    fn test_separation_vector() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], 2.0, [0.0; 3]);
        let sv = cn.separation_vector();
        assert!((sv[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_flip() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], 1.0, [0.0; 3]);
        let flipped = cn.flip();
        assert!((flipped.normal[1] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_is_approaching() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], 1.0, [0.0; 3]);
        assert!(cn.is_approaching([0.0, -5.0, 0.0]));
        assert!(!cn.is_approaching([0.0, 5.0, 0.0]));
    }

    #[test]
    fn test_reflect_velocity() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], 1.0, [0.0; 3]);
        let reflected = cn.reflect_velocity([0.0, -10.0, 0.0]);
        assert!((reflected[1] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_project_velocity() {
        let cn = ContactNormal::new([0.0, 1.0, 0.0], 1.0, [0.0; 3]);
        let projected = cn.project_velocity([3.0, -5.0, 2.0]);
        assert!((projected[1]).abs() < 1e-5);
        assert!((projected[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_with() {
        let a = ContactNormal::new([1.0, 0.0, 0.0], 0.0, [0.0; 3]);
        let b = ContactNormal::new([0.0, 1.0, 0.0], 0.0, [0.0; 3]);
        let angle = a.angle_with(&b);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }
}
