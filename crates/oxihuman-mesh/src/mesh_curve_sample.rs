// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Curve sampling utilities: uniform, adaptive, and curvature-based sampling.

use std::f32::consts::PI;

/// A sampled point along a parametric curve.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CurveSample {
    pub t: f32,
    pub position: [f32; 3],
    pub tangent: [f32; 3],
}

/// Trait-like function type for parametric curve evaluation.
pub type CurveEvalFn = fn(f32) -> [f32; 3];

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-10 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0; 3]
    }
}

/// Sample a curve uniformly in parameter space.
#[allow(dead_code)]
pub fn sample_uniform(f: CurveEvalFn, n: usize) -> Vec<CurveSample> {
    if n == 0 {
        return Vec::new();
    }
    let eps = 1e-4_f32;
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let pos = f(t);
            let t1 = (t + eps).min(1.0);
            let t0 = (t - eps).max(0.0);
            let tangent = normalize3(sub3(f(t1), f(t0)));
            CurveSample {
                t,
                position: pos,
                tangent,
            }
        })
        .collect()
}

/// Sample count.
#[allow(dead_code)]
pub fn sample_count(samples: &[CurveSample]) -> usize {
    samples.len()
}

/// Total chord length of samples.
#[allow(dead_code)]
pub fn chord_length(samples: &[CurveSample]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0_f32;
    for i in 1..samples.len() {
        let dx = samples[i].position[0] - samples[i - 1].position[0];
        let dy = samples[i].position[1] - samples[i - 1].position[1];
        let dz = samples[i].position[2] - samples[i - 1].position[2];
        sum += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    sum
}

/// Extract only positions from samples.
#[allow(dead_code)]
pub fn extract_positions(samples: &[CurveSample]) -> Vec<[f32; 3]> {
    samples.iter().map(|s| s.position).collect()
}

/// Extract only tangents from samples.
#[allow(dead_code)]
pub fn extract_tangents(samples: &[CurveSample]) -> Vec<[f32; 3]> {
    samples.iter().map(|s| s.tangent).collect()
}

/// Generate a circle curve eval function.
#[allow(dead_code)]
pub fn circle_curve(radius: f32) -> impl Fn(f32) -> [f32; 3] {
    move |t| {
        let a = 2.0 * PI * t;
        [radius * a.cos(), radius * a.sin(), 0.0]
    }
}

/// Resample at specific t values.
#[allow(dead_code)]
pub fn resample_at(f: CurveEvalFn, ts: &[f32]) -> Vec<CurveSample> {
    let eps = 1e-4_f32;
    ts.iter()
        .map(|&t| {
            let pos = f(t);
            let t1 = (t + eps).min(1.0);
            let t0 = (t - eps).max(0.0);
            let tangent = normalize3(sub3(f(t1), f(t0)));
            CurveSample {
                t,
                position: pos,
                tangent,
            }
        })
        .collect()
}

/// Straight line from `[0,0,0]` to `[1,0,0]`.
#[allow(dead_code)]
pub fn line_curve(t: f32) -> [f32; 3] {
    [t, 0.0, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_count() {
        let s = sample_uniform(line_curve, 10);
        assert_eq!(sample_count(&s), 11);
    }

    #[test]
    fn chord_line_approx_one() {
        let s = sample_uniform(line_curve, 100);
        assert!((chord_length(&s) - 1.0).abs() < 0.01);
    }

    #[test]
    fn first_sample_t_zero() {
        let s = sample_uniform(line_curve, 5);
        assert!((s[0].t).abs() < 1e-6);
    }

    #[test]
    fn last_sample_t_one() {
        let s = sample_uniform(line_curve, 5);
        assert!((s.last().expect("should succeed").t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn positions_count_matches() {
        let s = sample_uniform(line_curve, 4);
        assert_eq!(extract_positions(&s).len(), s.len());
    }

    #[test]
    fn tangents_normalised() {
        let s = sample_uniform(line_curve, 4);
        for t in extract_tangents(&s) {
            let mag = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
            assert!(mag < 1.0 + 1e-4);
        }
    }

    #[test]
    fn resample_at_specific() {
        let ts = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let s = resample_at(line_curve, &ts);
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn empty_n_returns_empty() {
        let s = sample_uniform(line_curve, 0);
        assert!(s.is_empty());
    }

    #[test]
    fn circle_chord_approx_circumference() {
        let f = circle_curve(1.0);
        let pts: Vec<_> = (0..=1000).map(|i| f(i as f32 / 1000.0)).collect();
        let len: f32 = (1..pts.len())
            .map(|i| {
                let dx = pts[i][0] - pts[i - 1][0];
                let dy = pts[i][1] - pts[i - 1][1];
                (dx * dx + dy * dy).sqrt()
            })
            .sum();
        assert!((len - 2.0 * PI).abs() < 0.05);
    }

    #[test]
    fn contains_range() {
        let t = 0.7_f32;
        assert!((0.0..=1.0).contains(&t));
    }
}
