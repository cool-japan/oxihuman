// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compute dispatch debug view — tracks compute shader dispatch counts and workgroup sizes.

/// Compute dispatch view configuration.
#[derive(Debug, Clone)]
pub struct ComputeDispatchView {
    pub enabled: bool,
    pub dispatches_per_frame: u32,
    pub workgroup_x: u32,
    pub workgroup_y: u32,
    pub workgroup_z: u32,
}

impl ComputeDispatchView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dispatches_per_frame: 0,
            workgroup_x: 8,
            workgroup_y: 8,
            workgroup_z: 1,
        }
    }
}

impl Default for ComputeDispatchView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new compute dispatch view.
pub fn new_compute_dispatch_view() -> ComputeDispatchView {
    ComputeDispatchView::new()
}

/// Enable or disable compute dispatch debug overlay.
pub fn cdv_set_enabled(v: &mut ComputeDispatchView, enabled: bool) {
    v.enabled = enabled;
}

/// Update dispatch count for current frame.
pub fn cdv_set_dispatches(v: &mut ComputeDispatchView, count: u32) {
    v.dispatches_per_frame = count;
}

/// Set workgroup size.
pub fn cdv_set_workgroup_size(v: &mut ComputeDispatchView, x: u32, y: u32, z: u32) {
    v.workgroup_x = x.max(1);
    v.workgroup_y = y.max(1);
    v.workgroup_z = z.max(1);
}

/// Compute total threads per dispatch.
pub fn cdv_threads_per_dispatch(v: &ComputeDispatchView) -> u64 {
    v.workgroup_x as u64 * v.workgroup_y as u64 * v.workgroup_z as u64
}

/// Compute total thread invocations across all dispatches this frame.
pub fn cdv_total_threads(v: &ComputeDispatchView) -> u64 {
    cdv_threads_per_dispatch(v).saturating_mul(v.dispatches_per_frame as u64)
}

/// Serialize to JSON-like string.
pub fn compute_dispatch_view_to_json(v: &ComputeDispatchView) -> String {
    format!(
        r#"{{"enabled":{},"dispatches_per_frame":{},"workgroup_x":{},"workgroup_y":{},"workgroup_z":{}}}"#,
        v.enabled, v.dispatches_per_frame, v.workgroup_x, v.workgroup_y, v.workgroup_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_compute_dispatch_view();
        assert!(!v.enabled);
        assert_eq!(v.workgroup_x, 8);
        assert_eq!(v.workgroup_z, 1);
    }

    #[test]
    fn test_enable() {
        let mut v = new_compute_dispatch_view();
        cdv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_dispatches() {
        let mut v = new_compute_dispatch_view();
        cdv_set_dispatches(&mut v, 42);
        assert_eq!(v.dispatches_per_frame, 42);
    }

    #[test]
    fn test_workgroup_size_set() {
        let mut v = new_compute_dispatch_view();
        cdv_set_workgroup_size(&mut v, 16, 16, 1);
        assert_eq!(v.workgroup_x, 16);
        assert_eq!(v.workgroup_y, 16);
    }

    #[test]
    fn test_workgroup_size_min() {
        let mut v = new_compute_dispatch_view();
        cdv_set_workgroup_size(&mut v, 0, 0, 0);
        assert_eq!(v.workgroup_x, 1);
        assert_eq!(v.workgroup_y, 1);
        assert_eq!(v.workgroup_z, 1);
    }

    #[test]
    fn test_threads_per_dispatch() {
        let mut v = new_compute_dispatch_view();
        cdv_set_workgroup_size(&mut v, 8, 8, 1);
        assert_eq!(cdv_threads_per_dispatch(&v), 64);
    }

    #[test]
    fn test_total_threads_zero_dispatches() {
        let v = new_compute_dispatch_view();
        assert_eq!(cdv_total_threads(&v), 0);
    }

    #[test]
    fn test_total_threads_computed() {
        let mut v = new_compute_dispatch_view();
        cdv_set_workgroup_size(&mut v, 4, 4, 1);
        cdv_set_dispatches(&mut v, 10);
        assert_eq!(cdv_total_threads(&v), 160);
    }

    #[test]
    fn test_json_keys() {
        let v = new_compute_dispatch_view();
        let s = compute_dispatch_view_to_json(&v);
        assert!(s.contains("workgroup_x"));
    }

    #[test]
    fn test_clone() {
        let v = new_compute_dispatch_view();
        let v2 = v.clone();
        assert_eq!(v2.workgroup_x, v.workgroup_x);
    }
}
