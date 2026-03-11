// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SdfDebugView {
    pub show_positive: bool,
    pub show_negative: bool,
    pub iso_value: f32,
    pub color_range: f32,
}

pub fn new_sdf_debug_view() -> SdfDebugView {
    SdfDebugView {
        show_positive: true,
        show_negative: true,
        iso_value: 0.0,
        color_range: 1.0,
    }
}

pub fn sdf_color(dist: f32, params: &SdfDebugView) -> [f32; 3] {
    let range = params.color_range.max(1e-9);
    let d = (dist / range).clamp(-1.0, 1.0);
    if d.abs() < 0.05 {
        /* near surface -> white */
        [1.0, 1.0, 1.0]
    } else if d > 0.0 {
        /* positive -> green */
        [0.0, d, 0.0]
    } else {
        /* negative -> red */
        [-d, 0.0, 0.0]
    }
}

pub fn sdf_contour_intensity(dist: f32, frequency: f32) -> f32 {
    let phase = dist * frequency;
    (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5
}

pub fn sdf_is_surface(dist: f32, eps: f32) -> bool {
    dist.abs() < eps
}

pub fn sdf_gradient_approx(f: impl Fn([f32; 3]) -> f32, p: [f32; 3], eps: f32) -> [f32; 3] {
    let dx = f([p[0] + eps, p[1], p[2]]) - f([p[0] - eps, p[1], p[2]]);
    let dy = f([p[0], p[1] + eps, p[2]]) - f([p[0], p[1] - eps, p[2]]);
    let dz = f([p[0], p[1], p[2] + eps]) - f([p[0], p[1], p[2] - eps]);
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
    [dx / len, dy / len, dz / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sdf_debug_view() {
        /* color_range defaults to 1 */
        let v = new_sdf_debug_view();
        assert!((v.color_range - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sdf_color_positive() {
        /* positive dist -> green channel */
        let v = new_sdf_debug_view();
        let c = sdf_color(0.5, &v);
        assert!(c[1] > 0.0 && c[0] < 1e-6);
    }

    #[test]
    fn test_sdf_color_negative() {
        /* negative dist -> red channel */
        let v = new_sdf_debug_view();
        let c = sdf_color(-0.5, &v);
        assert!(c[0] > 0.0 && c[1] < 1e-6);
    }

    #[test]
    fn test_sdf_is_surface() {
        /* dist near 0 is surface */
        assert!(sdf_is_surface(0.001, 0.01));
        assert!(!sdf_is_surface(0.5, 0.01));
    }

    #[test]
    fn test_sdf_gradient_approx_sphere() {
        /* gradient of sphere sdf at (1,0,0) should be ~(1,0,0) */
        let sphere = |p: [f32; 3]| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 1.0;
        let g = sdf_gradient_approx(sphere, [1.0, 0.0, 0.0], 0.001);
        assert!((g[0] - 1.0).abs() < 0.01);
    }
}
