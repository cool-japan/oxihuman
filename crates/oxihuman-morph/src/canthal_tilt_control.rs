// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Canthal tilt — outer vs inner eye corner height.

#![allow(dead_code)]

/// Canthal tilt parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CanthalTilt {
    /// Left eye canthal tilt (-1 = negative, 0 = neutral, 1 = positive).
    pub left_tilt: f32,
    /// Right eye canthal tilt (-1 = negative, 0 = neutral, 1 = positive).
    pub right_tilt: f32,
}

/// Create a default `CanthalTilt` with neutral values.
#[allow(dead_code)]
pub fn default_canthal_tilt() -> CanthalTilt {
    CanthalTilt {
        left_tilt: 0.0,
        right_tilt: 0.0,
    }
}

/// Apply canthal tilt to a weight slice.
///
/// Expects at least 2 entries: [left_tilt, right_tilt].
#[allow(dead_code)]
pub fn apply_canthal_tilt(weights: &mut [f32], ct: &CanthalTilt) {
    if !weights.is_empty() {
        weights[0] = ct.left_tilt;
    }
    if weights.len() >= 2 {
        weights[1] = ct.right_tilt;
    }
}

/// Linearly blend two `CanthalTilt` values.
#[allow(dead_code)]
pub fn canthal_tilt_blend(a: &CanthalTilt, b: &CanthalTilt, t: f32) -> CanthalTilt {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    CanthalTilt {
        left_tilt: lerp(a.left_tilt, b.left_tilt),
        right_tilt: lerp(a.right_tilt, b.right_tilt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let ct = default_canthal_tilt();
        assert_eq!(ct.left_tilt, 0.0);
        assert_eq!(ct.right_tilt, 0.0);
    }

    #[test]
    fn test_apply_both() {
        let ct = CanthalTilt { left_tilt: 0.5, right_tilt: -0.3 };
        let mut w = [0.0f32; 2];
        apply_canthal_tilt(&mut w, &ct);
        assert_eq!(w[0], 0.5);
        assert_eq!(w[1], -0.3);
    }

    #[test]
    fn test_apply_one() {
        let ct = CanthalTilt { left_tilt: 0.7, right_tilt: 0.2 };
        let mut w = [0.0f32; 1];
        apply_canthal_tilt(&mut w, &ct);
        assert_eq!(w[0], 0.7);
    }

    #[test]
    fn test_apply_empty() {
        let ct = default_canthal_tilt();
        let mut w: [f32; 0] = [];
        apply_canthal_tilt(&mut w, &ct);
    }

    #[test]
    fn test_blend_zero() {
        let a = CanthalTilt { left_tilt: -1.0, right_tilt: -1.0 };
        let b = CanthalTilt { left_tilt: 1.0, right_tilt: 1.0 };
        let r = canthal_tilt_blend(&a, &b, 0.0);
        assert!((r.left_tilt - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = CanthalTilt { left_tilt: -1.0, right_tilt: -1.0 };
        let b = CanthalTilt { left_tilt: 1.0, right_tilt: 1.0 };
        let r = canthal_tilt_blend(&a, &b, 1.0);
        assert!((r.right_tilt - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = CanthalTilt { left_tilt: -1.0, right_tilt: -1.0 };
        let b = CanthalTilt { left_tilt: 1.0, right_tilt: 1.0 };
        let r = canthal_tilt_blend(&a, &b, 0.5);
        assert!(r.left_tilt.abs() < 1e-6);
    }

    #[test]
    fn test_clone_eq() {
        let ct = CanthalTilt { left_tilt: 0.3, right_tilt: 0.4 };
        assert_eq!(ct.clone(), ct);
    }

    #[test]
    fn test_negative_tilt() {
        let ct = CanthalTilt { left_tilt: -0.8, right_tilt: -0.5 };
        let mut w = [0.0f32; 2];
        apply_canthal_tilt(&mut w, &ct);
        assert!(w[0] < 0.0);
        assert!(w[1] < 0.0);
    }
}
