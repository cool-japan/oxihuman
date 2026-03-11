// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct ToonView {
    pub num_bands: u32,
    pub outline_width: f32,
    pub show_outline: bool,
}

pub fn new_toon_view(bands: u32) -> ToonView {
    ToonView {
        num_bands: bands.max(1),
        outline_width: 0.05,
        show_outline: true,
    }
}

pub fn toon_quantize(value: f32, bands: u32) -> f32 {
    let n = bands.max(1) as f32;
    (value * n).floor() / n
}

pub fn toon_outline_factor(normal_dot_view: f32, width: f32) -> f32 {
    if normal_dot_view < width {
        1.0
    } else {
        0.0
    }
}

pub fn toon_shade(cos_theta: f32, bands: u32) -> [f32; 3] {
    let q = toon_quantize(cos_theta.max(0.0), bands);
    [q, q, q]
}

pub fn toon_is_outline(ndv: f32, width: f32) -> bool {
    ndv < width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_toon_view() {
        /* bands is stored correctly */
        let v = new_toon_view(4);
        assert_eq!(v.num_bands, 4);
    }

    #[test]
    fn test_toon_quantize() {
        /* value is quantized to band steps */
        let q = toon_quantize(0.9, 4);
        assert!((q - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_toon_outline_factor_inside() {
        /* inside outline threshold */
        let f = toon_outline_factor(0.02, 0.05);
        assert_eq!(f, 1.0);
    }

    #[test]
    fn test_toon_outline_factor_outside() {
        /* outside outline threshold */
        let f = toon_outline_factor(0.1, 0.05);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn test_toon_shade_gray() {
        /* shade returns gray-scale (r=g=b) */
        let c = toon_shade(0.8, 4);
        assert!((c[0] - c[1]).abs() < 1e-6);
    }
}
