// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lip protrusion (forward projection) control.

#![allow(dead_code)]

/// Lip protrusion parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LipProtrusion {
    /// Upper lip forward projection (0 = retracted, 1 = protruded).
    pub upper_protrusion: f32,
    /// Lower lip forward projection (0 = retracted, 1 = protruded).
    pub lower_protrusion: f32,
}

/// Create a default `LipProtrusion` with neutral values.
#[allow(dead_code)]
pub fn default_lip_protrusion() -> LipProtrusion {
    LipProtrusion {
        upper_protrusion: 0.0,
        lower_protrusion: 0.0,
    }
}

/// Apply lip protrusion to a weight slice.
///
/// Expects at least 2 entries: [upper_protrusion, lower_protrusion].
#[allow(dead_code)]
pub fn apply_lip_protrusion(weights: &mut [f32], lp: &LipProtrusion) {
    if !weights.is_empty() {
        weights[0] = lp.upper_protrusion;
    }
    if weights.len() >= 2 {
        weights[1] = lp.lower_protrusion;
    }
}

/// Linearly blend two `LipProtrusion` values.
#[allow(dead_code)]
pub fn lip_protrusion_blend(a: &LipProtrusion, b: &LipProtrusion, t: f32) -> LipProtrusion {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    LipProtrusion {
        upper_protrusion: lerp(a.upper_protrusion, b.upper_protrusion),
        lower_protrusion: lerp(a.lower_protrusion, b.lower_protrusion),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_no_protrusion() {
        let lp = default_lip_protrusion();
        assert_eq!(lp.upper_protrusion, 0.0);
        assert_eq!(lp.lower_protrusion, 0.0);
    }

    #[test]
    fn test_apply_both() {
        let lp = LipProtrusion { upper_protrusion: 0.6, lower_protrusion: 0.4 };
        let mut w = [0.0f32; 2];
        apply_lip_protrusion(&mut w, &lp);
        assert_eq!(w[0], 0.6);
        assert_eq!(w[1], 0.4);
    }

    #[test]
    fn test_apply_one() {
        let lp = LipProtrusion { upper_protrusion: 0.8, lower_protrusion: 0.2 };
        let mut w = [0.0f32; 1];
        apply_lip_protrusion(&mut w, &lp);
        assert_eq!(w[0], 0.8);
    }

    #[test]
    fn test_apply_empty() {
        let lp = default_lip_protrusion();
        let mut w: [f32; 0] = [];
        apply_lip_protrusion(&mut w, &lp);
    }

    #[test]
    fn test_blend_zero() {
        let a = LipProtrusion { upper_protrusion: 0.3, lower_protrusion: 0.2 };
        let b = LipProtrusion { upper_protrusion: 0.9, lower_protrusion: 0.8 };
        let r = lip_protrusion_blend(&a, &b, 0.0);
        assert!((r.upper_protrusion - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = LipProtrusion { upper_protrusion: 0.3, lower_protrusion: 0.2 };
        let b = LipProtrusion { upper_protrusion: 0.9, lower_protrusion: 0.8 };
        let r = lip_protrusion_blend(&a, &b, 1.0);
        assert!((r.lower_protrusion - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_blend_half() {
        let a = LipProtrusion { upper_protrusion: 0.0, lower_protrusion: 0.0 };
        let b = LipProtrusion { upper_protrusion: 1.0, lower_protrusion: 1.0 };
        let r = lip_protrusion_blend(&a, &b, 0.5);
        assert!((r.upper_protrusion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone_eq() {
        let lp = default_lip_protrusion();
        assert_eq!(lp.clone(), lp);
    }

    #[test]
    fn test_debug() {
        let lp = LipProtrusion { upper_protrusion: 0.1, lower_protrusion: 0.9 };
        let s = format!("{:?}", lp);
        assert!(s.contains("LipProtrusion"));
    }
}
