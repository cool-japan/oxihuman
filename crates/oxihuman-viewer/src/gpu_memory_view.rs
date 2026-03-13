// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GPU memory usage visualization — shows texture, buffer, and total VRAM consumption.

/// GPU memory view configuration.
#[derive(Debug, Clone)]
pub struct GpuMemoryView {
    pub enabled: bool,
    pub texture_bytes: u64,
    pub buffer_bytes: u64,
    pub render_target_bytes: u64,
    pub budget_bytes: u64,
}

impl GpuMemoryView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            texture_bytes: 0,
            buffer_bytes: 0,
            render_target_bytes: 0,
            budget_bytes: 4 * 1024 * 1024 * 1024, /* 4 GiB default */
        }
    }
}

impl Default for GpuMemoryView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new GPU memory view.
pub fn new_gpu_memory_view() -> GpuMemoryView {
    GpuMemoryView::new()
}

/// Enable or disable GPU memory overlay.
pub fn gmv_set_enabled(v: &mut GpuMemoryView, enabled: bool) {
    v.enabled = enabled;
}

/// Update texture memory usage.
pub fn gmv_set_texture_bytes(v: &mut GpuMemoryView, bytes: u64) {
    v.texture_bytes = bytes;
}

/// Update buffer memory usage.
pub fn gmv_set_buffer_bytes(v: &mut GpuMemoryView, bytes: u64) {
    v.buffer_bytes = bytes;
}

/// Set the VRAM budget.
pub fn gmv_set_budget_bytes(v: &mut GpuMemoryView, budget: u64) {
    v.budget_bytes = budget.max(1);
}

/// Compute total GPU memory usage.
pub fn gmv_total_bytes(v: &GpuMemoryView) -> u64 {
    v.texture_bytes
        .saturating_add(v.buffer_bytes)
        .saturating_add(v.render_target_bytes)
}

/// Compute usage fraction (0–1 relative to budget).
pub fn gmv_usage_fraction(v: &GpuMemoryView) -> f32 {
    (gmv_total_bytes(v) as f32 / v.budget_bytes as f32).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn gpu_memory_view_to_json(v: &GpuMemoryView) -> String {
    format!(
        r#"{{"enabled":{},"texture_bytes":{},"buffer_bytes":{},"render_target_bytes":{},"budget_bytes":{}}}"#,
        v.enabled, v.texture_bytes, v.buffer_bytes, v.render_target_bytes, v.budget_bytes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_gpu_memory_view();
        assert!(!v.enabled);
        assert_eq!(v.texture_bytes, 0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_gpu_memory_view();
        gmv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_texture_bytes_set() {
        let mut v = new_gpu_memory_view();
        gmv_set_texture_bytes(&mut v, 1024 * 1024);
        assert_eq!(v.texture_bytes, 1024 * 1024);
    }

    #[test]
    fn test_buffer_bytes_set() {
        let mut v = new_gpu_memory_view();
        gmv_set_buffer_bytes(&mut v, 512 * 1024);
        assert_eq!(v.buffer_bytes, 512 * 1024);
    }

    #[test]
    fn test_budget_min() {
        let mut v = new_gpu_memory_view();
        gmv_set_budget_bytes(&mut v, 0);
        assert_eq!(v.budget_bytes, 1);
    }

    #[test]
    fn test_total_bytes() {
        let mut v = new_gpu_memory_view();
        gmv_set_texture_bytes(&mut v, 100);
        gmv_set_buffer_bytes(&mut v, 200);
        v.render_target_bytes = 50;
        assert_eq!(gmv_total_bytes(&v), 350);
    }

    #[test]
    fn test_usage_fraction_zero() {
        let v = new_gpu_memory_view();
        assert_eq!(gmv_usage_fraction(&v), 0.0);
    }

    #[test]
    fn test_usage_fraction_clamped() {
        let mut v = new_gpu_memory_view();
        gmv_set_budget_bytes(&mut v, 1);
        gmv_set_texture_bytes(&mut v, u64::MAX / 2);
        assert_eq!(gmv_usage_fraction(&v), 1.0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_gpu_memory_view();
        let s = gpu_memory_view_to_json(&v);
        assert!(s.contains("budget_bytes"));
    }

    #[test]
    fn test_clone() {
        let v = new_gpu_memory_view();
        let v2 = v.clone();
        assert_eq!(v2.budget_bytes, v.budget_bytes);
    }
}
