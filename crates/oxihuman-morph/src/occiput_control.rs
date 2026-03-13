// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Occiput (back of head) shape control.

/// Parameters controlling the shape of the occipital region.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OcciputControl {
    /// How far the occiput protrudes posteriorly, normalised 0..1.
    pub protrusion: f32,
    /// Lateral width of the occipital bone, normalised 0..1.
    pub width: f32,
    /// Flatness of the occiput surface (0 = rounded, 1 = flat).
    pub flatness: f32,
}

/// Return a default (neutral) occiput control.
#[allow(dead_code)]
pub fn default_occiput_control() -> OcciputControl {
    OcciputControl {
        protrusion: 0.5,
        width: 0.5,
        flatness: 0.0,
    }
}

/// Apply occiput parameters to a morph-weight slice.
#[allow(dead_code)]
pub fn apply_occiput_control(weights: &mut [f32], oc: &OcciputControl) {
    if !weights.is_empty() { weights[0] = oc.protrusion; }
    if weights.len() > 1 { weights[1] = oc.width; }
    if weights.len() > 2 { weights[2] = oc.flatness; }
}

/// Linear blend between two occiput controls.
#[allow(dead_code)]
pub fn occiput_blend(a: &OcciputControl, b: &OcciputControl, t: f32) -> OcciputControl {
    let t = t.clamp(0.0, 1.0);
    OcciputControl {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        width:      a.width      + (b.width      - a.width)      * t,
        flatness:   a.flatness   + (b.flatness   - a.flatness)   * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_protrusion_is_half() {
        let oc = default_occiput_control();
        assert_eq!(oc.protrusion, 0.5);
    }

    #[test]
    fn default_width_is_half() {
        let oc = default_occiput_control();
        assert_eq!(oc.width, 0.5);
    }

    #[test]
    fn default_flatness_is_zero() {
        let oc = default_occiput_control();
        assert_eq!(oc.flatness, 0.0);
    }

    #[test]
    fn apply_writes_three_weights() {
        let oc = OcciputControl { protrusion: 0.1, width: 0.2, flatness: 0.3 };
        let mut w = [0.0_f32; 3];
        apply_occiput_control(&mut w, &oc);
        assert_eq!(w[0], 0.1);
        assert_eq!(w[1], 0.2);
        assert_eq!(w[2], 0.3);
    }

    #[test]
    fn apply_handles_short_slice() {
        let oc = OcciputControl { protrusion: 0.7, width: 0.8, flatness: 0.9 };
        let mut w = [0.0_f32; 1];
        apply_occiput_control(&mut w, &oc);
        assert_eq!(w[0], 0.7);
    }

    #[test]
    fn blend_at_zero_returns_a() {
        let a = default_occiput_control();
        let b = OcciputControl { protrusion: 1.0, width: 1.0, flatness: 1.0 };
        assert_eq!(occiput_blend(&a, &b, 0.0), a);
    }

    #[test]
    fn blend_at_one_returns_b() {
        let a = default_occiput_control();
        let b = OcciputControl { protrusion: 1.0, width: 1.0, flatness: 1.0 };
        assert_eq!(occiput_blend(&a, &b, 1.0), b);
    }

    #[test]
    fn blend_midpoint() {
        let a = OcciputControl { protrusion: 0.0, width: 0.0, flatness: 0.0 };
        let b = OcciputControl { protrusion: 1.0, width: 1.0, flatness: 1.0 };
        let c = occiput_blend(&a, &b, 0.5);
        assert!((c.protrusion - 0.5).abs() < 1e-6);
        assert!((c.width - 0.5).abs() < 1e-6);
        assert!((c.flatness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn blend_clamps_below() {
        let a = default_occiput_control();
        let b = OcciputControl { protrusion: 1.0, width: 1.0, flatness: 1.0 };
        assert_eq!(occiput_blend(&a, &b, -0.5), a);
    }

    #[test]
    fn apply_empty_no_panic() {
        let oc = default_occiput_control();
        let mut w: [f32; 0] = [];
        apply_occiput_control(&mut w, &oc);
    }
}
