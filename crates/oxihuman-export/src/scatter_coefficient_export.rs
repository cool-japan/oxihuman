// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct ScatterCoefficients {
    pub wavelengths_nm: Vec<f32>,
    pub absorption: Vec<f32>,
    pub scattering: Vec<f32>,
    pub anisotropy: Vec<f32>,
}

pub fn new_scatter_coefficients() -> ScatterCoefficients {
    ScatterCoefficients {
        wavelengths_nm: Vec::new(),
        absorption: Vec::new(),
        scattering: Vec::new(),
        anisotropy: Vec::new(),
    }
}

pub fn scatter_push(s: &mut ScatterCoefficients, wl: f32, abs: f32, scat: f32, g: f32) {
    s.wavelengths_nm.push(wl);
    s.absorption.push(abs);
    s.scattering.push(scat);
    s.anisotropy.push(g.clamp(-1.0, 1.0));
}

pub fn scatter_extinction(s: &ScatterCoefficients, i: usize) -> f32 {
    if i < s.absorption.len() && i < s.scattering.len() {
        s.absorption[i] + s.scattering[i]
    } else {
        0.0
    }
}

pub fn scatter_albedo(s: &ScatterCoefficients, i: usize) -> f32 {
    let ext = scatter_extinction(s, i);
    if ext < 1e-10 {
        return 0.0;
    }
    if i < s.scattering.len() {
        s.scattering[i] / ext
    } else {
        0.0
    }
}

pub fn scatter_count(s: &ScatterCoefficients) -> usize {
    s.wavelengths_nm.len()
}

pub fn scatter_to_csv(s: &ScatterCoefficients) -> String {
    let mut out = String::from("wavelength_nm,absorption,scattering,anisotropy\n");
    for i in 0..s.wavelengths_nm.len() {
        out.push_str(&format!(
            "{:.2},{:.6},{:.6},{:.4}\n",
            s.wavelengths_nm[i], s.absorption[i], s.scattering[i], s.anisotropy[i]
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scatter_coefficients() {
        /* starts empty */
        let s = new_scatter_coefficients();
        assert_eq!(scatter_count(&s), 0);
    }

    #[test]
    fn test_scatter_push() {
        /* push adds entry */
        let mut s = new_scatter_coefficients();
        scatter_push(&mut s, 550.0, 0.01, 1.0, 0.8);
        assert_eq!(scatter_count(&s), 1);
    }

    #[test]
    fn test_scatter_extinction() {
        /* extinction = abs + scat */
        let mut s = new_scatter_coefficients();
        scatter_push(&mut s, 550.0, 0.1, 0.9, 0.5);
        assert!((scatter_extinction(&s, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_scatter_albedo() {
        /* albedo = scat/extinction */
        let mut s = new_scatter_coefficients();
        scatter_push(&mut s, 550.0, 0.1, 0.9, 0.5);
        assert!((scatter_albedo(&s, 0) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_scatter_to_csv() {
        /* csv has header */
        let s = new_scatter_coefficients();
        let csv = scatter_to_csv(&s);
        assert!(csv.contains("wavelength_nm"));
    }

    #[test]
    fn test_scatter_albedo_oob() {
        /* out-of-bounds returns 0 */
        let s = new_scatter_coefficients();
        assert_eq!(scatter_albedo(&s, 0), 0.0);
    }
}
