// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Draw-call sorting utilities: front-to-back, back-to-front, by material, etc.

/// Sort strategy for draw calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortStrategy {
    /// Opaque objects: front-to-back to maximise early-z rejection.
    FrontToBack,
    /// Transparent objects: back-to-front for correct blending.
    BackToFront,
    /// Group by material id to minimise state changes.
    ByMaterial,
    /// Group by pipeline/shader id.
    ByPipeline,
    /// No sorting — preserve submission order.
    None,
}

/// A lightweight key used to sort a draw call.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct DrawSortKey {
    /// Distance from the camera (used for depth sorting).
    pub depth: f32,
    /// Material id (lower = earlier).
    pub material_id: u32,
    /// Pipeline id.
    pub pipeline_id: u32,
    /// Render layer (higher = later).
    pub layer: u8,
}

impl DrawSortKey {
    /// Create a new sort key.
    #[allow(dead_code)]
    pub fn new(depth: f32, material_id: u32, pipeline_id: u32, layer: u8) -> Self {
        Self {
            depth,
            material_id,
            pipeline_id,
            layer,
        }
    }
}

/// A draw call with its sort key and an opaque handle index.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SortedDraw {
    pub key: DrawSortKey,
    pub handle: usize,
}

/// Sort a slice of `SortedDraw` in-place according to `strategy`.
#[allow(dead_code)]
pub fn sort_draws(draws: &mut [SortedDraw], strategy: SortStrategy) {
    match strategy {
        SortStrategy::FrontToBack => draws.sort_by(|a, b| {
            a.key
                .depth
                .partial_cmp(&b.key.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        SortStrategy::BackToFront => draws.sort_by(|a, b| {
            b.key
                .depth
                .partial_cmp(&a.key.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        SortStrategy::ByMaterial => draws.sort_by_key(|d| (d.key.layer, d.key.material_id)),
        SortStrategy::ByPipeline => draws.sort_by_key(|d| (d.key.layer, d.key.pipeline_id)),
        SortStrategy::None => {}
    }
}

/// Build a sort key optimised for opaque rendering.
#[allow(dead_code)]
pub fn opaque_key(depth: f32, material_id: u32, pipeline_id: u32) -> DrawSortKey {
    DrawSortKey::new(depth, material_id, pipeline_id, 0)
}

/// Build a sort key for transparent rendering.
#[allow(dead_code)]
pub fn transparent_key(depth: f32, material_id: u32, pipeline_id: u32) -> DrawSortKey {
    DrawSortKey::new(depth, material_id, pipeline_id, 128)
}

/// Return true when the draw list is sorted front-to-back.
#[allow(dead_code)]
pub fn is_sorted_front_to_back(draws: &[SortedDraw]) -> bool {
    draws.windows(2).all(|w| w[0].key.depth <= w[1].key.depth)
}

/// Return true when the draw list is sorted back-to-front.
#[allow(dead_code)]
pub fn is_sorted_back_to_front(draws: &[SortedDraw]) -> bool {
    draws.windows(2).all(|w| w[0].key.depth >= w[1].key.depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_draw(depth: f32, mat: u32, pipe: u32, layer: u8, handle: usize) -> SortedDraw {
        SortedDraw {
            key: DrawSortKey::new(depth, mat, pipe, layer),
            handle,
        }
    }

    #[test]
    fn front_to_back_sorts_ascending() {
        let mut draws = vec![
            make_draw(3.0, 0, 0, 0, 0),
            make_draw(1.0, 0, 0, 0, 1),
            make_draw(2.0, 0, 0, 0, 2),
        ];
        sort_draws(&mut draws, SortStrategy::FrontToBack);
        assert!(is_sorted_front_to_back(&draws));
    }

    #[test]
    fn back_to_front_sorts_descending() {
        let mut draws = vec![
            make_draw(1.0, 0, 0, 0, 0),
            make_draw(3.0, 0, 0, 0, 1),
            make_draw(2.0, 0, 0, 0, 2),
        ];
        sort_draws(&mut draws, SortStrategy::BackToFront);
        assert!(is_sorted_back_to_front(&draws));
    }

    #[test]
    fn by_material_groups() {
        let mut draws = vec![
            make_draw(1.0, 2, 0, 0, 0),
            make_draw(2.0, 1, 0, 0, 1),
            make_draw(3.0, 2, 0, 0, 2),
        ];
        sort_draws(&mut draws, SortStrategy::ByMaterial);
        assert!(draws[0].key.material_id <= draws[1].key.material_id);
    }

    #[test]
    fn by_pipeline_groups() {
        let mut draws = vec![make_draw(1.0, 0, 3, 0, 0), make_draw(2.0, 0, 1, 0, 1)];
        sort_draws(&mut draws, SortStrategy::ByPipeline);
        assert!(draws[0].key.pipeline_id <= draws[1].key.pipeline_id);
    }

    #[test]
    fn none_preserves_order() {
        let mut draws = vec![make_draw(5.0, 0, 0, 0, 0), make_draw(1.0, 0, 0, 0, 1)];
        sort_draws(&mut draws, SortStrategy::None);
        assert_eq!(draws[0].handle, 0);
        assert_eq!(draws[1].handle, 1);
    }

    #[test]
    fn opaque_key_layer_zero() {
        let k = opaque_key(1.0, 0, 0);
        assert_eq!(k.layer, 0);
    }

    #[test]
    fn transparent_key_layer_nonzero() {
        let k = transparent_key(1.0, 0, 0);
        assert!(k.layer > 0);
    }

    #[test]
    fn empty_slice_sorts_cleanly() {
        let mut draws: Vec<SortedDraw> = vec![];
        sort_draws(&mut draws, SortStrategy::FrontToBack);
        assert!(draws.is_empty());
    }

    #[test]
    fn is_sorted_single_element() {
        let draws = vec![make_draw(1.0, 0, 0, 0, 0)];
        assert!(is_sorted_front_to_back(&draws));
        assert!(is_sorted_back_to_front(&draws));
    }
}
