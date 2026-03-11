// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct IrradianceCacheView {
    pub show_samples: bool,
    pub show_interpolation: bool,
    pub radius_scale: f32,
}

pub fn new_irradiance_cache_view() -> IrradianceCacheView {
    IrradianceCacheView {
        show_samples: true,
        show_interpolation: false,
        radius_scale: 1.0,
    }
}

pub fn ic_sample_color(irradiance: [f32; 3]) -> [f32; 3] {
    [
        irradiance[0].clamp(0.0, 1.0),
        irradiance[1].clamp(0.0, 1.0),
        irradiance[2].clamp(0.0, 1.0),
    ]
}

pub fn ic_weight_color(weight: f32) -> [f32; 3] {
    let w = weight.clamp(0.0, 1.0);
    [w, w * 0.5, 0.0]
}

pub fn ic_sample_radius_color(radius: f32, max_radius: f32) -> [f32; 3] {
    let t = if max_radius < 1e-9 {
        0.0
    } else {
        (radius / max_radius).clamp(0.0, 1.0)
    };
    [0.0, t, 1.0 - t]
}

pub fn ic_validity_color(error: f32, threshold: f32) -> [f32; 3] {
    if error < threshold {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_irradiance_cache_view() {
        /* radius_scale defaults to 1 */
        let v = new_irradiance_cache_view();
        assert!((v.radius_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ic_sample_color() {
        /* passthrough clamped */
        let c = ic_sample_color([0.5, 0.3, 0.8]);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ic_weight_color() {
        /* weight=0 -> black */
        let c = ic_weight_color(0.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_ic_sample_radius_color() {
        /* radius=max -> [0,1,0] */
        let c = ic_sample_radius_color(1.0, 1.0);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ic_validity_color() {
        /* low error -> green, high -> red */
        let g = ic_validity_color(0.001, 0.01);
        let r = ic_validity_color(0.1, 0.01);
        assert!((g[1] - 1.0).abs() < 1e-6);
        assert!((r[0] - 1.0).abs() < 1e-6);
    }
}
