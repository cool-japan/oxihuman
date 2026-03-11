// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Capsule-ray intersection tests.

use std::f32::consts::PI;

/// Ray in 3D space.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Ray {
    pub origin: [f32; 3],
    /// Must be normalised.
    pub direction: [f32; 3],
}

/// Capsule: segment from `a` to `b` with radius `r`.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct RayCapsule {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub radius: f32,
}

fn dot(u: [f32; 3], v: [f32; 3]) -> f32 {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn length_sq(v: [f32; 3]) -> f32 {
    dot(v, v)
}

/// Closest point on segment `[a,b]` to point p; returns parameter t in `[0,1]`.
#[allow(dead_code)]
pub fn closest_t_on_segment(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> f32 {
    let ab = sub(b, a);
    let ap = sub(p, a);
    let denom = length_sq(ab);
    if denom < 1e-12 {
        return 0.0;
    }
    (dot(ap, ab) / denom).clamp(0.0, 1.0)
}

/// Ray–capsule intersection; returns smallest positive t or None.
#[allow(dead_code)]
pub fn ray_capsule_intersect(ray: &Ray, cap: &RayCapsule) -> Option<f32> {
    // Parameterise by reducing to ray-cylinder then checking caps.
    let ab = sub(cap.b, cap.a);
    let ao = sub(ray.origin, cap.a);
    let d = ray.direction;

    let ab_len_sq = length_sq(ab);
    if ab_len_sq < 1e-12 {
        // Degenerate capsule → sphere
        return ray_sphere_intersect(ray, cap.a, cap.radius);
    }

    let ab_unit = scale(ab, 1.0 / ab_len_sq.sqrt());
    let d_par = dot(d, ab_unit);
    let ao_par = dot(ao, ab_unit);

    let d_perp = sub(d, scale(ab_unit, d_par));
    let ao_perp = sub(ao, scale(ab_unit, ao_par));

    let a_coef = length_sq(d_perp);
    let b_coef = 2.0 * dot(d_perp, ao_perp);
    let c_coef = length_sq(ao_perp) - cap.radius * cap.radius;

    let discriminant = b_coef * b_coef - 4.0 * a_coef * c_coef;

    let mut t_min = f32::MAX;

    if a_coef > 1e-12 && discriminant >= 0.0 {
        let sqrt_disc = discriminant.sqrt();
        for sign in &[-1.0_f32, 1.0_f32] {
            let t = (-b_coef + sign * sqrt_disc) / (2.0 * a_coef);
            if t < 0.0 {
                continue;
            }
            let hit_point = add(ray.origin, scale(d, t));
            let proj = dot(sub(hit_point, cap.a), ab_unit);
            let seg_len = ab_len_sq.sqrt();
            if (0.0..=seg_len).contains(&proj) && t < t_min {
                t_min = t;
            }
        }
    }

    // Check hemispherical caps.
    for &center in &[cap.a, cap.b] {
        if let Some(t) = ray_sphere_intersect(ray, center, cap.radius) {
            if t < t_min {
                t_min = t;
            }
        }
    }

    if t_min < f32::MAX {
        Some(t_min)
    } else {
        None
    }
}

fn ray_sphere_intersect(ray: &Ray, center: [f32; 3], radius: f32) -> Option<f32> {
    let oc = sub(ray.origin, center);
    let a = length_sq(ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = length_sq(oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t >= 0.0 {
        Some(t)
    } else {
        let t2 = (-b + disc.sqrt()) / (2.0 * a);
        if t2 >= 0.0 {
            Some(t2)
        } else {
            None
        }
    }
}

/// Normalise a 3-vector.
#[allow(dead_code)]
pub fn normalise(v: [f32; 3]) -> [f32; 3] {
    let len = length_sq(v).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    scale(v, 1.0 / len)
}

/// Capsule surface area.
#[allow(dead_code)]
pub fn capsule_surface_area(cap: &RayCapsule) -> f32 {
    let seg_len = {
        let d = sub(cap.b, cap.a);
        length_sq(d).sqrt()
    };
    4.0 * PI * cap.radius * cap.radius + 2.0 * PI * cap.radius * seg_len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ray_along_x(ox: f32) -> Ray {
        Ray {
            origin: [ox, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        }
    }

    #[test]
    fn test_hit_cylinder_body() {
        let cap = RayCapsule {
            a: [5.0, 0.0, -1.0],
            b: [5.0, 0.0, 1.0],
            radius: 0.5,
        };
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        assert!(ray_capsule_intersect(&ray, &cap).is_some());
    }

    #[test]
    fn test_miss() {
        let cap = RayCapsule {
            a: [5.0, 5.0, -1.0],
            b: [5.0, 5.0, 1.0],
            radius: 0.5,
        };
        let ray = ray_along_x(0.0);
        assert!(ray_capsule_intersect(&ray, &cap).is_none());
    }

    #[test]
    fn test_hit_cap_sphere() {
        let cap = RayCapsule {
            a: [3.0, 0.0, 0.0],
            b: [3.0, 0.0, 0.0],
            radius: 1.0,
        };
        let ray = ray_along_x(0.0);
        assert!(ray_capsule_intersect(&ray, &cap).is_some());
    }

    #[test]
    fn test_ray_behind() {
        let cap = RayCapsule {
            a: [-5.0, 0.0, -1.0],
            b: [-5.0, 0.0, 1.0],
            radius: 0.5,
        };
        let ray = ray_along_x(0.0);
        // capsule is behind the ray origin in +x direction
        assert!(ray_capsule_intersect(&ray, &cap).is_none());
    }

    #[test]
    fn test_normalise() {
        let v = normalise([3.0, 4.0, 0.0]);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len - 1.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_closest_t_on_segment() {
        let t = closest_t_on_segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        assert!((t - 0.5_f32).abs() < 1e-5);
    }

    #[test]
    fn test_surface_area() {
        let cap = RayCapsule {
            a: [0.0, 0.0, 0.0],
            b: [0.0, 0.0, 0.0],
            radius: 1.0,
        };
        let area = capsule_surface_area(&cap);
        assert!((area - 4.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_hit_returns_positive_t() {
        let cap = RayCapsule {
            a: [5.0, 0.0, -1.0],
            b: [5.0, 0.0, 1.0],
            radius: 0.5,
        };
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [1.0, 0.0, 0.0],
        };
        let t = ray_capsule_intersect(&ray, &cap).unwrap();
        assert!(t > 0.0);
    }

    #[test]
    fn test_degenerate_capsule_sphere_hit() {
        let cap = RayCapsule {
            a: [3.0, 0.0, 0.0],
            b: [3.0, 0.0, 0.0],
            radius: 0.5,
        };
        let ray = ray_along_x(0.0);
        let t = ray_capsule_intersect(&ray, &cap).unwrap();
        assert!((t - 2.5_f32).abs() < 1e-4);
    }
}
