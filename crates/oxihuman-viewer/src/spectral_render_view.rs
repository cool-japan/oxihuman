// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SpectralRenderView {
    pub wavelengths_nm: Vec<f32>,
    pub show_primaries: bool,
    pub show_spectrum: bool,
}

pub fn new_spectral_render_view() -> SpectralRenderView {
    SpectralRenderView {
        wavelengths_nm: vec![380.0, 450.0, 550.0, 650.0, 780.0],
        show_primaries: false,
        show_spectrum: true,
    }
}

/// Approximate visible spectrum wavelength to linear RGB.
pub fn spectral_wavelength_to_rgb(wl_nm: f32) -> [f32; 3] {
    if !(380.0..=780.0).contains(&wl_nm) {
        return [0.0, 0.0, 0.0];
    }
    let r = if wl_nm < 490.0 {
        0.0
    } else if wl_nm < 580.0 {
        (wl_nm - 490.0) / 90.0
    } else {
        1.0
    };
    let g = if wl_nm < 400.0 {
        0.0
    } else if wl_nm < 520.0 {
        (wl_nm - 400.0) / 120.0
    } else if wl_nm < 580.0 {
        1.0
    } else {
        1.0 - (wl_nm - 580.0) / 200.0
    };
    let b = if wl_nm < 400.0 {
        1.0
    } else if wl_nm < 490.0 {
        1.0 - (wl_nm - 400.0) / 90.0
    } else {
        0.0
    };
    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
}

/// Planck blackbody radiance stub (normalised).
pub fn spectral_energy_at(wl_nm: f32, blackbody_temp_k: f32) -> f32 {
    let wl_m = wl_nm * 1e-9;
    let h = 6.626e-34_f32;
    let c = 3.0e8_f32;
    let k = 1.38e-23_f32;
    let num = 2.0 * h * c * c / (wl_m.powi(5));
    let exp_arg = (h * c / (wl_m * k * blackbody_temp_k)).min(88.0);
    let denom = (exp_arg.exp() - 1.0).max(1e-30);
    (num / denom).min(1e30)
}

/// CIE 1931 stub — returns a rough approximation.
pub fn spectral_to_xyz(wl_nm: f32, power: f32) -> [f32; 3] {
    let rgb = spectral_wavelength_to_rgb(wl_nm);
    [rgb[0] * power, rgb[1] * power, rgb[2] * power]
}

pub fn spectral_is_visible(wl_nm: f32) -> bool {
    (380.0..=780.0).contains(&wl_nm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spectral_render_view() {
        /* wavelengths_nm should be non-empty */
        let v = new_spectral_render_view();
        assert!(!v.wavelengths_nm.is_empty());
    }

    #[test]
    fn test_spectral_wavelength_to_rgb_visible() {
        /* green at 550nm should have non-zero green component */
        let rgb = spectral_wavelength_to_rgb(550.0);
        assert!(rgb[1] > 0.0);
    }

    #[test]
    fn test_spectral_wavelength_to_rgb_invisible() {
        /* outside visible range is black */
        let rgb = spectral_wavelength_to_rgb(10.0);
        assert_eq!(rgb, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_spectral_is_visible() {
        /* 550 is visible, 1000 is not */
        assert!(spectral_is_visible(550.0));
        assert!(!spectral_is_visible(1000.0));
    }

    #[test]
    fn test_spectral_energy_at_positive() {
        /* energy should be positive for a 6000K blackbody at 550nm */
        let e = spectral_energy_at(550.0, 6000.0);
        assert!(e > 0.0);
    }
}
