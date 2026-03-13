// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct GiDebugView {
    pub show_irradiance: bool,
    pub show_radiance: bool,
    pub show_probes: bool,
    pub exposure: f32,
}

pub fn new_gi_debug_view() -> GiDebugView {
    GiDebugView {
        show_irradiance: true,
        show_radiance: false,
        show_probes: false,
        exposure: 1.0,
    }
}

pub fn gi_irradiance_color(irradiance: [f32; 3], exposure: f32) -> [f32; 3] {
    [
        (irradiance[0] * exposure).clamp(0.0, 1.0),
        (irradiance[1] * exposure).clamp(0.0, 1.0),
        (irradiance[2] * exposure).clamp(0.0, 1.0),
    ]
}

pub fn gi_probe_color(probe_weight: f32) -> [f32; 3] {
    let w = probe_weight.clamp(0.0, 1.0);
    [w, w * 0.5, 1.0 - w]
}

pub fn gi_radiance_to_ldr(radiance: [f32; 3], exposure: f32) -> [f32; 3] {
    /* Reinhard tone mapping */
    [
        (radiance[0] * exposure) / (1.0 + radiance[0] * exposure),
        (radiance[1] * exposure) / (1.0 + radiance[1] * exposure),
        (radiance[2] * exposure) / (1.0 + radiance[2] * exposure),
    ]
}

pub fn gi_is_converged(variance: f32, threshold: f32) -> bool {
    variance < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gi_debug_view() {
        /* exposure defaults to 1 */
        let v = new_gi_debug_view();
        assert!((v.exposure - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gi_irradiance_color() {
        /* irradiance clamped to [0,1] */
        let c = gi_irradiance_color([2.0, 0.5, 0.1], 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gi_probe_color() {
        /* weight=0 -> [0,0,1] */
        let c = gi_probe_color(0.0);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gi_radiance_to_ldr() {
        /* reinhard maps high radiance below 1 */
        let c = gi_radiance_to_ldr([100.0, 0.0, 0.0], 1.0);
        assert!(c[0] < 1.0 && c[0] > 0.9);
    }

    #[test]
    fn test_gi_is_converged() {
        assert!(gi_is_converged(0.001, 0.01));
        assert!(!gi_is_converged(0.1, 0.01));
    }
}
