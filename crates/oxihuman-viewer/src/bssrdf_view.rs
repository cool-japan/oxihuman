// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct BssrdfView {
    pub show_radius: bool,
    pub show_profile: bool,
    pub max_radius_mm: f32,
}

pub fn new_bssrdf_view() -> BssrdfView {
    BssrdfView {
        show_radius: true,
        show_profile: false,
        max_radius_mm: 10.0,
    }
}

/// Dipole diffusion approximation profile.
pub fn bssrdf_dipole_profile(r_mm: f32, sigma_t_prime: f32) -> f32 {
    let d = 1.0 / (3.0 * sigma_t_prime.max(1e-6));
    let r = r_mm.max(1e-6) * 1e-3; // mm to m
    let zr = 1.0 / sigma_t_prime.max(1e-6);
    let dpos = (r * r + zr * zr).sqrt();
    let tr = (sigma_t_prime * dpos).min(88.0);
    ((-tr).exp()) / (dpos * dpos) * (1.0 + sigma_t_prime * dpos) / (4.0 * std::f32::consts::PI * d)
}

pub fn bssrdf_mean_free_path(sigma_t: f32) -> f32 {
    1.0 / sigma_t.max(1e-9)
}

pub fn bssrdf_radius_color(r_mm: f32, max_r: f32) -> [f32; 3] {
    let t = (r_mm / max_r.max(1e-6)).clamp(0.0, 1.0);
    [t, 1.0 - t, 0.0]
}

pub fn bssrdf_is_within_radius(r: f32, max: f32) -> bool {
    r <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bssrdf_view() {
        /* max radius defaults to 10mm */
        let v = new_bssrdf_view();
        assert!((v.max_radius_mm - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_bssrdf_mean_free_path() {
        /* MFP = 1/sigma_t */
        let mfp = bssrdf_mean_free_path(2.0);
        assert!((mfp - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_bssrdf_dipole_profile_positive() {
        /* profile must be positive */
        let v = bssrdf_dipole_profile(1.0, 1000.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_bssrdf_is_within_radius() {
        /* within radius */
        assert!(bssrdf_is_within_radius(5.0, 10.0));
        assert!(!bssrdf_is_within_radius(15.0, 10.0));
    }

    #[test]
    fn test_bssrdf_radius_color() {
        /* at r=0, red=0 */
        let c = bssrdf_radius_color(0.0, 10.0);
        assert_eq!(c[0], 0.0);
    }
}
