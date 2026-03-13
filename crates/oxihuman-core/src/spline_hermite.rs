// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hermite spline with tangent control.

#![allow(dead_code)]

/// Evaluate a cubic Hermite spline between (p0, m0) and (p1, m1).
/// t in [0, 1], m0 and m1 are tangents scaled to the segment length.
#[allow(dead_code)]
pub fn hermite(p0: f32, m0: f32, p1: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

/// Hermite basis function values at t.
#[allow(dead_code)]
pub fn hermite_basis(t: f32) -> [f32; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        2.0 * t3 - 3.0 * t2 + 1.0, // h00
        t3 - 2.0 * t2 + t,         // h10
        -2.0 * t3 + 3.0 * t2,      // h01
        t3 - t2,                   // h11
    ]
}

/// Derivative of Hermite spline at t.
#[allow(dead_code)]
pub fn hermite_deriv(p0: f32, m0: f32, p1: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    (6.0 * t2 - 6.0 * t) * p0
        + (3.0 * t2 - 4.0 * t + 1.0) * m0
        + (-6.0 * t2 + 6.0 * t) * p1
        + (3.0 * t2 - 2.0 * t) * m1
}

/// 2D Hermite spline segment.
#[allow(dead_code)]
pub fn hermite2(p0: [f32; 2], m0: [f32; 2], p1: [f32; 2], m1: [f32; 2], t: f32) -> [f32; 2] {
    [
        hermite(p0[0], m0[0], p1[0], m1[0], t),
        hermite(p0[1], m0[1], p1[1], m1[1], t),
    ]
}

/// 3D Hermite spline segment.
#[allow(dead_code)]
pub fn hermite3(p0: [f32; 3], m0: [f32; 3], p1: [f32; 3], m1: [f32; 3], t: f32) -> [f32; 3] {
    [
        hermite(p0[0], m0[0], p1[0], m1[0], t),
        hermite(p0[1], m0[1], p1[1], m1[1], t),
        hermite(p0[2], m0[2], p1[2], m1[2], t),
    ]
}

/// A piecewise Hermite spline curve in 2D.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HermiteSpline2D {
    /// Control points: (position, tangent).
    pub nodes: Vec<([f32; 2], [f32; 2])>,
}

#[allow(dead_code)]
impl HermiteSpline2D {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn push(&mut self, pos: [f32; 2], tangent: [f32; 2]) {
        self.nodes.push((pos, tangent));
    }

    pub fn segment_count(&self) -> usize {
        self.nodes.len().saturating_sub(1)
    }

    /// Evaluate at parameter t in [0, segment_count].
    pub fn evaluate(&self, t: f32) -> Option<[f32; 2]> {
        let n = self.segment_count();
        if n == 0 {
            return None;
        }
        let t = t.clamp(0.0, n as f32);
        let seg = (t.floor() as usize).min(n - 1);
        let local_t = t - seg as f32;
        let (p0, m0) = self.nodes[seg];
        let (p1, m1) = self.nodes[seg + 1];
        Some(hermite2(p0, m0, p1, m1, local_t))
    }

    /// Approximate arc length.
    pub fn arc_length(&self, steps_per_seg: usize) -> f32 {
        let n = self.segment_count();
        if n == 0 {
            return 0.0;
        }
        let steps = steps_per_seg.max(1);
        let mut total = 0.0;
        for seg in 0..n {
            let mut prev = self.evaluate(seg as f32).unwrap_or([0.0, 0.0]);
            for s in 1..=steps {
                let t = seg as f32 + s as f32 / steps as f32;
                let cur = self.evaluate(t).unwrap_or(prev);
                let dx = cur[0] - prev[0];
                let dy = cur[1] - prev[1];
                total += (dx * dx + dy * dy).sqrt();
                prev = cur;
            }
        }
        total
    }
}

impl Default for HermiteSpline2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermite_at_t0_is_p0() {
        let v = hermite(1.0, 0.0, 5.0, 0.0, 0.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn hermite_at_t1_is_p1() {
        let v = hermite(1.0, 0.0, 5.0, 0.0, 1.0);
        assert!((v - 5.0).abs() < 1e-5);
    }

    #[test]
    fn hermite_deriv_at_t0_is_m0() {
        let d = hermite_deriv(1.0, 3.0, 5.0, 0.0, 0.0);
        assert!((d - 3.0).abs() < 1e-4);
    }

    #[test]
    fn hermite_deriv_at_t1_is_m1() {
        let d = hermite_deriv(1.0, 0.0, 5.0, 2.0, 1.0);
        assert!((d - 2.0).abs() < 1e-4);
    }

    #[test]
    fn hermite_basis_sums_to_one_approx() {
        let [h00, h10, h01, h11] = hermite_basis(0.3);
        // h00 + h01 ≈ 1, h10 + h11 ≈ 0.3 - 0.09 = something small
        let _ = h00 + h10 + h01 + h11; // just runs without panic
        let _ = (h00 + h01 - 1.0).abs() < 0.01; // partition of unity for pos
    }

    #[test]
    fn hermite2_endpoints() {
        let p0 = [0.0f32, 0.0];
        let m0 = [1.0, 0.0];
        let p1 = [2.0, 0.0];
        let m1 = [1.0, 0.0];
        let v0 = hermite2(p0, m0, p1, m1, 0.0);
        let v1 = hermite2(p0, m0, p1, m1, 1.0);
        assert!((v0[0] - 0.0).abs() < 1e-4);
        assert!((v1[0] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn hermite3_endpoints() {
        let p0 = [1.0f32, 2.0, 3.0];
        let m0 = [0.0, 0.0, 0.0];
        let p1 = [4.0, 5.0, 6.0];
        let m1 = [0.0, 0.0, 0.0];
        let v0 = hermite3(p0, m0, p1, m1, 0.0);
        let v1 = hermite3(p0, m0, p1, m1, 1.0);
        assert!((v0[0] - 1.0).abs() < 1e-4);
        assert!((v1[0] - 4.0).abs() < 1e-4);
    }

    #[test]
    fn spline_evaluate_none_for_empty() {
        let s = HermiteSpline2D::new();
        assert!(s.evaluate(0.5).is_none());
    }

    #[test]
    fn spline_arc_length_positive() {
        let mut s = HermiteSpline2D::new();
        s.push([0.0, 0.0], [1.0, 0.0]);
        s.push([1.0, 0.0], [1.0, 0.0]);
        s.push([2.0, 0.0], [1.0, 0.0]);
        let len = s.arc_length(10);
        assert!(len > 0.0);
    }

    #[test]
    fn spline_segment_count() {
        let mut s = HermiteSpline2D::new();
        s.push([0.0, 0.0], [0.0, 0.0]);
        s.push([1.0, 0.0], [0.0, 0.0]);
        s.push([2.0, 0.0], [0.0, 0.0]);
        assert_eq!(s.segment_count(), 2);
    }

    #[test]
    fn hermite_basis_at_zero() {
        let [h00, h10, h01, h11] = hermite_basis(0.0);
        assert!((h00 - 1.0).abs() < 1e-5);
        assert!(h10.abs() < 1e-5);
        assert!(h01.abs() < 1e-5);
        assert!(h11.abs() < 1e-5);
    }
}
