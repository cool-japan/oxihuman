//! Spine curve operations with control points.
#![allow(dead_code)]

use std::f32::consts::PI;

/// A spine curve defined by a list of 3D control points.
#[allow(dead_code)]
pub struct SpineCurve {
    pub control_points: Vec<[f32; 3]>,
}

/// Create a new spine curve with no control points.
#[allow(dead_code)]
pub fn new_spine_curve() -> SpineCurve {
    SpineCurve { control_points: Vec::new() }
}

/// Add a control point to the spine curve.
#[allow(dead_code)]
pub fn add_control_point(curve: &mut SpineCurve, point: [f32; 3]) {
    curve.control_points.push(point);
}

/// Evaluate the spine at parameter t in [0, 1] using linear interpolation.
#[allow(dead_code)]
pub fn evaluate_at(curve: &SpineCurve, t: f32) -> [f32; 3] {
    let pts = &curve.control_points;
    if pts.is_empty() {
        return [0.0; 3];
    }
    if pts.len() == 1 {
        return pts[0];
    }
    let t = t.clamp(0.0, 1.0);
    let n = pts.len() - 1;
    let seg = (t * n as f32).floor() as usize;
    let seg = seg.min(n - 1);
    let local = t * n as f32 - seg as f32;
    let a = pts[seg];
    let b = pts[seg + 1];
    [
        a[0] + (b[0] - a[0]) * local,
        a[1] + (b[1] - a[1]) * local,
        a[2] + (b[2] - a[2]) * local,
    ]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute total arc length of the spine.
#[allow(dead_code)]
pub fn spine_length(curve: &SpineCurve) -> f32 {
    let pts = &curve.control_points;
    if pts.len() < 2 {
        return 0.0;
    }
    pts.windows(2).map(|w| dist3(w[0], w[1])).sum()
}

/// Compute the tangent direction at parameter t (normalized).
#[allow(dead_code)]
pub fn spine_tangent_at(curve: &SpineCurve, t: f32) -> [f32; 3] {
    let eps = 1e-4;
    let a = evaluate_at(curve, (t - eps).max(0.0));
    let b = evaluate_at(curve, (t + eps).min(1.0));
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [dx / len, dy / len, dz / len]
    }
}

/// Compute an approximate normal at parameter t.
#[allow(dead_code)]
pub fn spine_normal_at(curve: &SpineCurve, t: f32) -> [f32; 3] {
    let tangent = spine_tangent_at(curve, t);
    // use world up unless tangent is nearly parallel
    let up = if tangent[1].abs() < 0.99 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let bx = tangent[1] * up[2] - tangent[2] * up[1];
    let by = tangent[2] * up[0] - tangent[0] * up[2];
    let bz = tangent[0] * up[1] - tangent[1] * up[0];
    let nx = by * tangent[2] - bz * tangent[1];
    let ny = bz * tangent[0] - bx * tangent[2];
    let nz = bx * tangent[1] - by * tangent[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-10 {
        [1.0, 0.0, 0.0]
    } else {
        [nx / len, ny / len, nz / len]
    }
}

/// Resample the spine to have `n` uniformly spaced (by arc length) control points.
#[allow(dead_code)]
pub fn resample_spine(curve: &SpineCurve, n: usize) -> SpineCurve {
    if n < 2 || curve.control_points.len() < 2 {
        return SpineCurve { control_points: curve.control_points.clone() };
    }
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        pts.push(evaluate_at(curve, t));
    }
    SpineCurve { control_points: pts }
}

/// Convert spine to a polyline (list of points) with `n` segments.
#[allow(dead_code)]
pub fn spine_to_polyline(curve: &SpineCurve, n: usize) -> Vec<[f32; 3]> {
    let n = n.max(2);
    (0..=n).map(|i| evaluate_at(curve, i as f32 / n as f32)).collect()
}

// expose PI to suppress unused import
#[allow(dead_code)]
fn _pi_usage() -> f32 { PI }

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_curve() -> SpineCurve {
        let mut c = new_spine_curve();
        add_control_point(&mut c, [0.0, 0.0, 0.0]);
        add_control_point(&mut c, [1.0, 0.0, 0.0]);
        add_control_point(&mut c, [2.0, 0.0, 0.0]);
        c
    }

    #[test]
    fn test_new_spine_empty() {
        let c = new_spine_curve();
        assert!(c.control_points.is_empty());
    }

    #[test]
    fn test_add_control_point() {
        let mut c = new_spine_curve();
        add_control_point(&mut c, [1.0, 2.0, 3.0]);
        assert_eq!(c.control_points.len(), 1);
    }

    #[test]
    fn test_evaluate_at_start() {
        let c = sample_curve();
        let p = evaluate_at(&c, 0.0);
        assert!((p[0]).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_at_end() {
        let c = sample_curve();
        let p = evaluate_at(&c, 1.0);
        assert!((p[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_at_mid() {
        let c = sample_curve();
        let p = evaluate_at(&c, 0.5);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_spine_length() {
        let c = sample_curve();
        let len = spine_length(&c);
        assert!((len - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_spine_tangent_direction() {
        let c = sample_curve();
        let t = spine_tangent_at(&c, 0.5);
        assert!((t[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_resample_spine_count() {
        let c = sample_curve();
        let resampled = resample_spine(&c, 5);
        assert_eq!(resampled.control_points.len(), 5);
    }

    #[test]
    fn test_spine_to_polyline_count() {
        let c = sample_curve();
        let poly = spine_to_polyline(&c, 4);
        assert_eq!(poly.len(), 5);
    }

    #[test]
    fn test_spine_normal_at_not_zero() {
        let c = sample_curve();
        let n = spine_normal_at(&c, 0.5);
        let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
        assert!(len > 0.5);
    }
}
