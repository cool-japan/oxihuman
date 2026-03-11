// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct PhotonMapView {
    pub show_caustics: bool,
    pub show_global: bool,
    pub photon_radius: f32,
}

pub fn new_photon_map_view() -> PhotonMapView {
    PhotonMapView {
        show_caustics: true,
        show_global: false,
        photon_radius: 0.1,
    }
}

pub fn photon_density_color(count: u32, max_count: u32) -> [f32; 3] {
    let t = if max_count == 0 {
        0.0
    } else {
        (count as f32 / max_count as f32).clamp(0.0, 1.0)
    };
    [t, t * 0.5, 0.0]
}

pub fn photon_power_color(power: [f32; 3]) -> [f32; 3] {
    let lum = (power[0] + power[1] + power[2]) / 3.0;
    let lum = lum.clamp(0.0, 1.0);
    [lum, lum, lum]
}

pub fn photon_direction_color(dir: [f32; 3]) -> [f32; 3] {
    [dir[0] * 0.5 + 0.5, dir[1] * 0.5 + 0.5, dir[2] * 0.5 + 0.5]
}

pub fn photon_is_caustic_contributor(bounce_count: u32) -> bool {
    bounce_count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_photon_map_view() {
        /* photon_radius defaults to 0.1 */
        let v = new_photon_map_view();
        assert!((v.photon_radius - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_photon_density_color_zero() {
        /* zero count -> black */
        let c = photon_density_color(0, 100);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_photon_power_color() {
        /* white power -> white */
        let c = photon_power_color([1.0, 1.0, 1.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_photon_direction_color() {
        /* (1,0,0) -> [1, 0.5, 0.5] */
        let c = photon_direction_color([1.0, 0.0, 0.0]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_photon_is_caustic_contributor() {
        /* >= 2 bounces is caustic */
        assert!(photon_is_caustic_contributor(2));
        assert!(!photon_is_caustic_contributor(1));
    }
}
