// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Isogeometric analysis (IGA) stub — uses NURBS basis functions to represent
//! both geometry and the solution field, unifying CAD and FEA descriptions.

/// B-spline basis function evaluated by the Cox-de Boor recursion.
pub fn bspline_basis(i: usize, p: usize, t: f64, knots: &[f64]) -> f64 {
    if p == 0 {
        if knots[i] <= t && t < knots[i + 1] {
            return 1.0;
        }
        return 0.0;
    }
    let d1 = knots[i + p] - knots[i];
    let d2 = knots[i + p + 1] - knots[i + 1];
    let left = if d1.abs() > 1e-14 {
        (t - knots[i]) / d1 * bspline_basis(i, p - 1, t, knots)
    } else {
        0.0
    };
    let right = if d2.abs() > 1e-14 {
        (knots[i + p + 1] - t) / d2 * bspline_basis(i + 1, p - 1, t, knots)
    } else {
        0.0
    };
    left + right
}

/// A 1-D IGA patch defined by a knot vector, degree, and control points.
pub struct IgaPatch1D {
    pub knots: Vec<f64>,
    pub degree: usize,
    pub ctrl_pts: Vec<[f64; 2]>, /* (x, y) control points */
    pub weights: Vec<f64>,
}

impl IgaPatch1D {
    /// Create a uniform knot-vector patch.
    pub fn new(degree: usize, ctrl_pts: Vec<[f64; 2]>) -> Self {
        let n = ctrl_pts.len();
        let knots = Self::uniform_knots(degree, n);
        let weights = vec![1.0; n];
        Self {
            knots,
            degree,
            ctrl_pts,
            weights,
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn uniform_knots(p: usize, n: usize) -> Vec<f64> {
        let m = n + p + 1;
        let mut knots = vec![0.0; m];
        /* clamped uniform */
        for i in 0..m {
            if i <= p {
                knots[i] = 0.0;
            } else if i >= m - p - 1 {
                knots[i] = 1.0;
            } else {
                knots[i] = (i - p) as f64 / (n - p) as f64;
            }
        }
        knots
    }

    /// Evaluate the NURBS curve at parameter `t` ∈ [0, 1).
    pub fn eval(&self, t: f64) -> [f64; 2] {
        let n = self.ctrl_pts.len();
        let p = self.degree;
        let mut wx = 0.0f64;
        let mut wy = 0.0f64;
        let mut w_sum = 0.0f64;
        for i in 0..n {
            let b = bspline_basis(i, p, t, &self.knots);
            let wi = self.weights[i] * b;
            wx += wi * self.ctrl_pts[i][0];
            wy += wi * self.ctrl_pts[i][1];
            w_sum += wi;
        }
        if w_sum.abs() < 1e-14 {
            return [0.0, 0.0];
        }
        [wx / w_sum, wy / w_sum]
    }

    /// Number of basis functions (= number of control points).
    pub fn basis_count(&self) -> usize {
        self.ctrl_pts.len()
    }

    /// Knot vector length.
    pub fn knot_count(&self) -> usize {
        self.knots.len()
    }
}

/// Create a 1-D IGA patch.
pub fn new_iga_patch(degree: usize, ctrl_pts: Vec<[f64; 2]>) -> IgaPatch1D {
    IgaPatch1D::new(degree, ctrl_pts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_patch() -> IgaPatch1D {
        /* linear NURBS from (0,0) to (1,0) */
        IgaPatch1D::new(1, vec![[0.0, 0.0], [1.0, 0.0]])
    }

    #[test]
    fn test_eval_start() {
        let p = line_patch();
        let pt = p.eval(0.0);
        assert!((pt[0] - 0.0).abs() < 1e-9); /* start point */
    }

    #[test]
    fn test_eval_midpoint() {
        let p = line_patch();
        let pt = p.eval(0.5);
        assert!((pt[0] - 0.5).abs() < 1e-9); /* midpoint */
    }

    #[test]
    fn test_basis_count() {
        let p = line_patch();
        assert_eq!(p.basis_count(), 2); /* two control points */
    }

    #[test]
    fn test_knot_count() {
        let p = line_patch();
        /* n + p + 1 = 2 + 1 + 1 = 4 */
        assert_eq!(p.knot_count(), 4); /* correct knot vector length */
    }

    #[test]
    fn test_quadratic_basis_sum() {
        /* sum of all basis functions at any t should be 1 */
        let p = IgaPatch1D::new(2, vec![[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]]);
        let n = p.basis_count();
        let t = 0.4;
        let sum: f64 = (0..n).map(|i| bspline_basis(i, 2, t, &p.knots)).sum();
        assert!((sum - 1.0).abs() < 1e-10); /* partition of unity */
    }

    #[test]
    fn test_bspline_basis_p0() {
        let knots = vec![0.0, 0.5, 1.0];
        let b = bspline_basis(0, 0, 0.25, &knots);
        assert!((b - 1.0).abs() < 1e-10); /* inside interval */
    }

    #[test]
    fn test_new_helper() {
        let p = new_iga_patch(1, vec![[0.0, 0.0], [2.0, 0.0]]);
        assert_eq!(p.basis_count(), 2); /* helper works */
    }

    #[test]
    fn test_eval_y_zero_on_line() {
        let p = line_patch();
        let pt = p.eval(0.3);
        assert!(pt[1].abs() < 1e-9); /* y is always 0 on the line */
    }

    #[test]
    fn test_weights_default_one() {
        let p = line_patch();
        assert!(p.weights.iter().all(|&w| (w - 1.0).abs() < 1e-10)); /* unit weights */
    }
}
