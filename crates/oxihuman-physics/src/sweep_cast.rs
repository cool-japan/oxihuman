// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// Result of a sweep cast query.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SweepResult {
    /// Whether the sweep hit something.
    pub hit: bool,
    /// Parameter along the sweep direction at impact (0..=dist).
    pub t: f32,
    /// Surface normal at impact (unit vector).
    pub normal: [f32; 3],
    /// World-space impact point.
    pub point: [f32; 3],
}

/// Return a no-hit `SweepResult`.
#[allow(dead_code)]
pub fn sweep_result_none() -> SweepResult {
    SweepResult {
        hit: false,
        t: f32::INFINITY,
        normal: [0.0, 1.0, 0.0],
        point: [0.0; 3],
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l > 1e-8 {
        [a[0] / l, a[1] / l, a[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// Sweep a sphere against an infinite plane defined by normal `plane_n` and offset `plane_d`.
/// The sphere moves from `center` in direction `dir` (normalized) for up to `dist` units.
#[allow(dead_code)]
pub fn sphere_sweep(
    center: [f32; 3],
    radius: f32,
    dir: [f32; 3],
    dist: f32,
    plane_n: [f32; 3],
    plane_d: f32,
) -> SweepResult {
    // Distance of sphere center to plane
    let d_center = dot3(center, plane_n) - plane_d;
    let d_vel = dot3(dir, plane_n);

    // If moving away from the plane, no hit
    if d_vel >= 0.0 {
        return sweep_result_none();
    }

    // Time to hit (sphere surface touches plane when d_center - radius == d_vel * t)
    let t = (d_center - radius) / (-d_vel);
    if t < 0.0 || t > dist {
        return sweep_result_none();
    }

    let point = add3(center, scale3(dir, t));
    SweepResult {
        hit: true,
        t,
        normal: plane_n,
        point,
    }
}

/// Sweep a capsule (segment from `cap_a` to `cap_b` with radius) against an AABB.
/// This is a simplified stub that checks if the capsule's swept path intersects the AABB.
#[allow(dead_code)]
pub fn capsule_sweep_aabb(
    cap_a: [f32; 3],
    cap_b: [f32; 3],
    radius: f32,
    dir: [f32; 3],
    box_min: [f32; 3],
    box_max: [f32; 3],
) -> SweepResult {
    // Expand the AABB by radius
    let expanded_min = [box_min[0] - radius, box_min[1] - radius, box_min[2] - radius];
    let expanded_max = [box_max[0] + radius, box_max[1] + radius, box_max[2] + radius];

    // Use the capsule midpoint as the test origin
    let origin = [
        (cap_a[0] + cap_b[0]) * 0.5,
        (cap_a[1] + cap_b[1]) * 0.5,
        (cap_a[2] + cap_b[2]) * 0.5,
    ];

    // Slab method for ray-AABB intersection
    let mut t_min = 0.0f32;
    let mut t_max = f32::INFINITY;

    for i in 0..3 {
        if dir[i].abs() < 1e-8 {
            if origin[i] < expanded_min[i] || origin[i] > expanded_max[i] {
                return sweep_result_none();
            }
        } else {
            let inv_d = 1.0 / dir[i];
            let t1 = (expanded_min[i] - origin[i]) * inv_d;
            let t2 = (expanded_max[i] - origin[i]) * inv_d;
            let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
        }
    }

    if t_min > t_max || t_max < 0.0 {
        return sweep_result_none();
    }

    let t = t_min.max(0.0);
    let point = add3(origin, scale3(dir, t));

    // Compute approximate normal (nearest AABB face)
    let center_box = [
        (expanded_min[0] + expanded_max[0]) * 0.5,
        (expanded_min[1] + expanded_max[1]) * 0.5,
        (expanded_min[2] + expanded_max[2]) * 0.5,
    ];
    let diff = [point[0] - center_box[0], point[1] - center_box[1], point[2] - center_box[2]];
    let half = [
        (expanded_max[0] - expanded_min[0]) * 0.5,
        (expanded_max[1] - expanded_min[1]) * 0.5,
        (expanded_max[2] - expanded_min[2]) * 0.5,
    ];

    let mut best_axis = 0;
    let mut best_val = (diff[0] / half[0].max(1e-8)).abs();
    for i in 1..3 {
        let v = (diff[i] / half[i].max(1e-8)).abs();
        if v > best_val {
            best_val = v;
            best_axis = i;
        }
    }
    let mut normal = [0.0f32; 3];
    normal[best_axis] = if diff[best_axis] >= 0.0 { 1.0 } else { -1.0 };

    SweepResult {
        hit: true,
        t,
        normal,
        point,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_sweep_hits_ground_plane() {
        // Sphere at [0, 2, 0], radius 0.5, moving downward
        let result = sphere_sweep(
            [0.0, 2.0, 0.0],
            0.5,
            [0.0, -1.0, 0.0],
            10.0,
            [0.0, 1.0, 0.0],
            0.0,
        );
        assert!(result.hit);
        assert!((result.t - 1.5).abs() < 1e-4);
    }

    #[test]
    fn sphere_sweep_moving_away_no_hit() {
        let result = sphere_sweep(
            [0.0, 2.0, 0.0],
            0.5,
            [0.0, 1.0, 0.0],
            10.0,
            [0.0, 1.0, 0.0],
            0.0,
        );
        assert!(!result.hit);
    }

    #[test]
    fn sphere_sweep_too_far_no_hit() {
        let result = sphere_sweep(
            [0.0, 100.0, 0.0],
            0.5,
            [0.0, -1.0, 0.0],
            1.0,
            [0.0, 1.0, 0.0],
            0.0,
        );
        assert!(!result.hit);
    }

    #[test]
    fn sweep_result_none_not_hit() {
        let r = sweep_result_none();
        assert!(!r.hit);
    }

    #[test]
    fn sweep_result_none_t_infinite() {
        let r = sweep_result_none();
        assert!(r.t.is_infinite());
    }

    #[test]
    fn capsule_sweep_aabb_hits() {
        // Capsule moving toward AABB
        let result = capsule_sweep_aabb(
            [-0.1, 5.0, -0.1],
            [0.1, 5.0, 0.1],
            0.1,
            [0.0, -1.0, 0.0],
            [-1.0, 0.0, -1.0],
            [1.0, 1.0, 1.0],
        );
        assert!(result.hit);
    }

    #[test]
    fn capsule_sweep_aabb_miss() {
        // Capsule moving away from AABB
        let result = capsule_sweep_aabb(
            [-0.1, 5.0, -0.1],
            [0.1, 5.0, 0.1],
            0.1,
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, -1.0],
            [1.0, 1.0, 1.0],
        );
        assert!(!result.hit);
    }

    #[test]
    fn sphere_sweep_t_positive_on_hit() {
        let result = sphere_sweep(
            [0.0, 3.0, 0.0],
            0.5,
            [0.0, -1.0, 0.0],
            10.0,
            [0.0, 1.0, 0.0],
            0.0,
        );
        assert!(result.hit);
        assert!(result.t > 0.0);
    }

    #[test]
    fn sphere_sweep_point_on_plane() {
        let result = sphere_sweep(
            [0.0, 3.0, 0.0],
            0.5,
            [0.0, -1.0, 0.0],
            10.0,
            [0.0, 1.0, 0.0],
            0.0,
        );
        assert!(result.hit);
        // Impact point y should be near sphere radius
        assert!((result.point[1] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn capsule_sweep_hit_t_nonnegative() {
        let result = capsule_sweep_aabb(
            [-0.1, 5.0, -0.1],
            [0.1, 5.0, 0.1],
            0.1,
            [0.0, -1.0, 0.0],
            [-1.0, 0.0, -1.0],
            [1.0, 1.0, 1.0],
        );
        assert!(result.hit);
        assert!(result.t >= 0.0);
    }
}
