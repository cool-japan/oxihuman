// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fiber/filament body with bending and torsion (v2, distinct from fiber_body.rs).

#![allow(dead_code)]

/// A single segment of a fiber.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FiberSegment {
    /// Start position.
    pub p0: [f32; 3],
    /// End position.
    pub p1: [f32; 3],
    /// Linear velocity of end point.
    pub velocity: [f32; 3],
    /// Mass of segment.
    pub mass: f32,
    /// Bending stiffness.
    pub bend_stiffness: f32,
    /// Torsion stiffness.
    pub torsion_stiffness: f32,
    /// Rest length.
    pub rest_length: f32,
    /// Stretch stiffness.
    pub stretch_stiffness: f32,
}

/// A fiber body: chain of segments.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FiberBodyV2 {
    pub segments: Vec<FiberSegment>,
    /// Root (pinned) position.
    pub root: [f32; 3],
    pub is_root_pinned: bool,
}

/// Create a straight fiber with `n` segments along the y axis.
#[allow(dead_code)]
pub fn new_fiber_body_v2(
    n: usize,
    length: f32,
    mass_per_seg: f32,
    bend_stiffness: f32,
    torsion_stiffness: f32,
    stretch_stiffness: f32,
) -> FiberBodyV2 {
    let seg_len = length / n.max(1) as f32;
    let mut segments = Vec::with_capacity(n);
    for i in 0..n {
        let y0 = i as f32 * seg_len;
        let y1 = (i + 1) as f32 * seg_len;
        segments.push(FiberSegment {
            p0: [0.0, y0, 0.0],
            p1: [0.0, y1, 0.0],
            velocity: [0.0; 3],
            mass: mass_per_seg,
            bend_stiffness,
            torsion_stiffness,
            rest_length: seg_len,
            stretch_stiffness,
        });
    }
    FiberBodyV2 {
        segments,
        root: [0.0; 3],
        is_root_pinned: true,
    }
}

/// Segment count.
#[allow(dead_code)]
pub fn fiber_segment_count(b: &FiberBodyV2) -> usize {
    b.segments.len()
}

/// Compute length of a segment.
#[allow(dead_code)]
pub fn fiber_segment_length(seg: &FiberSegment) -> f32 {
    let d = [
        seg.p1[0] - seg.p0[0],
        seg.p1[1] - seg.p0[1],
        seg.p1[2] - seg.p0[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Compute stretch force along a segment.
#[allow(dead_code)]
pub fn fiber_stretch_force(seg: &FiberSegment) -> [f32; 3] {
    let len = fiber_segment_length(seg);
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    let stretch = len - seg.rest_length;
    let d = [
        seg.p1[0] - seg.p0[0],
        seg.p1[1] - seg.p0[1],
        seg.p1[2] - seg.p0[2],
    ];
    let n = [d[0] / len, d[1] / len, d[2] / len];
    let f = seg.stretch_stiffness * stretch;
    [f * n[0], f * n[1], f * n[2]]
}

/// Compute bending angle between two consecutive segments.
#[allow(dead_code)]
pub fn fiber_bending_angle(s0: &FiberSegment, s1: &FiberSegment) -> f32 {
    let d0 = [
        s0.p1[0] - s0.p0[0],
        s0.p1[1] - s0.p0[1],
        s0.p1[2] - s0.p0[2],
    ];
    let d1 = [
        s1.p1[0] - s1.p0[0],
        s1.p1[1] - s1.p0[1],
        s1.p1[2] - s1.p0[2],
    ];
    let len0 = (d0[0] * d0[0] + d0[1] * d0[1] + d0[2] * d0[2]).sqrt();
    let len1 = (d1[0] * d1[0] + d1[1] * d1[1] + d1[2] * d1[2]).sqrt();
    if len0 < f32::EPSILON || len1 < f32::EPSILON {
        return 0.0;
    }
    let dot = (d0[0] * d1[0] + d0[1] * d1[1] + d0[2] * d1[2]) / (len0 * len1);
    dot.clamp(-1.0, 1.0).acos()
}

/// Step: apply gravity to each segment end point.
#[allow(dead_code)]
pub fn fiber_step(b: &mut FiberBodyV2, gravity: [f32; 3], dt: f32) {
    for (i, seg) in b.segments.iter_mut().enumerate() {
        if i == 0 && b.is_root_pinned {
            seg.p0 = b.root;
        }
        // Apply gravity to velocity
        seg.velocity[0] += gravity[0] * dt;
        seg.velocity[1] += gravity[1] * dt;
        seg.velocity[2] += gravity[2] * dt;
        // Integrate end point
        seg.p1[0] += seg.velocity[0] * dt;
        seg.p1[1] += seg.velocity[1] * dt;
        seg.p1[2] += seg.velocity[2] * dt;
    }
    // Re-link segments (start of next = end of previous)
    for i in 1..b.segments.len() {
        let prev_end = b.segments[i - 1].p1;
        b.segments[i].p0 = prev_end;
    }
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn fiber_kinetic_energy(b: &FiberBodyV2) -> f32 {
    b.segments
        .iter()
        .map(|s| {
            let v2 = s.velocity[0] * s.velocity[0]
                + s.velocity[1] * s.velocity[1]
                + s.velocity[2] * s.velocity[2];
            0.5 * s.mass * v2
        })
        .sum()
}

/// Tip position (end of last segment).
#[allow(dead_code)]
pub fn fiber_tip(b: &FiberBodyV2) -> Option<[f32; 3]> {
    b.segments.last().map(|s| s.p1)
}

/// Total rest length.
#[allow(dead_code)]
pub fn fiber_rest_length(b: &FiberBodyV2) -> f32 {
    b.segments.iter().map(|s| s.rest_length).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_fiber() -> FiberBodyV2 {
        new_fiber_body_v2(5, 1.0, 0.1, 10.0, 5.0, 50.0)
    }

    #[test]
    fn segment_count_correct() {
        let b = default_fiber();
        assert_eq!(fiber_segment_count(&b), 5);
    }

    #[test]
    fn rest_length_correct() {
        let b = default_fiber();
        assert!((fiber_rest_length(&b) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn tip_exists() {
        let b = default_fiber();
        assert!(fiber_tip(&b).is_some());
    }

    #[test]
    fn segment_length_at_rest() {
        let b = default_fiber();
        for s in &b.segments {
            let l = fiber_segment_length(s);
            assert!((l - 0.2).abs() < 1e-4);
        }
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let b = default_fiber();
        assert_eq!(fiber_kinetic_energy(&b), 0.0);
    }

    #[test]
    fn step_gravity_moves_tip_down() {
        let mut b = default_fiber();
        let before = fiber_tip(&b).unwrap()[1];
        fiber_step(&mut b, [0.0, -9.81, 0.0], 0.1);
        let after = fiber_tip(&b).unwrap()[1];
        assert!(after < before);
    }

    #[test]
    fn root_pinned_stays_fixed() {
        let mut b = default_fiber();
        fiber_step(&mut b, [0.0, -9.81, 0.0], 0.1);
        assert_eq!(b.segments[0].p0, b.root);
    }

    #[test]
    fn stretch_force_zero_at_rest() {
        let b = default_fiber();
        let f = fiber_stretch_force(&b.segments[0]);
        // At rest length, stretch should be near zero
        let mag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        assert!(mag < 1e-3);
    }

    #[test]
    fn bending_angle_straight() {
        let b = default_fiber();
        if b.segments.len() >= 2 {
            let angle = fiber_bending_angle(&b.segments[0], &b.segments[1]);
            // Straight fiber: angle should be 0
            assert!(angle < 0.01);
        }
    }

    #[test]
    fn kinetic_energy_increases_with_gravity() {
        let mut b = default_fiber();
        fiber_step(&mut b, [0.0, -9.81, 0.0], 0.1);
        assert!(fiber_kinetic_energy(&b) > 0.0);
    }
}
