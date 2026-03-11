// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! InstanceRenderer — GPU-instanced rendering utilities.

#![allow(dead_code)]

/// A single instance transform.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceData {
    pub transform: [[f32; 4]; 4],
}

/// A batch of instances sharing a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceBatch {
    pub mesh_id: u32,
    pub instances: Vec<InstanceData>,
}

/// Renderer that collects instance batches.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InstanceRenderer {
    pub batches: Vec<InstanceBatch>,
}

/// Create an empty instance renderer.
#[allow(dead_code)]
pub fn new_instance_renderer() -> InstanceRenderer {
    InstanceRenderer::default()
}

/// Add a batch of instances for a given mesh.
#[allow(dead_code)]
pub fn add_instance_batch(renderer: &mut InstanceRenderer, batch: InstanceBatch) {
    renderer.batches.push(batch);
}

/// Number of batches.
#[allow(dead_code)]
pub fn batch_count(renderer: &InstanceRenderer) -> usize {
    renderer.batches.len()
}

/// Total instance count across all batches.
#[allow(dead_code)]
pub fn total_instance_count(renderer: &InstanceRenderer) -> usize {
    renderer.batches.iter().map(|b| b.instances.len()).sum()
}

/// Get instance data at `(batch_idx, instance_idx)`.
#[allow(dead_code)]
pub fn instance_at(renderer: &InstanceRenderer, batch_idx: usize, instance_idx: usize) -> Option<&InstanceData> {
    renderer.batches.get(batch_idx).and_then(|b| b.instances.get(instance_idx))
}

/// Stub render call — returns total instances rendered.
#[allow(dead_code)]
pub fn render_instances_stub(renderer: &InstanceRenderer) -> usize {
    total_instance_count(renderer)
}

/// Return the mesh id of a batch.
#[allow(dead_code)]
pub fn batch_mesh_id(renderer: &InstanceRenderer, batch_idx: usize) -> Option<u32> {
    renderer.batches.get(batch_idx).map(|b| b.mesh_id)
}

/// Clear all batches.
#[allow(dead_code)]
pub fn clear_instance_batches(renderer: &mut InstanceRenderer) {
    renderer.batches.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    fn sample_batch(mesh_id: u32, count: usize) -> InstanceBatch {
        InstanceBatch {
            mesh_id,
            instances: (0..count)
                .map(|_| InstanceData { transform: identity() })
                .collect(),
        }
    }

    #[test]
    fn test_new_instance_renderer() {
        let r = new_instance_renderer();
        assert_eq!(batch_count(&r), 0);
    }

    #[test]
    fn test_add_instance_batch() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(1, 3));
        assert_eq!(batch_count(&r), 1);
    }

    #[test]
    fn test_batch_count() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(1, 2));
        add_instance_batch(&mut r, sample_batch(2, 5));
        assert_eq!(batch_count(&r), 2);
    }

    #[test]
    fn test_total_instance_count() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(1, 2));
        add_instance_batch(&mut r, sample_batch(2, 3));
        assert_eq!(total_instance_count(&r), 5);
    }

    #[test]
    fn test_instance_at() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(1, 2));
        assert!(instance_at(&r, 0, 0).is_some());
        assert!(instance_at(&r, 0, 2).is_none());
    }

    #[test]
    fn test_render_instances_stub() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(1, 4));
        assert_eq!(render_instances_stub(&r), 4);
    }

    #[test]
    fn test_batch_mesh_id() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(42, 1));
        assert_eq!(batch_mesh_id(&r, 0), Some(42));
        assert_eq!(batch_mesh_id(&r, 1), None);
    }

    #[test]
    fn test_clear_instance_batches() {
        let mut r = new_instance_renderer();
        add_instance_batch(&mut r, sample_batch(1, 2));
        clear_instance_batches(&mut r);
        assert_eq!(batch_count(&r), 0);
    }

    #[test]
    fn test_total_instance_count_empty() {
        let r = new_instance_renderer();
        assert_eq!(total_instance_count(&r), 0);
    }

    #[test]
    fn test_instance_at_invalid_batch() {
        let r = new_instance_renderer();
        assert!(instance_at(&r, 0, 0).is_none());
    }
}
