// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! NURBS curve evaluation and sampling.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NurbsConfig {
    pub degree: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NurbsCurve {
    pub control_points: Vec<[f32; 3]>,
    pub knots: Vec<f32>,
    pub weights: Vec<f32>,
    pub config: NurbsConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NurbsSample {
    pub position: [f32; 3],
    pub t: f32,
}

#[allow(dead_code)]
pub fn default_nurbs_config() -> NurbsConfig {
    NurbsConfig { degree: 3 }
}

/// Build a uniform clamped knot vector for `n` control points and degree `p`.
#[allow(clippy::needless_range_loop)]
fn uniform_clamped_knots(n: usize, p: usize) -> Vec<f32> {
    let m = n + p + 1;
    let mut knots = vec![0.0f32; m];
    for i in 0..m {
        if i < p + 1 {
            knots[i] = 0.0;
        } else if i >= m - p - 1 {
            knots[i] = 1.0;
        } else {
            knots[i] = (i - p) as f32 / (m - 2 * p - 1) as f32;
        }
    }
    knots
}

#[allow(dead_code)]
pub fn new_nurbs_curve(control_points: Vec<[f32; 3]>, degree: usize) -> NurbsCurve {
    let n = control_points.len();
    let knots = uniform_clamped_knots(n, degree);
    let weights = vec![1.0f32; n];
    NurbsCurve {
        control_points,
        knots,
        weights,
        config: NurbsConfig { degree },
    }
}

/// Cox-de Boor basis function N_{i,p}(t).
fn basis(i: usize, p: usize, t: f32, knots: &[f32]) -> f32 {
    if p == 0 {
        let lo = knots[i];
        let hi = knots[i + 1];
        if lo <= t && t < hi { 1.0 } else { 0.0 }
    } else {
        let d1 = knots[i + p] - knots[i];
        let d2 = knots[i + p + 1] - knots[i + 1];
        let left = if d1.abs() < 1e-9 {
            0.0
        } else {
            (t - knots[i]) / d1 * basis(i, p - 1, t, knots)
        };
        let right = if d2.abs() < 1e-9 {
            0.0
        } else {
            (knots[i + p + 1] - t) / d2 * basis(i + 1, p - 1, t, knots)
        };
        left + right
    }
}

#[allow(dead_code)]
pub fn nurbs_evaluate(curve: &NurbsCurve, t: f32) -> [f32; 3] {
    // Clamp t to [0, 1 - eps] to avoid boundary issues.
    let t = t.clamp(0.0, 1.0 - 1e-7);
    let n = curve.control_points.len();
    let p = curve.config.degree;
    let mut num = [0.0f32; 3];
    let mut denom = 0.0f32;
    for i in 0..n {
        let b = basis(i, p, t, &curve.knots) * curve.weights[i];
        let cp = curve.control_points[i];
        num[0] += b * cp[0];
        num[1] += b * cp[1];
        num[2] += b * cp[2];
        denom += b;
    }
    if denom.abs() < 1e-12 {
        return curve.control_points[0];
    }
    [num[0] / denom, num[1] / denom, num[2] / denom]
}

#[allow(dead_code)]
pub fn nurbs_sample(curve: &NurbsCurve, n_samples: usize) -> Vec<NurbsSample> {
    let n = n_samples.max(2);
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            NurbsSample {
                position: nurbs_evaluate(curve, t),
                t,
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn nurbs_control_count(curve: &NurbsCurve) -> usize {
    curve.control_points.len()
}

#[allow(dead_code)]
pub fn nurbs_arc_length_approx(curve: &NurbsCurve, n_samples: usize) -> f32 {
    let samples = nurbs_sample(curve, n_samples);
    let mut len = 0.0f32;
    for w in samples.windows(2) {
        let a = w[0].position;
        let b = w[1].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        len += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    len
}

#[allow(dead_code)]
pub fn nurbs_to_json(curve: &NurbsCurve) -> String {
    format!(
        "{{\"degree\":{},\"control_points\":{},\"knots\":{}}}",
        curve.config.degree,
        curve.control_points.len(),
        curve.knots.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_curve() -> NurbsCurve {
        new_nurbs_curve(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
            3,
        )
    }

    #[test]
    fn test_default_config() {
        let cfg = default_nurbs_config();
        assert_eq!(cfg.degree, 3);
    }

    #[test]
    fn test_new_curve_control_count() {
        let c = line_curve();
        assert_eq!(nurbs_control_count(&c), 4);
    }

    #[test]
    fn test_knot_vector_length() {
        let c = line_curve();
        // m = n + p + 1 = 4 + 3 + 1 = 8
        assert_eq!(c.knots.len(), 8);
    }

    #[test]
    fn test_evaluate_start() {
        let c = line_curve();
        let p = nurbs_evaluate(&c, 0.0);
        assert!((p[0] - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_end() {
        let c = line_curve();
        let p = nurbs_evaluate(&c, 1.0 - 1e-6);
        // Near endpoint should approach last control point
        assert!(p[0] > 2.0);
    }

    #[test]
    fn test_sample_count() {
        let c = line_curve();
        let samples = nurbs_sample(&c, 10);
        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_arc_length_positive() {
        let c = line_curve();
        let len = nurbs_arc_length_approx(&c, 20);
        assert!(len > 0.0);
    }

    #[test]
    fn test_to_json() {
        let c = line_curve();
        let j = nurbs_to_json(&c);
        assert!(j.contains("degree"));
    }
}
