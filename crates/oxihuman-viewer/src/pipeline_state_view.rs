// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pipeline state debug view — visualizes PSO bindings and state change counts.

/// Pipeline state view configuration.
#[derive(Debug, Clone)]
pub struct PipelineStateView {
    pub enabled: bool,
    pub state_changes_per_frame: u32,
    pub active_pipelines: u32,
    pub cached_pipelines: u32,
    pub show_state_diff: bool,
}

impl PipelineStateView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            state_changes_per_frame: 0,
            active_pipelines: 0,
            cached_pipelines: 0,
            show_state_diff: false,
        }
    }
}

impl Default for PipelineStateView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pipeline state view.
pub fn new_pipeline_state_view() -> PipelineStateView {
    PipelineStateView::new()
}

/// Enable or disable pipeline state debug overlay.
pub fn psv_set_enabled(v: &mut PipelineStateView, enabled: bool) {
    v.enabled = enabled;
}

/// Update pipeline counts and state change frequency.
pub fn psv_update_stats(v: &mut PipelineStateView, changes: u32, active: u32, cached: u32) {
    v.state_changes_per_frame = changes;
    v.active_pipelines = active;
    v.cached_pipelines = cached;
}

/// Toggle state diff visualization.
pub fn psv_set_show_state_diff(v: &mut PipelineStateView, show: bool) {
    v.show_state_diff = show;
}

/// Compute cache hit ratio.
pub fn psv_cache_hit_ratio(v: &PipelineStateView) -> f32 {
    let total = v.active_pipelines + v.cached_pipelines;
    if total == 0 {
        return 0.0;
    }
    (v.cached_pipelines as f32 / total as f32).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn pipeline_state_view_to_json(v: &PipelineStateView) -> String {
    format!(
        r#"{{"enabled":{},"state_changes_per_frame":{},"active_pipelines":{},"cached_pipelines":{},"show_state_diff":{}}}"#,
        v.enabled,
        v.state_changes_per_frame,
        v.active_pipelines,
        v.cached_pipelines,
        v.show_state_diff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_pipeline_state_view();
        assert!(!v.enabled);
        assert_eq!(v.state_changes_per_frame, 0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_pipeline_state_view();
        psv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_stats() {
        let mut v = new_pipeline_state_view();
        psv_update_stats(&mut v, 200, 15, 85);
        assert_eq!(v.state_changes_per_frame, 200);
        assert_eq!(v.active_pipelines, 15);
        assert_eq!(v.cached_pipelines, 85);
    }

    #[test]
    fn test_show_state_diff_toggle() {
        let mut v = new_pipeline_state_view();
        psv_set_show_state_diff(&mut v, true);
        assert!(v.show_state_diff);
    }

    #[test]
    fn test_cache_hit_ratio_zero_total() {
        let v = new_pipeline_state_view();
        assert_eq!(psv_cache_hit_ratio(&v), 0.0);
    }

    #[test]
    fn test_cache_hit_ratio_partial() {
        let mut v = new_pipeline_state_view();
        psv_update_stats(&mut v, 0, 20, 80);
        assert!((psv_cache_hit_ratio(&v) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_cache_hit_ratio_in_range() {
        let mut v = new_pipeline_state_view();
        psv_update_stats(&mut v, 10, 50, 50);
        let ratio = psv_cache_hit_ratio(&v);
        assert!((0.0..=1.0).contains(&ratio));
    }

    #[test]
    fn test_json_keys() {
        let v = new_pipeline_state_view();
        let s = pipeline_state_view_to_json(&v);
        assert!(s.contains("cached_pipelines"));
    }

    #[test]
    fn test_clone() {
        let v = new_pipeline_state_view();
        let v2 = v.clone();
        assert_eq!(v2.active_pipelines, v.active_pipelines);
    }
}
