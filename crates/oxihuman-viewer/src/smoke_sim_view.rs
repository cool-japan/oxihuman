// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SmokeView {
    pub show_density: bool,
    pub show_velocity: bool,
    pub show_temperature: bool,
}

pub fn new_smoke_view() -> SmokeView {
    SmokeView {
        show_density: true,
        show_velocity: false,
        show_temperature: false,
    }
}

pub fn smoke_density_color(density: f32) -> [f32; 3] {
    let g = (1.0 - density.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    [g, g, g]
}

pub fn smoke_velocity_color(vel: [f32; 3]) -> [f32; 3] {
    let mag = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
    let n = mag.max(1e-6);
    [
        (vel[0] / n * 0.5 + 0.5).clamp(0.0, 1.0),
        (vel[1] / n * 0.5 + 0.5).clamp(0.0, 1.0),
        (vel[2] / n * 0.5 + 0.5).clamp(0.0, 1.0),
    ]
}

pub fn smoke_temperature_color(temp_k: f32) -> [f32; 3] {
    let t = ((temp_k - 273.0) / 727.0).clamp(0.0, 1.0);
    [t, 0.0, 1.0 - t]
}

pub fn smoke_opacity(density: f32, step_size: f32) -> f32 {
    1.0 - (-density * step_size).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_smoke_view() {
        /* show_density defaults to true */
        let v = new_smoke_view();
        assert!(v.show_density);
    }

    #[test]
    fn test_smoke_density_color_zero() {
        /* zero density -> white */
        let c = smoke_density_color(0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_smoke_density_color_full() {
        /* full density -> black */
        let c = smoke_density_color(1.0);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_smoke_opacity_positive() {
        /* opacity is positive for positive density */
        let o = smoke_opacity(1.0, 0.1);
        assert!(o > 0.0 && o < 1.0);
    }

    #[test]
    fn test_smoke_velocity_color_magnitude() {
        /* upward velocity should have high green */
        let c = smoke_velocity_color([0.0, 1.0, 0.0]);
        assert!(c[1] > 0.9);
    }
}
