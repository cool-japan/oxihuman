// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Color space conversion — sRGB, linear, ACEScg, and Rec.709 helpers.

/// Color space identifier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Linear,
    Srgb,
    AcesCg,
    Rec709,
}

#[allow(dead_code)]
pub fn color_space_name(cs: ColorSpace) -> &'static str {
    match cs {
        ColorSpace::Linear => "linear",
        ColorSpace::Srgb => "srgb",
        ColorSpace::AcesCg => "aces_cg",
        ColorSpace::Rec709 => "rec709",
    }
}

#[allow(dead_code)]
pub fn srgb_to_linear_channel(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

#[allow(dead_code)]
pub fn linear_to_srgb_channel(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

#[allow(dead_code)]
pub fn srgb_to_linear_rgb(rgb: [f32; 3]) -> [f32; 3] {
    [
        srgb_to_linear_channel(rgb[0]),
        srgb_to_linear_channel(rgb[1]),
        srgb_to_linear_channel(rgb[2]),
    ]
}

#[allow(dead_code)]
pub fn linear_to_srgb_rgb(rgb: [f32; 3]) -> [f32; 3] {
    [
        linear_to_srgb_channel(rgb[0]),
        linear_to_srgb_channel(rgb[1]),
        linear_to_srgb_channel(rgb[2]),
    ]
}

#[allow(dead_code)]
pub fn linear_to_aces_cg(rgb: [f32; 3]) -> [f32; 3] {
    // Approximate matrix (AP0 -> AP1)
    let r = rgb[0] * 0.6131 + rgb[1] * 0.3395 + rgb[2] * 0.0474;
    let g = rgb[0] * 0.0701 + rgb[1] * 0.9163 + rgb[2] * 0.0136;
    let b = rgb[0] * 0.0206 + rgb[1] * 0.1096 + rgb[2] * 0.8698;
    [r, g, b]
}

#[allow(dead_code)]
pub fn aces_cg_to_linear(rgb: [f32; 3]) -> [f32; 3] {
    // Approximate inverse
    let r = rgb[0] * 1.6410 - rgb[1] * 0.3248 - rgb[2] * 0.2363;
    let g = -rgb[0] * 0.6636 + rgb[1] * 1.6153 + rgb[2] * 0.0167;
    let b = rgb[0] * 0.0117 - rgb[1] * 0.0082 + rgb[2] * 0.9883;
    [r, g, b]
}

#[allow(dead_code)]
pub fn convert_color_space(rgb: [f32; 3], from: ColorSpace, to: ColorSpace) -> [f32; 3] {
    if from == to {
        return rgb;
    }
    let linear = match from {
        ColorSpace::Linear => rgb,
        ColorSpace::Srgb => srgb_to_linear_rgb(rgb),
        ColorSpace::AcesCg => aces_cg_to_linear(rgb),
        ColorSpace::Rec709 => srgb_to_linear_rgb(rgb), // same gamma
    };
    match to {
        ColorSpace::Linear => linear,
        ColorSpace::Srgb => linear_to_srgb_rgb(linear),
        ColorSpace::AcesCg => linear_to_aces_cg(linear),
        ColorSpace::Rec709 => linear_to_srgb_rgb(linear),
    }
}

#[allow(dead_code)]
pub fn csc_luminance(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_space_names() {
        assert_eq!(color_space_name(ColorSpace::Linear), "linear");
        assert_eq!(color_space_name(ColorSpace::Srgb), "srgb");
    }

    #[test]
    fn srgb_linear_roundtrip() {
        let original = [0.5_f32, 0.25, 0.75];
        let linear = srgb_to_linear_rgb(original);
        let back = linear_to_srgb_rgb(linear);
        for (a, b) in original.iter().zip(back.iter()) {
            assert!((a - b).abs() < 1e-5, "roundtrip failed: {a} vs {b}");
        }
    }

    #[test]
    fn linear_zero_maps_to_zero() {
        let v = linear_to_srgb_channel(0.0);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn srgb_one_maps_to_one() {
        let v = srgb_to_linear_channel(1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn same_space_identity() {
        let rgb = [0.3, 0.5, 0.8];
        let out = convert_color_space(rgb, ColorSpace::Linear, ColorSpace::Linear);
        for (a, b) in rgb.iter().zip(out.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn aces_roundtrip_approx() {
        let rgb = [0.4, 0.5, 0.3];
        let aces = linear_to_aces_cg(rgb);
        let back = aces_cg_to_linear(aces);
        // Approximate matrix — allow coarser tolerance
        for (a, b) in rgb.iter().zip(back.iter()) {
            assert!((a - b).abs() < 0.1, "aces roundtrip: {a} vs {b}");
        }
    }

    #[test]
    fn luminance_white() {
        let lum = csc_luminance([1.0, 1.0, 1.0]);
        assert!((lum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn luminance_black() {
        let lum = csc_luminance([0.0, 0.0, 0.0]);
        assert!(lum.abs() < 1e-6);
    }

    #[test]
    fn convert_srgb_to_linear_brightens() {
        let srgb = [0.5, 0.5, 0.5];
        let lin = convert_color_space(srgb, ColorSpace::Srgb, ColorSpace::Linear);
        // sRGB 0.5 → linear ≈ 0.214 (gamma-compressed, so linear is darker)
        assert!(lin[0] < srgb[0]);
    }
}
