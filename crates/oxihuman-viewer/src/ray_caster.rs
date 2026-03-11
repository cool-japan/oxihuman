// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ray casting for mouse picking.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RayConfig {
    pub max_distance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RayHit {
    pub t: f32,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub object_id: u32,
}

#[allow(dead_code)]
pub fn default_ray_config() -> RayConfig {
    RayConfig { max_distance: 1000.0 }
}

#[allow(dead_code)]
pub fn new_ray(origin: [f32; 3], direction: [f32; 3]) -> Ray {
    Ray { origin, direction }
}

/// Returns the point along the ray at parameter `t`.
#[allow(dead_code)]
pub fn ray_at(ray: &Ray, t: f32) -> [f32; 3] {
    [
        ray.origin[0] + ray.direction[0] * t,
        ray.origin[1] + ray.direction[1] * t,
        ray.origin[2] + ray.direction[2] * t,
    ]
}

/// Intersect ray with sphere at `center` with `radius`. Returns `Some(t)` on hit.
#[allow(dead_code)]
pub fn ray_intersect_sphere(ray: &Ray, center: [f32; 3], radius: f32) -> Option<f32> {
    let oc = [
        ray.origin[0] - center[0],
        ray.origin[1] - center[1],
        ray.origin[2] - center[2],
    ];
    let d = &ray.direction;
    let a = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let b = 2.0 * (oc[0] * d[0] + oc[1] * d[1] + oc[2] * d[2]);
    let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t > 0.0 { Some(t) } else { None }
}

/// Möller–Trumbore ray-triangle intersection. Returns `Some(t)` on hit.
#[allow(dead_code)]
pub fn ray_intersect_triangle(
    ray: &Ray,
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let eps = 1e-7_f32;
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let d = &ray.direction;
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < eps {
        return None;
    }
    let f = 1.0 / a;
    let s = [
        ray.origin[0] - v0[0],
        ray.origin[1] - v0[1],
        ray.origin[2] - v0[2],
    ];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    if t > eps { Some(t) } else { None }
}

/// Test ray against AABB defined by min/max corners. Returns `Some(t)` on hit.
#[allow(dead_code)]
pub fn ray_intersect_aabb(ray: &Ray, aabb_min: [f32; 3], aabb_max: [f32; 3]) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    for i in 0..3 {
        let d = ray.direction[i];
        if d.abs() < 1e-8 {
            if ray.origin[i] < aabb_min[i] || ray.origin[i] > aabb_max[i] {
                return None;
            }
        } else {
            let t1 = (aabb_min[i] - ray.origin[i]) / d;
            let t2 = (aabb_max[i] - ray.origin[i]) / d;
            let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
            if t_min > t_max {
                return None;
            }
        }
    }
    if t_max < 0.0 {
        return None;
    }
    let t = if t_min >= 0.0 { t_min } else { t_max };
    Some(t)
}

/// Normalize the ray direction in place.
#[allow(dead_code)]
pub fn ray_normalize_direction(ray: &mut Ray) {
    let d = &mut ray.direction;
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len > 1e-10 {
        d[0] /= len;
        d[1] /= len;
        d[2] /= len;
    }
}

#[allow(dead_code)]
pub fn ray_to_json(ray: &Ray) -> String {
    format!(
        r#"{{"origin":[{:.4},{:.4},{:.4}],"direction":[{:.4},{:.4},{:.4}]}}"#,
        ray.origin[0], ray.origin[1], ray.origin[2],
        ray.direction[0], ray.direction[1], ray.direction[2]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ray_config();
        assert_eq!(cfg.max_distance, 1000.0);
    }

    #[test]
    fn test_new_ray() {
        let r = new_ray([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(r.origin[2], 0.0);
        assert_eq!(r.direction[2], 1.0);
    }

    #[test]
    fn test_ray_at() {
        let r = new_ray([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = ray_at(&r, 3.0);
        assert!((p[0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_intersect_sphere_hit() {
        let r = new_ray([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let hit = ray_intersect_sphere(&r, [0.0, 0.0, 0.0], 1.0);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_intersect_sphere_miss() {
        let r = new_ray([5.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let hit = ray_intersect_sphere(&r, [0.0, 0.0, 0.0], 1.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_intersect_triangle_hit() {
        let r = new_ray([0.1, 0.1, -1.0], [0.0, 0.0, 1.0]);
        let hit = ray_intersect_triangle(
            &r,
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!(hit.is_some());
    }

    #[test]
    fn test_intersect_aabb_hit() {
        let r = new_ray([0.5, 0.5, -5.0], [0.0, 0.0, 1.0]);
        let hit = ray_intersect_aabb(&r, [0.0, 0.0, -1.0], [1.0, 1.0, 1.0]);
        assert!(hit.is_some());
    }

    #[test]
    fn test_normalize_direction() {
        let mut r = new_ray([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]);
        ray_normalize_direction(&mut r);
        let len = (r.direction[0].powi(2) + r.direction[1].powi(2) + r.direction[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_to_json() {
        let r = new_ray([1.0, 2.0, 3.0], [0.0, 1.0, 0.0]);
        let j = ray_to_json(&r);
        assert!(j.contains("origin"));
        assert!(j.contains("direction"));
    }
}
