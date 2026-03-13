// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Epicanthic fold prominence control.

#![allow(dead_code)]

/// Epicanthic fold parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EpicanthicFold {
    /// Left eye fold prominence (0 = absent, 1 = prominent).
    pub left_fold: f32,
    /// Right eye fold prominence (0 = absent, 1 = prominent).
    pub right_fold: f32,
}

/// Create a default `EpicanthicFold` with no fold.
#[allow(dead_code)]
pub fn default_epicanthic_fold() -> EpicanthicFold {
    EpicanthicFold {
        left_fold: 0.0,
        right_fold: 0.0,
    }
}

/// Apply epicanthic fold to a weight slice.
///
/// Expects at least 2 entries: [left_fold, right_fold].
#[allow(dead_code)]
pub fn apply_epicanthic_fold(weights: &mut [f32], ef: &EpicanthicFold) {
    if !weights.is_empty() {
        weights[0] = ef.left_fold;
    }
    if weights.len() >= 2 {
        weights[1] = ef.right_fold;
    }
}

/// Linearly blend two `EpicanthicFold` values.
#[allow(dead_code)]
pub fn epicanthic_blend(a: &EpicanthicFold, b: &EpicanthicFold, t: f32) -> EpicanthicFold {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    EpicanthicFold {
        left_fold: lerp(a.left_fold, b.left_fold),
        right_fold: lerp(a.right_fold, b.right_fold),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_no_fold() {
        let ef = default_epicanthic_fold();
        assert_eq!(ef.left_fold, 0.0);
        assert_eq!(ef.right_fold, 0.0);
    }

    #[test]
    fn test_apply_full() {
        let ef = EpicanthicFold { left_fold: 0.7, right_fold: 0.8 };
        let mut w = [0.0f32; 2];
        apply_epicanthic_fold(&mut w, &ef);
        assert_eq!(w[0], 0.7);
        assert_eq!(w[1], 0.8);
    }

    #[test]
    fn test_apply_single() {
        let ef = EpicanthicFold { left_fold: 0.5, right_fold: 0.3 };
        let mut w = [0.0f32; 1];
        apply_epicanthic_fold(&mut w, &ef);
        assert_eq!(w[0], 0.5);
    }

    #[test]
    fn test_apply_empty() {
        let ef = default_epicanthic_fold();
        let mut w: [f32; 0] = [];
        apply_epicanthic_fold(&mut w, &ef);
    }

    #[test]
    fn test_blend_at_zero() {
        let a = EpicanthicFold { left_fold: 0.2, right_fold: 0.4 };
        let b = EpicanthicFold { left_fold: 0.8, right_fold: 0.6 };
        let r = epicanthic_blend(&a, &b, 0.0);
        assert!((r.left_fold - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        let a = EpicanthicFold { left_fold: 0.2, right_fold: 0.4 };
        let b = EpicanthicFold { left_fold: 0.8, right_fold: 0.6 };
        let r = epicanthic_blend(&a, &b, 1.0);
        assert!((r.right_fold - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_quarter() {
        let a = EpicanthicFold { left_fold: 0.0, right_fold: 0.0 };
        let b = EpicanthicFold { left_fold: 1.0, right_fold: 1.0 };
        let r = epicanthic_blend(&a, &b, 0.25);
        assert!((r.left_fold - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_symmetric_blend() {
        let a = EpicanthicFold { left_fold: 0.0, right_fold: 0.0 };
        let b = EpicanthicFold { left_fold: 1.0, right_fold: 1.0 };
        let r = epicanthic_blend(&a, &b, 0.5);
        assert!((r.left_fold - r.right_fold).abs() < 1e-6);
    }

    #[test]
    fn test_clone_eq() {
        let ef = EpicanthicFold { left_fold: 0.6, right_fold: 0.4 };
        assert_eq!(ef.clone(), ef);
    }
}
