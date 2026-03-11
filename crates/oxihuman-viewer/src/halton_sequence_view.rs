// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct HaltonView {
    pub base_x: u32,
    pub base_y: u32,
    pub sequence_length: u32,
}

pub fn new_halton_view() -> HaltonView {
    HaltonView {
        base_x: 2,
        base_y: 3,
        sequence_length: 16,
    }
}

pub fn halton_sample(mut index: u32, base: u32) -> f32 {
    let mut result = 0.0f32;
    let mut denom = 1.0f32;
    while index > 0 {
        denom *= base as f32;
        result += (index % base) as f32 / denom;
        index /= base;
    }
    result
}

pub fn halton_point(v: &HaltonView, index: u32) -> (f32, f32) {
    (
        halton_sample(index + 1, v.base_x),
        halton_sample(index + 1, v.base_y),
    )
}

pub fn halton_jitter(v: &HaltonView, frame: u32) -> (f32, f32) {
    let idx = frame % v.sequence_length;
    let (x, y) = halton_point(v, idx);
    (x - 0.5, y - 0.5)
}

pub fn halton_is_coprime_bases(v: &HaltonView) -> bool {
    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    gcd(v.base_x, v.base_y) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default bases */
        let v = new_halton_view();
        assert_eq!(v.base_x, 2);
        assert_eq!(v.base_y, 3);
    }

    #[test]
    fn test_sample_range() {
        /* sample in [0, 1) */
        let s = halton_sample(5, 2);
        assert!((0.0..1.0).contains(&s));
    }

    #[test]
    fn test_jitter_range() {
        /* jitter in [-0.5, 0.5] */
        let v = new_halton_view();
        let (jx, jy) = halton_jitter(&v, 0);
        assert!((-0.5..=0.5).contains(&jx));
        assert!((-0.5..=0.5).contains(&jy));
    }

    #[test]
    fn test_coprime_bases_by_default() {
        /* 2 and 3 are coprime */
        let v = new_halton_view();
        assert!(halton_is_coprime_bases(&v));
    }

    #[test]
    fn test_sequence_wraps() {
        /* frame wraps around sequence_length */
        let v = new_halton_view();
        let (x0, y0) = halton_jitter(&v, 0);
        let (x16, y16) = halton_jitter(&v, 16);
        assert!((x0 - x16).abs() < 1e-6);
        assert!((y0 - y16).abs() < 1e-6);
    }
}
