// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FsrMode {
    UltraPerformance,
    Performance,
    Balanced,
    Quality,
}

pub struct FsrView {
    pub mode: FsrMode,
    pub sharpness: f32,
    pub enabled: bool,
}

pub fn new_fsr_view() -> FsrView {
    FsrView {
        mode: FsrMode::Quality,
        sharpness: 0.5,
        enabled: false,
    }
}

pub fn fsr_set_mode(v: &mut FsrView, m: FsrMode) {
    v.mode = m;
}

pub fn fsr_scale_factor(v: &FsrView) -> f32 {
    match v.mode {
        FsrMode::UltraPerformance => 0.333,
        FsrMode::Performance => 0.5,
        FsrMode::Balanced => 0.586,
        FsrMode::Quality => 0.667,
    }
}

pub fn fsr_is_enabled(v: &FsrView) -> bool {
    v.enabled
}

pub fn fsr_render_resolution(v: &FsrView, output_w: u32, output_h: u32) -> (u32, u32) {
    let s = fsr_scale_factor(v);
    (
        (output_w as f32 * s).round() as u32,
        (output_h as f32 * s).round() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* disabled by default */
        let v = new_fsr_view();
        assert!(!fsr_is_enabled(&v));
    }

    #[test]
    fn test_scale_factor_quality() {
        /* quality is ~0.667 */
        let v = new_fsr_view();
        assert!((fsr_scale_factor(&v) - 0.667).abs() < 1e-4);
    }

    #[test]
    fn test_scale_factor_ultra_performance() {
        /* ultra performance is ~0.333 */
        let mut v = new_fsr_view();
        fsr_set_mode(&mut v, FsrMode::UltraPerformance);
        assert!((fsr_scale_factor(&v) - 0.333).abs() < 1e-4);
    }

    #[test]
    fn test_render_resolution() {
        /* render resolution is scaled down */
        let v = new_fsr_view();
        let (rw, _rh) = fsr_render_resolution(&v, 1920, 1080);
        assert!(rw < 1920);
    }

    #[test]
    fn test_set_mode() {
        /* mode can be set */
        let mut v = new_fsr_view();
        fsr_set_mode(&mut v, FsrMode::Balanced);
        assert_eq!(v.mode, FsrMode::Balanced);
    }
}
