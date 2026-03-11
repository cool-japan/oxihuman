// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Eyebrow arch height and position control.

#![allow(dead_code)]

/// Eyebrow arch parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BrowArch {
    /// Left brow arch height (0 = flat, 1 = high arch).
    pub left_arch: f32,
    /// Right brow arch height (0 = flat, 1 = high arch).
    pub right_arch: f32,
    /// Peak offset along the brow (-1 = toward nose, 1 = toward temple).
    pub peak_offset: f32,
}

/// Create a default `BrowArch` with moderate arch and centered peak.
#[allow(dead_code)]
pub fn default_brow_arch() -> BrowArch {
    BrowArch {
        left_arch: 0.5,
        right_arch: 0.5,
        peak_offset: 0.0,
    }
}

/// Apply brow arch to a weight slice.
///
/// Expects at least 3 entries: [left_arch, right_arch, peak_offset].
#[allow(dead_code)]
pub fn apply_brow_arch(weights: &mut [f32], ba: &BrowArch) {
    if !weights.is_empty() {
        weights[0] = ba.left_arch;
    }
    if weights.len() >= 2 {
        weights[1] = ba.right_arch;
    }
    if weights.len() >= 3 {
        weights[2] = ba.peak_offset;
    }
}

/// Linearly blend two `BrowArch` values.
#[allow(dead_code)]
pub fn brow_arch_blend(a: &BrowArch, b: &BrowArch, t: f32) -> BrowArch {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    BrowArch {
        left_arch: lerp(a.left_arch, b.left_arch),
        right_arch: lerp(a.right_arch, b.right_arch),
        peak_offset: lerp(a.peak_offset, b.peak_offset),
    }
}

/// Compute how symmetric the arch is (0 = fully asymmetric, 1 = perfectly symmetric).
#[allow(dead_code)]
pub fn arch_symmetry(ba: &BrowArch) -> f32 {
    let diff = (ba.left_arch - ba.right_arch).abs();
    (1.0 - diff).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let ba = default_brow_arch();
        assert_eq!(ba.left_arch, 0.5);
        assert_eq!(ba.right_arch, 0.5);
        assert_eq!(ba.peak_offset, 0.0);
    }

    #[test]
    fn test_apply_full() {
        let ba = BrowArch { left_arch: 0.7, right_arch: 0.8, peak_offset: -0.2 };
        let mut w = [0.0f32; 3];
        apply_brow_arch(&mut w, &ba);
        assert_eq!(w[0], 0.7);
        assert_eq!(w[1], 0.8);
        assert_eq!(w[2], -0.2);
    }

    #[test]
    fn test_apply_empty() {
        let ba = default_brow_arch();
        let mut w: [f32; 0] = [];
        apply_brow_arch(&mut w, &ba);
    }

    #[test]
    fn test_blend_zero() {
        let a = BrowArch { left_arch: 0.2, right_arch: 0.3, peak_offset: -0.5 };
        let b = BrowArch { left_arch: 0.8, right_arch: 0.7, peak_offset: 0.5 };
        let r = brow_arch_blend(&a, &b, 0.0);
        assert!((r.left_arch - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = BrowArch { left_arch: 0.2, right_arch: 0.3, peak_offset: -0.5 };
        let b = BrowArch { left_arch: 0.8, right_arch: 0.7, peak_offset: 0.5 };
        let r = brow_arch_blend(&a, &b, 1.0);
        assert!((r.peak_offset - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_arch_symmetry_perfect() {
        let ba = BrowArch { left_arch: 0.5, right_arch: 0.5, peak_offset: 0.0 };
        assert!((arch_symmetry(&ba) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_arch_symmetry_asymmetric() {
        let ba = BrowArch { left_arch: 0.0, right_arch: 1.0, peak_offset: 0.0 };
        assert!((arch_symmetry(&ba)).abs() < 1e-6);
    }

    #[test]
    fn test_arch_symmetry_partial() {
        let ba = BrowArch { left_arch: 0.3, right_arch: 0.7, peak_offset: 0.0 };
        let sym = arch_symmetry(&ba);
        assert!(sym > 0.0 && sym < 1.0);
    }
}
