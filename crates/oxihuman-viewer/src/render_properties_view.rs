#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Render engine properties panel view.

/// Render engine properties panel state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPropertiesView {
    /// Engine: 0=EEVEE, 1=Cycles, 2=Workbench, 3=Custom.
    pub engine: u8,
    pub samples: u32,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub frame_start: u32,
    pub frame_end: u32,
    pub fps: f32,
}

/// Create a default `RenderPropertiesView` with 1920×1080, 128 samples, 24 fps.
#[allow(dead_code)]
pub fn default_render_properties_view() -> RenderPropertiesView {
    RenderPropertiesView {
        engine: 0,
        samples: 128,
        resolution_x: 1920,
        resolution_y: 1080,
        frame_start: 1,
        frame_end: 250,
        fps: 24.0,
    }
}

/// Set the output resolution.
#[allow(dead_code)]
pub fn set_resolution(view: &mut RenderPropertiesView, x: u32, y: u32) {
    view.resolution_x = x.max(1);
    view.resolution_y = y.max(1);
}

/// Return the aspect ratio (width / height).
#[allow(dead_code)]
pub fn resolution_aspect(view: &RenderPropertiesView) -> f32 {
    view.resolution_x as f32 / view.resolution_y.max(1) as f32
}

/// Return a static name for a render engine code.
///
/// * 0 → "EEVEE"
/// * 1 → "Cycles"
/// * 2 → "Workbench"
/// * _ → "Custom"
#[allow(dead_code)]
pub fn render_engine_name(engine: u8) -> &'static str {
    match engine {
        0 => "EEVEE",
        1 => "Cycles",
        2 => "Workbench",
        _ => "Custom",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_resolution() {
        let v = default_render_properties_view();
        assert_eq!(v.resolution_x, 1920);
        assert_eq!(v.resolution_y, 1080);
    }

    #[test]
    fn default_samples() {
        let v = default_render_properties_view();
        assert_eq!(v.samples, 128);
    }

    #[test]
    fn default_fps() {
        let v = default_render_properties_view();
        assert!((v.fps - 24.0).abs() < 1e-6);
    }

    #[test]
    fn set_resolution_valid() {
        let mut v = default_render_properties_view();
        set_resolution(&mut v, 2560, 1440);
        assert_eq!(v.resolution_x, 2560);
        assert_eq!(v.resolution_y, 1440);
    }

    #[test]
    fn set_resolution_clamps_to_one() {
        let mut v = default_render_properties_view();
        set_resolution(&mut v, 0, 0);
        assert_eq!(v.resolution_x, 1);
        assert_eq!(v.resolution_y, 1);
    }

    #[test]
    fn resolution_aspect_16_9() {
        let v = default_render_properties_view();
        assert!((resolution_aspect(&v) - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn resolution_aspect_square() {
        let mut v = default_render_properties_view();
        set_resolution(&mut v, 512, 512);
        assert!((resolution_aspect(&v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn render_engine_name_eevee() {
        assert_eq!(render_engine_name(0), "EEVEE");
    }

    #[test]
    fn render_engine_name_cycles() {
        assert_eq!(render_engine_name(1), "Cycles");
    }

    #[test]
    fn render_engine_name_custom() {
        assert_eq!(render_engine_name(200), "Custom");
    }
}
