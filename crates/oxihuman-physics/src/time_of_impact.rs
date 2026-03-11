// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Time of impact (TOI) calculation between swept AABBs.

#![allow(dead_code)]

/// A swept AABB with position and velocity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SweptAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub velocity: [f32; 3],
}

/// Compute the time of impact between two swept AABBs using separating axis theorem.
/// Returns None if they never intersect within [0, 1] time interval.
#[allow(dead_code)]
pub fn swept_aabb_toi(a: &SweptAabb, b: &SweptAabb) -> Option<f32> {
    // Relative velocity of b with respect to a
    let rel_vel = [
        b.velocity[0] - a.velocity[0],
        b.velocity[1] - a.velocity[1],
        b.velocity[2] - a.velocity[2],
    ];

    let mut t_enter = 0.0f32;
    let mut t_exit = 1.0f32;

    let axes = [
        (a.min[0], a.max[0], b.min[0], b.max[0], rel_vel[0]),
        (a.min[1], a.max[1], b.min[1], b.max[1], rel_vel[1]),
        (a.min[2], a.max[2], b.min[2], b.max[2], rel_vel[2]),
    ];

    for (a_min, a_max, b_min, b_max, v) in axes {
        if v.abs() < 1e-9 {
            // No relative motion on this axis
            if b_max < a_min || b_min > a_max {
                return None; // separated and not moving
            }
        } else {
            let t1 = (a_min - b_max) / v;
            let t2 = (a_max - b_min) / v;
            let (t_in, t_out) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_enter = t_enter.max(t_in);
            t_exit = t_exit.min(t_out);
            if t_enter > t_exit {
                return None;
            }
        }
    }

    if (0.0..=1.0).contains(&t_enter) {
        Some(t_enter)
    } else {
        None
    }
}

/// Return true if two static AABBs overlap.
#[allow(dead_code)]
pub fn aabbs_overlap(
    a_min: [f32; 3],
    a_max: [f32; 3],
    b_min: [f32; 3],
    b_max: [f32; 3],
) -> bool {
    for i in 0..3 {
        if a_max[i] < b_min[i] || b_max[i] < a_min[i] {
            return false;
        }
    }
    true
}

/// Return the AABB position of a swept AABB at time t.
#[allow(dead_code)]
pub fn swept_aabb_at_t(swept: &SweptAabb, t: f32) -> ([f32; 3], [f32; 3]) {
    let min = [
        swept.min[0] + swept.velocity[0] * t,
        swept.min[1] + swept.velocity[1] * t,
        swept.min[2] + swept.velocity[2] * t,
    ];
    let max = [
        swept.max[0] + swept.velocity[0] * t,
        swept.max[1] + swept.velocity[1] * t,
        swept.max[2] + swept.velocity[2] * t,
    ];
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_swept(min: [f32; 3], max: [f32; 3], vel: [f32; 3]) -> SweptAabb {
        SweptAabb { min, max, velocity: vel }
    }

    #[test]
    fn test_overlapping_static_aabbs_toi_zero() {
        let a = make_swept([0.0; 3], [1.0; 3], [0.0; 3]);
        let b = make_swept([0.5; 3], [1.5; 3], [0.0; 3]);
        // Already overlapping
        let toi = swept_aabb_toi(&a, &b);
        assert!(toi.is_some());
        assert!(toi.unwrap() <= 1.0);
    }

    #[test]
    fn test_separated_static_no_toi() {
        let a = make_swept([0.0; 3], [1.0; 3], [0.0; 3]);
        let b = make_swept([5.0; 3], [6.0; 3], [0.0; 3]);
        assert!(swept_aabb_toi(&a, &b).is_none());
    }

    #[test]
    fn test_moving_towards_each_other() {
        let a = make_swept([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0; 3]);
        let b = make_swept([3.0, 0.0, 0.0], [4.0, 1.0, 1.0], [-2.0, 0.0, 0.0]);
        let toi = swept_aabb_toi(&a, &b);
        assert!(toi.is_some());
    }

    #[test]
    fn test_moving_away_no_toi() {
        let a = make_swept([0.0; 3], [1.0; 3], [0.0; 3]);
        let b = make_swept([5.0; 3], [6.0; 3], [10.0, 0.0, 0.0]);
        assert!(swept_aabb_toi(&a, &b).is_none());
    }

    #[test]
    fn test_aabbs_overlap_true() {
        assert!(aabbs_overlap(
            [0.0; 3], [2.0; 3],
            [1.0; 3], [3.0; 3]
        ));
    }

    #[test]
    fn test_aabbs_overlap_false() {
        assert!(!aabbs_overlap(
            [0.0; 3], [1.0; 3],
            [2.0; 3], [3.0; 3]
        ));
    }

    #[test]
    fn test_swept_aabb_at_t_zero() {
        let s = make_swept([0.0; 3], [1.0; 3], [1.0; 3]);
        let (mn, mx) = swept_aabb_at_t(&s, 0.0);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [1.0; 3]);
    }

    #[test]
    fn test_swept_aabb_at_t_one() {
        let s = make_swept([0.0; 3], [1.0; 3], [2.0, 0.0, 0.0]);
        let (mn, _mx) = swept_aabb_at_t(&s, 1.0);
        assert!((mn[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_toi_value_in_range() {
        let a = make_swept([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0; 3]);
        let b = make_swept([2.0, 0.0, 0.0], [3.0, 1.0, 1.0], [-3.0, 0.0, 0.0]);
        if let Some(t) = swept_aabb_toi(&a, &b) {
            assert!((0.0..=1.0).contains(&t));
        }
    }
}
