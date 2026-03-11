// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shader compilation status view — tracks and visualizes shader compile state.

/// Shader compilation entry status.
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderStatus {
    Pending,
    Compiling,
    Ready,
    Error,
}

/// Shader compile view configuration.
#[derive(Debug, Clone)]
pub struct ShaderCompileView {
    pub enabled: bool,
    pub total_shaders: u32,
    pub compiled_shaders: u32,
    pub error_count: u32,
    pub show_error_details: bool,
}

impl ShaderCompileView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_shaders: 0,
            compiled_shaders: 0,
            error_count: 0,
            show_error_details: true,
        }
    }
}

impl Default for ShaderCompileView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new shader compile view.
pub fn new_shader_compile_view() -> ShaderCompileView {
    ShaderCompileView::new()
}

/// Enable or disable shader compile overlay.
pub fn scv_set_enabled(v: &mut ShaderCompileView, enabled: bool) {
    v.enabled = enabled;
}

/// Update shader compile counts.
pub fn scv_update_counts(v: &mut ShaderCompileView, total: u32, compiled: u32, errors: u32) {
    v.total_shaders = total;
    v.compiled_shaders = compiled.min(total);
    v.error_count = errors;
}

/// Toggle error detail display.
pub fn scv_set_show_error_details(v: &mut ShaderCompileView, show: bool) {
    v.show_error_details = show;
}

/// Compute compile progress fraction (0–1).
pub fn scv_progress(v: &ShaderCompileView) -> f32 {
    if v.total_shaders == 0 {
        return 1.0;
    }
    (v.compiled_shaders as f32 / v.total_shaders as f32).clamp(0.0, 1.0)
}

/// Returns true if all shaders compiled without errors.
pub fn scv_all_ready(v: &ShaderCompileView) -> bool {
    v.total_shaders > 0 && v.compiled_shaders == v.total_shaders && v.error_count == 0
}

/// Serialize to JSON-like string.
pub fn shader_compile_view_to_json(v: &ShaderCompileView) -> String {
    format!(
        r#"{{"enabled":{},"total_shaders":{},"compiled_shaders":{},"error_count":{},"show_error_details":{}}}"#,
        v.enabled, v.total_shaders, v.compiled_shaders, v.error_count, v.show_error_details
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_shader_compile_view();
        assert!(!v.enabled);
        assert_eq!(v.error_count, 0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_shader_compile_view();
        scv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_counts() {
        let mut v = new_shader_compile_view();
        scv_update_counts(&mut v, 20, 15, 1);
        assert_eq!(v.total_shaders, 20);
        assert_eq!(v.compiled_shaders, 15);
        assert_eq!(v.error_count, 1);
    }

    #[test]
    fn test_compiled_capped_at_total() {
        let mut v = new_shader_compile_view();
        scv_update_counts(&mut v, 10, 20, 0);
        assert_eq!(v.compiled_shaders, 10);
    }

    #[test]
    fn test_progress_zero_total() {
        let v = new_shader_compile_view();
        assert_eq!(scv_progress(&v), 1.0);
    }

    #[test]
    fn test_progress_partial() {
        let mut v = new_shader_compile_view();
        scv_update_counts(&mut v, 10, 5, 0);
        assert!((scv_progress(&v) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_all_ready_false_with_errors() {
        let mut v = new_shader_compile_view();
        scv_update_counts(&mut v, 10, 10, 2);
        assert!(!scv_all_ready(&v));
    }

    #[test]
    fn test_all_ready_true() {
        let mut v = new_shader_compile_view();
        scv_update_counts(&mut v, 10, 10, 0);
        assert!(scv_all_ready(&v));
    }

    #[test]
    fn test_json_keys() {
        let v = new_shader_compile_view();
        let s = shader_compile_view_to_json(&v);
        assert!(s.contains("error_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_shader_compile_view();
        let v2 = v.clone();
        assert_eq!(v2.total_shaders, v.total_shaders);
    }
}
