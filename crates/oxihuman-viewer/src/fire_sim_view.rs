// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct FireView {
    pub show_temperature: bool,
    pub show_density: bool,
    pub min_temp_k: f32,
    pub max_temp_k: f32,
}

pub fn new_fire_view() -> FireView {
    FireView {
        show_temperature: true,
        show_density: false,
        min_temp_k: 300.0,
        max_temp_k: 3000.0,
    }
}

/// Blackbody-inspired color: black -> red -> orange -> yellow -> white.
pub fn fire_temperature_color(temp_k: f32) -> [f32; 3] {
    let t = ((temp_k - 300.0) / 2700.0).clamp(0.0, 1.0);
    if t < 0.25 {
        let s = t / 0.25;
        [s, 0.0, 0.0]
    } else if t < 0.5 {
        let s = (t - 0.25) / 0.25;
        [1.0, s * 0.5, 0.0]
    } else if t < 0.75 {
        let s = (t - 0.5) / 0.25;
        [1.0, 0.5 + s * 0.5, 0.0]
    } else {
        let s = (t - 0.75) / 0.25;
        [1.0, 1.0, s]
    }
}

pub fn fire_density_color(density: f32) -> [f32; 3] {
    let d = density.clamp(0.0, 1.0);
    [d * 0.5, d * 0.3, d * 0.1]
}

pub fn fire_is_hot(temp_k: f32) -> bool {
    temp_k > 1000.0
}

pub fn fire_blackbody_color(temp_k: f32) -> [f32; 3] {
    fire_temperature_color(temp_k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fire_view() {
        /* max temp defaults to 3000K */
        let v = new_fire_view();
        assert!((v.max_temp_k - 3000.0).abs() < 1e-6);
    }

    #[test]
    fn test_fire_temperature_color_cold() {
        /* at 300K (t=0) -> black */
        let c = fire_temperature_color(300.0);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_fire_temperature_color_hot() {
        /* at very high temp -> white */
        let c = fire_temperature_color(3000.0);
        assert!(c[0] > 0.9);
    }

    #[test]
    fn test_fire_is_hot() {
        /* above 1000K is hot */
        assert!(fire_is_hot(1500.0));
        assert!(!fire_is_hot(500.0));
    }

    #[test]
    fn test_fire_density_color() {
        /* zero density -> zero color */
        let c = fire_density_color(0.0);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }
}
