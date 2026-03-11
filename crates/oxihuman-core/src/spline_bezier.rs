// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cubic Bezier spline math.

#![allow(dead_code)]

/// Evaluates a cubic Bezier curve at parameter `t` in [0, 1].
#[allow(dead_code)]
pub fn bezier_eval(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    t: f32,
) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    let t2 = t * t;
    let t3 = t2 * t;
    [
        u3 * p0[0] + 3.0 * u2 * t * p1[0] + 3.0 * u * t2 * p2[0] + t3 * p3[0],
        u3 * p0[1] + 3.0 * u2 * t * p1[1] + 3.0 * u * t2 * p2[1] + t3 * p3[1],
    ]
}

/// Returns the tangent (derivative) of a cubic Bezier curve at parameter `t`.
#[allow(dead_code)]
pub fn bezier_tangent(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    t: f32,
) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    // B'(t) = 3[(p1-p0)(1-t)^2 + 2(p2-p1)(1-t)t + (p3-p2)t^2]
    let c0 = 3.0 * u * u;
    let c1 = 6.0 * u * t;
    let c2 = 3.0 * t * t;
    [
        c0 * (p1[0] - p0[0]) + c1 * (p2[0] - p1[0]) + c2 * (p3[0] - p2[0]),
        c0 * (p1[1] - p0[1]) + c1 * (p2[1] - p1[1]) + c2 * (p3[1] - p2[1]),
    ]
}

/// Approximates the arc length of a cubic Bezier curve using `steps` samples.
#[allow(dead_code)]
pub fn bezier_arc_length(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    steps: u32,
) -> f32 {
    if steps == 0 {
        return 0.0;
    }
    let n = steps as f32;
    let mut length = 0.0;
    let mut prev = bezier_eval(p0, p1, p2, p3, 0.0);
    for i in 1..=steps {
        let t = i as f32 / n;
        let curr = bezier_eval(p0, p1, p2, p3, t);
        let dx = curr[0] - prev[0];
        let dy = curr[1] - prev[1];
        length += (dx * dx + dy * dy).sqrt();
        prev = curr;
    }
    length
}

/// Splits a cubic Bezier at parameter `t` using de Casteljau's algorithm.
/// Returns the 7 control points: [p0, q1, q2, mid, r1, r2, p3].
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn bezier_split(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    t: f32,
) -> ([f32; 2], [f32; 2], [f32; 2], [f32; 2], [f32; 2], [f32; 2], [f32; 2]) {
    let t = t.clamp(0.0, 1.0);
    let lerp2 = |a: [f32; 2], b: [f32; 2], t: f32| -> [f32; 2] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
    };
    let q0 = lerp2(p0, p1, t);
    let q1 = lerp2(p1, p2, t);
    let q2 = lerp2(p2, p3, t);
    let r0 = lerp2(q0, q1, t);
    let r1 = lerp2(q1, q2, t);
    let mid = lerp2(r0, r1, t);
    (p0, q0, r0, mid, r1, q2, p3)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    #[test]
    fn test_eval_t0() {
        let p = bezier_eval([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 0.0);
        assert!((p[0]).abs() < EPS);
        assert!((p[1]).abs() < EPS);
    }

    #[test]
    fn test_eval_t1() {
        let p = bezier_eval([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 1.0);
        assert!((p[0] - 3.0).abs() < EPS);
    }

    #[test]
    fn test_eval_midpoint_linear() {
        // For a "linear" bezier (collinear control points), midpoint should be halfway
        let p = bezier_eval([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 0.5);
        assert!((p[0] - 1.5).abs() < EPS);
    }

    #[test]
    fn test_tangent_t0() {
        let t = bezier_tangent([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 0.0);
        // tangent at t=0 is 3*(p1-p0)
        assert!((t[0] - 3.0).abs() < EPS);
    }

    #[test]
    fn test_tangent_t1() {
        let t = bezier_tangent([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 1.0);
        // tangent at t=1 is 3*(p3-p2)
        assert!((t[0] - 3.0).abs() < EPS);
    }

    #[test]
    fn test_arc_length_zero_steps() {
        let l = bezier_arc_length([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 0);
        assert!(l.abs() < EPS);
    }

    #[test]
    fn test_arc_length_linear_approx() {
        let l = bezier_arc_length([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 100);
        // A straight-line bezier has arc length = 3.0
        assert!((l - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_split_midpoint() {
        let (p0, _, _, mid, _, _, p3) =
            bezier_split([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 0.5);
        assert!((p0[0]).abs() < EPS);
        assert!((p3[0] - 3.0).abs() < EPS);
        assert!((mid[0] - 1.5).abs() < EPS);
    }

    #[test]
    fn test_split_t0() {
        let (p0s, _, _, mid, _, _, _) =
            bezier_split([0.0, 0.0], [1.0, 1.0], [2.0, 1.0], [3.0, 0.0], 0.0);
        assert!((p0s[0]).abs() < EPS);
        assert!((mid[0]).abs() < EPS);
    }
}
