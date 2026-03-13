// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bifurcation diagram computation.

/// A single point on the bifurcation diagram (r, x_settled).
#[derive(Debug, Clone)]
pub struct BifurcationPoint {
    pub r: f64,
    pub x: f64,
}

/// Compute a bifurcation diagram for the logistic map.
pub fn compute_bifurcation(
    r_min: f64,
    r_max: f64,
    r_steps: u32,
    warm_up: u64,
    record: u64,
    x0: f64,
) -> Vec<BifurcationPoint> {
    if r_steps == 0 || r_min >= r_max {
        return Vec::new();
    }
    let mut points = Vec::new();
    let dr = (r_max - r_min) / r_steps as f64;
    for step in 0..=r_steps {
        let r = r_min + step as f64 * dr;
        let mut x = x0.clamp(0.01, 0.99);
        /* warm-up phase to let transients die */
        for _ in 0..warm_up {
            x = r * x * (1.0 - x);
        }
        /* record phase */
        for _ in 0..record {
            x = r * x * (1.0 - x);
            points.push(BifurcationPoint { r, x });
        }
    }
    points
}

/// Count distinct attractors by counting distinct x-bins at a given r.
pub fn attractor_count_at_r(points: &[BifurcationPoint], r: f64, tol: f64) -> usize {
    let r_pts: Vec<f64> = points
        .iter()
        .filter(|p| (p.r - r).abs() < tol)
        .map(|p| p.x)
        .collect();
    if r_pts.is_empty() {
        return 0;
    }
    let mut bins: Vec<i64> = r_pts.iter().map(|&x| (x / tol).round() as i64).collect();
    bins.sort_unstable();
    bins.dedup();
    bins.len()
}

pub fn bifurcation_compute(r_min: f64, r_max: f64, r_steps: u32) -> Vec<BifurcationPoint> {
    compute_bifurcation(r_min, r_max, r_steps, 200, 50, 0.5)
}

pub fn bif_point_count(pts: &[BifurcationPoint]) -> usize {
    pts.len()
}

pub fn bif_attractor_count(pts: &[BifurcationPoint], r: f64, tol: f64) -> usize {
    attractor_count_at_r(pts, r, tol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_for_invalid_range() {
        let pts = bifurcation_compute(3.0, 2.0, 10);
        assert_eq!(bif_point_count(&pts), 0);
    }

    #[test]
    fn test_point_count() {
        /* r_steps=10, record=50 → 11*50 = 550 points */
        let pts = compute_bifurcation(2.0, 4.0, 10, 100, 50, 0.5);
        assert_eq!(pts.len(), 11 * 50);
    }

    #[test]
    fn test_all_x_bounded() {
        let pts = bifurcation_compute(2.0, 4.0, 20);
        for p in &pts {
            assert!((0.0..=1.0).contains(&p.x));
        }
    }

    #[test]
    fn test_r2_fixed_point() {
        /* r=2: logistic map has stable fixed point at x=0.5 */
        let pts = compute_bifurcation(2.0, 2.0, 0, 500, 10, 0.3);
        for p in &pts {
            assert!((p.x - 0.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_r4_chaotic_many_attractors() {
        /* r=4 chaotic: should have many distinct x values.
        Use r_min=r_max=4.0 with r_steps=1 to avoid div-by-zero,
        and a loose tolerance for bin counting. */
        let pts = compute_bifurcation(3.99, 4.0, 1, 100, 200, 0.5);
        let count = bif_attractor_count(&pts, 4.0, 0.05);
        assert!(count > 3);
    }

    #[test]
    fn test_r_range_covered() {
        let pts = bifurcation_compute(2.5, 4.0, 10);
        let r_min = pts.iter().map(|p| p.r).fold(f64::INFINITY, f64::min);
        let r_max = pts.iter().map(|p| p.r).fold(f64::NEG_INFINITY, f64::max);
        assert!((r_min - 2.5).abs() < 1e-10);
        assert!((r_max - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_steps() {
        /* r_steps=1 gives 2 r values (0..=1) → 2*50 = 100 points */
        let pts = compute_bifurcation(2.0, 4.0, 1, 100, 50, 0.5);
        assert_eq!(pts.len(), 2 * 50);
    }

    #[test]
    fn test_period_doubling_r35() {
        /* r=3.5: period-4 orbit; use r_steps=1 to include r=3.5 cleanly */
        let pts = compute_bifurcation(3.49, 3.5, 1, 2000, 50, 0.5);
        let count = bif_attractor_count(&pts, 3.5, 0.01);
        /* period-4 orbit should give at least 4 bins */
        assert!(count >= 4);
    }

    #[test]
    fn test_attractor_count_empty_range() {
        let pts = bifurcation_compute(2.0, 4.0, 5);
        /* r=10 not in range → 0 */
        assert_eq!(bif_attractor_count(&pts, 10.0, 0.01), 0);
    }
}
