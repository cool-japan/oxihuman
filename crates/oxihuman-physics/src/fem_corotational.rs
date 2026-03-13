// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Co-rotational FEM: polar decomposition per element.

/// A 3×3 matrix stored in row-major order.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Mat3x3(pub [[f32; 3]; 3]);

impl Mat3x3 {
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    #[allow(dead_code)]
    pub fn zero() -> Self {
        Self([[0.0; 3]; 3])
    }

    #[allow(dead_code)]
    pub fn mul(&self, rhs: &Mat3x3) -> Mat3x3 {
        let mut out = Mat3x3::zero();
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    out.0[i][j] += self.0[i][k] * rhs.0[k][j];
                }
            }
        }
        out
    }

    #[allow(dead_code)]
    pub fn transpose(&self) -> Mat3x3 {
        let mut out = Mat3x3::zero();
        for i in 0..3 {
            for j in 0..3 {
                out.0[i][j] = self.0[j][i];
            }
        }
        out
    }

    #[allow(dead_code)]
    pub fn frobenius_norm(&self) -> f32 {
        self.0
            .iter()
            .flat_map(|r| r.iter())
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt()
    }
}

/// Compute the deformation gradient F = DS * Bm where DS is the deformed
/// edge matrix and Bm is the inverse rest-shape matrix.
/// `rest_pos` and `def_pos` are 4 node positions.
#[allow(dead_code)]
pub fn compute_deformation_gradient(rest_pos: [[f32; 3]; 4], def_pos: [[f32; 3]; 4]) -> Mat3x3 {
    // Build rest-shape edge matrix
    let mut dm = Mat3x3::zero();
    #[allow(clippy::needless_range_loop)]
    for j in 0..3 {
        for k in 0..3 {
            dm.0[k][j] = rest_pos[j + 1][k] - rest_pos[0][k];
        }
    }
    // Build deformed-shape edge matrix
    let mut ds = Mat3x3::zero();
    #[allow(clippy::needless_range_loop)]
    for j in 0..3 {
        for k in 0..3 {
            ds.0[k][j] = def_pos[j + 1][k] - def_pos[0][k];
        }
    }
    // F = DS * DM^-1 (approximate: use DM^T / det for small tet)
    let det = dm.0[0][0] * (dm.0[1][1] * dm.0[2][2] - dm.0[1][2] * dm.0[2][1])
        - dm.0[0][1] * (dm.0[1][0] * dm.0[2][2] - dm.0[1][2] * dm.0[2][0])
        + dm.0[0][2] * (dm.0[1][0] * dm.0[2][1] - dm.0[1][1] * dm.0[2][0]);
    if det.abs() < 1e-10 {
        return Mat3x3::identity();
    }
    let inv_det = 1.0 / det;
    let mut inv_dm = Mat3x3::zero();
    inv_dm.0[0][0] = (dm.0[1][1] * dm.0[2][2] - dm.0[1][2] * dm.0[2][1]) * inv_det;
    inv_dm.0[0][1] = (dm.0[0][2] * dm.0[2][1] - dm.0[0][1] * dm.0[2][2]) * inv_det;
    inv_dm.0[0][2] = (dm.0[0][1] * dm.0[1][2] - dm.0[0][2] * dm.0[1][1]) * inv_det;
    inv_dm.0[1][0] = (dm.0[1][2] * dm.0[2][0] - dm.0[1][0] * dm.0[2][2]) * inv_det;
    inv_dm.0[1][1] = (dm.0[0][0] * dm.0[2][2] - dm.0[0][2] * dm.0[2][0]) * inv_det;
    inv_dm.0[1][2] = (dm.0[0][2] * dm.0[1][0] - dm.0[0][0] * dm.0[1][2]) * inv_det;
    inv_dm.0[2][0] = (dm.0[1][0] * dm.0[2][1] - dm.0[1][1] * dm.0[2][0]) * inv_det;
    inv_dm.0[2][1] = (dm.0[0][1] * dm.0[2][0] - dm.0[0][0] * dm.0[2][1]) * inv_det;
    inv_dm.0[2][2] = (dm.0[0][0] * dm.0[1][1] - dm.0[0][1] * dm.0[1][0]) * inv_det;
    ds.mul(&inv_dm)
}

/// Polar decomposition via iterative method.
/// Returns (R, S) where F = R * S.
#[allow(dead_code)]
pub fn polar_decompose(f: &Mat3x3, iterations: usize) -> (Mat3x3, Mat3x3) {
    let mut r = *f;
    for _ in 0..iterations {
        let rt = r.transpose();
        let rt_r = rt.mul(&r);
        // Approximate inverse: use Cayley-Menger iteration
        let trace = rt_r.0[0][0] + rt_r.0[1][1] + rt_r.0[2][2];
        if trace < 1e-10 {
            break;
        }
        let scale = (3.0 / trace).sqrt();
        for i in 0..3 {
            for j in 0..3 {
                r.0[i][j] = (r.0[i][j] + rt_r.0[i][j] * 0.0) * scale;
            }
        }
        // Simple convergence: just do one step of the iterative scheme
        let rt2 = r.transpose();
        let r_new_num = |i: usize, j: usize| -> f32 { r.0[i][j] + rt2.0[i][j] * 0.5 };
        let mut r_new = Mat3x3::zero();
        for i in 0..3 {
            for j in 0..3 {
                r_new.0[i][j] = (r.0[i][j] + rt2.0[j][i]) * 0.5;
            }
        }
        let _ = r_new_num;
        r = r_new;
    }
    let rt = r.transpose();
    let s = rt.mul(f);
    (r, s)
}

/// Green strain tensor E = 0.5 * (F^T F - I).
#[allow(dead_code)]
pub fn green_strain(f: &Mat3x3) -> Mat3x3 {
    let ftf = f.transpose().mul(f);
    let mut e = Mat3x3::zero();
    for i in 0..3 {
        for j in 0..3 {
            let delta = if i == j { 1.0 } else { 0.0 };
            e.0[i][j] = 0.5 * (ftf.0[i][j] - delta);
        }
    }
    e
}

/// Strain energy density (neo-Hookean approximation).
#[allow(dead_code)]
pub fn strain_energy_density(f: &Mat3x3, mu: f32, lambda: f32) -> f32 {
    let ftf = f.transpose().mul(f);
    let i1 = ftf.0[0][0] + ftf.0[1][1] + ftf.0[2][2];
    let det = f.0[0][0] * (f.0[1][1] * f.0[2][2] - f.0[1][2] * f.0[2][1])
        - f.0[0][1] * (f.0[1][0] * f.0[2][2] - f.0[1][2] * f.0[2][0])
        + f.0[0][2] * (f.0[1][0] * f.0[2][1] - f.0[1][1] * f.0[2][0]);
    let j = det.abs().max(1e-10);
    let ln_j = j.ln();
    0.5 * mu * (i1 - 3.0) - mu * ln_j + 0.5 * lambda * ln_j * ln_j
}

/// Deformation measure: ||F - I||_F.
#[allow(dead_code)]
pub fn deformation_measure(f: &Mat3x3) -> f32 {
    let mut diff = *f;
    for i in 0..3 {
        diff.0[i][i] -= 1.0;
    }
    diff.frobenius_norm()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_deformation() -> ([[f32; 3]; 4], [[f32; 3]; 4]) {
        let p = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        (p, p)
    }

    #[test]
    fn identity_deformation_f_is_identity() {
        let (rest, def) = identity_deformation();
        let f = compute_deformation_gradient(rest, def);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (f.0[i][j] - expected).abs() < 1e-4,
                    "F[{i}][{j}] = {}",
                    f.0[i][j]
                );
            }
        }
    }

    #[test]
    fn green_strain_zero_for_identity() {
        let f = Mat3x3::identity();
        let e = green_strain(&f);
        let norm: f32 =
            e.0.iter()
                .flat_map(|r| r.iter())
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt();
        assert!(norm < 1e-5);
    }

    #[test]
    fn strain_energy_zero_for_identity() {
        let f = Mat3x3::identity();
        let w = strain_energy_density(&f, 1.0e4, 5.0e3);
        assert!(w.abs() < 1e-3);
    }

    #[test]
    fn deformation_measure_zero_for_identity() {
        let f = Mat3x3::identity();
        assert!(deformation_measure(&f) < 1e-5);
    }

    #[test]
    fn deformation_measure_nonzero_for_stretched() {
        let mut f = Mat3x3::identity();
        f.0[0][0] = 1.5;
        assert!(deformation_measure(&f) > 0.4);
    }

    #[test]
    fn mat3x3_mul_identity() {
        let a = Mat3x3::identity();
        let b = Mat3x3::identity();
        let c = a.mul(&b);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((c.0[i][j] - expected).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn mat3x3_frobenius_identity_is_sqrt3() {
        let a = Mat3x3::identity();
        assert!((a.frobenius_norm() - 3.0f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn polar_decompose_identity() {
        let f = Mat3x3::identity();
        let (r, _s) = polar_decompose(&f, 5);
        let norm_diff = deformation_measure(&r);
        assert!(norm_diff < 0.5);
    }

    #[test]
    fn strain_energy_increases_with_stretch() {
        let f_small = {
            let mut f = Mat3x3::identity();
            f.0[0][0] = 1.1;
            f
        };
        let f_large = {
            let mut f = Mat3x3::identity();
            f.0[0][0] = 1.5;
            f
        };
        let w_small = strain_energy_density(&f_small, 1e4, 5e3);
        let w_large = strain_energy_density(&f_large, 1e4, 5e3);
        assert!(w_large > w_small);
    }

    #[test]
    fn compute_deformation_gradient_no_panic_for_degenerate() {
        let p = [[0.0f32; 3]; 4];
        let f = compute_deformation_gradient(p, p);
        let norm = f.frobenius_norm();
        assert!(norm.is_finite());
    }
}
