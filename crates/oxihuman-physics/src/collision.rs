// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Geometric collision detection and response for physics primitives.
//!
//! Provides sphere, capsule, AABB, and plane primitives together with
//! pairwise overlap tests and a simple separation-based contact resolver.

// ── Private math helpers ──────────────────────────────────────────────────────

#[allow(dead_code)]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[allow(dead_code)]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

#[allow(dead_code)]
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        vec3_scale(v, 1.0 / len)
    }
}

#[allow(dead_code)]
fn vec3_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    vec3_add(vec3_scale(a, 1.0 - t), vec3_scale(b, t))
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Contact information returned by a collision test.
#[derive(Debug, Clone)]
pub struct Contact {
    /// World-space point of contact.
    pub point: [f32; 3],
    /// Contact normal pointing from shape A toward shape B.
    pub normal: [f32; 3],
    /// Penetration depth (positive means the shapes are overlapping).
    pub depth: f32,
}

/// A sphere primitive.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    /// Center of the sphere.
    pub center: [f32; 3],
    /// Radius of the sphere.
    pub radius: f32,
}

/// A capsule: a cylinder with hemispherical end caps.
#[derive(Debug, Clone, Copy)]
pub struct Capsule {
    /// One endpoint of the inner line segment.
    pub a: [f32; 3],
    /// Other endpoint of the inner line segment.
    pub b: [f32; 3],
    /// Radius of the capsule (applied uniformly around the segment).
    pub radius: f32,
}

/// An axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct CollisionAabb {
    /// Minimum corner (x, y, z).
    pub min: [f32; 3],
    /// Maximum corner (x, y, z).
    pub max: [f32; 3],
}

/// An infinite plane defined by a unit normal and signed distance from the origin.
#[derive(Debug, Clone, Copy)]
pub struct CollisionPlane {
    /// Unit outward normal of the plane.
    pub normal: [f32; 3],
    /// Signed distance from the world origin along the normal.
    pub d: f32,
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Returns the closest point on line segment (`a`, `b`) to point `p`.
pub fn closest_point_on_segment(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let ab = vec3_sub(b, a);
    let ap = vec3_sub(p, a);
    let len_sq = vec3_dot(ab, ab);
    if len_sq < 1e-14 {
        return a;
    }
    let t = (vec3_dot(ap, ab) / len_sq).clamp(0.0, 1.0);
    vec3_add(a, vec3_scale(ab, t))
}

/// Returns the closest point on `aabb` to point `p` (clamped to box surface).
pub fn closest_point_on_aabb(aabb: &CollisionAabb, p: [f32; 3]) -> [f32; 3] {
    [
        p[0].clamp(aabb.min[0], aabb.max[0]),
        p[1].clamp(aabb.min[1], aabb.max[1]),
        p[2].clamp(aabb.min[2], aabb.max[2]),
    ]
}

/// Returns the squared distance from point `p` to line segment (`a`, `b`).
pub fn sq_dist_point_segment(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> f32 {
    let closest = closest_point_on_segment(a, b, p);
    let d = vec3_sub(p, closest);
    vec3_dot(d, d)
}

/// Simple contact resolver: push `pos_a` and `pos_b` apart by equal halves along
/// the contact normal so they no longer penetrate.
///
/// Returns `(new_pos_a, new_pos_b)`.
pub fn resolve_contact(
    pos_a: [f32; 3],
    pos_b: [f32; 3],
    contact: &Contact,
) -> ([f32; 3], [f32; 3]) {
    let half = contact.depth * 0.5;
    let push_a = vec3_scale(contact.normal, -half);
    let push_b = vec3_scale(contact.normal, half);
    (vec3_add(pos_a, push_a), vec3_add(pos_b, push_b))
}

// ── Sphere tests ──────────────────────────────────────────────────────────────

/// Tests sphere–sphere collision.
///
/// Returns `Some(Contact)` when the spheres overlap, `None` otherwise.
pub fn sphere_sphere(a: &Sphere, b: &Sphere) -> Option<Contact> {
    let diff = vec3_sub(b.center, a.center);
    let dist_sq = vec3_dot(diff, diff);
    let sum_r = a.radius + b.radius;
    if dist_sq >= sum_r * sum_r {
        return None;
    }
    let dist = dist_sq.sqrt();
    let normal = if dist < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        vec3_scale(diff, 1.0 / dist)
    };
    let depth = sum_r - dist;
    // Contact point: on the surface of A toward B
    let point = vec3_add(a.center, vec3_scale(normal, a.radius));
    Some(Contact {
        point,
        normal,
        depth,
    })
}

/// Tests sphere–plane collision.
///
/// Returns `Some(Contact)` when the sphere penetrates the plane, `None` otherwise.
pub fn sphere_plane(sphere: &Sphere, plane: &CollisionPlane) -> Option<Contact> {
    let dist = vec3_dot(sphere.center, plane.normal) - plane.d;
    if dist >= sphere.radius {
        return None;
    }
    let depth = sphere.radius - dist;
    let normal = plane.normal;
    // Contact point: projection of sphere center onto the plane
    let point = vec3_sub(sphere.center, vec3_scale(normal, dist));
    Some(Contact {
        point,
        normal,
        depth,
    })
}

/// Tests sphere–AABB collision.
///
/// Returns `Some(Contact)` when the sphere overlaps the box, `None` otherwise.
pub fn sphere_aabb(sphere: &Sphere, aabb: &CollisionAabb) -> Option<Contact> {
    let closest = closest_point_on_aabb(aabb, sphere.center);
    let diff = vec3_sub(sphere.center, closest);
    let dist_sq = vec3_dot(diff, diff);
    if dist_sq >= sphere.radius * sphere.radius {
        return None;
    }
    let dist = dist_sq.sqrt();
    let normal = if dist < 1e-10 {
        // Center is inside the box – find the axis with smallest penetration
        best_aabb_normal(aabb, sphere.center)
    } else {
        vec3_scale(diff, 1.0 / dist)
    };
    let depth = sphere.radius - dist;
    Some(Contact {
        point: closest,
        normal,
        depth,
    })
}

/// Computes the AABB face normal pointing toward `p` for when `p` is inside the box.
fn best_aabb_normal(aabb: &CollisionAabb, p: [f32; 3]) -> [f32; 3] {
    // Find the axis along which p is closest to a face by checking all 3 explicitly
    let mins = [p[0] - aabb.min[0], p[1] - aabb.min[1], p[2] - aabb.min[2]];
    let maxs = [aabb.max[0] - p[0], aabb.max[1] - p[1], aabb.max[2] - p[2]];
    let dists = [
        mins[0].min(maxs[0]),
        mins[1].min(maxs[1]),
        mins[2].min(maxs[2]),
    ];

    let best_axis = if dists[0] <= dists[1] && dists[0] <= dists[2] {
        0usize
    } else if dists[1] <= dists[2] {
        1usize
    } else {
        2usize
    };

    let mut n = [0.0f32; 3];
    n[best_axis] = if mins[best_axis] < maxs[best_axis] {
        -1.0
    } else {
        1.0
    };
    n
}

// ── Capsule tests ─────────────────────────────────────────────────────────────

/// Tests capsule–sphere collision.
///
/// Returns `Some(Contact)` when the capsule and sphere overlap, `None` otherwise.
pub fn capsule_sphere(cap: &Capsule, sphere: &Sphere) -> Option<Contact> {
    // Treat the capsule as a sphere centered at the closest point on its segment
    let closest = closest_point_on_segment(cap.a, cap.b, sphere.center);
    let fake_sphere = Sphere {
        center: closest,
        radius: cap.radius,
    };
    sphere_sphere(&fake_sphere, sphere)
}

/// Tests capsule–plane collision.
///
/// Returns `Some(Contact)` when either end cap penetrates the plane, `None` otherwise.
pub fn capsule_plane(cap: &Capsule, plane: &CollisionPlane) -> Option<Contact> {
    // The deepest point of the capsule relative to the plane is the end cap
    // with the smaller signed distance to the plane.
    let dist_a = vec3_dot(cap.a, plane.normal) - plane.d;
    let dist_b = vec3_dot(cap.b, plane.normal) - plane.d;

    // Choose the end with the smallest (most negative) distance
    let (deepest_pt, dist) = if dist_a <= dist_b {
        (cap.a, dist_a)
    } else {
        (cap.b, dist_b)
    };

    if dist >= cap.radius {
        return None;
    }

    let depth = cap.radius - dist;
    let normal = plane.normal;
    let point = vec3_sub(deepest_pt, vec3_scale(normal, dist));
    Some(Contact {
        point,
        normal,
        depth,
    })
}

/// Tests capsule–capsule collision.
///
/// Returns `Some(Contact)` when the capsules overlap, `None` otherwise.
pub fn capsule_capsule(a: &Capsule, b: &Capsule) -> Option<Contact> {
    // Find the closest pair of points between the two segments
    let (pa, pb) = closest_points_between_segments(a.a, a.b, b.a, b.b);
    let sphere_a = Sphere {
        center: pa,
        radius: a.radius,
    };
    let sphere_b = Sphere {
        center: pb,
        radius: b.radius,
    };
    sphere_sphere(&sphere_a, &sphere_b)
}

/// Computes the closest point pair between two line segments (p0, p1) and (q0, q1).
///
/// Based on the algorithm from "Real-Time Collision Detection" by Ericson.
fn closest_points_between_segments(
    p0: [f32; 3],
    p1: [f32; 3],
    q0: [f32; 3],
    q1: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    let d1 = vec3_sub(p1, p0);
    let d2 = vec3_sub(q1, q0);
    let r = vec3_sub(p0, q0);

    let a = vec3_dot(d1, d1);
    let e = vec3_dot(d2, d2);
    let f = vec3_dot(d2, r);

    let (s, t) = if a < 1e-10 && e < 1e-10 {
        // Both degenerate
        (0.0f32, 0.0f32)
    } else if a < 1e-10 {
        // First degenerate
        (0.0, (f / e).clamp(0.0, 1.0))
    } else {
        let c = vec3_dot(d1, r);
        if e < 1e-10 {
            // Second degenerate
            ((-c / a).clamp(0.0, 1.0), 0.0)
        } else {
            let b_coeff = vec3_dot(d1, d2);
            let denom = a * e - b_coeff * b_coeff;
            let s = if denom.abs() > 1e-10 {
                ((b_coeff * f - c * e) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let t = (b_coeff * s + f) / e;
            if t < 0.0 {
                let s = (-c / a).clamp(0.0, 1.0);
                (s, 0.0)
            } else if t > 1.0 {
                let s = ((b_coeff - c) / a).clamp(0.0, 1.0);
                (s, 1.0)
            } else {
                (s, t)
            }
        }
    };

    let pa = vec3_add(p0, vec3_scale(d1, s));
    let pb = vec3_add(q0, vec3_scale(d2, t));
    (pa, pb)
}

// ── AABB tests ────────────────────────────────────────────────────────────────

/// Tests AABB–AABB collision (separating axis theorem on each axis).
///
/// Returns `Some(Contact)` when the boxes overlap, `None` otherwise.
pub fn aabb_aabb(a: &CollisionAabb, b: &CollisionAabb) -> Option<Contact> {
    // Compute overlap on each axis
    let overlap_x = a.max[0].min(b.max[0]) - a.min[0].max(b.min[0]);
    let overlap_y = a.max[1].min(b.max[1]) - a.min[1].max(b.min[1]);
    let overlap_z = a.max[2].min(b.max[2]) - a.min[2].max(b.min[2]);

    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
        return None;
    }

    // Choose the axis of minimum penetration
    let (depth, axis) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        (overlap_x, 0usize)
    } else if overlap_y <= overlap_z {
        (overlap_y, 1usize)
    } else {
        (overlap_z, 2usize)
    };

    // Direction: from A center to B center along the chosen axis
    let center_a = [
        (a.min[0] + a.max[0]) * 0.5,
        (a.min[1] + a.max[1]) * 0.5,
        (a.min[2] + a.max[2]) * 0.5,
    ];
    let center_b = [
        (b.min[0] + b.max[0]) * 0.5,
        (b.min[1] + b.max[1]) * 0.5,
        (b.min[2] + b.max[2]) * 0.5,
    ];

    let mut normal = [0.0f32; 3];
    normal[axis] = if center_b[axis] >= center_a[axis] {
        1.0
    } else {
        -1.0
    };

    // Contact point: midpoint of the overlap region on the separating face
    let point = [
        (a.min[0].max(b.min[0]) + a.max[0].min(b.max[0])) * 0.5,
        (a.min[1].max(b.min[1]) + a.max[1].min(b.max[1])) * 0.5,
        (a.min[2].max(b.min[2]) + a.max[2].min(b.max[2])) * 0.5,
    ];

    Some(Contact {
        point,
        normal,
        depth,
    })
}

/// Tests AABB–plane collision.
///
/// Returns `Some(Contact)` when any part of the box penetrates the plane.
pub fn aabb_plane(aabb: &CollisionAabb, plane: &CollisionPlane) -> Option<Contact> {
    // Project the box onto the plane normal and find the most negative vertex
    let center = [
        (aabb.min[0] + aabb.max[0]) * 0.5,
        (aabb.min[1] + aabb.max[1]) * 0.5,
        (aabb.min[2] + aabb.max[2]) * 0.5,
    ];
    let half_extents = [
        (aabb.max[0] - aabb.min[0]) * 0.5,
        (aabb.max[1] - aabb.min[1]) * 0.5,
        (aabb.max[2] - aabb.min[2]) * 0.5,
    ];

    // Effective radius of the box projected onto the plane normal
    let projected_radius = half_extents[0] * plane.normal[0].abs()
        + half_extents[1] * plane.normal[1].abs()
        + half_extents[2] * plane.normal[2].abs();

    let center_dist = vec3_dot(center, plane.normal) - plane.d;

    if center_dist >= projected_radius {
        return None;
    }

    let depth = projected_radius - center_dist;
    let normal = plane.normal;

    // Contact point: deepest corner projected onto the plane
    let deepest = [
        center[0] - plane.normal[0].signum() * half_extents[0],
        center[1] - plane.normal[1].signum() * half_extents[1],
        center[2] - plane.normal[2].signum() * half_extents[2],
    ];
    let deepest_dist = vec3_dot(deepest, plane.normal) - plane.d;
    let point = vec3_sub(deepest, vec3_scale(plane.normal, deepest_dist));

    Some(Contact {
        point,
        normal,
        depth,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Tolerance for floating-point comparisons
    const EPS: f32 = 1e-4;

    fn assert_approx(a: f32, b: f32, label: &str) {
        assert!(
            (a - b).abs() < EPS,
            "{label}: expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    // ── sphere_sphere ─────────────────────────────────────────────────────────

    #[test]
    fn sphere_sphere_overlap_detects() {
        let a = Sphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        };
        let b = Sphere {
            center: [1.5, 0.0, 0.0],
            radius: 1.0,
        };
        assert!(sphere_sphere(&a, &b).is_some());
    }

    #[test]
    fn sphere_sphere_no_overlap_none() {
        let a = Sphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        };
        let b = Sphere {
            center: [3.0, 0.0, 0.0],
            radius: 1.0,
        };
        assert!(sphere_sphere(&a, &b).is_none());
    }

    #[test]
    fn sphere_sphere_contact_depth_correct() {
        let a = Sphere {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
        };
        let b = Sphere {
            center: [1.5, 0.0, 0.0],
            radius: 1.0,
        };
        let contact = sphere_sphere(&a, &b).expect("should succeed");
        // Sum of radii = 2.0, distance = 1.5, depth = 0.5
        assert_approx(contact.depth, 0.5, "depth");
    }

    // ── sphere_plane ──────────────────────────────────────────────────────────

    #[test]
    fn sphere_plane_above_no_contact() {
        // Plane: y = 0 (normal = [0,1,0], d = 0)
        let plane = CollisionPlane {
            normal: [0.0, 1.0, 0.0],
            d: 0.0,
        };
        let sphere = Sphere {
            center: [0.0, 2.0, 0.0],
            radius: 1.0,
        };
        // Distance from center to plane = 2.0, radius = 1.0 → no contact
        assert!(sphere_plane(&sphere, &plane).is_none());
    }

    #[test]
    fn sphere_plane_below_contact() {
        let plane = CollisionPlane {
            normal: [0.0, 1.0, 0.0],
            d: 0.0,
        };
        let sphere = Sphere {
            center: [0.0, 0.5, 0.0],
            radius: 1.0,
        };
        let contact = sphere_plane(&sphere, &plane).expect("should succeed");
        // depth = 1.0 - 0.5 = 0.5
        assert_approx(contact.depth, 0.5, "sphere_plane depth");
        assert_approx(contact.normal[1], 1.0, "sphere_plane normal y");
    }

    // ── sphere_aabb ───────────────────────────────────────────────────────────

    #[test]
    fn sphere_aabb_inside_aabb_contact() {
        // Sphere centered just outside one face
        let aabb = CollisionAabb {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        let sphere = Sphere {
            center: [1.4, 0.0, 0.0],
            radius: 1.0,
        };
        // closest point = [1,0,0], dist = 0.4, depth = 0.6
        let contact = sphere_aabb(&sphere, &aabb).expect("should succeed");
        assert!(contact.depth > 0.0);
    }

    #[test]
    fn sphere_aabb_outside_no_contact() {
        let aabb = CollisionAabb {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        let sphere = Sphere {
            center: [3.0, 0.0, 0.0],
            radius: 1.0,
        };
        assert!(sphere_aabb(&sphere, &aabb).is_none());
    }

    // ── capsule_sphere ────────────────────────────────────────────────────────

    #[test]
    fn capsule_sphere_overlap() {
        let cap = Capsule {
            a: [0.0, 0.0, 0.0],
            b: [0.0, 2.0, 0.0],
            radius: 0.5,
        };
        let sphere = Sphere {
            center: [0.8, 1.0, 0.0],
            radius: 0.5,
        };
        // Distance from sphere center to segment = 0.8, sum of radii = 1.0
        assert!(capsule_sphere(&cap, &sphere).is_some());
    }

    #[test]
    fn capsule_sphere_no_overlap() {
        let cap = Capsule {
            a: [0.0, 0.0, 0.0],
            b: [0.0, 2.0, 0.0],
            radius: 0.3,
        };
        let sphere = Sphere {
            center: [2.0, 1.0, 0.0],
            radius: 0.3,
        };
        // Distance = 2.0, sum of radii = 0.6 → no overlap
        assert!(capsule_sphere(&cap, &sphere).is_none());
    }

    // ── capsule_plane ─────────────────────────────────────────────────────────

    #[test]
    fn capsule_plane_contact() {
        // Capsule endpoint near the floor (y = 0 plane)
        let plane = CollisionPlane {
            normal: [0.0, 1.0, 0.0],
            d: 0.0,
        };
        let cap = Capsule {
            a: [0.0, 0.4, 0.0],
            b: [0.0, 2.0, 0.0],
            radius: 0.5,
        };
        // Nearest endpoint at y=0.4, dist=0.4 < radius=0.5 → contact
        let contact = capsule_plane(&cap, &plane).expect("should succeed");
        assert!(contact.depth > 0.0);
        assert_approx(contact.normal[1], 1.0, "capsule_plane normal y");
    }

    // ── capsule_capsule ───────────────────────────────────────────────────────

    #[test]
    fn capsule_capsule_parallel_overlap() {
        let a = Capsule {
            a: [0.0, 0.0, 0.0],
            b: [2.0, 0.0, 0.0],
            radius: 0.5,
        };
        let b = Capsule {
            a: [0.0, 0.8, 0.0],
            b: [2.0, 0.8, 0.0],
            radius: 0.5,
        };
        // Parallel capsules 0.8 apart, sum of radii = 1.0 → overlap
        assert!(capsule_capsule(&a, &b).is_some());
    }

    #[test]
    fn capsule_capsule_no_overlap() {
        let a = Capsule {
            a: [0.0, 0.0, 0.0],
            b: [1.0, 0.0, 0.0],
            radius: 0.3,
        };
        let b = Capsule {
            a: [0.0, 2.0, 0.0],
            b: [1.0, 2.0, 0.0],
            radius: 0.3,
        };
        // 2.0 apart, sum of radii = 0.6 → no overlap
        assert!(capsule_capsule(&a, &b).is_none());
    }

    // ── aabb_aabb ─────────────────────────────────────────────────────────────

    #[test]
    fn aabb_aabb_overlap_detects() {
        let a = CollisionAabb {
            min: [0.0, 0.0, 0.0],
            max: [2.0, 2.0, 2.0],
        };
        let b = CollisionAabb {
            min: [1.0, 1.0, 1.0],
            max: [3.0, 3.0, 3.0],
        };
        let contact = aabb_aabb(&a, &b).expect("should succeed");
        assert_approx(contact.depth, 1.0, "aabb_aabb depth");
    }

    #[test]
    fn aabb_aabb_no_overlap_none() {
        let a = CollisionAabb {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        };
        let b = CollisionAabb {
            min: [2.0, 0.0, 0.0],
            max: [3.0, 1.0, 1.0],
        };
        assert!(aabb_aabb(&a, &b).is_none());
    }

    // ── closest_point_on_segment ──────────────────────────────────────────────

    #[test]
    fn closest_point_on_segment_midpoint() {
        let cp = closest_point_on_segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        assert_approx(cp[0], 1.0, "x");
        assert_approx(cp[1], 0.0, "y");
        assert_approx(cp[2], 0.0, "z");
    }

    #[test]
    fn closest_point_on_segment_endpoint() {
        // Point is beyond one end of the segment
        let cp = closest_point_on_segment([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
        assert_approx(cp[0], 1.0, "x clamped to end");
    }

    // ── sq_dist_point_segment ─────────────────────────────────────────────────

    #[test]
    fn sq_dist_point_segment_zero() {
        // Point on the segment → distance = 0
        let d = sq_dist_point_segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_approx(d, 0.0, "sq_dist zero");
    }

    // ── resolve_contact ───────────────────────────────────────────────────────

    #[test]
    fn resolve_contact_separates_objects() {
        // Two spheres overlapping by 0.2 along x axis
        let contact = Contact {
            point: [0.0, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            depth: 0.2,
        };
        let (new_a, new_b) = resolve_contact([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], &contact);
        // A pushed back by 0.1, B pushed forward by 0.1
        assert_approx(new_a[0], -0.1, "pos_a x");
        assert_approx(new_b[0], 0.1, "pos_b x");
    }

    // ── aabb_plane ────────────────────────────────────────────────────────────

    #[test]
    fn aabb_plane_below_contact() {
        // Box with bottom face at y = -0.2, plane at y = 0
        let aabb = CollisionAabb {
            min: [-1.0, -0.2, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        let plane = CollisionPlane {
            normal: [0.0, 1.0, 0.0],
            d: 0.0,
        };
        let contact = aabb_plane(&aabb, &plane).expect("should succeed");
        assert!(contact.depth > 0.0, "should have positive depth");
        assert_approx(contact.normal[1], 1.0, "normal y");
    }
}
