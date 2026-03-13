// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct OrderedDitherView {
    pub matrix_size: u32,
    pub strength: f32,
    pub color_bits: u32,
}

pub fn new_ordered_dither_view() -> OrderedDitherView {
    OrderedDitherView {
        matrix_size: 4,
        strength: 1.0,
        color_bits: 8,
    }
}

pub fn od_set_matrix_size(v: &mut OrderedDitherView, s: u32) {
    v.matrix_size = match s {
        2 | 4 | 8 => s,
        _ => 4,
    };
}

/// Bayer 4x4 threshold matrix value (normalized to [0, 1)).
pub fn od_bayer4_threshold(x: usize, y: usize) -> f32 {
    const BAYER4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    BAYER4[y % 4][x % 4] as f32 / 16.0
}

pub fn od_apply(v: &OrderedDitherView, value: f32, x: usize, y: usize) -> f32 {
    let threshold = od_bayer4_threshold(x, y) - 0.5;
    let scale = v.strength / (1 << v.color_bits) as f32;
    (value + threshold * scale).clamp(0.0, 1.0)
}

pub fn od_is_fine_grain(v: &OrderedDitherView) -> bool {
    v.matrix_size >= 8
}

pub fn od_blend(a: &OrderedDitherView, b: &OrderedDitherView, t: f32) -> OrderedDitherView {
    let t = t.clamp(0.0, 1.0);
    OrderedDitherView {
        matrix_size: if t < 0.5 {
            a.matrix_size
        } else {
            b.matrix_size
        },
        strength: a.strength + (b.strength - a.strength) * t,
        color_bits: if t < 0.5 { a.color_bits } else { b.color_bits },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default matrix size */
        let v = new_ordered_dither_view();
        assert_eq!(v.matrix_size, 4);
    }

    #[test]
    fn test_set_matrix_size_valid() {
        /* valid size accepted */
        let mut v = new_ordered_dither_view();
        od_set_matrix_size(&mut v, 8);
        assert_eq!(v.matrix_size, 8);
    }

    #[test]
    fn test_bayer_range() {
        /* threshold in [0, 1) */
        let t = od_bayer4_threshold(2, 3);
        assert!((0.0..1.0).contains(&t));
    }

    #[test]
    fn test_apply_clamped() {
        /* output clamped to [0, 1] */
        let v = new_ordered_dither_view();
        let out = od_apply(&v, 1.0, 0, 0);
        assert!((0.0..=1.0).contains(&out));
    }

    #[test]
    fn test_not_fine_grain_by_default() {
        /* 4x4 is not fine grain */
        let v = new_ordered_dither_view();
        assert!(!od_is_fine_grain(&v));
    }
}
