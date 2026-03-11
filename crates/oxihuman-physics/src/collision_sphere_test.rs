#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Result of a sphere collision test.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphereTest {
    pub hit: bool,
    pub distance: f32,
    pub point: [f32; 3],
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn sphere_sphere_test(c1: [f32; 3], r1: f32, c2: [f32; 3], r2: f32) -> SphereTest {
    let d = dist3(c1, c2);
    let combined = r1 + r2;
    SphereTest {
        hit: d < combined,
        distance: d - combined,
        point: [(c1[0] + c2[0]) * 0.5, (c1[1] + c2[1]) * 0.5, (c1[2] + c2[2]) * 0.5],
    }
}

#[allow(dead_code)]
pub fn sphere_plane_test(center: [f32; 3], radius: f32, plane_normal: [f32; 3], plane_d: f32) -> SphereTest {
    let dot = center[0] * plane_normal[0] + center[1] * plane_normal[1] + center[2] * plane_normal[2];
    let dist = dot - plane_d;
    SphereTest {
        hit: dist.abs() < radius,
        distance: dist.abs() - radius,
        point: [
            center[0] - plane_normal[0] * dist,
            center[1] - plane_normal[1] * dist,
            center[2] - plane_normal[2] * dist,
        ],
    }
}

#[allow(dead_code)]
pub fn sphere_aabb_test(center: [f32; 3], radius: f32, aabb_min: [f32; 3], aabb_max: [f32; 3]) -> SphereTest {
    let closest = [
        center[0].clamp(aabb_min[0], aabb_max[0]),
        center[1].clamp(aabb_min[1], aabb_max[1]),
        center[2].clamp(aabb_min[2], aabb_max[2]),
    ];
    let d = dist3(center, closest);
    SphereTest {
        hit: d < radius,
        distance: d - radius,
        point: closest,
    }
}

#[allow(dead_code)]
pub fn sphere_capsule_test(s_center: [f32; 3], s_radius: f32, cap_a: [f32; 3], cap_b: [f32; 3], cap_radius: f32) -> SphereTest {
    let closest = closest_point_on_segment(cap_a, cap_b, s_center);
    let d = dist3(s_center, closest);
    let combined = s_radius + cap_radius;
    SphereTest {
        hit: d < combined,
        distance: d - combined,
        point: closest,
    }
}

fn closest_point_on_segment(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let dot_ab = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    if dot_ab < 1e-12 {
        return a;
    }
    let t = ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / dot_ab).clamp(0.0, 1.0);
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

#[allow(dead_code)]
pub fn sphere_point_test(center: [f32; 3], radius: f32, point: [f32; 3]) -> SphereTest {
    let d = dist3(center, point);
    SphereTest {
        hit: d < radius,
        distance: d - radius,
        point,
    }
}

#[allow(dead_code)]
pub fn sphere_ray_test(center: [f32; 3], radius: f32, ray_origin: [f32; 3], ray_dir: [f32; 3]) -> SphereTest {
    let oc = [ray_origin[0] - center[0], ray_origin[1] - center[1], ray_origin[2] - center[2]];
    let a = ray_dir[0] * ray_dir[0] + ray_dir[1] * ray_dir[1] + ray_dir[2] * ray_dir[2];
    let b = 2.0 * (oc[0] * ray_dir[0] + oc[1] * ray_dir[1] + oc[2] * ray_dir[2]);
    let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return SphereTest { hit: false, distance: -1.0, point: [0.0; 3] };
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    let point = [
        ray_origin[0] + ray_dir[0] * t,
        ray_origin[1] + ray_dir[1] * t,
        ray_origin[2] + ray_dir[2] * t,
    ];
    SphereTest { hit: t >= 0.0, distance: t, point }
}

#[allow(dead_code)]
pub fn sphere_closest_point(center: [f32; 3], radius: f32, point: [f32; 3]) -> [f32; 3] {
    let d = dist3(center, point);
    if d < 1e-6 {
        return [center[0] + radius, center[1], center[2]];
    }
    let scale = radius / d;
    [
        center[0] + (point[0] - center[0]) * scale,
        center[1] + (point[1] - center[1]) * scale,
        center[2] + (point[2] - center[2]) * scale,
    ]
}

#[allow(dead_code)]
pub fn sphere_distance(center: [f32; 3], radius: f32, point: [f32; 3]) -> f32 {
    let d = dist3(center, point);
    (d - radius).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_sphere_hit() {
        let r = sphere_sphere_test([0.0; 3], 1.0, [1.0, 0.0, 0.0], 1.0);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_sphere_miss() {
        let r = sphere_sphere_test([0.0; 3], 0.1, [10.0, 0.0, 0.0], 0.1);
        assert!(!r.hit);
    }

    #[test]
    fn test_sphere_plane_hit() {
        let r = sphere_plane_test([0.0, 0.5, 0.0], 1.0, [0.0, 1.0, 0.0], 0.0);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_plane_miss() {
        let r = sphere_plane_test([0.0, 5.0, 0.0], 1.0, [0.0, 1.0, 0.0], 0.0);
        assert!(!r.hit);
    }

    #[test]
    fn test_sphere_aabb_hit() {
        let r = sphere_aabb_test([0.0; 3], 1.0, [-0.5; 3], [0.5; 3]);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_aabb_miss() {
        let r = sphere_aabb_test([10.0, 0.0, 0.0], 0.1, [-0.5; 3], [0.5; 3]);
        assert!(!r.hit);
    }

    #[test]
    fn test_sphere_capsule_hit() {
        let r = sphere_capsule_test([0.0; 3], 1.0, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_point_inside() {
        let r = sphere_point_test([0.0; 3], 1.0, [0.5, 0.0, 0.0]);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_ray_hit() {
        let r = sphere_ray_test([0.0; 3], 1.0, [-5.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(r.hit);
    }

    #[test]
    fn test_sphere_distance() {
        let d = sphere_distance([0.0; 3], 1.0, [3.0, 0.0, 0.0]);
        assert!((d - 2.0).abs() < 1e-5);
    }
}
