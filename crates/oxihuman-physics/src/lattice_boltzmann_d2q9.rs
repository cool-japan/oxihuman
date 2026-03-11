// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/* D2Q9 velocity set:
q: 0=rest, 1-4=cardinal, 5-8=diagonal */
const EX: [f32; 9] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, -1.0, -1.0, 1.0];
const EY: [f32; 9] = [0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 1.0, -1.0, -1.0];
const W: [f32; 9] = [
    4.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
];

pub struct LbmD2q9 {
    pub f: Vec<[f32; 9]>,
    pub width: usize,
    pub height: usize,
    pub tau: f32,
}

pub fn new_lbm_d2q9(w: usize, h: usize, tau: f32) -> LbmD2q9 {
    let n = w * h;
    let mut f = vec![[0.0f32; 9]; n];
    /* initialize to equilibrium with rho=1, u=0 */
    for cell in f.iter_mut() {
        for (q, wq) in W.iter().enumerate() {
            cell[q] = *wq;
        }
    }
    LbmD2q9 {
        f,
        width: w,
        height: h,
        tau,
    }
}

pub fn lbm_density(l: &LbmD2q9, x: usize, y: usize) -> f32 {
    let i = y * l.width + x;
    l.f[i].iter().sum()
}

pub fn lbm_velocity_x(l: &LbmD2q9, x: usize, y: usize) -> f32 {
    let i = y * l.width + x;
    let rho = lbm_density(l, x, y);
    if rho < 1e-12 {
        return 0.0;
    }
    let mom: f32 = l.f[i].iter().zip(EX.iter()).map(|(f, e)| f * e).sum();
    mom / rho
}

pub fn lbm_equilibrium(rho: f32, ux: f32, uy: f32, q: usize) -> f32 {
    let eu = EX[q] * ux + EY[q] * uy;
    let u2 = ux * ux + uy * uy;
    W[q] * rho * (1.0 + 3.0 * eu + 4.5 * eu * eu - 1.5 * u2)
}

pub fn lbm_step(l: &mut LbmD2q9) {
    let w = l.width;
    let h = l.height;
    /* Collide only (BGK) */
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let rho: f32 = l.f[i].iter().sum();
            let ux = if rho > 1e-12 {
                l.f[i]
                    .iter()
                    .zip(EX.iter())
                    .map(|(f, e)| f * e)
                    .sum::<f32>()
                    / rho
            } else {
                0.0
            };
            let uy = if rho > 1e-12 {
                l.f[i]
                    .iter()
                    .zip(EY.iter())
                    .map(|(f, e)| f * e)
                    .sum::<f32>()
                    / rho
            } else {
                0.0
            };
            for q in 0..9 {
                let feq = lbm_equilibrium(rho, ux, uy, q);
                l.f[i][q] += -(l.f[i][q] - feq) / l.tau;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lbm_d2q9() {
        /* initialized grid has correct dimensions */
        let l = new_lbm_d2q9(4, 4, 1.0);
        assert_eq!(l.f.len(), 16);
    }

    #[test]
    fn test_lbm_density_unity() {
        /* initial density should be 1.0 */
        let l = new_lbm_d2q9(4, 4, 1.0);
        let rho = lbm_density(&l, 0, 0);
        assert!((rho - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_lbm_velocity_x_zero() {
        /* initial x-velocity should be 0 */
        let l = new_lbm_d2q9(4, 4, 1.0);
        let vx = lbm_velocity_x(&l, 2, 2);
        assert!(vx.abs() < 1e-5);
    }

    #[test]
    fn test_lbm_equilibrium_rest() {
        /* rho=1, u=0, rest direction q=0 */
        let feq = lbm_equilibrium(1.0, 0.0, 0.0, 0);
        assert!((feq - W[0]).abs() < 1e-6);
    }

    #[test]
    fn test_lbm_step_no_panic() {
        /* step runs without panic */
        let mut l = new_lbm_d2q9(4, 4, 0.8);
        lbm_step(&mut l);
        assert!(lbm_density(&l, 0, 0).is_finite());
    }

    #[test]
    fn test_lbm_dimensions() {
        /* width and height stored correctly */
        let l = new_lbm_d2q9(6, 3, 1.0);
        assert_eq!(l.width, 6);
        assert_eq!(l.height, 3);
    }
}
