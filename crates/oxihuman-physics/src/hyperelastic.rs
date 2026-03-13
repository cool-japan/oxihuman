// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hyperelastic material (Neo-Hookean model).

#![allow(dead_code)]

/// Neo-Hookean hyperelastic material parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeoHookean {
    pub mu: f64,
    pub lambda: f64,
}

/// Deformation gradient F (3x3 row-major).
pub type DefGrad = [[f64; 3]; 3];

impl NeoHookean {
    /// Create from Young's modulus and Poisson's ratio.
    #[allow(dead_code)]
    pub fn from_young_poisson(young: f64, poisson: f64) -> Self {
        let mu = young / (2.0 * (1.0 + poisson));
        let lambda = young * poisson / ((1.0 + poisson) * (1.0 - 2.0 * poisson));
        Self { mu, lambda }
    }

    /// Compute strain energy density W(F) for Neo-Hookean.
    /// W = mu/2 * (I1 - 3) - mu * ln(J) + lambda/2 * ln(J)^2
    #[allow(dead_code)]
    pub fn strain_energy(&self, f: &DefGrad) -> f64 {
        let j = det3(f);
        if j <= 0.0 {
            return f64::INFINITY;
        }
        let i1 = trace_fte(f);
        let lnj = j.ln();
        self.mu / 2.0 * (i1 - 3.0) - self.mu * lnj + self.lambda / 2.0 * lnj * lnj
    }

    /// First Piola-Kirchhoff stress P = mu*(F - F^{-T}) + lambda*ln(J)*F^{-T}.
    #[allow(dead_code)]
    pub fn first_piola_kirchhoff(&self, f: &DefGrad) -> DefGrad {
        let j = det3(f);
        if j <= 0.0 {
            return [[0.0; 3]; 3];
        }
        let lnj = j.ln();
        let finv_t = transpose3(&inv3(f));
        let mut p = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for k in 0..3 {
                p[i][k] = self.mu * (f[i][k] - finv_t[i][k])
                    + self.lambda * lnj * finv_t[i][k];
            }
        }
        p
    }

    #[allow(dead_code)]
    pub fn is_stable(&self, f: &DefGrad) -> bool {
        det3(f) > 0.0
    }
}

/// 3x3 determinant.
#[allow(dead_code)]
pub fn det3(m: &DefGrad) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Trace of F^T * F = I1 = sum of squared singular values.
fn trace_fte(f: &DefGrad) -> f64 {
    let mut s = 0.0;
    for row in f {
        for &v in row {
            s += v * v;
        }
    }
    s
}

/// 3x3 matrix inverse.
#[allow(dead_code)]
pub fn inv3(m: &DefGrad) -> DefGrad {
    let d = det3(m);
    if d.abs() < 1e-15 {
        return [[0.0; 3]; 3];
    }
    let inv_d = 1.0 / d;
    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_d,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_d,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_d,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_d,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_d,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_d,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_d,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_d,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_d,
        ],
    ]
}

/// 3x3 transpose.
#[allow(dead_code)]
pub fn transpose3(m: &DefGrad) -> DefGrad {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Identity deformation gradient.
#[allow(dead_code)]
pub fn identity_f() -> DefGrad {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_energy_at_identity() {
        let mat = NeoHookean::from_young_poisson(1000.0, 0.3);
        let f = identity_f();
        let w = mat.strain_energy(&f);
        assert!(w.abs() < 1e-10, "W={w}");
    }

    #[test]
    fn test_energy_positive_under_stretch() {
        let mat = NeoHookean::from_young_poisson(1000.0, 0.3);
        let mut f = identity_f();
        f[0][0] = 1.5;
        assert!(mat.strain_energy(&f) > 0.0);
    }

    #[test]
    fn test_det_identity() {
        let f = identity_f();
        assert!((det3(&f) - 1.0).abs() < 1e-12);
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_inv3_identity() {
        let f = identity_f();
        let inv = inv3(&f);
        for i in 0..3 {
            for k in 0..3 {
                let expected = if i == k { 1.0 } else { 0.0 };
                assert!((inv[i][k] - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_transpose_identity() {
        let f = identity_f();
        let t = transpose3(&f);
        for i in 0..3 {
            for k in 0..3 {
                assert!((f[i][k] - t[i][k]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_first_piola_zero_at_identity() {
        let mat = NeoHookean::from_young_poisson(1000.0, 0.3);
        let f = identity_f();
        let p = mat.first_piola_kirchhoff(&f);
        for row in &p {
            for &v in row {
                assert!(v.abs() < 1e-9, "P={v}");
            }
        }
    }

    #[test]
    fn test_is_stable_identity() {
        let mat = NeoHookean::from_young_poisson(1000.0, 0.3);
        assert!(mat.is_stable(&identity_f()));
    }

    #[test]
    fn test_energy_infinity_inverted() {
        let mat = NeoHookean::from_young_poisson(1000.0, 0.3);
        let mut f = identity_f();
        f[0][0] = -1.0;
        assert!(!mat.strain_energy(&f).is_finite());
    }

    #[test]
    fn test_lame_params_positive() {
        let mat = NeoHookean::from_young_poisson(200e9, 0.3);
        assert!(mat.mu > 0.0 && mat.lambda > 0.0);
    }

    #[test]
    fn test_det_non_zero() {
        let f: DefGrad = [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]];
        assert!((det3(&f) - 24.0).abs() < 1e-10);
    }
}
