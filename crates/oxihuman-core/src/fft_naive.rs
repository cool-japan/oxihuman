// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Naive DFT/IDFT (O(n²)) for correctness reference.

#![allow(dead_code)]

use std::f64::consts::PI;

/// Complex number (f64).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    #[allow(dead_code)]
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[allow(dead_code)]
    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    #[allow(dead_code)]
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self { re: r * theta.cos(), im: r * theta.sin() }
    }

    #[allow(dead_code)]
    pub fn abs(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    #[allow(dead_code)]
    pub fn conj(&self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    #[allow(dead_code)]
    pub fn add(&self, other: &Self) -> Self {
        Self { re: self.re + other.re, im: self.im + other.im }
    }

    #[allow(dead_code)]
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

/// Naive DFT: X[k] = sum_{n=0}^{N-1} x[n] * exp(-2πi*k*n/N)
#[allow(dead_code)]
pub fn dft(input: &[Complex]) -> Vec<Complex> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    (0..n)
        .map(|k| {
            let mut sum = Complex::zero();
            for (j, &x) in input.iter().enumerate() {
                let angle = -2.0 * PI * k as f64 * j as f64 / n as f64;
                let w = Complex::from_polar(1.0, angle);
                sum = sum.add(&x.mul(&w));
            }
            sum
        })
        .collect()
}

/// Naive IDFT: x[n] = (1/N) * sum_{k=0}^{N-1} X[k] * exp(2πi*k*n/N)
#[allow(dead_code)]
pub fn idft(input: &[Complex]) -> Vec<Complex> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let scale = 1.0 / n as f64;
    (0..n)
        .map(|j| {
            let mut sum = Complex::zero();
            for (k, &x) in input.iter().enumerate() {
                let angle = 2.0 * PI * k as f64 * j as f64 / n as f64;
                let w = Complex::from_polar(1.0, angle);
                sum = sum.add(&x.mul(&w));
            }
            Complex::new(sum.re * scale, sum.im * scale)
        })
        .collect()
}

/// Compute power spectrum (|X[k]|^2).
#[allow(dead_code)]
pub fn power_spectrum(spectrum: &[Complex]) -> Vec<f64> {
    spectrum.iter().map(|c| c.abs() * c.abs()).collect()
}

/// Real-input convenience: wrap f64 slice as Complex with im=0.
#[allow(dead_code)]
pub fn real_to_complex(data: &[f64]) -> Vec<Complex> {
    data.iter().map(|&r| Complex::new(r, 0.0)).collect()
}

/// Extract real parts from complex output.
#[allow(dead_code)]
pub fn complex_to_real(data: &[Complex]) -> Vec<f64> {
    data.iter().map(|c| c.re).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dft_idft_roundtrip() {
        let input: Vec<Complex> = [1.0, 2.0, 3.0, 4.0f64]
            .iter()
            .map(|&r| Complex::new(r, 0.0))
            .collect();
        let spectrum = dft(&input);
        let reconstructed = idft(&spectrum);
        for (a, b) in input.iter().zip(reconstructed.iter()) {
            assert!((a.re - b.re).abs() < 1e-9, "re mismatch {} vs {}", a.re, b.re);
            assert!((a.im - b.im).abs() < 1e-9, "im mismatch {} vs {}", a.im, b.im);
        }
    }

    #[test]
    fn test_dc_component() {
        let n = 4;
        let input: Vec<Complex> = vec![Complex::new(1.0, 0.0); n];
        let spectrum = dft(&input);
        assert!((spectrum[0].re - 4.0).abs() < 1e-9);
        for &s in &spectrum[1..] {
            assert!(s.abs() < 1e-9);
        }
    }

    #[test]
    fn test_empty() {
        assert!(dft(&[]).is_empty());
        assert!(idft(&[]).is_empty());
    }

    #[test]
    fn test_complex_mul() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);
        let c = a.mul(&b);
        assert!((c.re - (-5.0)).abs() < 1e-9);
        assert!((c.im - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_complex_abs() {
        let c = Complex::new(3.0, 4.0);
        assert!((c.abs() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_power_spectrum() {
        let spectrum = vec![Complex::new(3.0, 4.0), Complex::new(0.0, 0.0)];
        let ps = power_spectrum(&spectrum);
        assert!((ps[0] - 25.0).abs() < 1e-9);
        assert!((ps[1]).abs() < 1e-9);
    }

    #[test]
    fn test_real_to_complex() {
        let data = [1.0, 2.0f64];
        let c = real_to_complex(&data);
        assert_eq!(c[0].re, 1.0);
        assert_eq!(c[0].im, 0.0);
    }

    #[test]
    fn test_complex_to_real() {
        let data = vec![Complex::new(3.0, 1.0), Complex::new(5.0, 2.0)];
        let r = complex_to_real(&data);
        assert_eq!(r, vec![3.0, 5.0]);
    }

    #[test]
    fn test_single_element() {
        let input = vec![Complex::new(7.0, 3.0)];
        let s = dft(&input);
        assert_eq!(s.len(), 1);
        assert!((s[0].re - 7.0).abs() < 1e-9);
        assert!((s[0].im - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_conj() {
        let c = Complex::new(2.0, -3.0);
        let conj = c.conj();
        assert_eq!(conj.re, 2.0);
        assert_eq!(conj.im, 3.0);
    }
}
