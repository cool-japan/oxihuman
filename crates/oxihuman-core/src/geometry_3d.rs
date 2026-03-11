// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn norm3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

/// Ray-sphere intersection. Returns smallest positive t, or None.
pub fn ray_sphere_intersect(
    origin: [f32; 3],
    dir: [f32; 3],
    center: [f32; 3],
    r: f32,
) -> Option<f32> {
    let oc = sub3(origin, center);
    let a = dot3(dir, dir);
    let b = 2.0 * dot3(oc, dir);
    let c = dot3(oc, oc) - r * r;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let sq = disc.sqrt();
    let t1 = (-b - sq) / (2.0 * a);
    let t2 = (-b + sq) / (2.0 * a);
    if t1 > 1e-6 {
        Some(t1)
    } else if t2 > 1e-6 {
        Some(t2)
    } else {
        None
    }
}

/// Ray-plane intersection. Plane defined by normal and scalar d (dot(n, x) = d). Returns t or None.
pub fn ray_plane_intersect(
    origin: [f32; 3],
    dir: [f32; 3],
    normal: [f32; 3],
    d: f32,
) -> Option<f32> {
    let denom = dot3(normal, dir);
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = (d - dot3(normal, origin)) / denom;
    if t >= 0.0 {
        Some(t)
    } else {
        None
    }
}

/// Normalized normal of triangle (a, b, c).
pub fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    norm3(cross3(ab, ac))
}

/// Barycentric coordinates of 2D point p in triangle (a, b, c).
pub fn barycentric_2d(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> [f32; 3] {
    let v0 = [b[0] - a[0], b[1] - a[1]];
    let v1 = [c[0] - a[0], c[1] - a[1]];
    let v2 = [p[0] - a[0], p[1] - a[1]];
    let d00 = v0[0] * v0[0] + v0[1] * v0[1];
    let d01 = v0[0] * v1[0] + v0[1] * v1[1];
    let d11 = v1[0] * v1[0] + v1[1] * v1[1];
    let d20 = v2[0] * v0[0] + v2[1] * v0[1];
    let d21 = v2[0] * v1[0] + v2[1] * v1[1];
    let inv = d00 * d11 - d01 * d01;
    if inv.abs() < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / inv;
    let w = (d00 * d21 - d01 * d20) / inv;
    let u = 1.0 - v - w;
    [u, v, w]
}

/// Volume of a sphere.
pub fn sphere_volume(r: f32) -> f32 {
    (4.0 / 3.0) * PI * r * r * r
}

/// Volume of a box.
pub fn box_volume(w: f32, h: f32, d: f32) -> f32 {
    w * h * d
}

/// Euclidean distance between two 3D points.
pub fn dist_3d(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(b, a))
}

/// Closest point on segment (a, b) to point p.
pub fn closest_point_on_segment(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ap = sub3(p, a);
    let len_sq = dot3(ab, ab);
    if len_sq < 1e-12 {
        return a;
    }
    let t = (dot3(ap, ab) / len_sq).clamp(0.0, 1.0);
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

/// Surface area of a sphere (uses TAU).
pub fn sphere_surface_area(r: f32) -> f32 {
    TAU * 2.0 * r * r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_sphere_hit() {
        /* ray along +z hitting sphere at origin */
        let t = ray_sphere_intersect([0.0, 0.0, -5.0], [0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 1.0);
        assert!(t.is_some());
        let t = t.unwrap();
        assert!((t - 4.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_sphere_miss() {
        /* ray along +x missing sphere at origin */
        let t = ray_sphere_intersect([0.0, 5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        assert!(t.is_none());
    }

    #[test]
    fn test_ray_plane_intersect() {
        /* ray along +y hitting y=5 plane */
        let t = ray_plane_intersect([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], 5.0);
        assert!(t.is_some());
        assert!((t.unwrap() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_normal() {
        /* flat triangle in XY plane -> normal along +Z */
        let n = triangle_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2].abs() > 0.99);
    }

    #[test]
    fn test_barycentric_centroid() {
        /* centroid has equal barycentric coords */
        let a = [0.0f32, 0.0];
        let b = [1.0, 0.0];
        let c = [0.0, 1.0];
        let cen = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
        let bary = barycentric_2d(cen, a, b, c);
        assert!((bary[0] - 1.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_sphere_volume() {
        /* unit sphere volume = 4/3 * pi */
        let v = sphere_volume(1.0);
        assert!((v - 4.0 / 3.0 * std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn test_box_volume() {
        let v = box_volume(2.0, 3.0, 4.0);
        assert!((v - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_dist_3d() {
        /* 3-4-5 right triangle */
        let d = dist_3d([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_closest_point_on_segment() {
        /* p at segment midpoint */
        let cp = closest_point_on_segment([0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!((cp[0] - 0.0).abs() < 1e-5);
        /* p beyond end */
        let cp2 = closest_point_on_segment([5.0, 0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!((cp2[0] - 2.0).abs() < 1e-5);
    }
}
