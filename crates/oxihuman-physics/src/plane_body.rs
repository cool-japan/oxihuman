// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Infinite plane used as a static collision surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlaneBody {
    pub normal: [f32; 3],
    pub offset: f32,
}

#[allow(dead_code)]
impl PlaneBody {
    pub fn new(normal: [f32; 3], offset: f32) -> Self {
        let n = normalize3(normal);
        Self { normal: n, offset }
    }
    pub fn from_point_normal(point: [f32; 3], normal: [f32; 3]) -> Self {
        let n = normalize3(normal);
        let offset = dot3(n, point);
        Self { normal: n, offset }
    }
    pub fn signed_distance(&self, p: [f32; 3]) -> f32 {
        dot3(self.normal, p) - self.offset
    }
    pub fn closest_point(&self, p: [f32; 3]) -> [f32; 3] {
        let d = self.signed_distance(p);
        [
            p[0] - d * self.normal[0],
            p[1] - d * self.normal[1],
            p[2] - d * self.normal[2],
        ]
    }
    pub fn is_above(&self, p: [f32; 3]) -> bool {
        self.signed_distance(p) > 0.0
    }
    pub fn is_below(&self, p: [f32; 3]) -> bool {
        self.signed_distance(p) < 0.0
    }
    pub fn project_velocity(&self, vel: [f32; 3]) -> [f32; 3] {
        let dn = dot3(vel, self.normal);
        [
            vel[0] - dn * self.normal[0],
            vel[1] - dn * self.normal[1],
            vel[2] - dn * self.normal[2],
        ]
    }
    pub fn reflect_velocity(&self, vel: [f32; 3], restitution: f32) -> [f32; 3] {
        let dn = dot3(vel, self.normal);
        [
            vel[0] - (1.0 + restitution) * dn * self.normal[0],
            vel[1] - (1.0 + restitution) * dn * self.normal[1],
            vel[2] - (1.0 + restitution) * dn * self.normal[2],
        ]
    }
    pub fn sphere_penetration(&self, center: [f32; 3], radius: f32) -> f32 {
        let d = self.signed_distance(center);
        if d < radius {
            radius - d
        } else {
            0.0
        }
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-10);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn new_plane_body(normal: [f32; 3], offset: f32) -> PlaneBody {
    PlaneBody::new(normal, offset)
}
#[allow(dead_code)]
pub fn plane_signed_dist(p: &PlaneBody, point: [f32; 3]) -> f32 {
    p.signed_distance(point)
}
#[allow(dead_code)]
pub fn plane_closest_point(p: &PlaneBody, point: [f32; 3]) -> [f32; 3] {
    p.closest_point(point)
}
#[allow(dead_code)]
pub fn plane_is_above(p: &PlaneBody, point: [f32; 3]) -> bool {
    p.is_above(point)
}
#[allow(dead_code)]
pub fn plane_reflect_vel(p: &PlaneBody, vel: [f32; 3], rest: f32) -> [f32; 3] {
    p.reflect_velocity(vel, rest)
}
#[allow(dead_code)]
pub fn plane_sphere_penetration(p: &PlaneBody, center: [f32; 3], radius: f32) -> f32 {
    p.sphere_penetration(center, radius)
}
#[allow(dead_code)]
pub fn plane_project_vel(p: &PlaneBody, vel: [f32; 3]) -> [f32; 3] {
    p.project_velocity(vel)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ground_plane_distance() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        let d = plane_signed_dist(&p, [0.0, 5.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-5);
    }
    #[test]
    fn test_below_plane() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        assert!(!plane_is_above(&p, [0.0, -1.0, 0.0]));
    }
    #[test]
    fn test_above_plane() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        assert!(plane_is_above(&p, [0.0, 1.0, 0.0]));
    }
    #[test]
    fn test_closest_point_on_plane() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        let cp = plane_closest_point(&p, [3.0, 5.0, 2.0]);
        assert!((cp[1]).abs() < 1e-5);
        assert!((cp[0] - 3.0).abs() < 1e-5);
    }
    #[test]
    fn test_reflect_velocity() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        let vel = [1.0, -1.0, 0.0];
        let r = plane_reflect_vel(&p, vel, 1.0);
        assert!((r[1] - 1.0).abs() < 1e-5);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_sphere_penetration() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        let pen = plane_sphere_penetration(&p, [0.0, 0.3, 0.0], 0.5);
        assert!((pen - 0.2).abs() < 1e-5);
    }
    #[test]
    fn test_no_penetration() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        let pen = plane_sphere_penetration(&p, [0.0, 2.0, 0.0], 0.5);
        assert_eq!(pen, 0.0);
    }
    #[test]
    fn test_project_velocity() {
        let p = new_plane_body([0.0, 1.0, 0.0], 0.0);
        let pv = plane_project_vel(&p, [1.0, 3.0, 0.0]);
        assert!(pv[1].abs() < 1e-5);
    }
    #[test]
    fn test_normal_normalized() {
        let p = new_plane_body([3.0, 4.0, 0.0], 0.0);
        let len = (p.normal[0].powi(2) + p.normal[1].powi(2) + p.normal[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_from_point_normal() {
        let plane = PlaneBody::from_point_normal([0.0, 2.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((plane_signed_dist(&plane, [0.0, 2.0, 0.0])).abs() < 1e-5);
    }
}
