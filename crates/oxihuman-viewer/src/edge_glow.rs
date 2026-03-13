// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Edge glow (rim lighting) effect for the viewer.

/// Edge glow configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeGlowConfig {
    pub color: [f32; 3],
    pub intensity: f32,
    pub power: f32,
    pub enabled: bool,
}

/// Edge glow result per vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeGlowResult {
    pub glow_factor: f32,
    pub final_color: [f32; 3],
}

/// Default edge glow config.
#[allow(dead_code)]
pub fn default_edge_glow_config() -> EdgeGlowConfig {
    EdgeGlowConfig {
        color: [0.3, 0.6, 1.0],
        intensity: 1.0,
        power: 3.0,
        enabled: false,
    }
}

/// Compute edge glow factor from view direction and normal.
#[allow(dead_code)]
pub fn compute_edge_glow(view_dir: [f32; 3], normal: [f32; 3], config: &EdgeGlowConfig) -> EdgeGlowResult {
    if !config.enabled {
        return EdgeGlowResult {
            glow_factor: 0.0,
            final_color: [0.0, 0.0, 0.0],
        };
    }
    let dot = view_dir[0] * normal[0] + view_dir[1] * normal[1] + view_dir[2] * normal[2];
    let rim = (1.0 - dot.abs()).max(0.0);
    let factor = rim.powf(config.power) * config.intensity;
    let factor = factor.clamp(0.0, 1.0);
    EdgeGlowResult {
        glow_factor: factor,
        final_color: [
            config.color[0] * factor,
            config.color[1] * factor,
            config.color[2] * factor,
        ],
    }
}

/// Enable edge glow.
#[allow(dead_code)]
pub fn enable_edge_glow(config: &mut EdgeGlowConfig) {
    config.enabled = true;
}

/// Disable edge glow.
#[allow(dead_code)]
pub fn disable_edge_glow(config: &mut EdgeGlowConfig) {
    config.enabled = false;
}

/// Set glow intensity.
#[allow(dead_code)]
pub fn set_edge_glow_intensity(config: &mut EdgeGlowConfig, intensity: f32) {
    config.intensity = intensity.clamp(0.0, 10.0);
}

/// Set glow color.
#[allow(dead_code)]
pub fn set_edge_glow_color(config: &mut EdgeGlowConfig, color: [f32; 3]) {
    config.color = color;
}

/// Set glow power (falloff exponent).
#[allow(dead_code)]
pub fn set_edge_glow_power(config: &mut EdgeGlowConfig, power: f32) {
    config.power = power.clamp(0.1, 20.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_edge_glow_config();
        assert!(!c.enabled);
    }

    #[test]
    fn test_disabled_glow() {
        let c = default_edge_glow_config();
        let r = compute_edge_glow([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], &c);
        assert!(r.glow_factor.abs() < 1e-6);
    }

    #[test]
    fn test_enabled_glow() {
        let mut c = default_edge_glow_config();
        enable_edge_glow(&mut c);
        let r = compute_edge_glow([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], &c);
        assert!(r.glow_factor > 0.0);
    }

    #[test]
    fn test_facing_normal_no_glow() {
        let mut c = default_edge_glow_config();
        enable_edge_glow(&mut c);
        let r = compute_edge_glow([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], &c);
        assert!(r.glow_factor < 0.01);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = default_edge_glow_config();
        enable_edge_glow(&mut c);
        assert!(c.enabled);
        disable_edge_glow(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_intensity() {
        let mut c = default_edge_glow_config();
        set_edge_glow_intensity(&mut c, 5.0);
        assert!((c.intensity - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_color() {
        let mut c = default_edge_glow_config();
        set_edge_glow_color(&mut c, [1.0, 0.0, 0.0]);
        assert!((c.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_power() {
        let mut c = default_edge_glow_config();
        set_edge_glow_power(&mut c, 5.0);
        assert!((c.power - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_intensity() {
        let mut c = default_edge_glow_config();
        set_edge_glow_intensity(&mut c, 100.0);
        assert!((c.intensity - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_glow_color_output() {
        let mut c = default_edge_glow_config();
        enable_edge_glow(&mut c);
        c.color = [1.0, 0.0, 0.0];
        let r = compute_edge_glow([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], &c);
        assert!(r.final_color[0] > 0.0);
        assert!(r.final_color[1].abs() < 1e-6);
    }
}
