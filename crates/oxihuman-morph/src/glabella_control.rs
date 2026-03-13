// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Glabella (between brows) shape morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlabellaConfig {
    pub max_depth: f32,
    pub max_width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlabellaState {
    pub depth: f32,
    pub width: f32,
    pub height: f32,
}

#[allow(dead_code)]
pub fn default_glabella_config() -> GlabellaConfig {
    GlabellaConfig {
        max_depth: 1.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_glabella_state() -> GlabellaState {
    GlabellaState {
        depth: 0.0,
        width: 0.5,
        height: 0.5,
    }
}

#[allow(dead_code)]
pub fn glabella_set_depth(state: &mut GlabellaState, cfg: &GlabellaConfig, v: f32) {
    state.depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn glabella_set_width(state: &mut GlabellaState, cfg: &GlabellaConfig, v: f32) {
    state.width = v.clamp(0.0, cfg.max_width);
}

#[allow(dead_code)]
pub fn glabella_set_height(state: &mut GlabellaState, v: f32) {
    state.height = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn glabella_reset(state: &mut GlabellaState) {
    *state = new_glabella_state();
}

#[allow(dead_code)]
pub fn glabella_to_weights(state: &GlabellaState) -> [f32; 3] {
    [state.depth, state.width, state.height]
}

#[allow(dead_code)]
pub fn glabella_to_json(state: &GlabellaState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"height":{:.4}}}"#,
        state.depth, state.width, state.height
    )
}

#[allow(dead_code)]
pub fn glabella_clamp(state: &mut GlabellaState, cfg: &GlabellaConfig) {
    state.depth = state.depth.clamp(0.0, cfg.max_depth);
    state.width = state.width.clamp(0.0, cfg.max_width);
    state.height = state.height.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_glabella_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_glabella_state();
        assert!((s.depth - 0.0).abs() < 1e-6);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth_clamps() {
        let cfg = default_glabella_config();
        let mut s = new_glabella_state();
        glabella_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
        glabella_set_depth(&mut s, &cfg, -1.0);
        assert!((s.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_glabella_config();
        let mut s = new_glabella_state();
        glabella_set_width(&mut s, &cfg, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamps() {
        let mut s = new_glabella_state();
        glabella_set_height(&mut s, 2.0);
        assert!((s.height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_glabella_config();
        let mut s = new_glabella_state();
        glabella_set_depth(&mut s, &cfg, 0.8);
        glabella_reset(&mut s);
        assert!((s.depth - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_glabella_state();
        let w = glabella_to_weights(&s);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_glabella_state();
        let j = glabella_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("width"));
        assert!(j.contains("height"));
    }

    #[test]
    fn test_clamp() {
        let cfg = default_glabella_config();
        let mut s = new_glabella_state();
        s.depth = 10.0;
        s.height = -1.0;
        glabella_clamp(&mut s, &cfg);
        assert!((s.depth - 1.0).abs() < 1e-6);
        assert!((s.height - 0.0).abs() < 1e-6);
    }
}

// ── GlabellaControl (simple blend API) ────────────────────────────────────────

/// Simple glabella morph parameters (blend API).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GlabellaControl {
    /// Width of the glabellar region, normalised 0..1.
    pub width: f32,
    /// Anterior protrusion, normalised 0..1.
    pub protrusion: f32,
    /// Depth of the vertical groove, normalised 0..1.
    pub vertical_groove: f32,
}

/// Return a default glabella control.
#[allow(dead_code)]
pub fn default_glabella_control() -> GlabellaControl {
    GlabellaControl {
        width: 0.5,
        protrusion: 0.5,
        vertical_groove: 0.0,
    }
}

/// Apply glabella control to a morph-weight slice.
#[allow(dead_code)]
pub fn apply_glabella_control(weights: &mut [f32], gc: &GlabellaControl) {
    if !weights.is_empty() {
        weights[0] = gc.width;
    }
    if weights.len() > 1 {
        weights[1] = gc.protrusion;
    }
    if weights.len() > 2 {
        weights[2] = gc.vertical_groove;
    }
}

/// Linear blend between two glabella controls.
#[allow(dead_code)]
pub fn glabella_blend(a: &GlabellaControl, b: &GlabellaControl, t: f32) -> GlabellaControl {
    let t = t.clamp(0.0, 1.0);
    GlabellaControl {
        width: a.width + (b.width - a.width) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        vertical_groove: a.vertical_groove + (b.vertical_groove - a.vertical_groove) * t,
    }
}
