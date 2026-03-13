#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Subtle facial asymmetry micromorph.

/// Represents small left/right offsets for facial asymmetry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MicroAsymmetry {
    pub left_offset: Vec<f32>,
    pub right_offset: Vec<f32>,
    /// Overall magnitude scalar applied on top of the offsets.
    pub magnitude: f32,
}

/// Create a default `MicroAsymmetry` with `n_morphs` entries per side.
///
/// Uses a deterministic pattern derived from the index.
#[allow(dead_code)]
pub fn default_micro_asymmetry(n_morphs: usize) -> MicroAsymmetry {
    use std::f32::consts::PI;
    let left_offset = (0..n_morphs)
        .map(|i| ((i as f32) * PI / (n_morphs.max(1) as f32)).sin() * 0.02)
        .collect();
    let right_offset = (0..n_morphs)
        .map(|i| -((i as f32) * PI / (n_morphs.max(1) as f32)).sin() * 0.015)
        .collect();
    MicroAsymmetry {
        left_offset,
        right_offset,
        magnitude: 1.0,
    }
}

/// Apply the asymmetry offsets to a morph-weight slice.
///
/// The slice is assumed to hold left-side weights in the first half and
/// right-side weights in the second half.  Offsets are added scaled by
/// `ma.magnitude`.
#[allow(dead_code)]
pub fn apply_micro_asymmetry(weights: &mut [f32], ma: &MicroAsymmetry) {
    let n = weights.len() / 2;
    for i in 0..n {
        if i < ma.left_offset.len() {
            weights[i] += ma.left_offset[i] * ma.magnitude;
        }
        if i < ma.right_offset.len() {
            weights[n + i] += ma.right_offset[i] * ma.magnitude;
        }
    }
}

/// Return the RMS magnitude of all offsets.
#[allow(dead_code)]
pub fn asymmetry_magnitude(ma: &MicroAsymmetry) -> f32 {
    let all: Vec<f32> = ma.left_offset.iter().chain(ma.right_offset.iter()).copied().collect();
    if all.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = all.iter().map(|x| x * x).sum();
    (sum_sq / all.len() as f32).sqrt()
}

/// Return a `MicroAsymmetry` with all-zero offsets of length `n`.
#[allow(dead_code)]
pub fn zero_asymmetry(n: usize) -> MicroAsymmetry {
    MicroAsymmetry {
        left_offset: vec![0.0; n],
        right_offset: vec![0.0; n],
        magnitude: 0.0,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_asymmetry_is_all_zeros() {
        let ma = zero_asymmetry(4);
        assert!(ma.left_offset.iter().all(|&v| v.abs() < 1e-9));
        assert!(ma.right_offset.iter().all(|&v| v.abs() < 1e-9));
        assert!((ma.magnitude).abs() < 1e-9);
    }

    #[test]
    fn default_micro_asymmetry_length() {
        let ma = default_micro_asymmetry(6);
        assert_eq!(ma.left_offset.len(), 6);
        assert_eq!(ma.right_offset.len(), 6);
    }

    #[test]
    fn default_micro_asymmetry_zero_count() {
        let ma = default_micro_asymmetry(0);
        assert!(ma.left_offset.is_empty());
        assert!(ma.right_offset.is_empty());
    }

    #[test]
    fn asymmetry_magnitude_zero() {
        let ma = zero_asymmetry(4);
        assert!(asymmetry_magnitude(&ma).abs() < 1e-6);
    }

    #[test]
    fn asymmetry_magnitude_positive() {
        let ma = default_micro_asymmetry(4);
        let mag = asymmetry_magnitude(&ma);
        assert!(mag >= 0.0);
    }

    #[test]
    fn apply_micro_asymmetry_modifies_weights() {
        let ma = default_micro_asymmetry(4);
        let mut w = vec![0.0_f32; 8];
        apply_micro_asymmetry(&mut w, &ma);
        // At least one weight should be non-zero
        assert!(w.iter().any(|&v| v.abs() > 1e-7));
    }

    #[test]
    fn apply_micro_asymmetry_zero_no_change() {
        let ma = zero_asymmetry(4);
        let mut w = vec![0.5_f32; 8];
        apply_micro_asymmetry(&mut w, &ma);
        assert!(w.iter().all(|&v| (v - 0.5).abs() < 1e-9));
    }

    #[test]
    fn apply_micro_asymmetry_short_slice() {
        let ma = default_micro_asymmetry(4);
        let mut w = vec![0.0_f32; 2];
        apply_micro_asymmetry(&mut w, &ma); // must not panic
    }

    #[test]
    fn asymmetry_magnitude_empty() {
        let ma = MicroAsymmetry { left_offset: vec![], right_offset: vec![], magnitude: 1.0 };
        assert!((asymmetry_magnitude(&ma)).abs() < 1e-9);
    }

    #[test]
    fn zero_asymmetry_n_zero() {
        let ma = zero_asymmetry(0);
        assert!(ma.left_offset.is_empty());
        assert!(ma.right_offset.is_empty());
    }
}
