// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Transform/editor gizmo rendering.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GizmoConfig {
    pub size: f32,
    pub mode: GizmoMode,
    pub snap: bool,
    pub snap_increment: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GizmoState {
    pub active_axis: u8,
    pub dragging: bool,
    pub value: f32,
}

#[allow(dead_code)]
pub fn default_gizmo_config() -> GizmoConfig {
    GizmoConfig {
        size: 1.0,
        mode: GizmoMode::Translate,
        snap: false,
        snap_increment: 0.25,
    }
}

#[allow(dead_code)]
pub fn new_gizmo_state() -> GizmoState {
    GizmoState { active_axis: 0, dragging: false, value: 0.0 }
}

#[allow(dead_code)]
pub fn gizmo_begin_drag(state: &mut GizmoState, axis: u8) {
    state.active_axis = axis;
    state.dragging = true;
    state.value = 0.0;
}

#[allow(dead_code)]
pub fn gizmo_end_drag(state: &mut GizmoState) {
    state.dragging = false;
}

#[allow(dead_code)]
pub fn gizmo_update(state: &mut GizmoState, delta: f32) {
    if state.dragging {
        state.value += delta;
    }
}

#[allow(dead_code)]
pub fn gizmo_set_mode(cfg: &mut GizmoConfig, mode: GizmoMode) {
    cfg.mode = mode;
}

/// Returns the value snapped to the nearest `snap_increment` if snapping is on.
#[allow(dead_code)]
pub fn gizmo_snapped_value(state: &GizmoState, cfg: &GizmoConfig) -> f32 {
    if cfg.snap && cfg.snap_increment > 1e-10 {
        (state.value / cfg.snap_increment).round() * cfg.snap_increment
    } else {
        state.value
    }
}

#[allow(dead_code)]
pub fn gizmo_to_json(state: &GizmoState, cfg: &GizmoConfig) -> String {
    let mode = match cfg.mode {
        GizmoMode::Translate => "translate",
        GizmoMode::Rotate    => "rotate",
        GizmoMode::Scale     => "scale",
    };
    format!(
        r#"{{"mode":"{mode}","active_axis":{},"dragging":{},"value":{:.4},"snap":{}}}"#,
        state.active_axis, state.dragging, state.value, cfg.snap
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_gizmo_config();
        assert_eq!(cfg.mode, GizmoMode::Translate);
        assert!(!cfg.snap);
        assert!((cfg.snap_increment - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_gizmo_state();
        assert!(!s.dragging);
        assert_eq!(s.active_axis, 0);
    }

    #[test]
    fn test_begin_drag() {
        let mut s = new_gizmo_state();
        gizmo_begin_drag(&mut s, 2);
        assert!(s.dragging);
        assert_eq!(s.active_axis, 2);
    }

    #[test]
    fn test_end_drag() {
        let mut s = new_gizmo_state();
        gizmo_begin_drag(&mut s, 1);
        gizmo_end_drag(&mut s);
        assert!(!s.dragging);
    }

    #[test]
    fn test_update_accumulates_when_dragging() {
        let mut s = new_gizmo_state();
        gizmo_begin_drag(&mut s, 0);
        gizmo_update(&mut s, 1.5);
        gizmo_update(&mut s, 0.5);
        assert!((s.value - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_ignored_when_not_dragging() {
        let mut s = new_gizmo_state();
        gizmo_update(&mut s, 5.0);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn test_set_mode() {
        let mut cfg = default_gizmo_config();
        gizmo_set_mode(&mut cfg, GizmoMode::Scale);
        assert_eq!(cfg.mode, GizmoMode::Scale);
    }

    #[test]
    fn test_snapped_value_no_snap() {
        let mut s = new_gizmo_state();
        let cfg = default_gizmo_config();
        gizmo_begin_drag(&mut s, 0);
        gizmo_update(&mut s, 0.37);
        assert!((gizmo_snapped_value(&s, &cfg) - 0.37).abs() < 1e-5);
    }

    #[test]
    fn test_snapped_value_with_snap() {
        let mut s = new_gizmo_state();
        let mut cfg = default_gizmo_config();
        cfg.snap = true;
        gizmo_begin_drag(&mut s, 0);
        gizmo_update(&mut s, 0.37);
        // 0.37 rounds to 0.25
        assert!((gizmo_snapped_value(&s, &cfg) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_to_json_contains_mode() {
        let s = new_gizmo_state();
        let cfg = default_gizmo_config();
        let j = gizmo_to_json(&s, &cfg);
        assert!(j.contains("translate"));
    }
}
