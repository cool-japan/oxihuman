// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SkinColorSpectrum {
    pub wavelengths_nm: Vec<f32>,
    pub reflectance: Vec<f32>,
    pub skin_type: u8,
}

pub fn new_skin_color_spectrum(skin_type: u8) -> SkinColorSpectrum {
    SkinColorSpectrum {
        wavelengths_nm: vec![],
        reflectance: vec![],
        skin_type,
    }
}

pub fn spectrum_push(s: &mut SkinColorSpectrum, wl: f32, r: f32) {
    s.wavelengths_nm.push(wl);
    s.reflectance.push(r);
}

pub fn spectrum_mean_reflectance(s: &SkinColorSpectrum) -> f32 {
    if s.reflectance.is_empty() {
        return 0.0;
    }
    s.reflectance.iter().sum::<f32>() / s.reflectance.len() as f32
}

/// Stub: return skin-type-based approximate RGB.
pub fn spectrum_to_rgb(s: &SkinColorSpectrum) -> [f32; 3] {
    match s.skin_type {
        1 => [0.98, 0.89, 0.82],
        2 => [0.95, 0.82, 0.72],
        3 => [0.88, 0.72, 0.58],
        4 => [0.75, 0.58, 0.42],
        5 => [0.55, 0.40, 0.28],
        _ => [0.35, 0.24, 0.16],
    }
}

pub fn spectrum_to_csv(s: &SkinColorSpectrum) -> String {
    let mut out = String::from("wavelength_nm,reflectance\n");
    for (wl, r) in s.wavelengths_nm.iter().zip(s.reflectance.iter()) {
        out.push_str(&format!("{wl:.2},{r:.4}\n"));
    }
    out
}

pub fn spectrum_count(s: &SkinColorSpectrum) -> usize {
    s.reflectance.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_color_spectrum() {
        /* construction */
        let s = new_skin_color_spectrum(2);
        assert_eq!(s.skin_type, 2);
        assert_eq!(spectrum_count(&s), 0);
    }

    #[test]
    fn test_push_and_count() {
        /* push adds entries */
        let mut s = new_skin_color_spectrum(1);
        spectrum_push(&mut s, 550.0, 0.5);
        assert_eq!(spectrum_count(&s), 1);
    }

    #[test]
    fn test_mean_reflectance() {
        /* mean of 0.4 and 0.6 = 0.5 */
        let mut s = new_skin_color_spectrum(1);
        spectrum_push(&mut s, 450.0, 0.4);
        spectrum_push(&mut s, 650.0, 0.6);
        assert!((spectrum_mean_reflectance(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_rgb_skin_type() {
        /* type 1 is lightest */
        let s = new_skin_color_spectrum(1);
        let c = spectrum_to_rgb(&s);
        assert!(c[0] > 0.9);
    }

    #[test]
    fn test_to_csv_format() {
        /* CSV has header and data rows */
        let mut s = new_skin_color_spectrum(1);
        spectrum_push(&mut s, 550.0, 0.5);
        let csv = spectrum_to_csv(&s);
        assert!(csv.contains("wavelength_nm"));
        assert!(csv.contains("550.00"));
    }

    #[test]
    fn test_mean_empty() {
        /* empty => 0 */
        let s = new_skin_color_spectrum(3);
        assert!((spectrum_mean_reflectance(&s)).abs() < 1e-6);
    }
}
