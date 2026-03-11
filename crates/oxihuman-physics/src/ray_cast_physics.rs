#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ray casting against physics shapes.

/// A physics ray.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsRayCast {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub max_distance: f32,
}

/// Result of a ray hit.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RayHit {
    pub distance: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(v);
    if l < 1e-10 { [0.0, 0.0, 0.0] } else { [v[0] / l, v[1] / l, v[2] / l] }
}

#[allow(dead_code)]
pub fn new_physics_ray(origin: [f32; 3], direction: [f32; 3], max_distance: f32) -> PhysicsRayCast {
    PhysicsRayCast {
        origin,
        direction: vec3_normalize(direction),
        max_distance,
    }
}

#[allow(dead_code)]
pub fn cast_ray_sphere(
    ray: &PhysicsRayCast,
    center: [f32; 3], radius: f32,
) -> Option<RayHit> {
    let oc = vec3_sub(ray.origin, center);
    let b = vec3_dot(oc, ray.direction);
    let c = vec3_dot(oc, oc) - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    if t < 0.0 || t > ray.max_distance {
        return None;
    }
    let point = [
        ray.origin[0] + ray.direction[0] * t,
        ray.origin[1] + ray.direction[1] * t,
        ray.origin[2] + ray.direction[2] * t,
    ];
    let normal = vec3_normalize(vec3_sub(point, center));
    Some(RayHit { distance: t, point, normal })
}

#[allow(dead_code)]
pub fn cast_ray_box(
    ray: &PhysicsRayCast,
    center: [f32; 3], half: [f32; 3],
) -> Option<RayHit> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    let mut hit_normal = [0.0f32; 3];

    for i in 0..3 {
        if ray.direction[i].abs() < 1e-10 {
            if ray.origin[i] < center[i] - half[i] || ray.origin[i] > center[i] + half[i] {
                return None;
            }
        } else {
            let inv_d = 1.0 / ray.direction[i];
            let mut t1 = (center[i] - half[i] - ray.origin[i]) * inv_d;
            let mut t2 = (center[i] + half[i] - ray.origin[i]) * inv_d;
            let mut n = [0.0f32; 3];
            n[i] = -1.0;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
                n[i] = 1.0;
            }
            if t1 > tmin {
                tmin = t1;
                hit_normal = n;
            }
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
    }
    if tmin < 0.0 || tmin > ray.max_distance {
        return None;
    }
    let point = [
        ray.origin[0] + ray.direction[0] * tmin,
        ray.origin[1] + ray.direction[1] * tmin,
        ray.origin[2] + ray.direction[2] * tmin,
    ];
    Some(RayHit { distance: tmin, point, normal: hit_normal })
}

#[allow(dead_code)]
pub fn cast_ray_capsule(
    ray: &PhysicsRayCast,
    _a: [f32; 3], _b: [f32; 3], radius: f32,
) -> Option<RayHit> {
    // Simplified: treat as sphere at midpoint.
    let center = [
        (_a[0] + _b[0]) * 0.5,
        (_a[1] + _b[1]) * 0.5,
        (_a[2] + _b[2]) * 0.5,
    ];
    let hl = vec3_len(vec3_sub(_a, _b)) * 0.5;
    cast_ray_sphere(ray, center, radius + hl)
}

#[allow(dead_code)]
pub fn ray_hit_distance(hit: &RayHit) -> f32 {
    hit.distance
}

#[allow(dead_code)]
pub fn ray_hit_normal(hit: &RayHit) -> [f32; 3] {
    hit.normal
}

#[allow(dead_code)]
pub fn ray_hit_point(hit: &RayHit) -> [f32; 3] {
    hit.point
}

#[allow(dead_code)]
pub fn ray_closest_hit(hits: &[RayHit]) -> Option<&RayHit> {
    hits.iter().min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_sphere_hit() {
        let ray = new_physics_ray([0.0, 0.0, -5.0], [0.0, 0.0, 1.0], 100.0);
        let hit = cast_ray_sphere(&ray, [0.0, 0.0, 0.0], 1.0).unwrap();
        assert!((hit.distance - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_sphere_miss() {
        let ray = new_physics_ray([0.0, 0.0, -5.0], [0.0, 1.0, 0.0], 100.0);
        assert!(cast_ray_sphere(&ray, [0.0, 0.0, 0.0], 1.0).is_none());
    }

    #[test]
    fn test_ray_box_hit() {
        let ray = new_physics_ray([-5.0, 0.0, 0.0], [1.0, 0.0, 0.0], 100.0);
        let hit = cast_ray_box(&ray, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();
        assert!((hit.distance - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_box_miss() {
        let ray = new_physics_ray([-5.0, 0.0, 0.0], [0.0, 1.0, 0.0], 100.0);
        assert!(cast_ray_box(&ray, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).is_none());
    }

    #[test]
    fn test_ray_capsule() {
        let ray = new_physics_ray([0.0, 0.0, -10.0], [0.0, 0.0, 1.0], 100.0);
        let hit = cast_ray_capsule(&ray, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(hit.is_some());
    }

    #[test]
    fn test_hit_distance() {
        let hit = RayHit { distance: 3.0, point: [0.0; 3], normal: [0.0, 1.0, 0.0] };
        assert!((ray_hit_distance(&hit) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_hit_normal() {
        let hit = RayHit { distance: 1.0, point: [0.0; 3], normal: [0.0, 1.0, 0.0] };
        assert_eq!(ray_hit_normal(&hit), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_hit_point() {
        let hit = RayHit { distance: 1.0, point: [1.0, 2.0, 3.0], normal: [0.0; 3] };
        assert_eq!(ray_hit_point(&hit), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_closest_hit() {
        let hits = vec![
            RayHit { distance: 5.0, point: [0.0; 3], normal: [0.0; 3] },
            RayHit { distance: 2.0, point: [0.0; 3], normal: [0.0; 3] },
            RayHit { distance: 8.0, point: [0.0; 3], normal: [0.0; 3] },
        ];
        let closest = ray_closest_hit(&hits).unwrap();
        assert!((closest.distance - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_closest_hit_empty() {
        let hits: Vec<RayHit> = vec![];
        assert!(ray_closest_hit(&hits).is_none());
    }
}
