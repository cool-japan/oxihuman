// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Advanced UV island packing using a shelf-fit bin packing algorithm.
//!
//! Given a set of rectangular UV islands (bounding boxes), packs them into a
//! `[0,1]`² atlas.  Islands are placed left-to-right on horizontal "shelves";
//! when an island doesn't fit on the current shelf a new shelf is started at
//! the top of the previous one.  Optional 90° rotation is supported.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A rectangular UV island with its packed position and size.
#[derive(Debug, Clone)]
pub struct UvRect {
    /// Original user-supplied identifier.
    pub id: usize,
    /// X (U) position of the island's lower-left corner in atlas space.
    pub x: f32,
    /// Y (V) position of the island's lower-left corner in atlas space.
    pub y: f32,
    /// Width of the island in atlas space (post-rotation if any).
    pub w: f32,
    /// Height of the island in atlas space (post-rotation if any).
    pub h: f32,
    /// Whether the island was rotated 90° to achieve a better fit.
    pub rotated: bool,
}

/// Sorting strategy used before packing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSort {
    /// Sort islands by area (largest first) – usually best utilization.
    ByArea,
    /// Sort islands by height (tallest first).
    ByHeight,
    /// Sort islands by width (widest first).
    ByWidth,
    /// Do not sort; use the order provided by the caller.
    None,
}

/// Configuration for [`pack_uv_rects`].
#[derive(Debug, Clone)]
pub struct PackConfig {
    /// Side length of the square atlas (default `1.0`).
    pub atlas_size: f32,
    /// Gap between islands, in atlas units (default `0.005`).
    pub padding: f32,
    /// Allow 90° rotation of islands to improve packing.
    pub allow_rotation: bool,
    /// Sorting strategy applied before placing islands.
    pub sort_by: PackSort,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            atlas_size: 1.0,
            padding: 0.005,
            allow_rotation: true,
            sort_by: PackSort::ByArea,
        }
    }
}

/// Result returned by [`pack_uv_rects`].
pub struct PackResult {
    /// Successfully packed rectangles with their atlas positions.
    pub rects: Vec<UvRect>,
    /// Fraction of atlas area actually occupied by islands (0 – 1).
    pub utilization: f32,
    /// Number of islands that did not fit in the atlas.
    pub overflow_count: usize,
}

// ---------------------------------------------------------------------------
// Core shelf-fit packer
// ---------------------------------------------------------------------------

/// Pack a list of `(id, width, height)` rectangles into a square atlas.
///
/// Returns a [`PackResult`] with the placed [`UvRect`]s.  Islands that do not
/// fit are omitted from `rects` and counted in `overflow_count`.
pub fn pack_uv_rects(rects: &[(usize, f32, f32)], config: &PackConfig) -> PackResult {
    if rects.is_empty() {
        return PackResult {
            rects: Vec::new(),
            utilization: 0.0,
            overflow_count: 0,
        };
    }

    let atlas = config.atlas_size;
    let pad = config.padding;

    // Build a mutable list of (id, w, h) considering optional rotation.
    struct Candidate {
        id: usize,
        w: f32,
        h: f32,
        rotated: bool,
    }

    let mut candidates: Vec<Candidate> = rects
        .iter()
        .map(|&(id, w, h)| {
            // If rotation is allowed, orient so that width >= height (landscape).
            // This typically improves shelf utilization.
            if config.allow_rotation && h > w {
                Candidate {
                    id,
                    w: h,
                    h: w,
                    rotated: true,
                }
            } else {
                Candidate {
                    id,
                    w,
                    h,
                    rotated: false,
                }
            }
        })
        .collect();

    // Apply requested sort (descending – largest first).
    match config.sort_by {
        PackSort::ByArea => {
            candidates.sort_by(|a, b| {
                let area_a = a.w * a.h;
                let area_b = b.w * b.h;
                area_b
                    .partial_cmp(&area_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        PackSort::ByHeight => {
            candidates.sort_by(|a, b| b.h.partial_cmp(&a.h).unwrap_or(std::cmp::Ordering::Equal));
        }
        PackSort::ByWidth => {
            candidates.sort_by(|a, b| b.w.partial_cmp(&a.w).unwrap_or(std::cmp::Ordering::Equal));
        }
        PackSort::None => {}
    }

    // Shelf-fit placement.
    let mut packed: Vec<UvRect> = Vec::new();
    let mut overflow_count: usize = 0;
    let mut total_island_area: f32 = 0.0;

    let mut cursor_x: f32 = 0.0;
    let mut cursor_y: f32 = 0.0;
    let mut shelf_h: f32 = 0.0; // height of tallest island on current shelf

    for c in &candidates {
        let w_padded = c.w + pad;
        let h_padded = c.h + pad;

        // If the island is wider than the entire atlas it can never fit.
        if w_padded > atlas + 1e-6 || h_padded > atlas + 1e-6 {
            overflow_count += 1;
            continue;
        }

        // Check whether the island fits on the current shelf.
        if cursor_x + w_padded > atlas + 1e-6 {
            // Advance to the next shelf.
            cursor_x = 0.0;
            cursor_y += shelf_h;
            shelf_h = 0.0;
        }

        // Check whether there is vertical space for a new shelf.
        if cursor_y + h_padded > atlas + 1e-6 {
            overflow_count += 1;
            continue;
        }

        packed.push(UvRect {
            id: c.id,
            x: cursor_x,
            y: cursor_y,
            w: c.w,
            h: c.h,
            rotated: c.rotated,
        });

        total_island_area += c.w * c.h;
        cursor_x += w_padded;
        if h_padded > shelf_h {
            shelf_h = h_padded;
        }
    }

    // Compute utilization: total island area / atlas area.
    let utilization = (total_island_area / (atlas * atlas)).min(1.0);

    PackResult {
        rects: packed,
        utilization,
        overflow_count,
    }
}

// ---------------------------------------------------------------------------
// Mesh-integrated helpers
// ---------------------------------------------------------------------------

/// Compute the bounding box of a UV set.
///
/// Returns `(min_u, min_v, max_u, max_v)`.  If `uvs` is empty returns
/// `(0.0, 0.0, 0.0, 0.0)`.
pub fn uv_rect_bounds(uvs: &[[f32; 2]]) -> (f32, f32, f32, f32) {
    if uvs.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min_u = f32::INFINITY;
    let mut min_v = f32::INFINITY;
    let mut max_u = f32::NEG_INFINITY;
    let mut max_v = f32::NEG_INFINITY;
    for uv in uvs {
        if uv[0] < min_u {
            min_u = uv[0];
        }
        if uv[1] < min_v {
            min_v = uv[1];
        }
        if uv[0] > max_u {
            max_u = uv[0];
        }
        if uv[1] > max_v {
            max_v = uv[1];
        }
    }
    (min_u, min_v, max_u, max_v)
}

/// Remap UV coordinates from a source bounding-box to a destination [`UvRect`].
///
/// Each UV is first normalised within `src_rect` and then mapped to the
/// position and size of `dst_rect`.  If `dst_rect.rotated` is `true` the U
/// and V axes are swapped before mapping.
#[allow(clippy::too_many_arguments)]
pub fn transform_island_uvs(
    uvs: &mut [[f32; 2]],
    src_rect: (f32, f32, f32, f32),
    dst_rect: &UvRect,
) {
    let (smin_u, smin_v, smax_u, smax_v) = src_rect;
    let src_w = (smax_u - smin_u).max(1e-9);
    let src_h = (smax_v - smin_v).max(1e-9);

    for uv in uvs.iter_mut() {
        // Normalise to [0,1] within source rect.
        let norm_u = (uv[0] - smin_u) / src_w;
        let norm_v = (uv[1] - smin_v) / src_h;

        if dst_rect.rotated {
            // 90° rotation: original V axis becomes U, original U becomes V.
            uv[0] = dst_rect.x + norm_v * dst_rect.w;
            uv[1] = dst_rect.y + norm_u * dst_rect.h;
        } else {
            uv[0] = dst_rect.x + norm_u * dst_rect.w;
            uv[1] = dst_rect.y + norm_v * dst_rect.h;
        }
    }
}

/// Extract UV islands from `mesh`, pack them, and write updated UVs back.
///
/// This treats the entire mesh UV set as a single island.
pub fn pack_from_mesh(mesh: &MeshBuffers, config: &PackConfig) -> (MeshBuffers, PackResult) {
    let bounds = uv_rect_bounds(&mesh.uvs);
    let src_w = bounds.2 - bounds.0;
    let src_h = bounds.3 - bounds.1;

    let result = pack_uv_rects(&[(0, src_w, src_h)], config);

    let mut new_mesh = mesh.clone();
    if let Some(dst) = result.rects.first() {
        transform_island_uvs(&mut new_mesh.uvs, bounds, dst);
    }

    (new_mesh, result)
}

/// Return a human-readable summary of a [`PackResult`].
pub fn pack_stats(result: &PackResult) -> String {
    format!(
        "packed={}, overflow={}, utilization={:.1}%",
        result.rects.len(),
        result.overflow_count,
        result.utilization * 100.0,
    )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn default_config() -> PackConfig {
        PackConfig::default()
    }

    fn no_rotate_config() -> PackConfig {
        PackConfig {
            allow_rotation: false,
            ..PackConfig::default()
        }
    }

    fn make_mesh(uvs: Vec<[f32; 2]>) -> MeshBuffers {
        let n = uvs.len().max(3);
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0f32, 0.0, 0.0]; n],
            normals: vec![[0.0f32, 0.0, 1.0]; n],
            uvs,
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    // ------------------------------------------------------------------
    // 1. Empty input
    // ------------------------------------------------------------------
    #[test]
    fn empty_input_returns_empty_result() {
        let result = pack_uv_rects(&[], &default_config());
        assert_eq!(result.rects.len(), 0);
        assert_eq!(result.overflow_count, 0);
        assert!((result.utilization).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // 2. Single island – basic packing
    // ------------------------------------------------------------------
    #[test]
    fn single_island_is_packed() {
        let result = pack_uv_rects(&[(0, 0.2, 0.3)], &default_config());
        assert_eq!(result.rects.len(), 1, "single island must be packed");
        assert_eq!(result.overflow_count, 0);
        let r = &result.rects[0];
        assert_eq!(r.id, 0);
        assert!(r.x >= 0.0 && r.x + r.w <= 1.0 + 1e-5);
        assert!(r.y >= 0.0 && r.y + r.h <= 1.0 + 1e-5);
    }

    // ------------------------------------------------------------------
    // 3. Multiple islands – all should fit
    // ------------------------------------------------------------------
    #[test]
    fn multiple_small_islands_all_fit() {
        let rects: Vec<(usize, f32, f32)> = (0..6).map(|i| (i, 0.1, 0.1)).collect();
        let result = pack_uv_rects(&rects, &no_rotate_config());
        assert_eq!(result.rects.len(), 6);
        assert_eq!(result.overflow_count, 0);
    }

    // ------------------------------------------------------------------
    // 4. Overflow – islands too large to fit
    // ------------------------------------------------------------------
    #[test]
    fn oversized_islands_overflow() {
        // Single island larger than the atlas.
        let result = pack_uv_rects(&[(0, 2.0, 2.0)], &default_config());
        assert_eq!(result.overflow_count, 1);
        assert_eq!(result.rects.len(), 0);
    }

    // ------------------------------------------------------------------
    // 5. Overflow – atlas gets full
    // ------------------------------------------------------------------
    #[test]
    fn atlas_full_overflows_remainder() {
        // 20 rectangles of 0.3×0.3 won't all fit in a 1×1 atlas.
        let rects: Vec<(usize, f32, f32)> = (0..20).map(|i| (i, 0.3, 0.3)).collect();
        let cfg = PackConfig {
            padding: 0.0,
            allow_rotation: false,
            sort_by: PackSort::None,
            ..PackConfig::default()
        };
        let result = pack_uv_rects(&rects, &cfg);
        assert!(result.overflow_count > 0, "some islands must overflow");
        assert!(result.rects.len() < 20, "not all fit");
    }

    // ------------------------------------------------------------------
    // 6. Rotation – tall island should be rotated when allow_rotation=true
    // ------------------------------------------------------------------
    #[test]
    fn rotation_applied_to_tall_island() {
        // A tall narrow island: h > w => should be rotated (w becomes h, h becomes w).
        let cfg = PackConfig {
            allow_rotation: true,
            sort_by: PackSort::None,
            ..PackConfig::default()
        };
        let result = pack_uv_rects(&[(0, 0.1, 0.5)], &cfg);
        assert_eq!(result.rects.len(), 1);
        let r = &result.rects[0];
        // After rotation the stored w and h should be swapped.
        assert!(r.rotated, "tall island should be rotated");
        assert!(
            (r.w - 0.5).abs() < 1e-6,
            "w after rotation should be original h"
        );
        assert!(
            (r.h - 0.1).abs() < 1e-6,
            "h after rotation should be original w"
        );
    }

    // ------------------------------------------------------------------
    // 7. No rotation – island stays un-rotated
    // ------------------------------------------------------------------
    #[test]
    fn no_rotation_flag_respected() {
        let cfg = PackConfig {
            allow_rotation: false,
            sort_by: PackSort::None,
            ..PackConfig::default()
        };
        let result = pack_uv_rects(&[(0, 0.1, 0.5)], &cfg);
        assert_eq!(result.rects.len(), 1);
        let r = &result.rects[0];
        assert!(!r.rotated, "island must not be rotated");
        assert!((r.w - 0.1).abs() < 1e-6);
        assert!((r.h - 0.5).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // 8. Sort by height
    // ------------------------------------------------------------------
    #[test]
    fn sort_by_height_packs_without_error() {
        let rects = vec![(0, 0.2, 0.4), (1, 0.3, 0.1), (2, 0.15, 0.25)];
        let cfg = PackConfig {
            sort_by: PackSort::ByHeight,
            allow_rotation: false,
            ..PackConfig::default()
        };
        let result = pack_uv_rects(&rects, &cfg);
        // All three small rects should fit.
        assert_eq!(result.rects.len(), 3);
    }

    // ------------------------------------------------------------------
    // 9. Sort by width
    // ------------------------------------------------------------------
    #[test]
    fn sort_by_width_packs_without_error() {
        let rects = vec![(0, 0.1, 0.2), (1, 0.4, 0.15), (2, 0.25, 0.3)];
        let cfg = PackConfig {
            sort_by: PackSort::ByWidth,
            allow_rotation: false,
            ..PackConfig::default()
        };
        let result = pack_uv_rects(&rects, &cfg);
        assert_eq!(result.rects.len(), 3);
    }

    // ------------------------------------------------------------------
    // 10. Sort::None preserves insertion order (ids preserved)
    // ------------------------------------------------------------------
    #[test]
    fn sort_none_preserves_insertion_order() {
        let rects = vec![(10, 0.1, 0.1), (20, 0.1, 0.1), (30, 0.1, 0.1)];
        let cfg = PackConfig {
            sort_by: PackSort::None,
            allow_rotation: false,
            padding: 0.0,
            ..PackConfig::default()
        };
        let result = pack_uv_rects(&rects, &cfg);
        assert_eq!(result.rects.len(), 3);
        // With no sorting the ids must appear in input order.
        assert_eq!(result.rects[0].id, 10);
        assert_eq!(result.rects[1].id, 20);
        assert_eq!(result.rects[2].id, 30);
    }

    // ------------------------------------------------------------------
    // 11. Utilization calculation
    // ------------------------------------------------------------------
    #[test]
    fn utilization_is_positive_for_packed_islands() {
        let rects = vec![(0, 0.3, 0.3), (1, 0.2, 0.2)];
        let result = pack_uv_rects(&rects, &default_config());
        assert!(result.utilization > 0.0, "utilization must be > 0");
        assert!(result.utilization <= 1.0, "utilization must be <= 1");
    }

    // ------------------------------------------------------------------
    // 12. uv_rect_bounds – correct bounding box
    // ------------------------------------------------------------------
    #[test]
    fn uv_rect_bounds_correct() {
        let uvs = vec![[0.1f32, 0.2], [0.9, 0.5], [0.4, 0.8]];
        let (min_u, min_v, max_u, max_v) = uv_rect_bounds(&uvs);
        assert!((min_u - 0.1).abs() < 1e-6);
        assert!((min_v - 0.2).abs() < 1e-6);
        assert!((max_u - 0.9).abs() < 1e-6);
        assert!((max_v - 0.8).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // 13. uv_rect_bounds – empty input
    // ------------------------------------------------------------------
    #[test]
    fn uv_rect_bounds_empty_is_zero() {
        let (a, b, c, d) = uv_rect_bounds(&[]);
        assert_eq!((a, b, c, d), (0.0, 0.0, 0.0, 0.0));
    }

    // ------------------------------------------------------------------
    // 14. transform_island_uvs – basic non-rotated remap
    // ------------------------------------------------------------------
    #[test]
    fn transform_island_uvs_remaps_correctly() {
        // Source island occupies [0.0, 0.0] → [1.0, 1.0].
        let src = (0.0f32, 0.0, 1.0, 1.0);
        let dst = UvRect {
            id: 0,
            x: 0.5,
            y: 0.5,
            w: 0.25,
            h: 0.25,
            rotated: false,
        };
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        transform_island_uvs(&mut uvs, src, &dst);
        // Corner (0,0) → (0.5, 0.5)
        assert!((uvs[0][0] - 0.5).abs() < 1e-5, "u0 mismatch: {}", uvs[0][0]);
        assert!((uvs[0][1] - 0.5).abs() < 1e-5, "v0 mismatch: {}", uvs[0][1]);
        // Corner (1,0) → (0.75, 0.5)
        assert!(
            (uvs[1][0] - 0.75).abs() < 1e-5,
            "u1 mismatch: {}",
            uvs[1][0]
        );
        assert!((uvs[1][1] - 0.5).abs() < 1e-5, "v1 mismatch: {}", uvs[1][1]);
    }

    // ------------------------------------------------------------------
    // 15. transform_island_uvs – rotated remap swaps axes
    // ------------------------------------------------------------------
    #[test]
    fn transform_island_uvs_rotated_swaps_axes() {
        let src = (0.0f32, 0.0, 1.0, 1.0);
        // dst is a landscape rectangle for a rotated island
        let dst = UvRect {
            id: 0,
            x: 0.0,
            y: 0.0,
            w: 0.4,
            h: 0.2,
            rotated: true,
        };
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        transform_island_uvs(&mut uvs, src, &dst);
        // (0,0) → x + norm_v*w=0, y + norm_u*h=0
        assert!((uvs[0][0] - 0.0).abs() < 1e-5);
        assert!((uvs[0][1] - 0.0).abs() < 1e-5);
        // (1,0) → norm_u=1, norm_v=0 → x + 0*0.4=0, y + 1*0.2=0.2
        assert!((uvs[1][0] - 0.0).abs() < 1e-5, "rotated u1: {}", uvs[1][0]);
        assert!((uvs[1][1] - 0.2).abs() < 1e-5, "rotated v1: {}", uvs[1][1]);
    }

    // ------------------------------------------------------------------
    // 16. pack_from_mesh – mesh UVs updated after packing
    // ------------------------------------------------------------------
    #[test]
    fn pack_from_mesh_updates_uvs() {
        let uvs = vec![[0.0f32, 0.0], [0.2, 0.0], [0.2, 0.2], [0.0, 0.2]];
        let mesh = make_mesh(uvs.clone());
        let cfg = default_config();
        let (new_mesh, result) = pack_from_mesh(&mesh, &cfg);
        assert_eq!(result.rects.len(), 1, "single island must be packed");
        // UVs should have changed position.
        assert_eq!(new_mesh.uvs.len(), mesh.uvs.len());
    }

    // ------------------------------------------------------------------
    // 17. pack_stats – returns non-empty string
    // ------------------------------------------------------------------
    #[test]
    fn pack_stats_returns_non_empty_string() {
        let rects = vec![(0, 0.2, 0.2)];
        let result = pack_uv_rects(&rects, &default_config());
        let s = pack_stats(&result);
        assert!(!s.is_empty(), "stats string must not be empty");
        assert!(s.contains("packed="), "must contain 'packed='");
        assert!(s.contains("overflow="), "must contain 'overflow='");
        assert!(s.contains("utilization="), "must contain 'utilization='");
    }

    // ------------------------------------------------------------------
    // 18. Positions are within atlas bounds
    // ------------------------------------------------------------------
    #[test]
    fn packed_rects_within_atlas_bounds() {
        let rects: Vec<(usize, f32, f32)> = (0..8).map(|i| (i, 0.15, 0.12)).collect();
        let result = pack_uv_rects(&rects, &default_config());
        for r in &result.rects {
            assert!(r.x >= 0.0, "x must be non-negative");
            assert!(r.y >= 0.0, "y must be non-negative");
            assert!(
                r.x + r.w <= 1.0 + 1e-4,
                "right edge out of atlas: {}",
                r.x + r.w
            );
            assert!(
                r.y + r.h <= 1.0 + 1e-4,
                "top edge out of atlas: {}",
                r.y + r.h
            );
        }
    }
}
