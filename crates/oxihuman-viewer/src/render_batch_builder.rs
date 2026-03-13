// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Render batch builder for efficient draw call management.

/// Batch item.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchItem {
    pub mesh_id: u32,
    pub material_id: u32,
    pub transform_index: u32,
    pub sort_key: u64,
}

/// A render batch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderBatchGroup {
    pub material_id: u32,
    pub items: Vec<BatchItem>,
}

/// Batch builder.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderBatchBuilder {
    pub items: Vec<BatchItem>,
}

/// Create a new batch builder.
#[allow(dead_code)]
pub fn new_render_batch_builder() -> RenderBatchBuilder {
    RenderBatchBuilder { items: Vec::new() }
}

/// Add an item.
#[allow(dead_code)]
pub fn add_batch_item(builder: &mut RenderBatchBuilder, item: BatchItem) {
    builder.items.push(item);
}

/// Build grouped batches by material.
#[allow(dead_code)]
pub fn build_batches(builder: &RenderBatchBuilder) -> Vec<RenderBatchGroup> {
    let mut sorted = builder.items.clone();
    sorted.sort_by_key(|i| i.material_id);

    let mut groups: Vec<RenderBatchGroup> = Vec::new();
    for item in sorted {
        if groups.last().is_some_and(|g| g.material_id == item.material_id) {
            if let Some(g) = groups.last_mut() {
                g.items.push(item);
            }
        } else {
            groups.push(RenderBatchGroup {
                material_id: item.material_id,
                items: vec![item],
            });
        }
    }
    groups
}

/// Total item count.
#[allow(dead_code)]
pub fn batch_item_count(builder: &RenderBatchBuilder) -> usize {
    builder.items.len()
}

/// Clear builder.
#[allow(dead_code)]
pub fn clear_batch_builder(builder: &mut RenderBatchBuilder) {
    builder.items.clear();
}

/// Sort items by sort key.
#[allow(dead_code)]
pub fn sort_by_key(builder: &mut RenderBatchBuilder) {
    builder.items.sort_by_key(|i| i.sort_key);
}

/// Create a batch item.
#[allow(dead_code)]
pub fn new_batch_item(mesh_id: u32, material_id: u32, transform_index: u32) -> BatchItem {
    BatchItem {
        mesh_id,
        material_id,
        transform_index,
        sort_key: ((material_id as u64) << 32) | mesh_id as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_builder() {
        let b = new_render_batch_builder();
        assert!(b.items.is_empty());
    }

    #[test]
    fn test_add_item() {
        let mut b = new_render_batch_builder();
        add_batch_item(&mut b, new_batch_item(0, 0, 0));
        assert_eq!(batch_item_count(&b), 1);
    }

    #[test]
    fn test_build_single_material() {
        let mut b = new_render_batch_builder();
        add_batch_item(&mut b, new_batch_item(0, 1, 0));
        add_batch_item(&mut b, new_batch_item(1, 1, 1));
        let groups = build_batches(&b);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].items.len(), 2);
    }

    #[test]
    fn test_build_multiple_materials() {
        let mut b = new_render_batch_builder();
        add_batch_item(&mut b, new_batch_item(0, 1, 0));
        add_batch_item(&mut b, new_batch_item(1, 2, 1));
        let groups = build_batches(&b);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut b = new_render_batch_builder();
        add_batch_item(&mut b, new_batch_item(0, 0, 0));
        clear_batch_builder(&mut b);
        assert_eq!(batch_item_count(&b), 0);
    }

    #[test]
    fn test_sort_by_key() {
        let mut b = new_render_batch_builder();
        let mut item1 = new_batch_item(0, 0, 0);
        item1.sort_key = 10;
        let mut item2 = new_batch_item(1, 0, 1);
        item2.sort_key = 5;
        add_batch_item(&mut b, item1);
        add_batch_item(&mut b, item2);
        sort_by_key(&mut b);
        assert!(b.items[0].sort_key <= b.items[1].sort_key);
    }

    #[test]
    fn test_empty_build() {
        let b = new_render_batch_builder();
        let groups = build_batches(&b);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_batch_item_sort_key() {
        let item = new_batch_item(5, 3, 0);
        assert_eq!(item.sort_key, (3u64 << 32) | 5);
    }

    #[test]
    fn test_grouped_material_id() {
        let mut b = new_render_batch_builder();
        add_batch_item(&mut b, new_batch_item(0, 7, 0));
        let groups = build_batches(&b);
        assert_eq!(groups[0].material_id, 7);
    }

    #[test]
    fn test_many_items() {
        let mut b = new_render_batch_builder();
        for i in 0..100 {
            add_batch_item(&mut b, new_batch_item(i, i % 5, i));
        }
        let groups = build_batches(&b);
        assert_eq!(groups.len(), 5);
    }
}
