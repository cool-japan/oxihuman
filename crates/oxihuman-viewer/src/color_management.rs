// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Color management state (view transform, exposure, gamma).

#![allow(dead_code)]

/// Color management state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorManagement {
    pub exposure: f32,
    pub gamma: f32,
    pub view_transform: u8,
    pub look: u8,
    pub sequencer_colorspace: u8,
}

/// Returns a default `ColorManagement` (linear workflow, gamma 2.2).
#[allow(dead_code)]
pub fn default_color_management() -> ColorManagement {
    ColorManagement {
        exposure: 0.0,
        gamma: 2.2,
        view_transform: 0,
        look: 0,
        sequencer_colorspace: 0,
    }
}

/// Applies exposure compensation (2^exposure multiplication).
#[allow(dead_code)]
pub fn apply_exposure(color: [f32; 4], exposure: f32) -> [f32; 4] {
    let factor = (2.0f32).powf(exposure);
    [
        color[0] * factor,
        color[1] * factor,
        color[2] * factor,
        color[3],
    ]
}

/// Applies gamma correction to RGB channels.
#[allow(dead_code)]
pub fn apply_gamma(color: [f32; 4], gamma: f32) -> [f32; 4] {
    let inv_gamma = if gamma.abs() < f32::EPSILON { 1.0 } else { 1.0 / gamma };
    [
        color[0].max(0.0).powf(inv_gamma),
        color[1].max(0.0).powf(inv_gamma),
        color[2].max(0.0).powf(inv_gamma),
        color[3],
    ]
}

/// Applies both exposure and gamma to convert linear to display space.
#[allow(dead_code)]
pub fn linear_to_display(cm: &ColorManagement, color: [f32; 4]) -> [f32; 4] {
    let exposed = apply_exposure(color, cm.exposure);
    apply_gamma(exposed, cm.gamma)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_management() {
        let cm = default_color_management();
        assert!((cm.exposure - 0.0).abs() < 1e-6);
        assert!((cm.gamma - 2.2).abs() < 1e-5);
    }

    #[test]
    fn test_apply_exposure_zero() {
        let c = [0.5, 0.5, 0.5, 1.0];
        let out = apply_exposure(c, 0.0);
        assert!((out[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_exposure_positive() {
        let c = [0.5, 0.5, 0.5, 1.0];
        let out = apply_exposure(c, 1.0); // factor = 2
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_exposure_negative() {
        let c = [1.0, 1.0, 1.0, 1.0];
        let out = apply_exposure(c, -1.0); // factor = 0.5
        assert!((out[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_exposure_preserves_alpha() {
        let c = [0.5, 0.5, 0.5, 0.75];
        let out = apply_exposure(c, 0.0);
        assert!((out[3] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_apply_gamma_one() {
        let c = [0.5, 0.5, 0.5, 1.0];
        let out = apply_gamma(c, 1.0);
        assert!((out[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_gamma_clamps_negative_channel() {
        let c = [-0.1, 0.5, 0.5, 1.0];
        let out = apply_gamma(c, 2.2);
        assert!((out[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_to_display_round_trip() {
        let cm = default_color_management();
        let c = [0.5, 0.5, 0.5, 1.0];
        let out = linear_to_display(&cm, c);
        // alpha unchanged
        assert!((out[3] - 1.0).abs() < 1e-6);
        // output should differ from input due to gamma
        assert!((out[0] - c[0]).abs() > 1e-4);
    }

    #[test]
    fn test_linear_to_display_zero_gamma_fallback() {
        let mut cm = default_color_management();
        cm.gamma = 0.0;
        let c = [0.5, 0.5, 0.5, 1.0];
        let out = linear_to_display(&cm, c);
        // inv_gamma = 1.0, so output equals exposed input
        assert!((out[0] - 0.5).abs() < 1e-5);
    }
}
