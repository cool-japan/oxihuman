// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Haar wavelet transform 1D.

#![allow(dead_code)]

/// Perform in-place 1D Haar wavelet forward transform.
/// Input length must be a power of two.
#[allow(dead_code)]
pub fn haar_forward(data: &mut [f64]) {
    let n = data.len();
    if n < 2 || !n.is_power_of_two() {
        return;
    }
    let mut len = n;
    while len >= 2 {
        haar_step_forward(&mut data[..len]);
        len /= 2;
    }
}

fn haar_step_forward(data: &mut [f64]) {
    let half = data.len() / 2;
    let mut temp = vec![0.0f64; data.len()];
    let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;
    for i in 0..half {
        temp[i] = (data[2 * i] + data[2 * i + 1]) * sqrt2_inv;
        temp[half + i] = (data[2 * i] - data[2 * i + 1]) * sqrt2_inv;
    }
    data.copy_from_slice(&temp);
}

/// Perform in-place 1D Haar wavelet inverse transform.
#[allow(dead_code)]
pub fn haar_inverse(data: &mut [f64]) {
    let n = data.len();
    if n < 2 || !n.is_power_of_two() {
        return;
    }
    let mut len = 2usize;
    while len <= n {
        haar_step_inverse(&mut data[..len]);
        len *= 2;
    }
}

fn haar_step_inverse(data: &mut [f64]) {
    let half = data.len() / 2;
    let mut temp = vec![0.0f64; data.len()];
    let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;
    for i in 0..half {
        temp[2 * i] = (data[i] + data[half + i]) * sqrt2_inv;
        temp[2 * i + 1] = (data[i] - data[half + i]) * sqrt2_inv;
    }
    data.copy_from_slice(&temp);
}

/// Compute energy of signal (sum of squares).
#[allow(dead_code)]
pub fn signal_energy(data: &[f64]) -> f64 {
    data.iter().map(|&x| x * x).sum()
}

/// Zero out detail coefficients below threshold (hard thresholding).
#[allow(dead_code)]
pub fn hard_threshold(data: &mut [f64], threshold: f64, keep_approx: usize) {
    for v in data.iter_mut().skip(keep_approx) {
        if v.abs() < threshold {
            *v = 0.0;
        }
    }
}

/// Count non-zero coefficients.
#[allow(dead_code)]
pub fn nonzero_count(data: &[f64]) -> usize {
    data.iter().filter(|&&x| x.abs() > 1e-12).count()
}

/// Check if length is valid (power of two, >= 2).
#[allow(dead_code)]
pub fn is_valid_length(n: usize) -> bool {
    n >= 2 && n.is_power_of_two()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_inverse_roundtrip() {
        let original = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut data = original;
        haar_forward(&mut data);
        haar_inverse(&mut data);
        for (a, b) in original.iter().zip(data.iter()) {
            assert!((a - b).abs() < 1e-10, "mismatch {a} vs {b}");
        }
    }

    #[test]
    fn test_constant_signal() {
        let mut data = [2.0f64; 8];
        haar_forward(&mut data);
        for &v in &data[1..] {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_energy_preserved() {
        let original = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0f64];
        let e0 = signal_energy(&original);
        let mut data = original;
        haar_forward(&mut data);
        let e1 = signal_energy(&data);
        assert!((e0 - e1).abs() < 1e-9, "energy not preserved: {e0} vs {e1}");
    }

    #[test]
    fn test_invalid_length_noop() {
        let mut data = [1.0, 2.0, 3.0];
        let copy = data;
        haar_forward(&mut data);
        assert_eq!(data, copy);
    }

    #[test]
    fn test_length_two() {
        let original = [3.0, 5.0f64];
        let mut data = original;
        haar_forward(&mut data);
        haar_inverse(&mut data);
        for (a, b) in original.iter().zip(data.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_hard_threshold() {
        let mut data = [10.0, 0.01, -0.005, 3.0f64];
        hard_threshold(&mut data, 0.1, 1);
        assert_eq!(data[1], 0.0);
        assert_eq!(data[2], 0.0);
        assert_eq!(data[3], 3.0);
    }

    #[test]
    fn test_nonzero_count() {
        let data = [1.0, 0.0, -2.0, 0.0, 3.0f64];
        assert_eq!(nonzero_count(&data), 3);
    }

    #[test]
    fn test_is_valid_length() {
        assert!(is_valid_length(8));
        assert!(!is_valid_length(3));
        assert!(!is_valid_length(1));
        assert!(is_valid_length(2));
    }

    #[test]
    fn test_signal_energy() {
        let data = [3.0, 4.0f64];
        assert!((signal_energy(&data) - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_all_zeros() {
        let mut data = [0.0f64; 8];
        haar_forward(&mut data);
        assert!(data.iter().all(|&x| x.abs() < 1e-12));
    }
}
