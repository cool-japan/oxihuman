// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bone deform envelope debug visualization view stub.

/// Envelope display mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeDisplayMode {
    Wireframe,
    Solid,
    SolidWithEdges,
}

/// Bone envelope debug view configuration.
#[derive(Debug, Clone)]
pub struct BoneEnvelopeDebugView {
    pub display_mode: EnvelopeDisplayMode,
    pub color: [f32; 4],
    pub opacity: f32,
    pub show_head_tail: bool,
    pub enabled: bool,
}

impl BoneEnvelopeDebugView {
    pub fn new() -> Self {
        BoneEnvelopeDebugView {
            display_mode: EnvelopeDisplayMode::Wireframe,
            color: [0.0, 0.8, 0.2, 1.0],
            opacity: 0.5,
            show_head_tail: true,
            enabled: true,
        }
    }
}

impl Default for BoneEnvelopeDebugView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new bone envelope debug view.
pub fn new_bone_envelope_debug_view() -> BoneEnvelopeDebugView {
    BoneEnvelopeDebugView::new()
}

/// Set display mode.
pub fn bedv_set_display_mode(view: &mut BoneEnvelopeDebugView, mode: EnvelopeDisplayMode) {
    view.display_mode = mode;
}

/// Set envelope color (RGBA).
pub fn bedv_set_color(view: &mut BoneEnvelopeDebugView, color: [f32; 4]) {
    view.color = color;
}

/// Set envelope opacity.
pub fn bedv_set_opacity(view: &mut BoneEnvelopeDebugView, opacity: f32) {
    view.opacity = opacity.clamp(0.0, 1.0);
}

/// Toggle head/tail markers.
pub fn bedv_show_head_tail(view: &mut BoneEnvelopeDebugView, show: bool) {
    view.show_head_tail = show;
}

/// Enable or disable.
pub fn bedv_set_enabled(view: &mut BoneEnvelopeDebugView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn bedv_to_json(view: &BoneEnvelopeDebugView) -> String {
    let mode = match view.display_mode {
        EnvelopeDisplayMode::Wireframe => "wireframe",
        EnvelopeDisplayMode::Solid => "solid",
        EnvelopeDisplayMode::SolidWithEdges => "solid_with_edges",
    };
    format!(
        r#"{{"display_mode":"{}","opacity":{},"show_head_tail":{},"enabled":{}}}"#,
        mode, view.opacity, view.show_head_tail, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_display_mode() {
        let v = new_bone_envelope_debug_view();
        assert_eq!(
            v.display_mode,
            EnvelopeDisplayMode::Wireframe /* default must be Wireframe */
        );
    }

    #[test]
    fn test_set_display_mode() {
        let mut v = new_bone_envelope_debug_view();
        bedv_set_display_mode(&mut v, EnvelopeDisplayMode::Solid);
        assert_eq!(
            v.display_mode,
            EnvelopeDisplayMode::Solid /* mode must be set */
        );
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_bone_envelope_debug_view();
        bedv_set_opacity(&mut v, 2.0);
        assert!((v.opacity - 1.0).abs() < 1e-6 /* opacity clamped to 1.0 */);
    }

    #[test]
    fn test_show_head_tail() {
        let mut v = new_bone_envelope_debug_view();
        bedv_show_head_tail(&mut v, false);
        assert!(!v.show_head_tail /* head_tail must be hidden */);
    }

    #[test]
    fn test_set_color() {
        let mut v = new_bone_envelope_debug_view();
        bedv_set_color(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.color[0] - 1.0).abs() < 1e-6 /* red channel must be 1.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_bone_envelope_debug_view();
        bedv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_display_mode() {
        let v = new_bone_envelope_debug_view();
        let j = bedv_to_json(&v);
        assert!(j.contains("\"display_mode\"") /* JSON must have display_mode */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_bone_envelope_debug_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_opacity() {
        let v = new_bone_envelope_debug_view();
        assert!((v.opacity - 0.5).abs() < 1e-6 /* default opacity must be 0.5 */);
    }
}
