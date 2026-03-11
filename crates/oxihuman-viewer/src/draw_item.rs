// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Draw item — lightweight render submission descriptor.

/// Draw item type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawItemKind {
    Opaque,
    Transparent,
    Shadow,
    Wireframe,
}

/// A single draw item for the render queue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawItem {
    pub kind: DrawItemKind,
    pub mesh_id: u32,
    pub material_id: u32,
    pub sort_key: u64,
    pub visible: bool,
}

/// Collection of draw items.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DrawItemBatch {
    pub items: Vec<DrawItem>,
}

#[allow(dead_code)]
pub fn new_draw_item(kind: DrawItemKind, mesh_id: u32, material_id: u32) -> DrawItem {
    DrawItem {
        kind,
        mesh_id,
        material_id,
        sort_key: 0,
        visible: true,
    }
}

#[allow(dead_code)]
pub fn di_set_sort_key(item: &mut DrawItem, key: u64) {
    item.sort_key = key;
}

#[allow(dead_code)]
pub fn di_set_visible(item: &mut DrawItem, visible: bool) {
    item.visible = visible;
}

#[allow(dead_code)]
pub fn di_kind_name(kind: DrawItemKind) -> &'static str {
    match kind {
        DrawItemKind::Opaque => "opaque",
        DrawItemKind::Transparent => "transparent",
        DrawItemKind::Shadow => "shadow",
        DrawItemKind::Wireframe => "wireframe",
    }
}

#[allow(dead_code)]
pub fn batch_add(batch: &mut DrawItemBatch, item: DrawItem) {
    batch.items.push(item);
}

#[allow(dead_code)]
pub fn batch_count(batch: &DrawItemBatch) -> usize {
    batch.items.len()
}

#[allow(dead_code)]
pub fn batch_visible_count(batch: &DrawItemBatch) -> usize {
    batch.items.iter().filter(|i| i.visible).count()
}

#[allow(dead_code)]
pub fn batch_sort_by_key(batch: &mut DrawItemBatch) {
    batch.items.sort_by_key(|i| i.sort_key);
}

#[allow(dead_code)]
pub fn batch_clear(batch: &mut DrawItemBatch) {
    batch.items.clear();
}

#[allow(dead_code)]
pub fn batch_count_by_kind(batch: &DrawItemBatch, kind: DrawItemKind) -> usize {
    batch.items.iter().filter(|i| i.kind == kind).count()
}

#[allow(dead_code)]
pub fn batch_to_json(batch: &DrawItemBatch) -> String {
    format!(
        r#"{{"total":{},"visible":{}}}"#,
        batch_count(batch),
        batch_visible_count(batch)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_item_visible() {
        let item = new_draw_item(DrawItemKind::Opaque, 0, 0);
        assert!(item.visible);
    }

    #[test]
    fn set_invisible() {
        let mut item = new_draw_item(DrawItemKind::Opaque, 0, 0);
        di_set_visible(&mut item, false);
        assert!(!item.visible);
    }

    #[test]
    fn kind_names() {
        assert_eq!(di_kind_name(DrawItemKind::Shadow), "shadow");
    }

    #[test]
    fn batch_empty_initially() {
        let batch = DrawItemBatch::default();
        assert_eq!(batch_count(&batch), 0);
    }

    #[test]
    fn batch_add_item() {
        let mut batch = DrawItemBatch::default();
        batch_add(&mut batch, new_draw_item(DrawItemKind::Opaque, 1, 1));
        assert_eq!(batch_count(&batch), 1);
    }

    #[test]
    fn visible_count_test() {
        let mut batch = DrawItemBatch::default();
        let mut item = new_draw_item(DrawItemKind::Transparent, 2, 2);
        di_set_visible(&mut item, false);
        batch_add(&mut batch, item);
        batch_add(&mut batch, new_draw_item(DrawItemKind::Opaque, 3, 3));
        assert_eq!(batch_visible_count(&batch), 1);
    }

    #[test]
    fn batch_sort() {
        let mut batch = DrawItemBatch::default();
        let mut a = new_draw_item(DrawItemKind::Opaque, 0, 0);
        di_set_sort_key(&mut a, 100);
        let mut b = new_draw_item(DrawItemKind::Opaque, 0, 0);
        di_set_sort_key(&mut b, 10);
        batch_add(&mut batch, a);
        batch_add(&mut batch, b);
        batch_sort_by_key(&mut batch);
        assert_eq!(batch.items[0].sort_key, 10);
    }

    #[test]
    fn clear_empties_batch() {
        let mut batch = DrawItemBatch::default();
        batch_add(&mut batch, new_draw_item(DrawItemKind::Opaque, 0, 0));
        batch_clear(&mut batch);
        assert_eq!(batch_count(&batch), 0);
    }

    #[test]
    fn count_by_kind_test() {
        let mut batch = DrawItemBatch::default();
        batch_add(&mut batch, new_draw_item(DrawItemKind::Shadow, 0, 0));
        batch_add(&mut batch, new_draw_item(DrawItemKind::Opaque, 1, 1));
        assert_eq!(batch_count_by_kind(&batch, DrawItemKind::Shadow), 1);
    }

    #[test]
    fn to_json_fields() {
        let batch = DrawItemBatch::default();
        let j = batch_to_json(&batch);
        assert!(j.contains("total"));
    }
}
