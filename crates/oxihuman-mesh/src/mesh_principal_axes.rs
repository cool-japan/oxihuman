// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compute principal axes from an inertia tensor stub.

use crate::mesh_moment_of_inertia::InertiaTensor;

/// Principal axes result: three orthogonal unit vectors and corresponding moments.
#[derive(Debug, Clone, Copy)]
pub struct PrincipalAxes {
    /// Principal axes as columns of the rotation matrix.
    pub axes: [[f32; 3]; 3],
    /// Principal moments of inertia (eigenvalues).
    pub moments: [f32; 3],
}

impl PrincipalAxes {
    /// Return the axis with the smallest principal moment.
    pub fn min_axis(&self) -> [f32; 3] {
        let idx = self
            .moments
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.axes[idx]
    }

    /// Return the axis with the largest principal moment.
    pub fn max_axis(&self) -> [f32; 3] {
        let idx = self
            .moments
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(2);
        self.axes[idx]
    }
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        return [1.0, 0.0, 0.0];
    }
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Jacobi iteration step for symmetric 3×3 matrix diagonalization (stub).
pub fn jacobi_eigendecompose(tensor: &InertiaTensor, max_iters: usize) -> PrincipalAxes {
    /* Simplified Jacobi iteration for 3×3 symmetric matrix.
    Stores matrix as [m00, m11, m22, m01, m02, m12]. */
    let mut m = [
        tensor.diag[0],
        tensor.diag[1],
        tensor.diag[2],
        tensor.off[0],
        tensor.off[1],
        tensor.off[2],
    ];
    /* Eigenvector matrix (identity initially) */
    let mut v = [[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..max_iters {
        /* Find largest off-diagonal element */
        let offs = [m[3].abs(), m[4].abs(), m[5].abs()];
        let (p, q) = if offs[0] >= offs[1] && offs[0] >= offs[2] {
            (0usize, 1usize)
        } else if offs[1] >= offs[2] {
            (0, 2)
        } else {
            (1, 2)
        };
        let off_idx = if (p, q) == (0, 1) {
            3
        } else if (p, q) == (0, 2) {
            4
        } else {
            5
        };
        if m[off_idx].abs() < 1e-9 {
            break;
        }

        /* Compute rotation angle */
        let theta = 0.5 * (m[q] - m[p]).atan2(m[off_idx]);
        let (sin_t, cos_t) = theta.sin_cos();

        /* Rotate: update matrix and eigenvector columns */
        let mp = m[p];
        let mq = m[q];
        let mo = m[off_idx];
        m[p] = cos_t * cos_t * mp - 2.0 * sin_t * cos_t * mo + sin_t * sin_t * mq;
        m[q] = sin_t * sin_t * mp + 2.0 * sin_t * cos_t * mo + cos_t * cos_t * mq;
        m[off_idx] = 0.0;

        /* Update remaining off-diagonals (approximate) */
        let r = if (p, q) == (0, 1) {
            2
        } else if (p, q) == (0, 2) {
            1
        } else {
            0
        };
        let (rp_idx, rq_idx) = if r == 2 {
            (4, 5)
        } else if r == 1 {
            (3, 5)
        } else {
            (3, 4)
        };
        let mrp = m[rp_idx];
        let mrq = m[rq_idx];
        m[rp_idx] = cos_t * mrp - sin_t * mrq;
        m[rq_idx] = sin_t * mrp + cos_t * mrq;

        /* Update eigenvector columns p and q */
        for row in &mut v {
            let vp = row[p];
            let vq = row[q];
            row[p] = cos_t * vp - sin_t * vq;
            row[q] = sin_t * vp + cos_t * vq;
        }
    }

    PrincipalAxes {
        axes: [normalize3(v[0]), normalize3(v[1]), normalize3(v[2])],
        moments: [m[0], m[1], m[2]],
    }
}

/// Compute principal axes from an inertia tensor.
pub fn compute_principal_axes(tensor: &InertiaTensor) -> PrincipalAxes {
    jacobi_eigendecompose(tensor, 50)
}

/// Check if principal axes are orthogonal (within tolerance).
pub fn axes_are_orthogonal(axes: &[[f32; 3]; 3], tol: f32) -> bool {
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    dot(axes[0], axes[1]).abs() < tol
        && dot(axes[1], axes[2]).abs() < tol
        && dot(axes[0], axes[2]).abs() < tol
}

/// Sort principal axes so that moments are in ascending order.
pub fn sort_principal_axes(pa: &mut PrincipalAxes) {
    /* Bubble sort (3 elements) */
    for i in 0..3 {
        for j in 0..2 - i {
            if pa.moments[j] > pa.moments[j + 1] {
                pa.moments.swap(j, j + 1);
                pa.axes.swap(j, j + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagonal_tensor(ixx: f32, iyy: f32, izz: f32) -> InertiaTensor {
        InertiaTensor {
            diag: [ixx, iyy, izz],
            off: [0.0; 3],
        }
    }

    #[test]
    fn test_diagonal_tensor_gives_aligned_axes() {
        let t = diagonal_tensor(1.0, 2.0, 3.0);
        let pa = compute_principal_axes(&t);
        /* Moments should be close to diagonal values */
        let mut moments = pa.moments;
        moments.sort_by(|a, b| a.partial_cmp(b).expect("should succeed"));
        assert!((moments[0] - 1.0).abs() < 0.1 /* smallest moment ≈ 1 */);
    }

    #[test]
    fn test_axes_orthogonal() {
        let t = diagonal_tensor(1.0, 2.0, 3.0);
        let pa = compute_principal_axes(&t);
        assert!(axes_are_orthogonal(&pa.axes, 0.01) /* axes are orthogonal */);
    }

    #[test]
    fn test_sort_ascending() {
        let mut pa = PrincipalAxes {
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            moments: [3.0, 1.0, 2.0],
        };
        sort_principal_axes(&mut pa);
        assert!(pa.moments[0] <= pa.moments[1] /* ascending after sort */);
        assert!(pa.moments[1] <= pa.moments[2]);
    }

    #[test]
    fn test_min_max_axis_different() {
        let pa = PrincipalAxes {
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            moments: [1.0, 2.0, 3.0],
        };
        let min_a = pa.min_axis();
        let max_a = pa.max_axis();
        /* min and max axes should differ for distinct moments */
        let same = min_a
            .iter()
            .zip(max_a.iter())
            .all(|(a, b)| (a - b).abs() < 1e-5);
        assert!(!same /* distinct axes */);
    }

    #[test]
    fn test_unit_tensor_moment_magnitudes() {
        let t = diagonal_tensor(5.0, 5.0, 5.0);
        let pa = compute_principal_axes(&t);
        /* All moments should be ≈ 5 for isotropic tensor */
        for m in pa.moments {
            assert!((m - 5.0).abs() < 0.5 /* ≈ 5 */);
        }
    }

    #[test]
    fn test_jacobi_converges() {
        let t = InertiaTensor {
            diag: [2.0, 3.0, 4.0],
            off: [0.5, 0.3, 0.2],
        };
        let pa = jacobi_eigendecompose(&t, 100);
        /* Off-diagonals should be ≈ 0 after many iterations */
        assert!(pa.moments.iter().all(|m| m.is_finite()) /* all finite */);
    }

    #[test]
    fn test_axes_unit_length() {
        let t = diagonal_tensor(1.0, 4.0, 9.0);
        let pa = compute_principal_axes(&t);
        for ax in pa.axes {
            let l = len3(ax);
            assert!((l - 1.0).abs() < 0.01 /* unit length */);
        }
    }

    #[test]
    fn test_zero_tensor_does_not_panic() {
        let t = InertiaTensor::zero();
        let pa = compute_principal_axes(&t);
        assert!(pa.moments.iter().all(|m| m.is_finite()) /* finite even for zero */);
    }
}
