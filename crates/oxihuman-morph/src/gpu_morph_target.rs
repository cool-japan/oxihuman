// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! GPU morph target upload stub.

/// Upload state for a GPU morph target buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuUploadState {
    Pending,
    Uploaded,
    Dirty,
    Failed,
}

/// GPU morph target descriptor.
#[derive(Debug, Clone)]
pub struct GpuMorphTarget {
    pub name: String,
    pub vertex_count: usize,
    pub gpu_buffer_id: u64,
    pub upload_state: GpuUploadState,
    pub weight: f32,
    pub enabled: bool,
}

impl GpuMorphTarget {
    pub fn new(name: impl Into<String>, vertex_count: usize) -> Self {
        GpuMorphTarget {
            name: name.into(),
            vertex_count,
            gpu_buffer_id: 0,
            upload_state: GpuUploadState::Pending,
            weight: 0.0,
            enabled: true,
        }
    }
}

/// Create a new GPU morph target.
pub fn new_gpu_morph_target(name: impl Into<String>, vertex_count: usize) -> GpuMorphTarget {
    GpuMorphTarget::new(name, vertex_count)
}

/// Simulate uploading data to the GPU (stub: sets state to Uploaded).
pub fn gmt_upload(target: &mut GpuMorphTarget, _data: &[[f32; 3]]) {
    /* Stub: mark as uploaded; no actual GPU call */
    target.upload_state = GpuUploadState::Uploaded;
    target.gpu_buffer_id = target.vertex_count as u64 * 7919;
}

/// Mark the target as dirty (needs re-upload).
pub fn gmt_mark_dirty(target: &mut GpuMorphTarget) {
    target.upload_state = GpuUploadState::Dirty;
}

/// Set the blend weight.
pub fn gmt_set_weight(target: &mut GpuMorphTarget, weight: f32) {
    target.weight = weight.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn gmt_set_enabled(target: &mut GpuMorphTarget, enabled: bool) {
    target.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn gmt_to_json(target: &GpuMorphTarget) -> String {
    let state = match target.upload_state {
        GpuUploadState::Pending => "pending",
        GpuUploadState::Uploaded => "uploaded",
        GpuUploadState::Dirty => "dirty",
        GpuUploadState::Failed => "failed",
    };
    format!(
        r#"{{"name":"{}","vertex_count":{},"state":"{}","weight":{},"enabled":{}}}"#,
        target.name, target.vertex_count, state, target.weight, target.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_pending() {
        let t = new_gpu_morph_target("mouth_open", 256);
        assert_eq!(
            t.upload_state,
            GpuUploadState::Pending, /* must start as Pending */
        );
    }

    #[test]
    fn test_upload_changes_state() {
        let mut t = new_gpu_morph_target("x", 4);
        gmt_upload(&mut t, &[[0.0; 3]; 4]);
        assert_eq!(
            t.upload_state,
            GpuUploadState::Uploaded, /* state must be Uploaded after upload */
        );
    }

    #[test]
    fn test_upload_sets_buffer_id() {
        let mut t = new_gpu_morph_target("x", 4);
        gmt_upload(&mut t, &[[0.0; 3]; 4]);
        assert_ne!(
            t.gpu_buffer_id,
            0, /* buffer id must be non-zero after upload */
        );
    }

    #[test]
    fn test_mark_dirty() {
        let mut t = new_gpu_morph_target("x", 4);
        gmt_upload(&mut t, &[[0.0; 3]; 4]);
        gmt_mark_dirty(&mut t);
        assert_eq!(
            t.upload_state,
            GpuUploadState::Dirty, /* state must be Dirty after mark */
        );
    }

    #[test]
    fn test_set_weight_clamped() {
        let mut t = new_gpu_morph_target("x", 2);
        gmt_set_weight(&mut t, 1.5);
        assert!((t.weight - 1.0).abs() < 1e-6, /* weight must be clamped to 1.0 */);
    }

    #[test]
    fn test_set_weight_zero() {
        let mut t = new_gpu_morph_target("x", 2);
        gmt_set_weight(&mut t, -0.5);
        assert!((t.weight).abs() < 1e-6, /* weight must be clamped to 0.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut t = new_gpu_morph_target("x", 2);
        gmt_set_enabled(&mut t, false);
        assert!(!t.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_state() {
        let t = new_gpu_morph_target("brow", 10);
        let j = gmt_to_json(&t);
        assert!(j.contains("\"state\"") /* json must contain state */,);
    }

    #[test]
    fn test_vertex_count_stored() {
        let t = new_gpu_morph_target("v", 512);
        assert_eq!(t.vertex_count, 512 /* vertex count must match */,);
    }

    #[test]
    fn test_enabled_default() {
        let t = new_gpu_morph_target("v", 1);
        assert!(t.enabled /* must be enabled by default */,);
    }
}
