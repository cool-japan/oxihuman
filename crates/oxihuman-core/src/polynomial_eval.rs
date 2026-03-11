// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Polynomial evaluation (Horner's method) and Newton root-finding.

/// Evaluate polynomial with coefficients `coeffs` (lowest degree first) at `x` using Horner's method.
pub fn poly_eval(coeffs: &[f64], x: f64) -> f64 {
    if coeffs.is_empty() {
        return 0.0;
    }
    let mut result = 0.0f64;
    for &c in coeffs.iter().rev() {
        result = result * x + c;
    }
    result
}

/// Evaluate the derivative of the polynomial at `x`.
pub fn poly_deriv(coeffs: &[f64], x: f64) -> f64 {
    if coeffs.len() < 2 {
        return 0.0;
    }
    /* derivative coefficients: c[i+1]*(i+1) */
    let deriv: Vec<f64> = coeffs[1..]
        .iter()
        .enumerate()
        .map(|(i, &c)| c * (i + 1) as f64)
        .collect();
    poly_eval(&deriv, x)
}

/// A polynomial with its coefficients.
pub struct Polynomial {
    pub coeffs: Vec<f64>,
}

/// Construct a polynomial from coefficients (lowest degree first).
pub fn new_polynomial(coeffs: Vec<f64>) -> Polynomial {
    Polynomial { coeffs }
}

impl Polynomial {
    /// Evaluate at `x`.
    pub fn eval(&self, x: f64) -> f64 {
        poly_eval(&self.coeffs, x)
    }

    /// Evaluate derivative at `x`.
    pub fn deriv_at(&self, x: f64) -> f64 {
        poly_deriv(&self.coeffs, x)
    }

    /// Degree of the polynomial.
    pub fn degree(&self) -> usize {
        if self.coeffs.is_empty() {
            0
        } else {
            self.coeffs.len() - 1
        }
    }

    /// Find a root near `x0` using Newton-Raphson iteration.
    /// Returns None if convergence not achieved within `max_iter`.
    pub fn newton_root(&self, x0: f64, max_iter: usize, tol: f64) -> Option<f64> {
        let mut x = x0;
        for _ in 0..max_iter {
            let fx = self.eval(x);
            if fx.abs() < tol {
                return Some(x);
            }
            let dfx = self.deriv_at(x);
            if dfx.abs() < 1e-15 {
                return None;
            }
            x -= fx / dfx;
        }
        if self.eval(x).abs() < tol {
            Some(x)
        } else {
            None
        }
    }

    /// Multiply two polynomials.
    pub fn mul(&self, other: &Polynomial) -> Polynomial {
        let n = self.coeffs.len();
        let m = other.coeffs.len();
        if n == 0 || m == 0 {
            return Polynomial { coeffs: vec![] };
        }
        let mut result = vec![0.0f64; n + m - 1];
        for (i, &a) in self.coeffs.iter().enumerate() {
            for (j, &b) in other.coeffs.iter().enumerate() {
                result[i + j] += a * b;
            }
        }
        Polynomial { coeffs: result }
    }
}

/// Horner evaluation (free function).
pub fn horner_eval(coeffs: &[f64], x: f64) -> f64 {
    poly_eval(coeffs, x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly_eval_constant() {
        /* constant polynomial [5] evaluates to 5 */
        assert!((poly_eval(&[5.0], 999.0) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_poly_eval_linear() {
        /* [1, 2] = 1 + 2x; at x=3 -> 7 */
        assert!((poly_eval(&[1.0, 2.0], 3.0) - 7.0).abs() < 1e-12);
    }

    #[test]
    fn test_poly_eval_quadratic() {
        /* [0, 0, 1] = x^2; at x=4 -> 16 */
        assert!((poly_eval(&[0.0, 0.0, 1.0], 4.0) - 16.0).abs() < 1e-12);
    }

    #[test]
    fn test_poly_deriv() {
        /* deriv of x^2 = 2x; at x=3 -> 6 */
        assert!((poly_deriv(&[0.0, 0.0, 1.0], 3.0) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_newton_root_sqrt2() {
        /* x^2 - 2 has root sqrt(2) */
        let p = new_polynomial(vec![-2.0, 0.0, 1.0]);
        let root = p.newton_root(1.5, 50, 1e-10).unwrap();
        assert!((root - 2.0f64.sqrt()).abs() < 1e-8);
    }

    #[test]
    fn test_degree() {
        /* degree of [1, 2, 3] is 2 */
        let p = new_polynomial(vec![1.0, 2.0, 3.0]);
        assert_eq!(p.degree(), 2);
    }

    #[test]
    fn test_poly_mul() {
        /* (1 + x)(1 - x) = 1 - x^2 => coeffs [-1, 0, 1] wait => (x+1)(−x+1) = 1-x^2 */
        let p1 = new_polynomial(vec![1.0, 1.0]); /* 1 + x */
        let p2 = new_polynomial(vec![1.0, -1.0]); /* 1 - x */
        let prod = p1.mul(&p2);
        /* result: 1 - x^2 => [1, 0, -1] */
        assert!((prod.coeffs[0] - 1.0).abs() < 1e-12);
        assert!(prod.coeffs[1].abs() < 1e-12);
        assert!((prod.coeffs[2] - (-1.0)).abs() < 1e-12);
    }

    #[test]
    fn test_horner_eval() {
        /* horner_eval is consistent with poly_eval */
        let coeffs = vec![1.0, -3.0, 2.0];
        let x = 5.0;
        assert!((horner_eval(&coeffs, x) - poly_eval(&coeffs, x)).abs() < 1e-12);
    }

    #[test]
    fn test_empty_poly_eval() {
        /* empty coefficients evaluates to 0 */
        assert_eq!(poly_eval(&[], 10.0), 0.0);
    }
}
