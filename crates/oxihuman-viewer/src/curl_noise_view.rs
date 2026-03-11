// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Curl noise field visualization — renders curl of a noise-based vector field.

use std::f32::consts::TAU;

/// Curl noise view configuration.
#[derive(Debug, Clone)]
pub struct CurlNoiseView {
    pub enabled: bool,
    pub arrow_density: u32,
    pub arrow_scale: f32,
    pub frequency: f32,
    pub show_magnitude: bool,
}

impl CurlNoiseView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            arrow_density: 16,
            arrow_scale: 0.1,
            frequency: 1.0,
            show_magnitude: true,
        }
    }
}

impl Default for CurlNoiseView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new curl noise view.
pub fn new_curl_noise_view() -> CurlNoiseView {
    CurlNoiseView::new()
}

/// Enable or disable curl noise display.
pub fn cnv_set_enabled(v: &mut CurlNoiseView, enabled: bool) {
    v.enabled = enabled;
}

/// Set arrow grid density per axis.
pub fn cnv_set_arrow_density(v: &mut CurlNoiseView, density: u32) {
    v.arrow_density = density.clamp(4, 128);
}

/// Set arrow length scale.
pub fn cnv_set_arrow_scale(v: &mut CurlNoiseView, scale: f32) {
    v.arrow_scale = scale.clamp(0.001, 5.0);
}

/// Set noise base frequency.
pub fn cnv_set_frequency(v: &mut CurlNoiseView, freq: f32) {
    v.frequency = freq.max(0.001);
}

/// Toggle magnitude color coding.
pub fn cnv_set_show_magnitude(v: &mut CurlNoiseView, show: bool) {
    v.show_magnitude = show;
}

/// Simple heuristic: angular frequency for color cycling.
pub fn cnv_angular_freq(v: &CurlNoiseView) -> f32 {
    v.frequency * TAU
}

/// Serialize to JSON-like string.
pub fn curl_noise_view_to_json(v: &CurlNoiseView) -> String {
    format!(
        r#"{{"enabled":{},"arrow_density":{},"arrow_scale":{:.4},"frequency":{:.4},"show_magnitude":{}}}"#,
        v.enabled, v.arrow_density, v.arrow_scale, v.frequency, v.show_magnitude
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_curl_noise_view();
        assert!(!v.enabled);
        assert_eq!(v.arrow_density, 16);
    }

    #[test]
    fn test_enable() {
        let mut v = new_curl_noise_view();
        cnv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_density_clamp_low() {
        let mut v = new_curl_noise_view();
        cnv_set_arrow_density(&mut v, 0);
        assert_eq!(v.arrow_density, 4);
    }

    #[test]
    fn test_density_clamp_high() {
        let mut v = new_curl_noise_view();
        cnv_set_arrow_density(&mut v, 200);
        assert_eq!(v.arrow_density, 128);
    }

    #[test]
    fn test_arrow_scale_set() {
        let mut v = new_curl_noise_view();
        cnv_set_arrow_scale(&mut v, 0.5);
        assert!((v.arrow_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_frequency_set() {
        let mut v = new_curl_noise_view();
        cnv_set_frequency(&mut v, 2.0);
        assert!((v.frequency - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_angular_freq() {
        let mut v = new_curl_noise_view();
        cnv_set_frequency(&mut v, 1.0);
        assert!((cnv_angular_freq(&v) - std::f32::consts::TAU).abs() < 1e-5);
    }

    #[test]
    fn test_magnitude_toggle() {
        let mut v = new_curl_noise_view();
        cnv_set_show_magnitude(&mut v, false);
        assert!(!v.show_magnitude);
    }

    #[test]
    fn test_json_keys() {
        let v = new_curl_noise_view();
        let s = curl_noise_view_to_json(&v);
        assert!(s.contains("arrow_density"));
    }

    #[test]
    fn test_clone() {
        let v = new_curl_noise_view();
        let v2 = v.clone();
        assert!((v2.frequency - v.frequency).abs() < 1e-6);
    }
}
