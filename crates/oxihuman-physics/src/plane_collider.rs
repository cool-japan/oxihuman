// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An infinite plane collider for ground and wall collision.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PlaneCollider {
    normal: [f32; 3],
    distance: f32,
    friction: f32,
    restitution: f32,
}

#[allow(dead_code)]
impl PlaneCollider {
    pub fn new(normal: [f32; 3], distance: f32) -> Self {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let n = if len > 1e-9 {
            [normal[0] / len, normal[1] / len, normal[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        };
        Self {
            normal: n,
            distance,
            friction: 0.5,
            restitution: 0.3,
        }
    }

    pub fn ground() -> Self {
        Self::new([0.0, 1.0, 0.0], 0.0)
    }

    pub fn wall_x(offset: f32) -> Self {
        Self::new([1.0, 0.0, 0.0], offset)
    }

    pub fn wall_z(offset: f32) -> Self {
        Self::new([0.0, 0.0, 1.0], offset)
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    pub fn normal(&self) -> [f32; 3] {
        self.normal
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn friction(&self) -> f32 {
        self.friction
    }

    pub fn restitution(&self) -> f32 {
        self.restitution
    }

    pub fn signed_distance(&self, point: [f32; 3]) -> f32 {
        point[0] * self.normal[0] + point[1] * self.normal[1] + point[2] * self.normal[2]
            - self.distance
    }

    pub fn is_penetrating(&self, point: [f32; 3]) -> bool {
        self.signed_distance(point) < 0.0
    }

    pub fn penetration_depth(&self, point: [f32; 3]) -> f32 {
        (-self.signed_distance(point)).max(0.0)
    }

    pub fn project_point(&self, point: [f32; 3]) -> [f32; 3] {
        let sd = self.signed_distance(point);
        [
            point[0] - self.normal[0] * sd,
            point[1] - self.normal[1] * sd,
            point[2] - self.normal[2] * sd,
        ]
    }

    pub fn reflect_velocity(&self, velocity: [f32; 3]) -> [f32; 3] {
        let vn = velocity[0] * self.normal[0]
            + velocity[1] * self.normal[1]
            + velocity[2] * self.normal[2];
        if vn >= 0.0 {
            return velocity;
        }
        [
            velocity[0] - (1.0 + self.restitution) * vn * self.normal[0],
            velocity[1] - (1.0 + self.restitution) * vn * self.normal[1],
            velocity[2] - (1.0 + self.restitution) * vn * self.normal[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground() {
        let p = PlaneCollider::ground();
        assert_eq!(p.normal(), [0.0, 1.0, 0.0]);
        assert!((p.distance()).abs() < 1e-6);
    }

    #[test]
    fn test_signed_distance_above() {
        let p = PlaneCollider::ground();
        let sd = p.signed_distance([0.0, 5.0, 0.0]);
        assert!((sd - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_signed_distance_below() {
        let p = PlaneCollider::ground();
        let sd = p.signed_distance([0.0, -2.0, 0.0]);
        assert!((sd - (-2.0)).abs() < 1e-5);
    }

    #[test]
    fn test_is_penetrating() {
        let p = PlaneCollider::ground();
        assert!(p.is_penetrating([0.0, -0.1, 0.0]));
        assert!(!p.is_penetrating([0.0, 0.1, 0.0]));
    }

    #[test]
    fn test_penetration_depth() {
        let p = PlaneCollider::ground();
        assert!((p.penetration_depth([0.0, -0.5, 0.0]) - 0.5).abs() < 1e-5);
        assert!((p.penetration_depth([0.0, 1.0, 0.0])).abs() < 1e-6);
    }

    #[test]
    fn test_project_point() {
        let p = PlaneCollider::ground();
        let proj = p.project_point([3.0, 5.0, 7.0]);
        assert!((proj[0] - 3.0).abs() < 1e-5);
        assert!((proj[1]).abs() < 1e-5);
        assert!((proj[2] - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_reflect_velocity() {
        let p = PlaneCollider::ground().with_restitution(1.0);
        let v = p.reflect_velocity([0.0, -10.0, 0.0]);
        assert!((v[1] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_reflect_velocity_no_bounce() {
        let p = PlaneCollider::ground().with_restitution(0.0);
        let v = p.reflect_velocity([0.0, -10.0, 0.0]);
        assert!(v[1].abs() < 1e-4);
    }

    #[test]
    fn test_wall_x() {
        let p = PlaneCollider::wall_x(5.0);
        let sd = p.signed_distance([3.0, 0.0, 0.0]);
        assert!((sd - (-2.0)).abs() < 1e-5);
    }

    #[test]
    fn test_with_friction() {
        let p = PlaneCollider::ground().with_friction(0.8);
        assert!((p.friction() - 0.8).abs() < 1e-6);
    }
}
