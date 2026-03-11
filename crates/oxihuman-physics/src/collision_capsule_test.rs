#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Result of a capsule collision test.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapsuleTest {
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

fn closest_on_seg(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> [f32; 3] {
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
pub fn capsule_capsule_test(
    a1: [f32; 3], a2: [f32; 3], ra: f32,
    b1: [f32; 3], b2: [f32; 3], rb: f32,
) -> CapsuleTest {
    // Simplified: closest point between two segments
    let _mid_a = [(a1[0] + a2[0]) * 0.5, (a1[1] + a2[1]) * 0.5, (a1[2] + a2[2]) * 0.5];
    let mid_b = [(b1[0] + b2[0]) * 0.5, (b1[1] + b2[1]) * 0.5, (b1[2] + b2[2]) * 0.5];
    let ca = closest_on_seg(a1, a2, mid_b);
    let cb = closest_on_seg(b1, b2, ca);
    let d = dist3(ca, cb);
    let combined = ra + rb;
    CapsuleTest {
        hit: d < combined,
        distance: d - combined,
        point: [(ca[0] + cb[0]) * 0.5, (ca[1] + cb[1]) * 0.5, (ca[2] + cb[2]) * 0.5],
    }
}

#[allow(dead_code)]
pub fn capsule_sphere_test(
    cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32,
    sphere_c: [f32; 3], sphere_r: f32,
) -> CapsuleTest {
    let closest = closest_on_seg(cap_a, cap_b, sphere_c);
    let d = dist3(closest, sphere_c);
    let combined = cap_r + sphere_r;
    CapsuleTest {
        hit: d < combined,
        distance: d - combined,
        point: closest,
    }
}

#[allow(dead_code)]
pub fn capsule_plane_test(
    cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32,
    plane_n: [f32; 3], plane_d: f32,
) -> CapsuleTest {
    let da = cap_a[0] * plane_n[0] + cap_a[1] * plane_n[1] + cap_a[2] * plane_n[2] - plane_d;
    let db = cap_b[0] * plane_n[0] + cap_b[1] * plane_n[1] + cap_b[2] * plane_n[2] - plane_d;
    let min_d = da.abs().min(db.abs());
    CapsuleTest {
        hit: min_d < cap_r,
        distance: min_d - cap_r,
        point: if da.abs() < db.abs() { cap_a } else { cap_b },
    }
}

#[allow(dead_code)]
pub fn capsule_aabb_test(
    cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32,
    aabb_min: [f32; 3], aabb_max: [f32; 3],
) -> CapsuleTest {
    let center = [
        (aabb_min[0] + aabb_max[0]) * 0.5,
        (aabb_min[1] + aabb_max[1]) * 0.5,
        (aabb_min[2] + aabb_max[2]) * 0.5,
    ];
    let closest = closest_on_seg(cap_a, cap_b, center);
    let clamped = [
        closest[0].clamp(aabb_min[0], aabb_max[0]),
        closest[1].clamp(aabb_min[1], aabb_max[1]),
        closest[2].clamp(aabb_min[2], aabb_max[2]),
    ];
    let d = dist3(closest, clamped);
    CapsuleTest {
        hit: d < cap_r,
        distance: d - cap_r,
        point: clamped,
    }
}

#[allow(dead_code)]
pub fn capsule_point_test(cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32, point: [f32; 3]) -> CapsuleTest {
    let closest = closest_on_seg(cap_a, cap_b, point);
    let d = dist3(closest, point);
    CapsuleTest {
        hit: d < cap_r,
        distance: d - cap_r,
        point: closest,
    }
}

#[allow(dead_code)]
pub fn capsule_ray_test(
    cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32,
    ray_o: [f32; 3], ray_d: [f32; 3],
) -> CapsuleTest {
    // Simplified: test ray against sphere at closest point on segment to ray origin
    let closest = closest_on_seg(cap_a, cap_b, ray_o);
    let oc = [ray_o[0] - closest[0], ray_o[1] - closest[1], ray_o[2] - closest[2]];
    let a = ray_d[0] * ray_d[0] + ray_d[1] * ray_d[1] + ray_d[2] * ray_d[2];
    let b = 2.0 * (oc[0] * ray_d[0] + oc[1] * ray_d[1] + oc[2] * ray_d[2]);
    let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - cap_r * cap_r;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return CapsuleTest { hit: false, distance: -1.0, point: [0.0; 3] };
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    let point = [ray_o[0] + ray_d[0] * t, ray_o[1] + ray_d[1] * t, ray_o[2] + ray_d[2] * t];
    CapsuleTest { hit: t >= 0.0, distance: t, point }
}

#[allow(dead_code)]
pub fn capsule_closest_point(cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32, point: [f32; 3]) -> [f32; 3] {
    let closest = closest_on_seg(cap_a, cap_b, point);
    let d = dist3(closest, point);
    if d < 1e-6 {
        return [closest[0] + cap_r, closest[1], closest[2]];
    }
    let scale = cap_r / d;
    [
        closest[0] + (point[0] - closest[0]) * scale,
        closest[1] + (point[1] - closest[1]) * scale,
        closest[2] + (point[2] - closest[2]) * scale,
    ]
}

#[allow(dead_code)]
pub fn capsule_distance(cap_a: [f32; 3], cap_b: [f32; 3], cap_r: f32, point: [f32; 3]) -> f32 {
    let closest = closest_on_seg(cap_a, cap_b, point);
    let d = dist3(closest, point);
    (d - cap_r).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_capsule_hit() {
        let r = capsule_capsule_test(
            [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5,
            [0.5, -1.0, 0.0], [0.5, 1.0, 0.0], 0.5,
        );
        assert!(r.hit);
    }

    #[test]
    fn test_capsule_capsule_miss() {
        let r = capsule_capsule_test(
            [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.1,
            [10.0, -1.0, 0.0], [10.0, 1.0, 0.0], 0.1,
        );
        assert!(!r.hit);
    }

    #[test]
    fn test_capsule_sphere_hit() {
        let r = capsule_sphere_test([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.5, [0.3, 0.0, 0.0], 0.5);
        assert!(r.hit);
    }

    #[test]
    fn test_capsule_sphere_miss() {
        let r = capsule_sphere_test([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.1, [10.0, 0.0, 0.0], 0.1);
        assert!(!r.hit);
    }

    #[test]
    fn test_capsule_plane_hit() {
        let r = capsule_plane_test([0.0, 0.3, 0.0], [0.0, 0.5, 0.0], 0.5, [0.0, 1.0, 0.0], 0.0);
        assert!(r.hit);
    }

    #[test]
    fn test_capsule_aabb_hit() {
        let r = capsule_aabb_test([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5, [-0.3; 3], [0.3; 3]);
        assert!(r.hit);
    }

    #[test]
    fn test_capsule_point_inside() {
        let r = capsule_point_test([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 1.0, [0.1, 0.0, 0.0]);
        assert!(r.hit);
    }

    #[test]
    fn test_capsule_ray_hit() {
        let r = capsule_ray_test([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 1.0, [-5.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(r.hit);
    }

    #[test]
    fn test_capsule_closest_point() {
        let cp = capsule_closest_point([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 1.0, [5.0, 0.0, 0.0]);
        assert!((cp[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_capsule_distance() {
        let d = capsule_distance([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 1.0, [3.0, 0.0, 0.0]);
        assert!((d - 2.0).abs() < 1e-5);
    }
}
