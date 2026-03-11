// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 1D Discrete Cosine Transform (DCT-II).

#![allow(dead_code)]

use std::f64::consts::PI;

/// Compute DCT-II of input, return result vector.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn dct2(input: &[f64]) -> Vec<f64> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let mut output = vec![0.0f64; n];
    for k in 0..n {
        let sum: f64 = input
            .iter()
            .enumerate()
            .map(|(i, &x)| x * (PI * k as f64 * (2 * i + 1) as f64 / (2 * n) as f64).cos())
            .sum();
        output[k] = sum;
    }
    output
}

/// Compute inverse DCT-II (DCT-III scaled) from DCT-II coefficients.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn idct2(coeffs: &[f64]) -> Vec<f64> {
    let n = coeffs.len();
    if n == 0 {
        return Vec::new();
    }
    let mut output = vec![0.0f64; n];
    for i in 0..n {
        let sum: f64 = coeffs
            .iter()
            .enumerate()
            .map(|(k, &c)| {
                let scale = if k == 0 { 1.0 } else { 2.0 };
                scale * c * (PI * k as f64 * (2 * i + 1) as f64 / (2 * n) as f64).cos()
            })
            .sum();
        output[i] = sum / (2 * n) as f64;
    }
    output
}

/// Orthonormal DCT-II (scaled).
#[allow(dead_code)]
pub fn dct2_ortho(input: &[f64]) -> Vec<f64> {
    let n = input.len();
    let mut coeffs = dct2(input);
    if n == 0 {
        return coeffs;
    }
    let sqrt2n = (2.0 * n as f64).sqrt();
    coeffs[0] /= (n as f64).sqrt();
    for c in coeffs.iter_mut().skip(1) {
        *c /= sqrt2n / (2.0f64).sqrt();
    }
    coeffs
}

/// Signal energy (sum of squares).
#[allow(dead_code)]
pub fn energy(data: &[f64]) -> f64 {
    data.iter().map(|&x| x * x).sum()
}

/// Zero out small-magnitude DCT coefficients.
#[allow(dead_code)]
pub fn truncate_small(coeffs: &mut [f64], threshold: f64) {
    for c in coeffs.iter_mut() {
        if c.abs() < threshold {
            *c = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dct_idct_roundtrip() {
        let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0f64];
        let coeffs = dct2(&input);
        let reconstructed = idct2(&coeffs);
        for (a, b) in input.iter().zip(reconstructed.iter()) {
            assert!((a - b).abs() < 1e-9, "mismatch {a} vs {b}");
        }
    }

    #[test]
    fn test_constant_signal() {
        let input = [3.0f64; 8];
        let coeffs = dct2(&input);
        for &c in &coeffs[1..] {
            assert!(c.abs() < 1e-9, "non-DC coefficient {c}");
        }
    }

    #[test]
    fn test_empty_input() {
        assert!(dct2(&[]).is_empty());
        assert!(idct2(&[]).is_empty());
    }

    #[test]
    fn test_single_element() {
        let input = [5.0f64];
        let c = dct2(&input);
        assert_eq!(c.len(), 1);
        assert!((c[0] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_energy_parseval() {
        let input = [1.0, 2.0, 3.0, 4.0f64];
        let coeffs = dct2(&input);
        let e_time = energy(&input);
        let e_freq: f64 = coeffs
            .iter()
            .enumerate()
            .map(|(k, &c)| if k == 0 { c * c / 4.0 } else { c * c / 2.0 })
            .sum();
        assert!((e_time - e_freq).abs() < 1e-6, "parseval: {e_time} vs {e_freq}");
    }

    #[test]
    fn test_dct_length() {
        let input = [1.0f64; 16];
        assert_eq!(dct2(&input).len(), 16);
    }

    #[test]
    fn test_truncate_small() {
        let mut coeffs = [0.001, 5.0, -0.0001, 3.0f64];
        truncate_small(&mut coeffs, 0.01);
        assert_eq!(coeffs[0], 0.0);
        assert_eq!(coeffs[1], 5.0);
    }

    #[test]
    fn test_ortho_length() {
        let input = [1.0, 2.0, 3.0, 4.0f64];
        assert_eq!(dct2_ortho(&input).len(), 4);
    }

    #[test]
    fn test_dct_linearity() {
        let a = [1.0, 0.0, 0.0, 0.0f64];
        let b = [0.0, 1.0, 0.0, 0.0f64];
        let ca = dct2(&a);
        let cb = dct2(&b);
        let ab: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
        let cab = dct2(&ab);
        for ((x, y), z) in ca.iter().zip(cb.iter()).zip(cab.iter()) {
            assert!((x + y - z).abs() < 1e-9);
        }
    }

    #[test]
    fn test_energy_function() {
        let data = [3.0, 4.0f64];
        assert!((energy(&data) - 25.0).abs() < 1e-10);
    }
}
