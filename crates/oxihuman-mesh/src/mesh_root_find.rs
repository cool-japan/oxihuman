// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a root-finding operation.
#[allow(dead_code)]
pub struct RootFindResult {
    pub root: f32,
    pub iterations: usize,
    pub converged: bool,
    pub residual: f32,
}

/// Bisection method for scalar function.
#[allow(dead_code)]
pub fn bisect<F>(f: F, mut lo: f32, mut hi: f32, tol: f32, max_iter: usize) -> RootFindResult
where
    F: Fn(f32) -> f32,
{
    let mut iters = 0;
    while iters < max_iter {
        let mid = (lo + hi) * 0.5;
        let fm = f(mid);
        if fm.abs() < tol || (hi - lo) < tol {
            return RootFindResult {
                root: mid,
                iterations: iters,
                converged: true,
                residual: fm.abs(),
            };
        }
        if f(lo) * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
        iters += 1;
    }
    let mid = (lo + hi) * 0.5;
    RootFindResult {
        root: mid,
        iterations: iters,
        converged: false,
        residual: f(mid).abs(),
    }
}

/// Newton–Raphson root finding.
#[allow(dead_code)]
pub fn newton_raphson<F, DF>(f: F, df: DF, mut x: f32, tol: f32, max_iter: usize) -> RootFindResult
where
    F: Fn(f32) -> f32,
    DF: Fn(f32) -> f32,
{
    let mut iters = 0;
    while iters < max_iter {
        let fx = f(x);
        if fx.abs() < tol {
            return RootFindResult {
                root: x,
                iterations: iters,
                converged: true,
                residual: fx.abs(),
            };
        }
        let dfx = df(x);
        if dfx.abs() < 1e-12 {
            break;
        }
        x -= fx / dfx;
        iters += 1;
    }
    let residual = f(x).abs();
    RootFindResult {
        root: x,
        iterations: iters,
        converged: false,
        residual,
    }
}

/// Regula falsi (false position) method.
#[allow(dead_code)]
pub fn regula_falsi<F>(f: F, mut a: f32, mut b: f32, tol: f32, max_iter: usize) -> RootFindResult
where
    F: Fn(f32) -> f32,
{
    let mut iters = 0;
    while iters < max_iter {
        let fa = f(a);
        let fb = f(b);
        if (fb - fa).abs() < 1e-14 {
            break;
        }
        let c = b - fb * (b - a) / (fb - fa);
        let fc = f(c);
        if fc.abs() < tol {
            return RootFindResult {
                root: c,
                iterations: iters,
                converged: true,
                residual: fc.abs(),
            };
        }
        if fa * fc < 0.0 {
            b = c;
        } else {
            a = c;
        }
        iters += 1;
    }
    let c = (a + b) * 0.5;
    RootFindResult {
        root: c,
        iterations: iters,
        converged: false,
        residual: f(c).abs(),
    }
}

/// Evaluate sign of a function at two points.
#[allow(dead_code)]
pub fn bracket_check<F>(f: &F, a: f32, b: f32) -> bool
where
    F: Fn(f32) -> f32,
{
    f(a) * f(b) < 0.0
}

/// Compute residual at a given point.
#[allow(dead_code)]
pub fn residual_at<F>(f: &F, x: f32) -> f32
where
    F: Fn(f32) -> f32,
{
    f(x).abs()
}

/// Serialize root find result to JSON.
#[allow(dead_code)]
pub fn root_find_to_json(r: &RootFindResult) -> String {
    format!(
        r#"{{"root":{:.6},"iterations":{},"converged":{}}}"#,
        r.root, r.iterations, r.converged
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn bisect_linear() {
        let r = bisect(|x| x - 2.0, 0.0, 4.0, 1e-5, 100);
        assert!((r.root - 2.0).abs() < 1e-4);
        assert!(r.converged);
    }

    #[test]
    fn bisect_quadratic() {
        let r = bisect(|x| x * x - 4.0, 0.0, 4.0, 1e-5, 100);
        assert!((r.root - 2.0).abs() < 1e-4);
    }

    #[test]
    fn newton_linear() {
        let r = newton_raphson(|x| x - 3.0, |_| 1.0, 0.0, 1e-6, 50);
        assert!((r.root - 3.0).abs() < 1e-5);
    }

    #[test]
    fn newton_sine_root() {
        let r = newton_raphson(|x| x.sin(), |x| x.cos(), 3.0, 1e-6, 50);
        assert!((r.root - PI).abs() < 1e-4 || r.root.abs() < 1e-4);
    }

    #[test]
    fn regula_falsi_linear() {
        let r = regula_falsi(|x| x - 1.5, 0.0, 3.0, 1e-5, 100);
        assert!((r.root - 1.5).abs() < 1e-4);
    }

    #[test]
    fn bracket_check_true() {
        assert!(bracket_check(&|x: f32| x - 2.0, 0.0, 4.0));
    }

    #[test]
    fn bracket_check_false() {
        assert!(!bracket_check(&|x: f32| x - 2.0, 3.0, 5.0));
    }

    #[test]
    fn residual_at_zero() {
        let r = residual_at(&|x: f32| x - 2.0, 2.0);
        assert!(r < 1e-6);
    }

    #[test]
    fn json_has_root() {
        let result = RootFindResult {
            root: 1.5,
            iterations: 3,
            converged: true,
            residual: 0.0,
        };
        let j = root_find_to_json(&result);
        assert!(j.contains("\"root\":1.500000"));
    }

    #[test]
    fn max_iter_reached() {
        let r = bisect(|_| 1.0_f32, 0.0, 1.0, 1e-12, 3);
        assert_eq!(r.iterations, 3);
    }
}
