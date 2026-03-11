// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Surface-based vertex sampling for improved physics proxy fitting.

// ── helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        None
    } else {
        Some([v[0] / len, v[1] / len, v[2] / len])
    }
}

/// Multiply 3×3 matrix (row-major flat array of 9 elements) by a vector.
#[inline]
fn mat3_mul_vec(m: &[f32; 9], v: [f32; 3]) -> [f32; 3] {
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

/// Run power iteration starting from `start` for `steps` steps.
/// Returns the dominant eigenvector and its approximate eigenvalue (Rayleigh quotient).
/// Returns `None` if the covariance matrix collapses the start vector to zero.
fn power_iter(cov: &[f32; 9], start: [f32; 3], steps: usize) -> Option<([f32; 3], f32)> {
    let mut v = normalize(start)?;
    for _ in 0..steps {
        let mv = mat3_mul_vec(cov, v);
        v = normalize(mv)?;
    }
    let mv = mat3_mul_vec(cov, v);
    let eigenval = dot(v, mv); // Rayleigh quotient
    Some((v, eigenval))
}

// ── public API ────────────────────────────────────────────────────────────────

/// Compute the principal axis of a point cloud using PCA (pure Rust, no external crate).
///
/// Returns `(centroid, axis_direction)` where `axis_direction` is the dominant eigenvector
/// (unit length) found by power iteration on the 3×3 covariance matrix.
/// Three starting vectors (`[1,0,0]`, `[0,1,0]`, `[0,0,1]`) are tried and the one with the
/// largest Rayleigh quotient is returned.
pub fn principal_axis(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let n = points.len();
    if n == 0 {
        return ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    }

    // ── centroid ──────────────────────────────────────────────────────────────
    let mut sum = [0.0f32; 3];
    for p in points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    let inv_n = 1.0 / n as f32;
    let centroid = [sum[0] * inv_n, sum[1] * inv_n, sum[2] * inv_n];

    if n == 1 {
        return (centroid, [1.0, 0.0, 0.0]);
    }

    // ── 3×3 covariance matrix (row-major) ────────────────────────────────────
    // cov[i*3+j] = mean( (p[i]-centroid[i]) * (p[j]-centroid[j]) )
    let mut cov = [0.0f32; 9];
    for p in points {
        let d = sub(*p, centroid);
        cov[0] += d[0] * d[0];
        cov[1] += d[0] * d[1];
        cov[2] += d[0] * d[2];
        cov[3] += d[1] * d[0];
        cov[4] += d[1] * d[1];
        cov[5] += d[1] * d[2];
        cov[6] += d[2] * d[0];
        cov[7] += d[2] * d[1];
        cov[8] += d[2] * d[2];
    }
    for c in &mut cov {
        *c *= inv_n;
    }

    // ── power iteration with three axis-aligned starting vectors ─────────────
    // Try all three canonical axes as starting vectors and pick the winner
    // (largest Rayleigh quotient). This handles the case where the dominant
    // variance is exactly along one axis (e.g. all points on the Y axis).
    let candidates = [[1.0f32, 0.0, 0.0], [0.0f32, 1.0, 0.0], [0.0f32, 0.0, 1.0]];

    let mut best_vec = [1.0f32, 0.0, 0.0];
    let mut best_val = f32::NEG_INFINITY;

    for &start in &candidates {
        if let Some((v, eigenval)) = power_iter(&cov, start, 20) {
            if eigenval > best_val {
                best_val = eigenval;
                best_vec = v;
            }
        }
    }

    (centroid, best_vec)
}

/// A capsule collision primitive defined by two endpoint centers and a radius.
#[derive(Debug, Clone, PartialEq)]
pub struct Capsule {
    /// One endpoint of the capsule axis.
    pub p0: [f32; 3],
    /// Other endpoint of the capsule axis.
    pub p1: [f32; 3],
    /// Radius of the capsule.
    pub radius: f32,
}

/// Fit a capsule along the principal axis of a point cloud.
///
/// Algorithm:
/// 1. Find (centroid, axis) via [`principal_axis`].
/// 2. Project all points onto the axis; record min/max projections t_min, t_max.
/// 3. `p0 = centroid + axis * t_min`, `p1 = centroid + axis * t_max`.
/// 4. `radius` = max perpendicular distance from any point to the axis line.
///    Clamped to at least 0.001.
pub fn fit_capsule(points: &[[f32; 3]]) -> Capsule {
    if points.is_empty() {
        return Capsule {
            p0: [0.0, 0.0, 0.0],
            p1: [0.0, 0.0, 0.0],
            radius: 0.001,
        };
    }

    let (centroid, axis) = principal_axis(points);

    // ── project onto axis ─────────────────────────────────────────────────────
    let mut t_min = f32::INFINITY;
    let mut t_max = f32::NEG_INFINITY;
    for p in points {
        let t = dot(sub(*p, centroid), axis);
        if t < t_min {
            t_min = t;
        }
        if t > t_max {
            t_max = t;
        }
    }

    let p0 = add(centroid, scale(axis, t_min));
    let p1 = add(centroid, scale(axis, t_max));

    // ── perpendicular radius ──────────────────────────────────────────────────
    let mut max_perp_sq = 0.0f32;
    for p in points {
        let d = sub(*p, centroid);
        let t = dot(d, axis);
        // perpendicular component: d - axis*t
        let perp = sub(d, scale(axis, t));
        let perp_sq = dot(perp, perp);
        if perp_sq > max_perp_sq {
            max_perp_sq = perp_sq;
        }
    }
    let radius = max_perp_sq.sqrt().max(0.001);

    Capsule { p0, p1, radius }
}

/// Given a set of vertex positions for a body part, return a fitted [`Capsule`].
pub fn proxy_from_vertices(verts: &[[f32; 3]]) -> Capsule {
    fit_capsule(verts)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Uniform sphere of points — the axis can point anywhere, but must be unit length.
    #[test]
    fn principal_axis_sphere_is_any_direction() {
        let mut pts = Vec::new();
        // Use a deterministic grid of points on the surface of a sphere
        for i in 0..10i32 {
            for j in 0..10i32 {
                let theta = std::f32::consts::PI * i as f32 / 9.0;
                let phi = 2.0 * std::f32::consts::PI * j as f32 / 10.0;
                pts.push([
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ]);
            }
        }
        let (_centroid, axis) = principal_axis(&pts);
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "axis must be unit length, got len={len}"
        );
    }

    /// Points elongated along X: axis should be approximately ±[1,0,0].
    #[test]
    fn principal_axis_elongated_cloud() {
        // 21 points: x from -10 to +10, y/z near 0
        let pts: Vec<[f32; 3]> = (-10..=10).map(|i| [i as f32, 0.0f32, 0.0f32]).collect();
        let (_centroid, axis) = principal_axis(&pts);
        // The dominant component must be along X (either +1 or -1)
        let x_component = axis[0].abs();
        assert!(
            x_component > 0.99,
            "axis should align with X, got {:?}",
            axis
        );
    }

    /// 10 points from (0,0,0) to (0,10,0): capsule should span y=0..10 with small radius.
    #[test]
    fn fit_capsule_line_of_points() {
        let pts: Vec<[f32; 3]> = (0..=10).map(|i| [0.0f32, i as f32, 0.0f32]).collect();
        let cap = fit_capsule(&pts);

        // p0 and p1 should be at y=0 and y=10 (or swapped)
        let y_low = cap.p0[1].min(cap.p1[1]);
        let y_high = cap.p0[1].max(cap.p1[1]);
        assert!((y_low - 0.0).abs() < 0.01, "y_low={y_low}");
        assert!((y_high - 10.0).abs() < 0.01, "y_high={y_high}");
        // x and z should be near 0
        assert!(cap.p0[0].abs() < 0.01 && cap.p0[2].abs() < 0.01);
        // radius is clamped to at least 0.001 (collinear points → 0 perp distance)
        assert!(cap.radius >= 0.001);
    }

    /// Single point → radius is 0.001 (clamped), p0 == p1 == centroid.
    #[test]
    fn fit_capsule_single_point() {
        let pts = vec![[3.0f32, 5.0, 7.0]];
        let cap = fit_capsule(&pts);
        assert!((cap.p0[0] - 3.0).abs() < 1e-5);
        assert!((cap.p0[1] - 5.0).abs() < 1e-5);
        assert!((cap.p0[2] - 7.0).abs() < 1e-5);
        assert!((cap.p1[0] - 3.0).abs() < 1e-5);
        assert!((cap.p1[1] - 5.0).abs() < 1e-5);
        assert!((cap.p1[2] - 7.0).abs() < 1e-5);
        assert!((cap.radius - 0.001).abs() < 1e-5);
    }

    /// All input points must lie within `radius` distance of the capsule axis segment.
    #[test]
    fn fit_capsule_radius_covers_all() {
        // A cloud of 50 points in a cluster
        let pts: Vec<[f32; 3]> = (0..50)
            .map(|i| {
                let t = i as f32 * 0.1;
                [t.cos() * 0.5, t, t.sin() * 0.3]
            })
            .collect();
        let cap = fit_capsule(&pts);

        let (centroid, axis) = principal_axis(&pts);

        for p in &pts {
            let d = sub(*p, centroid);
            let t = dot(d, axis);
            let perp = sub(d, scale(axis, t));
            let perp_dist = dot(perp, perp).sqrt();
            assert!(
                perp_dist <= cap.radius + 1e-4,
                "point {:?} is {perp_dist:.4} from axis, radius={:.4}",
                p,
                cap.radius
            );
        }
    }

    /// Verify proxy_from_vertices delegates correctly.
    #[test]
    fn proxy_from_vertices_delegates() {
        let pts = vec![[0.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]];
        let cap = proxy_from_vertices(&pts);
        assert!(cap.radius >= 0.001);
        let y_high = cap.p0[1].max(cap.p1[1]);
        assert!((y_high - 2.0).abs() < 0.01, "y_high={y_high}");
    }
}
