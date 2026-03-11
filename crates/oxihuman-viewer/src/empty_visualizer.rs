// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Empty/null object visualizer (axes cross, arrows, etc.).

#![allow(dead_code)]

/// Display type for empty objects.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EmptyDisplayType {
    Axes,
    ArrowsXYZ,
    Sphere,
    Cube,
    Circle,
}

/// Configuration for empty visualizer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmptyVisConfig {
    pub display_type: EmptyDisplayType,
    pub size: f32,
    pub color: [f32; 4],
}

/// Runtime state for empty visualizer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmptyVisState {
    pub position: [f32; 3],
    pub config: EmptyVisConfig,
    pub visible: bool,
}

#[allow(dead_code)]
pub fn default_empty_vis_config() -> EmptyVisConfig {
    EmptyVisConfig {
        display_type: EmptyDisplayType::Axes,
        size: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn new_empty_vis_state() -> EmptyVisState {
    EmptyVisState {
        position: [0.0, 0.0, 0.0],
        config: default_empty_vis_config(),
        visible: true,
    }
}

#[allow(dead_code)]
pub fn ev_set_visible(state: &mut EmptyVisState, v: bool) {
    state.visible = v;
}

#[allow(dead_code)]
pub fn ev_set_display_type(state: &mut EmptyVisState, t: EmptyDisplayType) {
    state.config.display_type = t;
}

#[allow(dead_code)]
pub fn ev_set_size(state: &mut EmptyVisState, size: f32) {
    state.config.size = size.max(0.001);
}

#[allow(dead_code)]
pub fn ev_to_json(state: &EmptyVisState) -> String {
    let p = &state.position;
    let c = &state.config.color;
    format!(
        r#"{{"type":"{}","position":[{:.4},{:.4},{:.4}],"size":{:.4},"color":[{:.4},{:.4},{:.4},{:.4}],"visible":{}}}"#,
        ev_type_name(state),
        p[0], p[1], p[2],
        state.config.size,
        c[0], c[1], c[2], c[3],
        state.visible
    )
}

#[allow(dead_code)]
pub fn ev_type_name(state: &EmptyVisState) -> &'static str {
    match state.config.display_type {
        EmptyDisplayType::Axes => "axes",
        EmptyDisplayType::ArrowsXYZ => "arrows_xyz",
        EmptyDisplayType::Sphere => "sphere",
        EmptyDisplayType::Cube => "cube",
        EmptyDisplayType::Circle => "circle",
    }
}

#[allow(dead_code)]
pub fn ev_reset(state: &mut EmptyVisState) {
    *state = new_empty_vis_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_empty_vis_config();
        assert_eq!(cfg.display_type, EmptyDisplayType::Axes);
        assert!((cfg.size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_visible() {
        let s = new_empty_vis_state();
        assert!(s.visible);
        assert_eq!(s.position, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_set_visible() {
        let mut s = new_empty_vis_state();
        ev_set_visible(&mut s, false);
        assert!(!s.visible);
    }

    #[test]
    fn test_set_display_type() {
        let mut s = new_empty_vis_state();
        ev_set_display_type(&mut s, EmptyDisplayType::Sphere);
        assert_eq!(s.config.display_type, EmptyDisplayType::Sphere);
    }

    #[test]
    fn test_set_size_clamps() {
        let mut s = new_empty_vis_state();
        ev_set_size(&mut s, -1.0);
        assert!(s.config.size >= 0.001);
    }

    #[test]
    fn test_type_name_axes() {
        let s = new_empty_vis_state();
        assert_eq!(ev_type_name(&s), "axes");
    }

    #[test]
    fn test_type_name_all() {
        let types = [
            (EmptyDisplayType::Axes, "axes"),
            (EmptyDisplayType::ArrowsXYZ, "arrows_xyz"),
            (EmptyDisplayType::Sphere, "sphere"),
            (EmptyDisplayType::Cube, "cube"),
            (EmptyDisplayType::Circle, "circle"),
        ];
        for (t, name) in types {
            let mut s = new_empty_vis_state();
            ev_set_display_type(&mut s, t);
            assert_eq!(ev_type_name(&s), name);
        }
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_empty_vis_state();
        let j = ev_to_json(&s);
        assert!(j.contains("type"));
        assert!(j.contains("position"));
        assert!(j.contains("size"));
        assert!(j.contains("visible"));
    }

    #[test]
    fn test_reset() {
        let mut s = new_empty_vis_state();
        ev_set_visible(&mut s, false);
        ev_set_size(&mut s, 5.0);
        ev_reset(&mut s);
        assert!(s.visible);
        assert!((s.config.size - 1.0).abs() < 1e-6);
    }
}
