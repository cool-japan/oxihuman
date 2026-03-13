// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! USD stage preview panel view.

/// USD stage rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsdRenderMode {
    WireFrame,
    Shaded,
    ShadedWire,
    Points,
}

/// Configuration for the USD preview panel.
#[derive(Debug, Clone)]
pub struct UsdPreviewConfig {
    pub render_mode: UsdRenderMode,
    pub show_proxy_geometry: bool,
    pub frame: f64,
}

impl Default for UsdPreviewConfig {
    fn default() -> Self {
        UsdPreviewConfig {
            render_mode: UsdRenderMode::Shaded,
            show_proxy_geometry: true,
            frame: 0.0,
        }
    }
}

/// State for the USD preview panel.
#[derive(Debug, Clone)]
pub struct UsdPreviewView {
    pub config: UsdPreviewConfig,
    pub stage_path: String,
    pub enabled: bool,
}

/// Create a new USD preview view.
pub fn new_usd_preview_view() -> UsdPreviewView {
    UsdPreviewView {
        config: UsdPreviewConfig::default(),
        stage_path: String::new(),
        enabled: true,
    }
}

/// Set the USD stage file path.
pub fn upv_set_stage(v: &mut UsdPreviewView, path: &str) {
    v.stage_path = path.to_string();
}

/// Set the current time frame.
pub fn upv_set_frame(v: &mut UsdPreviewView, frame: f64) {
    v.config.frame = frame;
}

/// Set the render mode.
pub fn upv_set_render_mode(v: &mut UsdPreviewView, mode: UsdRenderMode) {
    v.config.render_mode = mode;
}

/// Return whether the stage path is non-empty.
pub fn upv_has_stage(v: &UsdPreviewView) -> bool {
    !v.stage_path.is_empty()
}

/// Serialise to JSON.
pub fn upv_to_json(v: &UsdPreviewView) -> String {
    format!(
        r#"{{"stage_path":"{}","frame":{:.2},"enabled":{}}}"#,
        v.stage_path, v.config.frame, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_no_stage() {
        let v = new_usd_preview_view();
        assert!(!upv_has_stage(&v) /* no stage by default */);
    }

    #[test]
    fn set_stage_makes_has_stage_true() {
        let mut v = new_usd_preview_view();
        upv_set_stage(&mut v, "scene.usd");
        assert!(upv_has_stage(&v) /* stage set */);
    }

    #[test]
    fn set_frame() {
        let mut v = new_usd_preview_view();
        upv_set_frame(&mut v, 42.0);
        assert!((v.config.frame - 42.0).abs() < 1e-9 /* frame set */);
    }

    #[test]
    fn set_render_mode_wireframe() {
        let mut v = new_usd_preview_view();
        upv_set_render_mode(&mut v, UsdRenderMode::WireFrame);
        assert_eq!(
            v.config.render_mode,
            UsdRenderMode::WireFrame /* mode set */
        );
    }

    #[test]
    fn default_render_mode_shaded() {
        let v = new_usd_preview_view();
        assert_eq!(
            v.config.render_mode,
            UsdRenderMode::Shaded /* default mode */
        );
    }

    #[test]
    fn json_has_stage_path() {
        let mut v = new_usd_preview_view();
        upv_set_stage(&mut v, "test.usda");
        assert!(upv_to_json(&v).contains("test.usda") /* stage in json */);
    }

    #[test]
    fn enabled_default() {
        let v = new_usd_preview_view();
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn proxy_geometry_default_true() {
        let v = new_usd_preview_view();
        assert!(v.config.show_proxy_geometry /* proxy geometry on by default */);
    }
}
