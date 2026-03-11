// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct CausticDebugView {
    pub show_irradiance: bool,
    pub show_photons: bool,
    pub exposure: f32,
}

pub fn new_caustic_debug_view() -> CausticDebugView {
    CausticDebugView {
        show_irradiance: true,
        show_photons: false,
        exposure: 1.0,
    }
}

pub fn caustic_irradiance_color(irr: f32, exposure: f32) -> [f32; 3] {
    let v = (irr * exposure).clamp(0.0, 1.0);
    [v, v * 0.8, 0.0]
}

pub fn caustic_photon_hit_color(intensity: f32) -> [f32; 3] {
    let i = intensity.clamp(0.0, 1.0);
    [i, i, 0.0]
}

pub fn caustic_concentration_factor(photon_count: u32, area_m2: f32) -> f32 {
    if area_m2 < 1e-9 {
        0.0
    } else {
        photon_count as f32 / area_m2
    }
}

pub fn caustic_is_bright(irr: f32, threshold: f32) -> bool {
    irr > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_caustic_debug_view() {
        /* exposure defaults to 1 */
        let v = new_caustic_debug_view();
        assert!((v.exposure - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_caustic_irradiance_color_zero() {
        /* irr=0 -> black */
        let c = caustic_irradiance_color(0.0, 1.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_caustic_photon_hit_color() {
        /* intensity=1 -> [1,1,0] */
        let c = caustic_photon_hit_color(1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_caustic_concentration_factor() {
        /* 100 photons / 0.5 m2 = 200 */
        let f = caustic_concentration_factor(100, 0.5);
        assert!((f - 200.0).abs() < 1e-3);
    }

    #[test]
    fn test_caustic_is_bright() {
        assert!(caustic_is_bright(0.8, 0.5));
        assert!(!caustic_is_bright(0.2, 0.5));
    }
}
