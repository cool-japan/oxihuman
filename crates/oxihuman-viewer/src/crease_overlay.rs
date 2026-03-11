// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Crease/sharp edge overlay visualization.

#![allow(dead_code)]

/// Configuration for crease overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CreaseOverlayConfig {
    pub show_creases: bool,
    pub show_sharp: bool,
    pub crease_color: [f32; 4],
    pub sharp_color: [f32; 4],
}

/// Runtime state for crease overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CreaseOverlayState {
    pub enabled: bool,
    pub config: CreaseOverlayConfig,
}

#[allow(dead_code)]
pub fn default_crease_overlay_config() -> CreaseOverlayConfig {
    CreaseOverlayConfig {
        show_creases: true,
        show_sharp: true,
        crease_color: [1.0, 0.5, 0.0, 1.0],
        sharp_color: [1.0, 0.0, 0.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn new_crease_overlay_state() -> CreaseOverlayState {
    CreaseOverlayState {
        enabled: false,
        config: default_crease_overlay_config(),
    }
}

#[allow(dead_code)]
pub fn co_set_enabled(state: &mut CreaseOverlayState, v: bool) {
    state.enabled = v;
}

#[allow(dead_code)]
pub fn co_show_creases(state: &mut CreaseOverlayState, v: bool) {
    state.config.show_creases = v;
}

#[allow(dead_code)]
pub fn co_show_sharp(state: &mut CreaseOverlayState, v: bool) {
    state.config.show_sharp = v;
}

#[allow(dead_code)]
pub fn co_to_json(state: &CreaseOverlayState) -> String {
    let cc = &state.config.crease_color;
    let sc = &state.config.sharp_color;
    format!(
        r#"{{"enabled":{},"show_creases":{},"show_sharp":{},"crease_color":[{:.4},{:.4},{:.4},{:.4}],"sharp_color":[{:.4},{:.4},{:.4},{:.4}]}}"#,
        state.enabled,
        state.config.show_creases,
        state.config.show_sharp,
        cc[0], cc[1], cc[2], cc[3],
        sc[0], sc[1], sc[2], sc[3]
    )
}

#[allow(dead_code)]
pub fn co_reset(state: &mut CreaseOverlayState) {
    *state = new_crease_overlay_state();
}

#[allow(dead_code)]
pub fn co_is_enabled(state: &CreaseOverlayState) -> bool {
    state.enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_crease_overlay_config();
        assert!(cfg.show_creases);
        assert!(cfg.show_sharp);
    }

    #[test]
    fn test_new_state_disabled() {
        let s = new_crease_overlay_state();
        assert!(!co_is_enabled(&s));
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_crease_overlay_state();
        co_set_enabled(&mut s, true);
        assert!(co_is_enabled(&s));
    }

    #[test]
    fn test_show_creases_toggle() {
        let mut s = new_crease_overlay_state();
        co_show_creases(&mut s, false);
        assert!(!s.config.show_creases);
    }

    #[test]
    fn test_show_sharp_toggle() {
        let mut s = new_crease_overlay_state();
        co_show_sharp(&mut s, false);
        assert!(!s.config.show_sharp);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_crease_overlay_state();
        let j = co_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("show_creases"));
        assert!(j.contains("crease_color"));
    }

    #[test]
    fn test_reset() {
        let mut s = new_crease_overlay_state();
        co_set_enabled(&mut s, true);
        co_show_creases(&mut s, false);
        co_reset(&mut s);
        assert!(!s.enabled);
        assert!(s.config.show_creases);
    }

    #[test]
    fn test_crease_color_default_orange() {
        let cfg = default_crease_overlay_config();
        assert!((cfg.crease_color[0] - 1.0).abs() < 1e-6);
        assert!((cfg.crease_color[1] - 0.5).abs() < 1e-6);
    }
}
