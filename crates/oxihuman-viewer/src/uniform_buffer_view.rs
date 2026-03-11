// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Uniform buffer contents view — visualizes UBO bindings, sizes, and update frequency.

/// Uniform buffer view configuration.
#[derive(Debug, Clone)]
pub struct UniformBufferView {
    pub enabled: bool,
    pub total_ubos: u32,
    pub total_bytes: u64,
    pub updates_per_frame: u32,
    pub show_binding_slots: bool,
}

impl UniformBufferView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_ubos: 0,
            total_bytes: 0,
            updates_per_frame: 0,
            show_binding_slots: true,
        }
    }
}

impl Default for UniformBufferView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new uniform buffer view.
pub fn new_uniform_buffer_view() -> UniformBufferView {
    UniformBufferView::new()
}

/// Enable or disable uniform buffer debug overlay.
pub fn ubv_set_enabled(v: &mut UniformBufferView, enabled: bool) {
    v.enabled = enabled;
}

/// Update UBO stats.
pub fn ubv_update_stats(v: &mut UniformBufferView, ubo_count: u32, total_bytes: u64, updates: u32) {
    v.total_ubos = ubo_count;
    v.total_bytes = total_bytes;
    v.updates_per_frame = updates;
}

/// Toggle binding slot visualization.
pub fn ubv_set_show_binding_slots(v: &mut UniformBufferView, show: bool) {
    v.show_binding_slots = show;
}

/// Compute average bytes per UBO.
pub fn ubv_avg_bytes_per_ubo(v: &UniformBufferView) -> f64 {
    if v.total_ubos == 0 {
        return 0.0;
    }
    v.total_bytes as f64 / v.total_ubos as f64
}

/// Serialize to JSON-like string.
pub fn uniform_buffer_view_to_json(v: &UniformBufferView) -> String {
    format!(
        r#"{{"enabled":{},"total_ubos":{},"total_bytes":{},"updates_per_frame":{},"show_binding_slots":{}}}"#,
        v.enabled, v.total_ubos, v.total_bytes, v.updates_per_frame, v.show_binding_slots
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_uniform_buffer_view();
        assert!(!v.enabled);
        assert_eq!(v.total_ubos, 0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_uniform_buffer_view();
        ubv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_stats() {
        let mut v = new_uniform_buffer_view();
        ubv_update_stats(&mut v, 8, 4096, 60);
        assert_eq!(v.total_ubos, 8);
        assert_eq!(v.total_bytes, 4096);
        assert_eq!(v.updates_per_frame, 60);
    }

    #[test]
    fn test_show_binding_slots_toggle() {
        let mut v = new_uniform_buffer_view();
        ubv_set_show_binding_slots(&mut v, false);
        assert!(!v.show_binding_slots);
    }

    #[test]
    fn test_avg_bytes_per_ubo_zero() {
        let v = new_uniform_buffer_view();
        assert_eq!(ubv_avg_bytes_per_ubo(&v), 0.0);
    }

    #[test]
    fn test_avg_bytes_per_ubo_computed() {
        let mut v = new_uniform_buffer_view();
        ubv_update_stats(&mut v, 4, 1024, 0);
        assert!((ubv_avg_bytes_per_ubo(&v) - 256.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_uniform_buffer_view();
        let s = uniform_buffer_view_to_json(&v);
        assert!(s.contains("updates_per_frame"));
    }

    #[test]
    fn test_clone() {
        let v = new_uniform_buffer_view();
        let v2 = v.clone();
        assert_eq!(v2.total_ubos, v.total_ubos);
    }

    #[test]
    fn test_updates_per_frame_stored() {
        let mut v = new_uniform_buffer_view();
        ubv_update_stats(&mut v, 1, 256, 120);
        assert_eq!(v.updates_per_frame, 120);
    }
}
