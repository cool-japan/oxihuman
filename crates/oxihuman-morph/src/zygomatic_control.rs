// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Zygomatic arch (cheekbone prominence) control.

/// Parameters controlling the zygomatic arch shape.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ZygomaticControl {
    /// How prominent the cheekbone appears, normalised 0..1.
    pub prominence: f32,
    /// Lateral width of the zygomatic arch, normalised 0..1.
    pub width: f32,
    /// Vertical position of peak prominence, normalised 0..1.
    pub height: f32,
    /// Arch angle relative to neutral, in normalised units.
    pub angle: f32,
}

/// Return a default (neutral) zygomatic control.
#[allow(dead_code)]
pub fn default_zygomatic_control() -> ZygomaticControl {
    ZygomaticControl {
        prominence: 0.5,
        width: 0.5,
        height: 0.5,
        angle: 0.0,
    }
}

/// Apply zygomatic parameters to a morph-weight slice.
#[allow(dead_code)]
pub fn apply_zygomatic_control(weights: &mut [f32], zc: &ZygomaticControl) {
    if !weights.is_empty() { weights[0] = zc.prominence; }
    if weights.len() > 1 { weights[1] = zc.width; }
    if weights.len() > 2 { weights[2] = zc.height; }
    if weights.len() > 3 { weights[3] = zc.angle; }
}

/// Linear blend between two zygomatic controls.
#[allow(dead_code)]
pub fn zygomatic_blend(a: &ZygomaticControl, b: &ZygomaticControl, t: f32) -> ZygomaticControl {
    let t = t.clamp(0.0, 1.0);
    ZygomaticControl {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width:      a.width      + (b.width      - a.width)      * t,
        height:     a.height     + (b.height     - a.height)     * t,
        angle:      a.angle      + (b.angle      - a.angle)      * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prominence_is_half() {
        let zc = default_zygomatic_control();
        assert_eq!(zc.prominence, 0.5);
    }

    #[test]
    fn default_angle_is_zero() {
        let zc = default_zygomatic_control();
        assert_eq!(zc.angle, 0.0);
    }

    #[test]
    fn apply_writes_four_weights() {
        let zc = ZygomaticControl { prominence: 0.1, width: 0.2, height: 0.3, angle: 0.4 };
        let mut w = [0.0_f32; 4];
        apply_zygomatic_control(&mut w, &zc);
        assert_eq!(w[0], 0.1);
        assert_eq!(w[1], 0.2);
        assert_eq!(w[2], 0.3);
        assert_eq!(w[3], 0.4);
    }

    #[test]
    fn apply_handles_short_slice() {
        let zc = ZygomaticControl { prominence: 0.9, width: 0.8, height: 0.7, angle: 0.6 };
        let mut w = [0.0_f32; 2];
        apply_zygomatic_control(&mut w, &zc);
        assert_eq!(w[0], 0.9);
        assert_eq!(w[1], 0.8);
    }

    #[test]
    fn blend_at_zero_returns_a() {
        let a = default_zygomatic_control();
        let b = ZygomaticControl { prominence: 1.0, width: 1.0, height: 1.0, angle: 1.0 };
        assert_eq!(zygomatic_blend(&a, &b, 0.0), a);
    }

    #[test]
    fn blend_at_one_returns_b() {
        let a = default_zygomatic_control();
        let b = ZygomaticControl { prominence: 1.0, width: 1.0, height: 1.0, angle: 1.0 };
        assert_eq!(zygomatic_blend(&a, &b, 1.0), b);
    }

    #[test]
    fn blend_midpoint_prominence() {
        let a = ZygomaticControl { prominence: 0.0, width: 0.0, height: 0.0, angle: 0.0 };
        let b = ZygomaticControl { prominence: 1.0, width: 1.0, height: 1.0, angle: 1.0 };
        let c = zygomatic_blend(&a, &b, 0.5);
        assert!((c.prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn blend_clamps_above_one() {
        let a = default_zygomatic_control();
        let b = ZygomaticControl { prominence: 1.0, width: 1.0, height: 1.0, angle: 1.0 };
        assert_eq!(zygomatic_blend(&a, &b, 2.0), b);
    }

    #[test]
    fn apply_empty_no_panic() {
        let zc = default_zygomatic_control();
        let mut w: [f32; 0] = [];
        apply_zygomatic_control(&mut w, &zc);
    }
}
