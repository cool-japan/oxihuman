// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Instance batch — GPU-instanced draw batch management.

/// Per-instance transform (column-major 4×4, stored as 4×`[f32;4]`).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceTransform {
    pub cols: [[f32; 4]; 4],
}

impl Default for InstanceTransform {
    fn default() -> Self {
        // Identity matrix
        Self {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// Per-instance data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchInstance {
    pub id: u32,
    pub transform: InstanceTransform,
    pub color_tint: [f32; 4],
    pub lod_level: u8,
    pub visible: bool,
}

/// Instance batch.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InstanceBatch {
    pub mesh_id: u32,
    instances: Vec<BatchInstance>,
}

#[allow(dead_code)]
pub fn new_instance_batch(mesh_id: u32) -> InstanceBatch {
    InstanceBatch {
        mesh_id,
        instances: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn ib_add(batch: &mut InstanceBatch, id: u32, transform: InstanceTransform) {
    batch.instances.push(BatchInstance {
        id,
        transform,
        color_tint: [1.0, 1.0, 1.0, 1.0],
        lod_level: 0,
        visible: true,
    });
}

#[allow(dead_code)]
pub fn ib_remove(batch: &mut InstanceBatch, id: u32) {
    batch.instances.retain(|i| i.id != id);
}

#[allow(dead_code)]
pub fn ib_set_visible(batch: &mut InstanceBatch, id: u32, vis: bool) {
    for inst in batch.instances.iter_mut() {
        if inst.id == id {
            inst.visible = vis;
        }
    }
}

#[allow(dead_code)]
pub fn ib_set_tint(batch: &mut InstanceBatch, id: u32, tint: [f32; 4]) {
    for inst in batch.instances.iter_mut() {
        if inst.id == id {
            inst.color_tint = tint;
        }
    }
}

#[allow(dead_code)]
pub fn ib_count(batch: &InstanceBatch) -> usize {
    batch.instances.len()
}

#[allow(dead_code)]
pub fn ib_visible_count(batch: &InstanceBatch) -> usize {
    batch.instances.iter().filter(|i| i.visible).count()
}

#[allow(dead_code)]
pub fn ib_clear(batch: &mut InstanceBatch) {
    batch.instances.clear();
}

/// Extract visible transforms for upload.
#[allow(dead_code)]
pub fn ib_visible_transforms(batch: &InstanceBatch) -> Vec<InstanceTransform> {
    batch
        .instances
        .iter()
        .filter(|i| i.visible)
        .map(|i| i.transform)
        .collect()
}

/// Estimated GPU memory bytes (64 bytes per transform).
#[allow(dead_code)]
pub fn ib_memory_bytes(batch: &InstanceBatch) -> usize {
    batch.instances.len() * 64
}

#[allow(dead_code)]
pub fn ib_to_json(batch: &InstanceBatch) -> String {
    format!(
        "{{\"mesh_id\":{},\"count\":{},\"visible\":{}}}",
        batch.mesh_id,
        ib_count(batch),
        ib_visible_count(batch)
    )
}

/// Build a translation matrix (last column only).
#[allow(dead_code)]
pub fn ib_translation(tx: f32, ty: f32, tz: f32) -> InstanceTransform {
    let mut t = InstanceTransform::default();
    t.cols[3] = [tx, ty, tz, 1.0];
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_batch() {
        let b = new_instance_batch(1);
        assert_eq!(ib_count(&b), 0);
    }

    #[test]
    fn add_increments_count() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        assert_eq!(ib_count(&b), 1);
    }

    #[test]
    fn remove_by_id() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        ib_remove(&mut b, 1);
        assert_eq!(ib_count(&b), 0);
    }

    #[test]
    fn set_invisible_reduces_visible_count() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        ib_set_visible(&mut b, 1, false);
        assert_eq!(ib_visible_count(&b), 0);
    }

    #[test]
    fn visible_transforms_filters() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        ib_add(&mut b, 2, InstanceTransform::default());
        ib_set_visible(&mut b, 1, false);
        assert_eq!(ib_visible_transforms(&b).len(), 1);
    }

    #[test]
    fn clear_empties() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        ib_clear(&mut b);
        assert_eq!(ib_count(&b), 0);
    }

    #[test]
    fn memory_bytes_per_instance() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        assert_eq!(ib_memory_bytes(&b), 64);
    }

    #[test]
    fn translation_sets_last_column() {
        let t = ib_translation(1.0, 2.0, 3.0);
        assert!((t.cols[3][0] - 1.0).abs() < 1e-6);
        assert!((t.cols[3][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn json_has_mesh_id() {
        let b = new_instance_batch(42);
        assert!(ib_to_json(&b).contains("42"));
    }

    #[test]
    fn tint_set() {
        let mut b = new_instance_batch(1);
        ib_add(&mut b, 1, InstanceTransform::default());
        ib_set_tint(&mut b, 1, [1.0, 0.0, 0.0, 1.0]);
        assert!((b.instances[0].color_tint[0] - 1.0).abs() < 1e-6);
    }
}
