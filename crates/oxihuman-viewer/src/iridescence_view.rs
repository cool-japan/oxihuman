// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct IridescenceView {
    pub show_factor: bool,
    pub show_ior: bool,
    pub wavelength_nm_range: [f32; 2],
}

pub fn new_iridescence_view() -> IridescenceView {
    IridescenceView {
        show_factor: true,
        show_ior: false,
        wavelength_nm_range: [380.0, 700.0],
    }
}

pub fn iridescence_color_at(thickness_nm: f32, ior: f32) -> [f32; 3] {
    /* thin-film interference stub: map phase to RGB */
    let phase = (thickness_nm * ior * 0.001) % 1.0;
    [
        (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5,
        ((phase + 0.333) * std::f32::consts::TAU).sin() * 0.5 + 0.5,
        ((phase + 0.667) * std::f32::consts::TAU).sin() * 0.5 + 0.5,
    ]
}

pub fn iridescence_factor_color(factor: f32) -> [f32; 3] {
    let f = factor.clamp(0.0, 1.0);
    [f, f * 0.5, 1.0 - f]
}

pub fn iridescence_is_active(factor: f32) -> bool {
    factor > 0.01
}

pub fn iridescence_thickness_range() -> [f32; 2] {
    [100.0, 400.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_iridescence_view() {
        /* wavelength range starts at 380-700 nm */
        let v = new_iridescence_view();
        assert!((v.wavelength_nm_range[0] - 380.0).abs() < 1e-3);
    }

    #[test]
    fn test_iridescence_color_at_returns_valid() {
        /* all channels in [0,1] */
        let c = iridescence_color_at(200.0, 1.5);
        assert!(c[0] >= 0.0 && c[0] <= 1.0);
        assert!(c[1] >= 0.0 && c[1] <= 1.0);
        assert!(c[2] >= 0.0 && c[2] <= 1.0);
    }

    #[test]
    fn test_iridescence_factor_color() {
        /* factor=1 -> [1, 0.5, 0] */
        let c = iridescence_factor_color(1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_iridescence_is_active() {
        /* factor > 0.01 is active */
        assert!(iridescence_is_active(0.5));
        assert!(!iridescence_is_active(0.0));
    }

    #[test]
    fn test_iridescence_thickness_range() {
        /* range is 100-400 nm */
        let r = iridescence_thickness_range();
        assert!((r[0] - 100.0).abs() < 1e-3);
        assert!((r[1] - 400.0).abs() < 1e-3);
    }
}
