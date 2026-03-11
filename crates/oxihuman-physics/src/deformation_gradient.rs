// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A 3x3 deformation gradient tensor F for continuum mechanics.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeformationGradient {
    pub m: [[f32; 3]; 3],
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
impl DeformationGradient {
    pub fn identity() -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn from_rows(r0: [f32; 3], r1: [f32; 3], r2: [f32; 3]) -> Self {
        Self { m: [r0, r1, r2] }
    }

    pub fn determinant(&self) -> f32 {
        let m = &self.m;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    pub fn trace(&self) -> f32 {
        self.m[0][0] + self.m[1][1] + self.m[2][2]
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::identity();
        for i in 0..3 {
            for j in 0..3 {
                result.m[i][j] = self.m[j][i];
            }
        }
        result
    }

    pub fn multiply(&self, other: &DeformationGradient) -> Self {
        let mut result = Self { m: [[0.0; 3]; 3] };
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result.m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        result
    }

    /// Compute the right Cauchy-Green deformation tensor C = F^T * F.
    pub fn right_cauchy_green(&self) -> Self {
        self.transpose().multiply(self)
    }

    /// Compute the left Cauchy-Green deformation tensor B = F * F^T.
    pub fn left_cauchy_green(&self) -> Self {
        self.multiply(&self.transpose())
    }

    /// Compute the Green-Lagrange strain tensor E = 0.5*(C - I).
    pub fn green_lagrange_strain(&self) -> Self {
        let c = self.right_cauchy_green();
        let mut e = Self { m: [[0.0; 3]; 3] };
        for i in 0..3 {
            for j in 0..3 {
                let delta = if i == j { 1.0 } else { 0.0 };
                e.m[i][j] = 0.5 * (c.m[i][j] - delta);
            }
        }
        e
    }

    /// Frobenius norm of the tensor.
    pub fn frobenius_norm(&self) -> f32 {
        let mut sum = 0.0_f32;
        for i in 0..3 {
            for j in 0..3 {
                sum += self.m[i][j] * self.m[i][j];
            }
        }
        sum.sqrt()
    }

    pub fn scale(&self, s: f32) -> Self {
        let mut result = *self;
        for i in 0..3 {
            for j in 0..3 {
                result.m[i][j] *= s;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_determinant() {
        let f = DeformationGradient::identity();
        assert!((f.determinant() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_identity_trace() {
        let f = DeformationGradient::identity();
        assert!((f.trace() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_transpose() {
        let f = DeformationGradient::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        let ft = f.transpose();
        assert!((ft.m[0][1] - 4.0).abs() < 1e-6);
        assert!((ft.m[1][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiply_identity() {
        let f = DeformationGradient::from_rows([2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]);
        let i = DeformationGradient::identity();
        let result = f.multiply(&i);
        assert!((result.m[0][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_right_cauchy_green_identity() {
        let f = DeformationGradient::identity();
        let c = f.right_cauchy_green();
        assert!((c.determinant() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_green_lagrange_identity() {
        let f = DeformationGradient::identity();
        let e = f.green_lagrange_strain();
        assert!(e.frobenius_norm() < 1e-6);
    }

    #[test]
    fn test_frobenius_norm() {
        let f = DeformationGradient::identity();
        assert!((f.frobenius_norm() - 3.0_f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn test_scale() {
        let f = DeformationGradient::identity();
        let s = f.scale(2.0);
        assert!((s.m[0][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_determinant_scaled() {
        let f = DeformationGradient::identity().scale(2.0);
        assert!((f.determinant() - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_left_cauchy_green() {
        let f = DeformationGradient::identity();
        let b = f.left_cauchy_green();
        assert!((b.trace() - 3.0).abs() < 1e-6);
    }
}
