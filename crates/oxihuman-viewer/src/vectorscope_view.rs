// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Color vectorscope view stub.

/// Display mode for the vectorscope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorscopeMode {
    CbCr,
    IQ,
    RgbGamut,
}

/// A point on the vectorscope plot.
#[derive(Debug, Clone, Copy)]
pub struct VectorscopePoint {
    pub cb: f32,
    pub cr: f32,
    pub intensity: f32,
}

/// Vectorscope view configuration.
#[derive(Debug, Clone)]
pub struct VectorscopeView {
    pub mode: VectorscopeMode,
    pub points: Vec<VectorscopePoint>,
    pub max_points: usize,
    pub gain: f32,
    pub enabled: bool,
}

impl VectorscopeView {
    pub fn new(max_points: usize) -> Self {
        VectorscopeView {
            mode: VectorscopeMode::CbCr,
            points: Vec::with_capacity(max_points),
            max_points,
            gain: 1.0,
            enabled: true,
        }
    }
}

/// Create a new vectorscope view.
pub fn new_vectorscope_view(max_points: usize) -> VectorscopeView {
    VectorscopeView::new(max_points)
}

/// Add a pixel to the vectorscope plot.
pub fn vsv_add_pixel(vsv: &mut VectorscopeView, rgb: [f32; 3]) {
    /* Stub: converts RGB to Cb/Cr and adds as a point */
    if vsv.points.len() >= vsv.max_points {
        return;
    }
    let y = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
    let cb = -0.168736 * rgb[0] - 0.331264 * rgb[1] + 0.5 * rgb[2];
    let cr = 0.5 * rgb[0] - 0.418688 * rgb[1] - 0.081312 * rgb[2];
    vsv.points.push(VectorscopePoint {
        cb: cb * vsv.gain,
        cr: cr * vsv.gain,
        intensity: y,
    });
}

/// Clear all plotted points.
pub fn vsv_clear(vsv: &mut VectorscopeView) {
    vsv.points.clear();
}

/// Return point count.
pub fn vsv_point_count(vsv: &VectorscopeView) -> usize {
    vsv.points.len()
}

/// Set display mode.
pub fn vsv_set_mode(vsv: &mut VectorscopeView, mode: VectorscopeMode) {
    vsv.mode = mode;
}

/// Set gain.
pub fn vsv_set_gain(vsv: &mut VectorscopeView, gain: f32) {
    vsv.gain = gain.max(0.1);
}

/// Enable or disable.
pub fn vsv_set_enabled(vsv: &mut VectorscopeView, enabled: bool) {
    vsv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn vsv_to_json(vsv: &VectorscopeView) -> String {
    let mode = match vsv.mode {
        VectorscopeMode::CbCr => "cbcr",
        VectorscopeMode::IQ => "iq",
        VectorscopeMode::RgbGamut => "rgb_gamut",
    };
    format!(
        r#"{{"mode":"{}","point_count":{},"gain":{},"enabled":{}}}"#,
        mode,
        vsv.points.len(),
        vsv.gain,
        vsv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_no_points() {
        let v = new_vectorscope_view(1000);
        assert_eq!(vsv_point_count(&v), 0 /* no points initially */,);
    }

    #[test]
    fn test_add_pixel() {
        let mut v = new_vectorscope_view(1000);
        vsv_add_pixel(&mut v, [1.0, 0.0, 0.0]);
        assert_eq!(vsv_point_count(&v), 1 /* one point after add */,);
    }

    #[test]
    fn test_add_pixel_respects_max() {
        let mut v = new_vectorscope_view(2);
        vsv_add_pixel(&mut v, [1.0, 0.0, 0.0]);
        vsv_add_pixel(&mut v, [0.0, 1.0, 0.0]);
        vsv_add_pixel(&mut v, [0.0, 0.0, 1.0]);
        assert_eq!(vsv_point_count(&v), 2 /* must not exceed max_points */,);
    }

    #[test]
    fn test_clear() {
        let mut v = new_vectorscope_view(100);
        vsv_add_pixel(&mut v, [0.5; 3]);
        vsv_clear(&mut v);
        assert_eq!(vsv_point_count(&v), 0 /* points cleared */,);
    }

    #[test]
    fn test_set_mode() {
        let mut v = new_vectorscope_view(100);
        vsv_set_mode(&mut v, VectorscopeMode::IQ);
        assert_eq!(v.mode, VectorscopeMode::IQ /* mode must be set */,);
    }

    #[test]
    fn test_set_gain() {
        let mut v = new_vectorscope_view(100);
        vsv_set_gain(&mut v, 2.0);
        assert!((v.gain - 2.0).abs() < 1e-5 /* gain must be set */,);
    }

    #[test]
    fn test_gain_clamped_minimum() {
        let mut v = new_vectorscope_view(100);
        vsv_set_gain(&mut v, 0.0);
        assert!(v.gain >= 0.1 /* gain clamped to minimum 0.1 */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_vectorscope_view(100);
        vsv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_mode() {
        let v = new_vectorscope_view(100);
        let j = vsv_to_json(&v);
        assert!(j.contains("\"mode\"") /* json must contain mode */,);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_vectorscope_view(100);
        assert!(v.enabled /* must be enabled by default */,);
    }
}
