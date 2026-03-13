// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Capsule-capsule pair geometry: closest-point, overlap, and contact queries.

/// A capsule defined by two endpoint centers and a radius.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CapsulePrim {
    pub p0: [f32; 3],
    pub p1: [f32; 3],
    pub radius: f32,
}

#[allow(dead_code)]
impl CapsulePrim {
    pub fn new(p0: [f32; 3], p1: [f32; 3], radius: f32) -> Self {
        Self { p0, p1, radius }
    }

    pub fn length(&self) -> f32 {
        let dx = self.p1[0] - self.p0[0];
        let dy = self.p1[1] - self.p0[1];
        let dz = self.p1[2] - self.p0[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Closest point on segment [p0, p1] to point q. Returns parameter t in `[0,1]`.
#[allow(dead_code)]
pub fn closest_point_on_segment(p0: [f32; 3], p1: [f32; 3], q: [f32; 3]) -> (f32, [f32; 3]) {
    let d = sub3(p1, p0);
    let len_sq = dot3(d, d);
    if len_sq < 1e-12 {
        return (0.0, p0);
    }
    let t = (dot3(sub3(q, p0), d) / len_sq).clamp(0.0, 1.0);
    let pt = [p0[0] + t * d[0], p0[1] + t * d[1], p0[2] + t * d[2]];
    (t, pt)
}

/// Closest points between two line segments (returns parameters s, t and the points).
#[allow(dead_code)]
pub fn closest_points_segment_segment(
    a0: [f32; 3],
    a1: [f32; 3],
    b0: [f32; 3],
    b1: [f32; 3],
) -> (f32, f32, [f32; 3], [f32; 3]) {
    let d1 = sub3(a1, a0);
    let d2 = sub3(b1, b0);
    let r = sub3(a0, b0);

    let a = dot3(d1, d1);
    let e = dot3(d2, d2);
    let f = dot3(d2, r);

    let (s, t) = if a < 1e-9 && e < 1e-9 {
        (0.0, 0.0)
    } else if a < 1e-9 {
        (0.0, (f / e).clamp(0.0, 1.0))
    } else {
        let c = dot3(d1, r);
        if e < 1e-9 {
            ((-c / a).clamp(0.0, 1.0), 0.0)
        } else {
            let b = dot3(d1, d2);
            let denom = a * e - b * b;
            let s0 = if denom.abs() > 1e-9 {
                ((b * f - c * e) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let t0 = (b * s0 + f) / e;
            if t0 < 0.0 {
                ((-c / a).clamp(0.0, 1.0), 0.0)
            } else if t0 > 1.0 {
                (((b - c) / a).clamp(0.0, 1.0), 1.0)
            } else {
                (s0, t0)
            }
        }
    };

    let pa = [a0[0] + s * d1[0], a0[1] + s * d1[1], a0[2] + s * d1[2]];
    let pb = [b0[0] + t * d2[0], b0[1] + t * d2[1], b0[2] + t * d2[2]];
    (s, t, pa, pb)
}

/// Squared distance between closest points of two capsule segments.
#[allow(dead_code)]
pub fn capsule_pair_sq_dist(a: &CapsulePrim, b: &CapsulePrim) -> f32 {
    let (_, _, pa, pb) = closest_points_segment_segment(a.p0, a.p1, b.p0, b.p1);
    let d = sub3(pa, pb);
    dot3(d, d)
}

/// True if two capsules overlap (penetrate).
#[allow(dead_code)]
pub fn capsule_pair_overlap(a: &CapsulePrim, b: &CapsulePrim) -> bool {
    let sum_r = a.radius + b.radius;
    capsule_pair_sq_dist(a, b) <= sum_r * sum_r
}

/// Penetration depth between two capsules (positive means overlap).
#[allow(dead_code)]
pub fn capsule_pair_penetration(a: &CapsulePrim, b: &CapsulePrim) -> f32 {
    let sum_r = a.radius + b.radius;
    let dist = capsule_pair_sq_dist(a, b).sqrt();
    sum_r - dist
}

/// Contact normal from capsule a to capsule b (unit vector).
#[allow(dead_code)]
pub fn capsule_pair_normal(a: &CapsulePrim, b: &CapsulePrim) -> [f32; 3] {
    let (_, _, pa, pb) = closest_points_segment_segment(a.p0, a.p1, b.p0, b.p1);
    let d = sub3(pb, pa);
    let l = len3(d);
    if l < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [d[0] / l, d[1] / l, d[2] / l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_length() {
        let c = CapsulePrim::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5);
        assert!((c.length() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn closest_point_midpoint() {
        let (t, pt) = closest_point_on_segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        assert!((t - 0.5).abs() < 1e-5);
        assert!((pt[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn closest_point_clamped_start() {
        let (t, _) = closest_point_on_segment([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]);
        assert!((t - 0.0).abs() < 1e-5);
    }

    #[test]
    fn closest_point_clamped_end() {
        let (t, _) = closest_point_on_segment([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn capsule_pair_overlap_touching() {
        let a = CapsulePrim::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        let b = CapsulePrim::new([0.9, 0.0, 0.0], [0.9, 1.0, 0.0], 0.5);
        assert!(capsule_pair_overlap(&a, &b));
    }

    #[test]
    fn capsule_pair_no_overlap_far() {
        let a = CapsulePrim::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.1);
        let b = CapsulePrim::new([5.0, 0.0, 0.0], [5.0, 1.0, 0.0], 0.1);
        assert!(!capsule_pair_overlap(&a, &b));
    }

    #[test]
    fn capsule_pair_penetration_positive_when_overlap() {
        let a = CapsulePrim::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5);
        let b = CapsulePrim::new([0.5, 0.0, 0.0], [0.5, 1.0, 0.0], 0.5);
        assert!(capsule_pair_penetration(&a, &b) > 0.0);
    }

    #[test]
    fn capsule_pair_normal_unit_length() {
        let a = CapsulePrim::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.3);
        let b = CapsulePrim::new([1.0, 0.0, 0.0], [1.0, 1.0, 0.0], 0.3);
        let n = capsule_pair_normal(&a, &b);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn parallel_capsules_sq_dist() {
        let a = CapsulePrim::new([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.1);
        let b = CapsulePrim::new([3.0, 0.0, 0.0], [3.0, 2.0, 0.0], 0.1);
        let sq = capsule_pair_sq_dist(&a, &b);
        assert!((sq - 9.0).abs() < 1e-4);
    }
}
