// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cubic Bezier path with eval, split, and length approximation.

#![allow(dead_code)]

/// A cubic Bezier curve defined by 4 control points in 2D.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CubicBezier {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub p3: [f32; 2],
}

/// A path made up of connected cubic Bezier segments.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BezierPath {
    segments: Vec<CubicBezier>,
}

/// Create a new empty BezierPath.
#[allow(dead_code)]
pub fn new_bezier_path() -> BezierPath {
    BezierPath::default()
}

/// Create a cubic Bezier from 4 control points.
#[allow(dead_code)]
pub fn new_cubic_bezier(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2]) -> CubicBezier {
    CubicBezier { p0, p1, p2, p3 }
}

/// Evaluate a cubic Bezier at parameter `t` in [0, 1].
#[allow(dead_code)]
pub fn bezier_eval(b: &CubicBezier, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    let mt3 = mt2 * mt;
    let t3 = t2 * t;
    let w0 = mt3;
    let w1 = 3.0 * mt2 * t;
    let w2 = 3.0 * mt * t2;
    let w3 = t3;
    [
        w0 * b.p0[0] + w1 * b.p1[0] + w2 * b.p2[0] + w3 * b.p3[0],
        w0 * b.p0[1] + w1 * b.p1[1] + w2 * b.p2[1] + w3 * b.p3[1],
    ]
}

/// Split a cubic Bezier at parameter `t` using de Casteljau's algorithm.
/// Returns (left, right) sub-curves.
#[allow(dead_code)]
pub fn bezier_split(b: &CubicBezier, t: f32) -> (CubicBezier, CubicBezier) {
    let t = t.clamp(0.0, 1.0);
    let lerp = |a: [f32; 2], c: [f32; 2]| -> [f32; 2] {
        [a[0] + t * (c[0] - a[0]), a[1] + t * (c[1] - a[1])]
    };
    let p01 = lerp(b.p0, b.p1);
    let p12 = lerp(b.p1, b.p2);
    let p23 = lerp(b.p2, b.p3);
    let p012 = lerp(p01, p12);
    let p123 = lerp(p12, p23);
    let p0123 = lerp(p012, p123);
    (
        CubicBezier {
            p0: b.p0,
            p1: p01,
            p2: p012,
            p3: p0123,
        },
        CubicBezier {
            p0: p0123,
            p1: p123,
            p2: p23,
            p3: b.p3,
        },
    )
}

/// Approximate the arc length of a cubic Bezier using `n` linear segments.
#[allow(dead_code)]
pub fn bezier_length(b: &CubicBezier, n: usize) -> f32 {
    if n == 0 {
        return 0.0;
    }
    let mut len = 0.0f32;
    let mut prev = bezier_eval(b, 0.0);
    for i in 1..=n {
        let t = i as f32 / n as f32;
        let curr = bezier_eval(b, t);
        let dx = curr[0] - prev[0];
        let dy = curr[1] - prev[1];
        len += (dx * dx + dy * dy).sqrt();
        prev = curr;
    }
    len
}

/// Add a segment to the path.
#[allow(dead_code)]
pub fn path_add_segment(path: &mut BezierPath, seg: CubicBezier) {
    path.segments.push(seg);
}

/// Number of segments in the path.
#[allow(dead_code)]
pub fn path_segment_count(path: &BezierPath) -> usize {
    path.segments.len()
}

/// Approximate total length of the path.
#[allow(dead_code)]
pub fn path_length(path: &BezierPath, samples_per_seg: usize) -> f32 {
    path.segments
        .iter()
        .map(|s| bezier_length(s, samples_per_seg))
        .sum()
}

/// Evaluate path at normalized parameter `u` in [0, 1].
#[allow(dead_code)]
pub fn path_eval(path: &BezierPath, u: f32) -> Option<[f32; 2]> {
    if path.segments.is_empty() {
        return None;
    }
    let n = path.segments.len() as f32;
    let scaled = (u.clamp(0.0, 1.0) * n).min(n - 1.0 + 1e-6);
    let seg_idx = scaled.floor() as usize;
    let t = scaled - seg_idx as f32;
    let seg_idx = seg_idx.min(path.segments.len() - 1);
    Some(bezier_eval(&path.segments[seg_idx], t))
}

/// Get a reference to segment at index.
#[allow(dead_code)]
pub fn path_get_segment(path: &BezierPath, idx: usize) -> Option<&CubicBezier> {
    path.segments.get(idx)
}

/// Clear all segments from the path.
#[allow(dead_code)]
pub fn path_clear(path: &mut BezierPath) {
    path.segments.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_bezier() -> CubicBezier {
        new_cubic_bezier([0.0, 0.0], [0.333, 0.0], [0.667, 1.0], [1.0, 1.0])
    }

    #[test]
    fn test_eval_endpoints() {
        let b = unit_bezier();
        let start = bezier_eval(&b, 0.0);
        let end = bezier_eval(&b, 1.0);
        assert!((start[0] - 0.0).abs() < 1e-5);
        assert!((start[1] - 0.0).abs() < 1e-5);
        assert!((end[0] - 1.0).abs() < 1e-5);
        assert!((end[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_eval_midpoint() {
        let b = new_cubic_bezier([0.0, 0.0], [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        let mid = bezier_eval(&b, 0.5);
        assert!((mid[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_split_endpoints_preserved() {
        let b = unit_bezier();
        let (left, right) = bezier_split(&b, 0.5);
        assert!((left.p0[0] - b.p0[0]).abs() < 1e-5);
        assert!((right.p3[0] - b.p3[0]).abs() < 1e-5);
    }

    #[test]
    fn test_split_junction() {
        let b = unit_bezier();
        let (left, right) = bezier_split(&b, 0.5);
        assert!((left.p3[0] - right.p0[0]).abs() < 1e-5);
        assert!((left.p3[1] - right.p0[1]).abs() < 1e-5);
    }

    #[test]
    fn test_length_straight_line() {
        let b = new_cubic_bezier([0.0, 0.0], [0.333, 0.0], [0.667, 0.0], [1.0, 0.0]);
        let len = bezier_length(&b, 100);
        assert!((len - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_path_add_and_count() {
        let mut path = new_bezier_path();
        path_add_segment(&mut path, unit_bezier());
        assert_eq!(path_segment_count(&path), 1);
    }

    #[test]
    fn test_path_eval_empty() {
        let path = new_bezier_path();
        assert!(path_eval(&path, 0.5).is_none());
    }

    #[test]
    fn test_path_eval_single_segment() {
        let mut path = new_bezier_path();
        let b = new_cubic_bezier([0.0, 0.0], [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        path_add_segment(&mut path, b);
        let pt = path_eval(&path, 0.0).unwrap();
        assert!((pt[0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_path_clear() {
        let mut path = new_bezier_path();
        path_add_segment(&mut path, unit_bezier());
        path_clear(&mut path);
        assert_eq!(path_segment_count(&path), 0);
    }
}
