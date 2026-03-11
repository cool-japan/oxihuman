// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Available debug view modes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugViewMode {
    None,
    Wireframe,
    Normals,
    Depth,
    Albedo,
    Metallic,
    Roughness,
    AO,
}

/// Debug view state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderDebugView {
    pub mode: DebugViewMode,
    pub enabled: bool,
}

/// Create a new debug view (disabled by default).
#[allow(dead_code)]
pub fn new_render_debug_view() -> RenderDebugView {
    RenderDebugView {
        mode: DebugViewMode::None,
        enabled: false,
    }
}

/// Set the debug view mode.
#[allow(dead_code)]
pub fn set_debug_mode(view: &mut RenderDebugView, mode: DebugViewMode) {
    view.mode = mode;
    view.enabled = mode != DebugViewMode::None;
}

/// Return the name of the current debug mode.
#[allow(dead_code)]
pub fn debug_mode_name(view: &RenderDebugView) -> &'static str {
    match view.mode {
        DebugViewMode::None => "none",
        DebugViewMode::Wireframe => "wireframe",
        DebugViewMode::Normals => "normals",
        DebugViewMode::Depth => "depth",
        DebugViewMode::Albedo => "albedo",
        DebugViewMode::Metallic => "metallic",
        DebugViewMode::Roughness => "roughness",
        DebugViewMode::AO => "ao",
    }
}

/// Return the number of available debug modes.
#[allow(dead_code)]
pub fn debug_view_count() -> usize {
    8 // None + 7 modes
}

/// Check if debug view is enabled.
#[allow(dead_code)]
pub fn is_debug_enabled(view: &RenderDebugView) -> bool {
    view.enabled
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn debug_to_json(view: &RenderDebugView) -> String {
    format!(
        "{{\"mode\":\"{}\",\"enabled\":{}}}",
        debug_mode_name(view),
        view.enabled
    )
}

/// Toggle debug view on/off. If enabling, uses the last non-None mode or Wireframe.
#[allow(dead_code)]
pub fn toggle_debug(view: &mut RenderDebugView) {
    if view.enabled {
        view.enabled = false;
    } else {
        view.enabled = true;
        if view.mode == DebugViewMode::None {
            view.mode = DebugViewMode::Wireframe;
        }
    }
}

/// Reset debug view to disabled/None.
#[allow(dead_code)]
pub fn reset_debug_view(view: &mut RenderDebugView) {
    view.mode = DebugViewMode::None;
    view.enabled = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_disabled() {
        let v = new_render_debug_view();
        assert!(!is_debug_enabled(&v));
    }

    #[test]
    fn set_mode_enables() {
        let mut v = new_render_debug_view();
        set_debug_mode(&mut v, DebugViewMode::Normals);
        assert!(is_debug_enabled(&v));
    }

    #[test]
    fn mode_name() {
        let mut v = new_render_debug_view();
        set_debug_mode(&mut v, DebugViewMode::Depth);
        assert_eq!(debug_mode_name(&v), "depth");
    }

    #[test]
    fn view_count() {
        assert_eq!(debug_view_count(), 8);
    }

    #[test]
    fn toggle_on() {
        let mut v = new_render_debug_view();
        toggle_debug(&mut v);
        assert!(is_debug_enabled(&v));
    }

    #[test]
    fn toggle_off() {
        let mut v = new_render_debug_view();
        set_debug_mode(&mut v, DebugViewMode::Wireframe);
        toggle_debug(&mut v);
        assert!(!is_debug_enabled(&v));
    }

    #[test]
    fn reset_works() {
        let mut v = new_render_debug_view();
        set_debug_mode(&mut v, DebugViewMode::Normals);
        reset_debug_view(&mut v);
        assert!(!is_debug_enabled(&v));
        assert_eq!(v.mode, DebugViewMode::None);
    }

    #[test]
    fn to_json() {
        let v = new_render_debug_view();
        let j = debug_to_json(&v);
        assert!(j.contains("\"mode\""));
    }

    #[test]
    fn set_none_disables() {
        let mut v = new_render_debug_view();
        set_debug_mode(&mut v, DebugViewMode::Normals);
        set_debug_mode(&mut v, DebugViewMode::None);
        assert!(!is_debug_enabled(&v));
    }

    #[test]
    fn toggle_from_none_uses_wireframe() {
        let mut v = new_render_debug_view();
        toggle_debug(&mut v);
        assert_eq!(v.mode, DebugViewMode::Wireframe);
    }
}
