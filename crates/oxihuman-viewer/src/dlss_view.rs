// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DlssQuality {
    Performance,
    Balanced,
    Quality,
    UltraQuality,
}

pub struct DlssView {
    pub quality: DlssQuality,
    pub sharpness: f32,
    pub enabled: bool,
}

pub fn new_dlss_view() -> DlssView {
    DlssView {
        quality: DlssQuality::Quality,
        sharpness: 0.0,
        enabled: false,
    }
}

pub fn dlss_set_quality(v: &mut DlssView, q: DlssQuality) {
    v.quality = q;
}

pub fn dlss_scale_factor(v: &DlssView) -> f32 {
    match v.quality {
        DlssQuality::Performance => 0.5,
        DlssQuality::Balanced => 0.58,
        DlssQuality::Quality => 0.67,
        DlssQuality::UltraQuality => 0.77,
    }
}

pub fn dlss_is_enabled(v: &DlssView) -> bool {
    v.enabled
}

pub fn dlss_render_resolution(v: &DlssView, output_w: u32, output_h: u32) -> (u32, u32) {
    let s = dlss_scale_factor(v);
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
        let v = new_dlss_view();
        assert!(!dlss_is_enabled(&v));
    }

    #[test]
    fn test_scale_factor_performance() {
        /* performance is 0.5 */
        let mut v = new_dlss_view();
        dlss_set_quality(&mut v, DlssQuality::Performance);
        assert!((dlss_scale_factor(&v) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_scale_factor_quality() {
        /* quality is 0.67 */
        let v = new_dlss_view();
        assert!((dlss_scale_factor(&v) - 0.67).abs() < 1e-5);
    }

    #[test]
    fn test_render_resolution() {
        /* render resolution is scaled */
        let v = new_dlss_view();
        let (rw, _rh) = dlss_render_resolution(&v, 1920, 1080);
        assert!(rw < 1920);
    }

    #[test]
    fn test_set_quality() {
        /* quality can be set */
        let mut v = new_dlss_view();
        dlss_set_quality(&mut v, DlssQuality::Balanced);
        assert_eq!(v.quality, DlssQuality::Balanced);
    }
}
