// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct FluorescenceView {
    pub excitation_nm: f32,
    pub emission_nm: f32,
    pub quantum_yield: f32,
    pub show_stokes_shift: bool,
}

pub fn new_fluorescence_view(exc: f32, em: f32) -> FluorescenceView {
    FluorescenceView {
        excitation_nm: exc,
        emission_nm: em,
        quantum_yield: 0.5,
        show_stokes_shift: false,
    }
}

pub fn fluor_stokes_shift_nm(v: &FluorescenceView) -> f32 {
    v.emission_nm - v.excitation_nm
}

pub fn fluor_emission_color(v: &FluorescenceView) -> [f32; 3] {
    crate::spectral_render_view::spectral_wavelength_to_rgb(v.emission_nm)
}

pub fn fluor_is_uv_excited(v: &FluorescenceView) -> bool {
    v.excitation_nm < 400.0
}

pub fn fluor_energy_ratio(v: &FluorescenceView) -> f32 {
    if v.excitation_nm < 1.0 {
        return 0.0;
    }
    v.emission_nm / v.excitation_nm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fluorescence_view() {
        /* excitation and emission are stored */
        let v = new_fluorescence_view(365.0, 520.0);
        assert!((v.excitation_nm - 365.0).abs() < 1e-6);
    }

    #[test]
    fn test_fluor_stokes_shift() {
        /* stokes shift is emission - excitation */
        let v = new_fluorescence_view(365.0, 520.0);
        assert!((fluor_stokes_shift_nm(&v) - 155.0).abs() < 1e-4);
    }

    #[test]
    fn test_fluor_is_uv_excited() {
        /* 365nm UV excited */
        let v = new_fluorescence_view(365.0, 520.0);
        assert!(fluor_is_uv_excited(&v));
    }

    #[test]
    fn test_fluor_is_not_uv_excited() {
        /* 450nm is visible, not UV */
        let v = new_fluorescence_view(450.0, 520.0);
        assert!(!fluor_is_uv_excited(&v));
    }

    #[test]
    fn test_fluor_energy_ratio() {
        /* ratio = emission/excitation */
        let v = new_fluorescence_view(400.0, 600.0);
        assert!((fluor_energy_ratio(&v) - 1.5).abs() < 1e-4);
    }
}
