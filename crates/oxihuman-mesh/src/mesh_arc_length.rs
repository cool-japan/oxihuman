// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Arc-length parameterisation utilities for polyline curves.

use std::f32::consts::PI;

/// A parameterised curve sample.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArcSample {
    pub t: f32,
    pub arc_length: f32,
    pub position: [f32; 3],
}

/// Result of arc-length computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArcLengthResult {
    pub samples: Vec<ArcSample>,
    pub total_length: f32,
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute cumulative arc-length for a polyline.
#[allow(dead_code)]
pub fn compute_arc_length(points: &[[f32; 3]]) -> ArcLengthResult {
    if points.is_empty() {
        return ArcLengthResult {
            samples: Vec::new(),
            total_length: 0.0,
        };
    }
    let mut samples = Vec::with_capacity(points.len());
    let mut acc = 0.0_f32;
    samples.push(ArcSample {
        t: 0.0,
        arc_length: 0.0,
        position: points[0],
    });
    for i in 1..points.len() {
        acc += dist3(points[i - 1], points[i]);
        samples.push(ArcSample {
            t: 0.0,
            arc_length: acc,
            position: points[i],
        });
    }
    let total = acc;
    if total > 0.0 {
        for s in &mut samples {
            s.t = s.arc_length / total;
        }
    }
    ArcLengthResult {
        samples,
        total_length: total,
    }
}

/// Reparameterise curve to uniform arc-length spacing.
#[allow(dead_code)]
pub fn uniform_resample(points: &[[f32; 3]], n: usize) -> Vec<[f32; 3]> {
    if points.len() < 2 || n == 0 {
        return Vec::new();
    }
    let res = compute_arc_length(points);
    let total = res.total_length;
    if total == 0.0 {
        return vec![points[0]; n];
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let target = total * (i as f32) / ((n - 1).max(1) as f32);
        let pos = sample_at_arc_length(&res, target);
        out.push(pos);
    }
    out
}

/// Sample position at a given arc-length along a curve.
#[allow(dead_code)]
pub fn sample_at_arc_length(res: &ArcLengthResult, s: f32) -> [f32; 3] {
    let samples = &res.samples;
    if samples.is_empty() {
        return [0.0; 3];
    }
    if s <= 0.0 {
        return samples[0].position;
    }
    if s >= res.total_length {
        return samples[samples.len() - 1].position;
    }
    for i in 1..samples.len() {
        if samples[i].arc_length >= s {
            let prev = &samples[i - 1];
            let cur = &samples[i];
            let span = cur.arc_length - prev.arc_length;
            let alpha = if span > 0.0 {
                (s - prev.arc_length) / span
            } else {
                0.0
            };
            return [
                prev.position[0] + alpha * (cur.position[0] - prev.position[0]),
                prev.position[1] + alpha * (cur.position[1] - prev.position[1]),
                prev.position[2] + alpha * (cur.position[2] - prev.position[2]),
            ];
        }
    }
    samples[samples.len() - 1].position
}

/// Return total arc-length.
#[allow(dead_code)]
pub fn total_length(points: &[[f32; 3]]) -> f32 {
    compute_arc_length(points).total_length
}

/// Sample position at normalised parameter t in `[0,1]`.
#[allow(dead_code)]
pub fn sample_at_t(res: &ArcLengthResult, t: f32) -> [f32; 3] {
    let t_clamped = t.clamp(0.0, 1.0);
    sample_at_arc_length(res, t_clamped * res.total_length)
}

/// Number of samples stored.
#[allow(dead_code)]
pub fn sample_count(res: &ArcLengthResult) -> usize {
    res.samples.len()
}

/// Segment lengths of a polyline.
#[allow(dead_code)]
pub fn segment_lengths(points: &[[f32; 3]]) -> Vec<f32> {
    if points.len() < 2 {
        return Vec::new();
    }
    (1..points.len())
        .map(|i| dist3(points[i - 1], points[i]))
        .collect()
}

/// Generate a circular polyline with given radius and steps.
#[allow(dead_code)]
pub fn circle_polyline(radius: f32, steps: usize) -> Vec<[f32; 3]> {
    (0..=steps)
        .map(|i| {
            let theta = 2.0 * PI * (i as f32) / (steps as f32);
            [radius * theta.cos(), radius * theta.sin(), 0.0]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_pts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]
    }

    #[test]
    fn total_length_straight_line() {
        let pts = line_pts();
        assert!((total_length(&pts) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn sample_count_matches() {
        let res = compute_arc_length(&line_pts());
        assert_eq!(sample_count(&res), 4);
    }

    #[test]
    fn first_sample_t_is_zero() {
        let res = compute_arc_length(&line_pts());
        assert!((res.samples[0].t).abs() < 1e-6);
    }

    #[test]
    fn last_sample_t_is_one() {
        let res = compute_arc_length(&line_pts());
        let last = res.samples.last().unwrap();
        assert!((last.t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn sample_at_midpoint() {
        let res = compute_arc_length(&line_pts());
        let p = sample_at_t(&res, 0.5);
        assert!((p[0] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn uniform_resample_count() {
        let pts = line_pts();
        let out = uniform_resample(&pts, 7);
        assert_eq!(out.len(), 7);
    }

    #[test]
    fn segment_lengths_sum() {
        let lengths = segment_lengths(&line_pts());
        let sum: f32 = lengths.iter().sum();
        assert!((sum - 3.0).abs() < 1e-5);
    }

    #[test]
    fn circle_polyline_length_approx() {
        let pts = circle_polyline(1.0, 1000);
        let len = total_length(&pts);
        assert!((len - 2.0 * PI).abs() < 0.01);
    }

    #[test]
    fn empty_input_returns_zero() {
        assert!((total_length(&[])).abs() < 1e-6);
    }

    #[test]
    fn contains_check() {
        let t = 0.5_f32;
        assert!((0.0..=1.0).contains(&t));
    }
}
