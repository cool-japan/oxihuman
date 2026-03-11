// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Flow streamline visualization — traces paths along a velocity field.

/// Streamline view configuration.
#[derive(Debug, Clone)]
pub struct StreamlineView {
    pub enabled: bool,
    pub seed_count: u32,
    pub max_steps: u32,
    pub step_size: f32,
    pub line_opacity: f32,
}

impl StreamlineView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            seed_count: 64,
            max_steps: 256,
            step_size: 0.05,
            line_opacity: 0.8,
        }
    }
}

impl Default for StreamlineView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new streamline view.
pub fn new_streamline_view() -> StreamlineView {
    StreamlineView::new()
}

/// Enable or disable streamline display.
pub fn slv_set_enabled(v: &mut StreamlineView, enabled: bool) {
    v.enabled = enabled;
}

/// Set number of seed points.
pub fn slv_set_seed_count(v: &mut StreamlineView, count: u32) {
    v.seed_count = count.clamp(1, 1024);
}

/// Set maximum integration steps per streamline.
pub fn slv_set_max_steps(v: &mut StreamlineView, steps: u32) {
    v.max_steps = steps.clamp(8, 4096);
}

/// Set integration step size.
pub fn slv_set_step_size(v: &mut StreamlineView, size: f32) {
    v.step_size = size.clamp(0.001, 1.0);
}

/// Set streamline opacity.
pub fn slv_set_line_opacity(v: &mut StreamlineView, o: f32) {
    v.line_opacity = o.clamp(0.0, 1.0);
}

/// Estimated total vertex count across all streamlines.
pub fn slv_vertex_budget(v: &StreamlineView) -> u64 {
    v.seed_count as u64 * v.max_steps as u64
}

/// Serialize to JSON-like string.
pub fn streamline_view_to_json(v: &StreamlineView) -> String {
    format!(
        r#"{{"enabled":{},"seed_count":{},"max_steps":{},"step_size":{:.4},"line_opacity":{:.4}}}"#,
        v.enabled, v.seed_count, v.max_steps, v.step_size, v.line_opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_streamline_view();
        assert!(!v.enabled);
        assert_eq!(v.seed_count, 64);
        assert_eq!(v.max_steps, 256);
    }

    #[test]
    fn test_enable() {
        let mut v = new_streamline_view();
        slv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_seed_count_clamp() {
        let mut v = new_streamline_view();
        slv_set_seed_count(&mut v, 0);
        assert_eq!(v.seed_count, 1);
    }

    #[test]
    fn test_max_steps_clamp() {
        let mut v = new_streamline_view();
        slv_set_max_steps(&mut v, 1);
        assert_eq!(v.max_steps, 8);
    }

    #[test]
    fn test_step_size_set() {
        let mut v = new_streamline_view();
        slv_set_step_size(&mut v, 0.1);
        assert!((v.step_size - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_streamline_view();
        slv_set_line_opacity(&mut v, 2.0);
        assert_eq!(v.line_opacity, 1.0);
    }

    #[test]
    fn test_vertex_budget() {
        let v = new_streamline_view();
        let budget = slv_vertex_budget(&v);
        assert_eq!(budget, 64 * 256);
    }

    #[test]
    fn test_json_keys() {
        let v = new_streamline_view();
        let s = streamline_view_to_json(&v);
        assert!(s.contains("seed_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_streamline_view();
        let v2 = v.clone();
        assert!((v2.step_size - v.step_size).abs() < 1e-6);
    }
}
