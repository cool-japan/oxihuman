// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Draw call sorting — sorts draw calls by material, depth, and layer for efficiency.

/// Draw call sort key (higher = drawn later).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DrawSortKey(pub u64);

/// Sort strategy.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortStrategy {
    /// Sort front-to-back (opaque).
    FrontToBack,
    /// Sort back-to-front (transparent).
    BackToFront,
    /// Sort by material id to minimise state changes.
    ByMaterial,
}

/// A draw call entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawEntry {
    pub id: u32,
    pub material_id: u32,
    pub depth: f32,
    pub layer: u8,
    pub transparent: bool,
}

#[allow(dead_code)]
pub fn new_draw_entry(
    id: u32,
    material_id: u32,
    depth: f32,
    layer: u8,
    transparent: bool,
) -> DrawEntry {
    DrawEntry {
        id,
        material_id,
        depth,
        layer,
        transparent,
    }
}

#[allow(dead_code)]
pub fn dc_sort_key(entry: &DrawEntry, strategy: SortStrategy) -> DrawSortKey {
    let depth_bits = entry.depth.to_bits();
    let key = match strategy {
        SortStrategy::FrontToBack => {
            ((entry.layer as u64) << 56) | ((entry.material_id as u64) << 32) | depth_bits as u64
        }
        SortStrategy::BackToFront => {
            ((entry.layer as u64) << 56) | ((!depth_bits) as u64 & 0x00FF_FFFF_FFFF_FFFF)
        }
        SortStrategy::ByMaterial => {
            ((entry.layer as u64) << 56) | ((entry.material_id as u64) << 24)
        }
    };
    DrawSortKey(key)
}

#[allow(dead_code)]
pub fn dc_sort(entries: &mut [DrawEntry], strategy: SortStrategy) {
    entries.sort_by_key(|e| dc_sort_key(e, strategy));
}

#[allow(dead_code)]
pub fn dc_split_opaque_transparent(entries: &[DrawEntry]) -> (Vec<&DrawEntry>, Vec<&DrawEntry>) {
    let opaque: Vec<&DrawEntry> = entries.iter().filter(|e| !e.transparent).collect();
    let transparent: Vec<&DrawEntry> = entries.iter().filter(|e| e.transparent).collect();
    (opaque, transparent)
}

#[allow(dead_code)]
pub fn dc_count_by_material(entries: &[DrawEntry]) -> Vec<(u32, usize)> {
    let mut counts: Vec<(u32, usize)> = Vec::new();
    for e in entries {
        if let Some(entry) = counts.iter_mut().find(|(mat, _)| *mat == e.material_id) {
            entry.1 += 1;
        } else {
            counts.push((e.material_id, 1));
        }
    }
    counts
}

#[allow(dead_code)]
pub fn dc_batch_count(entries: &[DrawEntry]) -> usize {
    if entries.is_empty() {
        return 0;
    }
    let mut batches = 1usize;
    for i in 1..entries.len() {
        if entries[i].material_id != entries[i - 1].material_id {
            batches += 1;
        }
    }
    batches
}

#[allow(dead_code)]
pub fn dc_to_json_summary(entries: &[DrawEntry]) -> String {
    let (opaque, transparent) = dc_split_opaque_transparent(entries);
    format!(
        r#"{{"total":{},"opaque":{},"transparent":{}}}"#,
        entries.len(),
        opaque.len(),
        transparent.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_by_material_groups() {
        let mut entries = vec![
            new_draw_entry(0, 2, 1.0, 0, false),
            new_draw_entry(1, 1, 2.0, 0, false),
            new_draw_entry(2, 2, 3.0, 0, false),
        ];
        dc_sort(&mut entries, SortStrategy::ByMaterial);
        // After sorting by material the two mat=2 entries are adjacent (indices 1 and 2).
        assert_eq!(entries[1].material_id, entries[2].material_id);
    }

    #[test]
    fn sort_front_to_back_order() {
        let mut entries = vec![
            new_draw_entry(0, 0, 5.0, 0, false),
            new_draw_entry(1, 0, 1.0, 0, false),
            new_draw_entry(2, 0, 3.0, 0, false),
        ];
        dc_sort(&mut entries, SortStrategy::FrontToBack);
        assert!(entries[0].depth <= entries[1].depth);
    }

    #[test]
    fn split_opaque_transparent() {
        let entries = vec![
            new_draw_entry(0, 0, 1.0, 0, false),
            new_draw_entry(1, 0, 2.0, 0, true),
            new_draw_entry(2, 0, 3.0, 0, false),
        ];
        let (opaque, transparent) = dc_split_opaque_transparent(&entries);
        assert_eq!(opaque.len(), 2);
        assert_eq!(transparent.len(), 1);
    }

    #[test]
    fn batch_count_single() {
        let entries = vec![
            new_draw_entry(0, 1, 1.0, 0, false),
            new_draw_entry(1, 1, 2.0, 0, false),
        ];
        assert_eq!(dc_batch_count(&entries), 1);
    }

    #[test]
    fn batch_count_multiple() {
        let entries = vec![
            new_draw_entry(0, 1, 1.0, 0, false),
            new_draw_entry(1, 2, 2.0, 0, false),
        ];
        assert_eq!(dc_batch_count(&entries), 2);
    }

    #[test]
    fn batch_count_empty() {
        let entries: Vec<DrawEntry> = vec![];
        assert_eq!(dc_batch_count(&entries), 0);
    }

    #[test]
    fn count_by_material() {
        let entries = vec![
            new_draw_entry(0, 1, 1.0, 0, false),
            new_draw_entry(1, 1, 2.0, 0, false),
            new_draw_entry(2, 2, 3.0, 0, false),
        ];
        let counts = dc_count_by_material(&entries);
        let mat1 = counts.iter().find(|(m, _)| *m == 1).map(|(_, c)| *c);
        assert_eq!(mat1, Some(2));
    }

    #[test]
    fn sort_key_layer_priority() {
        let a = new_draw_entry(0, 0, 1.0, 0, false);
        let b = new_draw_entry(1, 0, 1.0, 1, false);
        let ka = dc_sort_key(&a, SortStrategy::ByMaterial);
        let kb = dc_sort_key(&b, SortStrategy::ByMaterial);
        assert!(kb > ka);
    }

    #[test]
    fn to_json_summary() {
        let entries = vec![new_draw_entry(0, 0, 1.0, 0, true)];
        let j = dc_to_json_summary(&entries);
        assert!(j.contains("transparent"));
    }
}
