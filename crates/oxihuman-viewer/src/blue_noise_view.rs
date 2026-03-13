// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BlueNoiseView {
    pub tile_size: u32,
    pub intensity: f32,
    pub temporal_animate: bool,
}

pub fn new_blue_noise_view() -> BlueNoiseView {
    BlueNoiseView {
        tile_size: 64,
        intensity: 1.0,
        temporal_animate: true,
    }
}

pub fn bn_set_intensity(v: &mut BlueNoiseView, i: f32) {
    v.intensity = i.clamp(0.0, 2.0);
}

/// Interleaved gradient noise approximation for blue-noise-like patterns.
pub fn bn_interleaved_gradient(x: f32, y: f32, frame: u32) -> f32 {
    let xf = x + 5.588_238 * frame as f32;
    let yf = y + 5.588_238 * frame as f32;
    (52.982_91 * xf + 9.187_902 * yf).fract()
}

pub fn bn_is_animated(v: &BlueNoiseView) -> bool {
    v.temporal_animate
}

pub fn bn_blend(a: &BlueNoiseView, b: &BlueNoiseView, t: f32) -> BlueNoiseView {
    let t = t.clamp(0.0, 1.0);
    let ts = (a.tile_size as f32 + (b.tile_size as f32 - a.tile_size as f32) * t).round() as u32;
    BlueNoiseView {
        tile_size: ts.max(1),
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        temporal_animate: if t < 0.5 {
            a.temporal_animate
        } else {
            b.temporal_animate
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default tile size */
        let v = new_blue_noise_view();
        assert_eq!(v.tile_size, 64);
    }

    #[test]
    fn test_set_intensity_clamped() {
        /* clamped to 2 */
        let mut v = new_blue_noise_view();
        bn_set_intensity(&mut v, 5.0);
        assert!((v.intensity - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_gradient_in_range() {
        /* result in [0, 1) */
        let g = bn_interleaved_gradient(10.0, 20.0, 3);
        assert!((0.0..1.0).contains(&g));
    }

    #[test]
    fn test_is_animated_by_default() {
        /* animated by default */
        let v = new_blue_noise_view();
        assert!(bn_is_animated(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint intensity */
        let a = BlueNoiseView {
            tile_size: 32,
            intensity: 0.0,
            temporal_animate: false,
        };
        let b = BlueNoiseView {
            tile_size: 64,
            intensity: 2.0,
            temporal_animate: true,
        };
        let c = bn_blend(&a, &b, 0.5);
        assert!((c.intensity - 1.0).abs() < 1e-5);
    }
}
