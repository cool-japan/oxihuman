// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! World-level spatial queries: ray casts, sphere casts, overlap tests against a body list.

/// A simplified body representation for world queries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorldBody {
    pub id: u32,
    pub center: [f32; 3],
    pub radius: f32, // bounding sphere radius
    pub layer: u32,
}

/// Ray cast hit result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RayHit {
    pub body_id: u32,
    pub t: f32,
    pub point: [f32; 3],
}

/// A world query system over a set of bodies.
#[allow(dead_code)]
#[derive(Debug)]
pub struct PhysicsWorldQuery {
    bodies: Vec<WorldBody>,
}

#[allow(dead_code)]
fn v3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

#[allow(dead_code)]
fn v3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[allow(dead_code)]
fn v3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2]]
}

#[allow(dead_code)]
fn v3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0]*s, v[1]*s, v[2]*s]
}

#[allow(dead_code)]
fn v3_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = v3_sub(a, b);
    (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
impl PhysicsWorldQuery {
    pub fn new() -> Self {
        Self { bodies: Vec::new() }
    }

    pub fn add_body(&mut self, body: WorldBody) {
        self.bodies.push(body);
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Ray cast against all bodies' bounding spheres. Returns closest hit.
    pub fn ray_cast(&self, origin: [f32; 3], dir: [f32; 3], max_dist: f32, layer_mask: u32) -> Option<RayHit> {
        let mut best: Option<RayHit> = None;
        for body in &self.bodies {
            if body.layer & layer_mask == 0 { continue; }
            if let Some(t) = ray_sphere(origin, dir, body.center, body.radius) {
                if t <= max_dist && best.as_ref().is_none_or(|b| t < b.t) {
                    best = Some(RayHit {
                        body_id: body.id,
                        t,
                        point: v3_add(origin, v3_scale(dir, t)),
                    });
                }
            }
        }
        best
    }

    /// Overlap sphere query: returns all bodies overlapping with the sphere.
    pub fn overlap_sphere(&self, center: [f32; 3], radius: f32, layer_mask: u32) -> Vec<u32> {
        self.bodies.iter()
            .filter(|b| b.layer & layer_mask != 0)
            .filter(|b| v3_dist(center, b.center) <= radius + b.radius)
            .map(|b| b.id)
            .collect()
    }

    /// Find closest body to a point.
    pub fn closest_body(&self, point: [f32; 3], layer_mask: u32) -> Option<u32> {
        self.bodies.iter()
            .filter(|b| b.layer & layer_mask != 0)
            .min_by(|a, b| {
                let da = v3_dist(point, a.center);
                let db = v3_dist(point, b.center);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|b| b.id)
    }

    /// Find all bodies within distance.
    pub fn bodies_within(&self, point: [f32; 3], distance: f32) -> Vec<u32> {
        self.bodies.iter()
            .filter(|b| v3_dist(point, b.center) - b.radius <= distance)
            .map(|b| b.id)
            .collect()
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
    }
}

#[allow(dead_code)]
fn ray_sphere(origin: [f32; 3], dir: [f32; 3], center: [f32; 3], radius: f32) -> Option<f32> {
    let oc = v3_sub(origin, center);
    let a = v3_dot(dir, dir);
    let b = 2.0 * v3_dot(oc, dir);
    let c = v3_dot(oc, oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 { return None; }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t >= 0.0 { Some(t) } else {
        let t2 = (-b + disc.sqrt()) / (2.0 * a);
        if t2 >= 0.0 { Some(t2) } else { None }
    }
}

impl Default for PhysicsWorldQuery {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_body(id: u32, pos: [f32; 3], r: f32) -> WorldBody {
        WorldBody { id, center: pos, radius: r, layer: 1 }
    }

    #[test]
    fn test_ray_cast_hit() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [5.0, 0.0, 0.0], 1.0));
        let hit = w.ray_cast([0.0;3], [1.0, 0.0, 0.0], 100.0, 1);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().body_id, 0);
    }

    #[test]
    fn test_ray_cast_miss() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [5.0, 5.0, 0.0], 0.5));
        let hit = w.ray_cast([0.0;3], [1.0, 0.0, 0.0], 100.0, 1);
        assert!(hit.is_none());
    }

    #[test]
    fn test_ray_cast_closest() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [10.0, 0.0, 0.0], 1.0));
        w.add_body(make_body(1, [5.0, 0.0, 0.0], 1.0));
        let hit = w.ray_cast([0.0;3], [1.0, 0.0, 0.0], 100.0, 1);
        assert_eq!(hit.unwrap().body_id, 1);
    }

    #[test]
    fn test_overlap_sphere() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [1.0, 0.0, 0.0], 0.5));
        w.add_body(make_body(1, [10.0, 0.0, 0.0], 0.5));
        let overlap = w.overlap_sphere([0.0;3], 2.0, 1);
        assert_eq!(overlap, vec![0]);
    }

    #[test]
    fn test_closest_body() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [10.0, 0.0, 0.0], 1.0));
        w.add_body(make_body(1, [2.0, 0.0, 0.0], 1.0));
        assert_eq!(w.closest_body([0.0;3], 1), Some(1));
    }

    #[test]
    fn test_layer_mask() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(WorldBody { id: 0, center: [1.0,0.0,0.0], radius: 1.0, layer: 2 });
        let hit = w.ray_cast([0.0;3], [1.0,0.0,0.0], 100.0, 1);
        assert!(hit.is_none()); // layer mismatch
    }

    #[test]
    fn test_bodies_within() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [1.0, 0.0, 0.0], 0.5));
        w.add_body(make_body(1, [5.0, 0.0, 0.0], 0.5));
        let r = w.bodies_within([0.0;3], 2.0);
        assert!(r.contains(&0));
        assert!(!r.contains(&1));
    }

    #[test]
    fn test_clear() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [0.0;3], 1.0));
        w.clear();
        assert_eq!(w.body_count(), 0);
    }

    #[test]
    fn test_body_count() {
        let mut w = PhysicsWorldQuery::new();
        w.add_body(make_body(0, [0.0;3], 1.0));
        w.add_body(make_body(1, [0.0;3], 1.0));
        assert_eq!(w.body_count(), 2);
    }

    #[test]
    fn test_ray_sphere_fn() {
        let t = ray_sphere([0.0;3], [1.0,0.0,0.0], [5.0,0.0,0.0], 1.0);
        assert!(t.is_some());
        assert!((t.unwrap() - 4.0).abs() < 0.01);
    }
}
