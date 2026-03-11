// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Catmull-Rom spline evaluation (uniform parameterization).

#![allow(dead_code)]

/// Evaluate a Catmull-Rom spline segment between p1 and p2,
/// given control points p0 and p3. t in [0, 1].
#[allow(dead_code)]
pub fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Evaluate Catmull-Rom for 2D points.
#[allow(dead_code)]
pub fn catmull_rom2(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    [
        catmull_rom(p0[0], p1[0], p2[0], p3[0], t),
        catmull_rom(p0[1], p1[1], p2[1], p3[1], t),
    ]
}

/// Evaluate Catmull-Rom for 3D points.
#[allow(dead_code)]
pub fn catmull_rom3(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    [
        catmull_rom(p0[0], p1[0], p2[0], p3[0], t),
        catmull_rom(p0[1], p1[1], p2[1], p3[1], t),
        catmull_rom(p0[2], p1[2], p2[2], p3[2], t),
    ]
}

/// A Catmull-Rom spline through a set of 2D control points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CatmullRomSpline2D {
    pub points: Vec<[f32; 2]>,
    pub looped: bool,
}

#[allow(dead_code)]
impl CatmullRomSpline2D {
    pub fn new(points: Vec<[f32; 2]>, looped: bool) -> Self {
        Self { points, looped }
    }

    /// Number of segments (requires at least 4 points for one segment).
    pub fn segment_count(&self) -> usize {
        if self.points.len() < 4 {
            return 0;
        }
        self.points.len() - 3
    }

    /// Evaluate at global parameter t in [0, segment_count].
    pub fn evaluate(&self, t: f32) -> Option<[f32; 2]> {
        let n = self.segment_count();
        if n == 0 {
            return None;
        }
        let t = t.clamp(0.0, n as f32);
        let seg = (t.floor() as usize).min(n - 1);
        let local_t = t - seg as f32;
        let i0 = seg;
        let i1 = seg + 1;
        let i2 = seg + 2;
        let i3 = seg + 3;
        Some(catmull_rom2(
            self.points[i0],
            self.points[i1],
            self.points[i2],
            self.points[i3],
            local_t,
        ))
    }

    /// Approximate arc length by sampling.
    pub fn arc_length(&self, samples_per_segment: usize) -> f32 {
        let n = self.segment_count();
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        let steps = samples_per_segment.max(1);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_rom_at_t0_equals_p1() {
        let v = catmull_rom(0.0, 1.0, 2.0, 3.0, 0.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn catmull_rom_at_t1_equals_p2() {
        let v = catmull_rom(0.0, 1.0, 2.0, 3.0, 1.0);
        assert!((v - 2.0).abs() < 1e-5);
    }

    #[test]
    fn catmull_rom2_endpoints() {
        let p0 = [0.0f32, 0.0];
        let p1 = [1.0, 0.0];
        let p2 = [2.0, 0.0];
        let p3 = [3.0, 0.0];
        let v0 = catmull_rom2(p0, p1, p2, p3, 0.0);
        let v1 = catmull_rom2(p0, p1, p2, p3, 1.0);
        assert!((v0[0] - 1.0).abs() < 1e-5);
        assert!((v1[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn catmull_rom3_linear_case() {
        // For collinear points, midpoint should be at mid value
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [2.0, 0.0, 0.0];
        let p3 = [3.0, 0.0, 0.0];
        let mid = catmull_rom3(p0, p1, p2, p3, 0.5);
        assert!((mid[0] - 1.5).abs() < 1e-4);
    }

    #[test]
    fn spline_segment_count() {
        let pts = vec![
            [0.0f32, 0.0],
            [1.0, 0.0],
            [2.0, 0.0],
            [3.0, 0.0],
            [4.0, 0.0],
        ];
        let s = CatmullRomSpline2D::new(pts, false);
        assert_eq!(s.segment_count(), 2);
    }

    #[test]
    fn spline_evaluate_start_near_p1() {
        let pts = vec![[0.0f32, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, 1.0]];
        let s = CatmullRomSpline2D::new(pts, false);
        let v = s.evaluate(0.0).unwrap();
        assert!((v[0] - 1.0).abs() < 1e-4 && (v[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn spline_evaluate_none_when_few_points() {
        let s = CatmullRomSpline2D::new(vec![[0.0, 0.0], [1.0, 1.0]], false);
        assert!(s.evaluate(0.5).is_none());
    }

    #[test]
    fn spline_arc_length_positive() {
        let pts = vec![
            [0.0f32, 0.0],
            [1.0, 0.0],
            [2.0, 0.0],
            [3.0, 0.0],
            [4.0, 0.0],
        ];
        let s = CatmullRomSpline2D::new(pts, false);
        let len = s.arc_length(10);
        assert!(len > 0.0);
    }

    #[test]
    fn spline_arc_length_zero_for_few_points() {
        let s = CatmullRomSpline2D::new(vec![[0.0, 0.0]], false);
        assert!((s.arc_length(10) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn catmull_rom_collinear_midpoint() {
        // Collinear control points: result at t=0.5 should be midpoint of p1..p2
        let v = catmull_rom(0.0, 1.0, 2.0, 3.0, 0.5);
        assert!((v - 1.5).abs() < 1e-4);
    }

    #[test]
    fn catmull_rom3_at_t0_t1() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [0.0, 1.0, 0.0];
        let p2 = [0.0, 2.0, 0.0];
        let p3 = [0.0, 3.0, 0.0];
        let v0 = catmull_rom3(p0, p1, p2, p3, 0.0);
        let v1 = catmull_rom3(p0, p1, p2, p3, 1.0);
        assert!((v0[1] - 1.0).abs() < 1e-4);
        assert!((v1[1] - 2.0).abs() < 1e-4);
    }
}
