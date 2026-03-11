#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ray casting utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

#[allow(dead_code)]
pub fn new_ray(origin: [f32; 3], dir: [f32; 3]) -> Ray {
    Ray { origin, direction: dir }
}

#[allow(dead_code)]
pub fn ray_at(ray: &Ray, t: f32) -> [f32; 3] {
    [
        ray.origin[0] + ray.direction[0] * t,
        ray.origin[1] + ray.direction[1] * t,
        ray.origin[2] + ray.direction[2] * t,
    ]
}

#[allow(dead_code)]
pub fn ray_sphere_intersect(ray: &Ray, center: [f32; 3], radius: f32) -> Option<f32> {
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
    if t >= 0.0 { Some(t) } else { None }
}

#[allow(dead_code)]
pub fn ray_plane_intersect(ray: &Ray, normal: [f32; 3], d: f32) -> Option<f32> {
    let denom = normal[0] * ray.direction[0]
        + normal[1] * ray.direction[1]
        + normal[2] * ray.direction[2];
    if denom.abs() < 1e-8 {
        return None;
    }
    let num = -(normal[0] * ray.origin[0]
        + normal[1] * ray.origin[1]
        + normal[2] * ray.origin[2]
        + d);
    let t = num / denom;
    if t >= 0.0 { Some(t) } else { None }
}

#[allow(dead_code)]
pub fn ray_aabb_intersect(ray: &Ray, min: [f32; 3], max: [f32; 3]) -> Option<f32> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    for i in 0..3 {
        let inv_d = if ray.direction[i].abs() > 1e-10 {
            1.0 / ray.direction[i]
        } else {
            f32::INFINITY
        };
        let mut t1 = (min[i] - ray.origin[i]) * inv_d;
        let mut t2 = (max[i] - ray.origin[i]) * inv_d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }
    if t_max < 0.0 {
        return None;
    }
    let t = if t_min < 0.0 { t_max } else { t_min };
    Some(t)
}

#[allow(dead_code)]
pub fn ray_normalize_dir(ray: &Ray) -> Ray {
    let d = &ray.direction;
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-10 {
        return ray.clone();
    }
    Ray {
        origin: ray.origin,
        direction: [d[0] / len, d[1] / len, d[2] / len],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_at() {
        let r = new_ray([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = ray_at(&r, 5.0);
        assert!((p[0] - 5.0).abs() < 1e-6);
        assert!(p[1].abs() < 1e-6);
    }

    #[test]
    fn test_sphere_hit() {
        let r = new_ray([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let t = ray_sphere_intersect(&r, [0.0, 0.0, 0.0], 1.0);
        assert!(t.is_some());
        assert!((t.unwrap() - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_sphere_miss() {
        let r = new_ray([0.0, 5.0, -5.0], [0.0, 0.0, 1.0]);
        let t = ray_sphere_intersect(&r, [0.0, 0.0, 0.0], 1.0);
        assert!(t.is_none());
    }

    #[test]
    fn test_plane_hit() {
        let r = new_ray([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
        let t = ray_plane_intersect(&r, [0.0, 0.0, 1.0], 0.0);
        assert!(t.is_some());
        assert!((t.unwrap() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_plane_parallel() {
        let r = new_ray([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let t = ray_plane_intersect(&r, [0.0, 0.0, 1.0], 0.0);
        assert!(t.is_none());
    }

    #[test]
    fn test_aabb_hit() {
        let r = new_ray([-5.0, 0.5, 0.5], [1.0, 0.0, 0.0]);
        let t = ray_aabb_intersect(&r, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(t.is_some());
    }

    #[test]
    fn test_aabb_miss() {
        let r = new_ray([-5.0, 5.0, 0.5], [1.0, 0.0, 0.0]);
        let t = ray_aabb_intersect(&r, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(t.is_none());
    }

    #[test]
    fn test_normalize_dir() {
        let r = new_ray([0.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
        let rn = ray_normalize_dir(&r);
        let d = &rn.direction;
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_diagonal() {
        let r = new_ray([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        let rn = ray_normalize_dir(&r);
        let d = &rn.direction;
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }
}
