// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vertex/face normal visualization.

#![allow(dead_code)]

/// Display mode for normals.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NormalDisplayMode {
    Vertex,
    Face,
    Loop,
}

/// Configuration for normal overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalOverlayConfig {
    pub mode: NormalDisplayMode,
    pub length: f32,
    pub color: [f32; 4],
}

/// Runtime state for normal overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NormalOverlayState {
    pub enabled: bool,
    pub config: NormalOverlayConfig,
}

#[allow(dead_code)]
pub fn default_normal_overlay_config() -> NormalOverlayConfig {
    NormalOverlayConfig {
        mode: NormalDisplayMode::Vertex,
        length: 0.1,
        color: [0.0, 0.5, 1.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn new_normal_overlay_state() -> NormalOverlayState {
    NormalOverlayState {
        enabled: false,
        config: default_normal_overlay_config(),
    }
}

#[allow(dead_code)]
pub fn no_set_enabled(state: &mut NormalOverlayState, v: bool) {
    state.enabled = v;
}

#[allow(dead_code)]
pub fn no_set_mode(state: &mut NormalOverlayState, mode: NormalDisplayMode) {
    state.config.mode = mode;
}

#[allow(dead_code)]
pub fn no_set_length(state: &mut NormalOverlayState, v: f32) {
    state.config.length = v.max(0.001);
}

#[allow(dead_code)]
pub fn no_to_json(state: &NormalOverlayState) -> String {
    let c = &state.config.color;
    format!(
        r#"{{"enabled":{},"mode":"{}","length":{:.4},"color":[{:.4},{:.4},{:.4},{:.4}]}}"#,
        state.enabled,
        no_mode_name(state),
        state.config.length,
        c[0], c[1], c[2], c[3]
    )
}

#[allow(dead_code)]
pub fn no_mode_name(state: &NormalOverlayState) -> &'static str {
    match state.config.mode {
        NormalDisplayMode::Vertex => "vertex",
        NormalDisplayMode::Face => "face",
        NormalDisplayMode::Loop => "loop",
    }
}

#[allow(dead_code)]
pub fn no_reset(state: &mut NormalOverlayState) {
    *state = new_normal_overlay_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_normal_overlay_config();
        assert!((cfg.length - 0.1).abs() < 1e-6);
        assert_eq!(cfg.mode, NormalDisplayMode::Vertex);
    }

    #[test]
    fn test_new_state_disabled() {
        let s = new_normal_overlay_state();
        assert!(!s.enabled);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_normal_overlay_state();
        no_set_enabled(&mut s, true);
        assert!(s.enabled);
    }

    #[test]
    fn test_set_mode() {
        let mut s = new_normal_overlay_state();
        no_set_mode(&mut s, NormalDisplayMode::Face);
        assert_eq!(s.config.mode, NormalDisplayMode::Face);
    }

    #[test]
    fn test_set_length_clamps() {
        let mut s = new_normal_overlay_state();
        no_set_length(&mut s, -1.0);
        assert!(s.config.length >= 0.001);
    }

    #[test]
    fn test_mode_name_vertex() {
        let s = new_normal_overlay_state();
        assert_eq!(no_mode_name(&s), "vertex");
    }

    #[test]
    fn test_mode_name_face() {
        let mut s = new_normal_overlay_state();
        no_set_mode(&mut s, NormalDisplayMode::Face);
        assert_eq!(no_mode_name(&s), "face");
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_normal_overlay_state();
        let j = no_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("mode"));
        assert!(j.contains("length"));
    }
}
