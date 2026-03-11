// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Blend shape preview rendering (weight slider).

#![allow(dead_code)]

/// Config for blend shape preview.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapePreviewConfig {
    /// Weight range minimum.
    pub weight_min: f32,
    /// Weight range maximum.
    pub weight_max: f32,
    /// Auto-cycle animation speed (0 = off).
    pub cycle_speed: f32,
    /// Show displacement magnitude as heat map.
    pub show_displacement_heatmap: bool,
    /// Max displacement scale for heat map.
    pub heatmap_max: f32,
}

#[allow(dead_code)]
impl Default for BlendShapePreviewConfig {
    fn default() -> Self {
        Self {
            weight_min: 0.0,
            weight_max: 1.0,
            cycle_speed: 0.0,
            show_displacement_heatmap: false,
            heatmap_max: 0.01,
        }
    }
}

/// State for blend shape preview.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapePreviewState {
    pub weight: f32,
    pub time: f32,
    pub shape_name: String,
}

#[allow(dead_code)]
impl Default for BlendShapePreviewState {
    fn default() -> Self {
        Self {
            weight: 0.0,
            time: 0.0,
            shape_name: String::new(),
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_blend_shape_preview_config() -> BlendShapePreviewConfig {
    BlendShapePreviewConfig::default()
}

/// Create default state.
#[allow(dead_code)]
pub fn new_blend_shape_preview_state() -> BlendShapePreviewState {
    BlendShapePreviewState::default()
}

/// Set weight, clamped to config range.
#[allow(dead_code)]
pub fn bsp_set_weight(state: &mut BlendShapePreviewState, cfg: &BlendShapePreviewConfig, w: f32) {
    state.weight = w.clamp(cfg.weight_min, cfg.weight_max);
}

/// Advance animation cycle.
#[allow(dead_code)]
pub fn bsp_update(state: &mut BlendShapePreviewState, cfg: &BlendShapePreviewConfig, dt: f32) {
    if cfg.cycle_speed > 0.0 {
        state.time += dt * cfg.cycle_speed;
        let t = (state.time.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        state.weight = cfg.weight_min + (cfg.weight_max - cfg.weight_min) * t;
    }
}

/// Reset state.
#[allow(dead_code)]
pub fn bsp_reset(state: &mut BlendShapePreviewState) {
    *state = BlendShapePreviewState::default();
}

/// Displacement heat map color (blue→red).
#[allow(dead_code)]
pub fn displacement_heat_color(magnitude: f32, max_magnitude: f32) -> [f32; 3] {
    let t = (magnitude / max_magnitude.max(1e-10)).clamp(0.0, 1.0);
    [t, 0.0, 1.0 - t]
}

/// Evaluate interpolated vertex position.
#[allow(dead_code)]
pub fn bsp_apply_blend(base: [f32; 3], delta: [f32; 3], weight: f32) -> [f32; 3] {
    [
        base[0] + delta[0] * weight,
        base[1] + delta[1] * weight,
        base[2] + delta[2] * weight,
    ]
}

/// Normalize weight to 0..1 relative to config range.
#[allow(dead_code)]
pub fn bsp_normalized_weight(state: &BlendShapePreviewState, cfg: &BlendShapePreviewConfig) -> f32 {
    let range = cfg.weight_max - cfg.weight_min;
    if range.abs() < 1e-10 {
        return 0.0;
    }
    (state.weight - cfg.weight_min) / range
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn blend_shape_preview_to_json(cfg: &BlendShapePreviewConfig) -> String {
    format!(
        r#"{{"weight_min":{:.4},"weight_max":{:.4},"cycle_speed":{:.4},"heatmap_max":{:.6}}}"#,
        cfg.weight_min, cfg.weight_max, cfg.cycle_speed, cfg.heatmap_max
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = BlendShapePreviewConfig::default();
        assert!((c.weight_max - 1.0).abs() < 1e-6);
        assert!(!c.show_displacement_heatmap);
    }

    #[test]
    fn test_set_weight_clamped() {
        let cfg = BlendShapePreviewConfig::default();
        let mut s = BlendShapePreviewState::default();
        bsp_set_weight(&mut s, &cfg, 5.0);
        assert!((s.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_negative_clamped() {
        let cfg = BlendShapePreviewConfig::default();
        let mut s = BlendShapePreviewState::default();
        bsp_set_weight(&mut s, &cfg, -1.0);
        assert!(s.weight < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut s = BlendShapePreviewState {
            weight: 0.7,
            time: 1.5,
            shape_name: "x".to_string(),
        };
        bsp_reset(&mut s);
        assert!(s.weight < 1e-6);
    }

    #[test]
    fn test_apply_blend() {
        let base = [0.0f32, 0.0, 0.0];
        let delta = [1.0, 0.0, 0.0];
        let r = bsp_apply_blend(base, delta, 0.5);
        assert!((r[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_displacement_heat_color_zero() {
        let c = displacement_heat_color(0.0, 1.0);
        assert!(c[0].abs() < 1e-6);
    }

    #[test]
    fn test_normalized_weight() {
        let cfg = BlendShapePreviewConfig::default();
        let s = BlendShapePreviewState {
            weight: 0.5,
            ..Default::default()
        };
        let n = bsp_normalized_weight(&s, &cfg);
        assert!((n - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_update_no_cycle() {
        let cfg = BlendShapePreviewConfig {
            cycle_speed: 0.0,
            ..Default::default()
        };
        let mut s = BlendShapePreviewState::default();
        bsp_update(&mut s, &cfg, 0.1);
        assert!(s.weight < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = blend_shape_preview_to_json(&BlendShapePreviewConfig::default());
        assert!(j.contains("weight_min"));
    }
}
