// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct ErrorDiffusionView {
    pub color_levels: u32,
    pub serpentine_scan: bool,
    pub strength: f32,
}

pub fn new_error_diffusion_view() -> ErrorDiffusionView {
    ErrorDiffusionView {
        color_levels: 256,
        serpentine_scan: true,
        strength: 1.0,
    }
}

pub fn ed_set_color_levels(v: &mut ErrorDiffusionView, n: u32) {
    v.color_levels = n.clamp(2, 256);
}

/// Floyd-Steinberg quantize a single channel value.
pub fn ed_quantize(v: &ErrorDiffusionView, value: f32) -> (f32, f32) {
    let levels = v.color_levels as f32;
    let quantized = (value * (levels - 1.0)).round() / (levels - 1.0);
    let error = (value - quantized) * v.strength;
    (quantized.clamp(0.0, 1.0), error)
}

/// Distribute error to neighbors (returns weights for right, down-left, down, down-right).
pub fn ed_floyd_steinberg_weights() -> (f32, f32, f32, f32) {
    (7.0 / 16.0, 3.0 / 16.0, 5.0 / 16.0, 1.0 / 16.0)
}

pub fn ed_is_high_fidelity(v: &ErrorDiffusionView) -> bool {
    v.color_levels >= 128
}

pub fn ed_blend(a: &ErrorDiffusionView, b: &ErrorDiffusionView, t: f32) -> ErrorDiffusionView {
    let t = t.clamp(0.0, 1.0);
    let cl = (a.color_levels as f32 + (b.color_levels as f32 - a.color_levels as f32) * t).round()
        as u32;
    ErrorDiffusionView {
        color_levels: cl.clamp(2, 256),
        serpentine_scan: if t < 0.5 {
            a.serpentine_scan
        } else {
            b.serpentine_scan
        },
        strength: a.strength + (b.strength - a.strength) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default color levels */
        let v = new_error_diffusion_view();
        assert_eq!(v.color_levels, 256);
    }

    #[test]
    fn test_quantize_zero() {
        /* zero quantizes to zero */
        let v = new_error_diffusion_view();
        let (q, _e) = ed_quantize(&v, 0.0);
        assert!(q.abs() < 1e-6);
    }

    #[test]
    fn test_floyd_steinberg_weights_sum() {
        /* weights sum to 1 */
        let (w0, w1, w2, w3) = ed_floyd_steinberg_weights();
        let sum = w0 + w1 + w2 + w3;
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_high_fidelity_by_default() {
        /* 256 levels is high fidelity */
        let v = new_error_diffusion_view();
        assert!(ed_is_high_fidelity(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint strength */
        let a = ErrorDiffusionView {
            color_levels: 16,
            serpentine_scan: false,
            strength: 0.0,
        };
        let b = ErrorDiffusionView {
            color_levels: 16,
            serpentine_scan: false,
            strength: 2.0,
        };
        let c = ed_blend(&a, &b, 0.5);
        assert!((c.strength - 1.0).abs() < 1e-5);
    }
}
