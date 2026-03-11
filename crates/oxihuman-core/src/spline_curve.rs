// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Catmull-Rom spline utilities.

/// Evaluate a Catmull-Rom spline segment between p1 and p2 at parameter t ∈ [0, 1].
/// p0 and p3 are the neighbouring control points.
pub fn catmull_rom_f32(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Evaluate a 2D Catmull-Rom spline at parameter t.
pub fn catmull_rom_2d(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    [
        catmull_rom_f32(p0[0], p1[0], p2[0], p3[0], t),
        catmull_rom_f32(p0[1], p1[1], p2[1], p3[1], t),
    ]
}

/// Evaluate a 3D Catmull-Rom spline at parameter t.
pub fn catmull_rom_3d(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    [
        catmull_rom_f32(p0[0], p1[0], p2[0], p3[0], t),
        catmull_rom_f32(p0[1], p1[1], p2[1], p3[1], t),
        catmull_rom_f32(p0[2], p1[2], p2[2], p3[2], t),
    ]
}

/// Sample a chain of Catmull-Rom segments at `steps_per_segment` points each.
/// `points` must have at least 4 elements. Returns sampled 2D points.
pub fn catmull_rom_chain_2d(points: &[[f32; 2]], steps_per_segment: usize) -> Vec<[f32; 2]> {
    let n = points.len();
    if n < 4 {
        return vec![];
    }
    let mut result = Vec::new();
    for seg in 0..(n - 3) {
        let p0 = points[seg];
        let p1 = points[seg + 1];
        let p2 = points[seg + 2];
        let p3 = points[seg + 3];
        for step in 0..steps_per_segment {
            let t = step as f32 / steps_per_segment as f32;
            result.push(catmull_rom_2d(p0, p1, p2, p3, t));
        }
    }
    result
}

/// Derivative of a Catmull-Rom segment at t.
pub fn catmull_rom_tangent_f32(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    0.5 * ((-p0 + p2)
        + 2.0 * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t
        + 3.0 * (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_at_t0() {
        /* at t=0 value equals p1 */
        let v = catmull_rom_f32(0.0, 1.0, 2.0, 3.0, 0.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_catmull_rom_at_t1() {
        /* at t=1 value equals p2 */
        let v = catmull_rom_f32(0.0, 1.0, 2.0, 3.0, 1.0);
        assert!((v - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_catmull_rom_2d_endpoints() {
        /* 2D: t=0 -> p1, t=1 -> p2 */
        let p = catmull_rom_2d([0.0, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, 1.0], 0.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
        assert!((p[1] - 1.0).abs() < 1e-5);
        let p = catmull_rom_2d([0.0, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, 1.0], 1.0);
        assert!((p[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_catmull_rom_3d_endpoints() {
        /* 3D: t=0 -> p1 */
        let p = catmull_rom_3d(
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0],
            0.0,
        );
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_chain_empty_too_short() {
        /* fewer than 4 points -> empty */
        let pts = vec![[0.0f32, 0.0], [1.0, 1.0]];
        assert!(catmull_rom_chain_2d(&pts, 5).is_empty());
    }

    #[test]
    fn test_chain_point_count() {
        /* 4 points, 10 steps -> 10 points from 1 segment */
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
        let r = catmull_rom_chain_2d(&pts, 10);
        assert_eq!(r.len(), 10);
    }

    #[test]
    fn test_tangent_at_zero() {
        /* tangent at t=0 for linear control = positive slope */
        let tan = catmull_rom_tangent_f32(0.0, 1.0, 2.0, 3.0, 0.0);
        assert!(tan > 0.0);
    }
}
