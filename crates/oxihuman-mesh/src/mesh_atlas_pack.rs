//! Rectangle-bin UV atlas packing using the shelf-packing algorithm.
//!
//! Packs rectangles (UV islands) into a fixed-size atlas texture.  Each shelf
//! holds rectangles of similar height; a new shelf is opened when the current
//! one is full.

#![allow(dead_code)]

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for atlas packing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtlasPackConfig {
    /// Width of the atlas texture in pixels.
    pub atlas_width: u32,
    /// Height of the atlas texture in pixels.
    pub atlas_height: u32,
    /// Padding in pixels between packed rectangles.
    pub padding: u32,
    /// Whether to allow rotating rectangles 90° if it saves space.
    pub allow_rotation: bool,
}

/// A rectangle to be packed into the atlas.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtlasRect {
    /// Logical identifier for this rectangle.
    pub id: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// X position after packing (0 if not placed).
    pub placed_x: u32,
    /// Y position after packing (0 if not placed).
    pub placed_y: u32,
    /// Whether this rectangle was successfully placed.
    pub placed: bool,
    /// Whether it was rotated during placement.
    pub rotated: bool,
}

/// Result of a packing operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtlasPackResult {
    /// All rectangles after the packing attempt.
    pub rects: Vec<AtlasRect>,
    /// Atlas width used.
    pub atlas_width: u32,
    /// Atlas height used.
    pub atlas_height: u32,
    /// Number of rectangles successfully placed.
    pub placed_count: u32,
    /// Number of rectangles that did not fit.
    pub unplaced_count: u32,
}

// ── Shelf-packing state (private) ─────────────────────────────────────────────

struct Shelf {
    x: u32,
    y: u32,
    height: u32,
    remaining_width: u32,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns a default `AtlasPackConfig`.
#[allow(dead_code)]
pub fn default_atlas_pack_config() -> AtlasPackConfig {
    AtlasPackConfig {
        atlas_width: 1024,
        atlas_height: 1024,
        padding: 1,
        allow_rotation: false,
    }
}

/// Pack `rects` into an atlas using the shelf algorithm.
///
/// Rectangles are sorted tallest-first before packing.
#[allow(dead_code)]
pub fn pack_atlas_rects(rects: &[AtlasRect], config: &AtlasPackConfig) -> AtlasPackResult {
    let pad = config.padding;
    let atlas_w = config.atlas_width;
    let atlas_h = config.atlas_height;

    // Work with indices sorted by descending height
    let mut order: Vec<usize> = (0..rects.len()).collect();
    order.sort_by(|&a, &b| rects[b].height.cmp(&rects[a].height));

    let mut out: Vec<AtlasRect> = rects.to_vec();
    let mut shelves: Vec<Shelf> = Vec::new();
    let mut cursor_y = 0u32;

    for &i in &order {
        let mut w = out[i].width;
        let mut h = out[i].height;

        // Optionally try rotation
        let mut rotated = false;
        if config.allow_rotation && h > w {
            std::mem::swap(&mut w, &mut h);
            rotated = true;
        }

        let needed_w = w + pad;
        let needed_h = h + pad;

        // Find a shelf with enough width and matching height
        let shelf_idx = shelves.iter().position(|s| {
            s.remaining_width >= needed_w && s.height >= h
        });

        if let Some(si) = shelf_idx {
            let s = &mut shelves[si];
            out[i].placed_x = s.x + atlas_w - s.remaining_width;
            out[i].placed_y = s.y;
            out[i].placed = true;
            out[i].rotated = rotated;
            s.remaining_width -= needed_w;
        } else if cursor_y + needed_h <= atlas_h && needed_w <= atlas_w {
            // Open a new shelf
            out[i].placed_x = 0;
            out[i].placed_y = cursor_y;
            out[i].placed = true;
            out[i].rotated = rotated;
            shelves.push(Shelf {
                x: 0,
                y: cursor_y,
                height: h,
                remaining_width: atlas_w - needed_w,
            });
            cursor_y += needed_h;
        } else {
            out[i].placed = false;
        }
    }

    let placed_count = out.iter().filter(|r| r.placed).count() as u32;
    let unplaced_count = out.iter().filter(|r| !r.placed).count() as u32;

    AtlasPackResult {
        rects: out,
        atlas_width: atlas_w,
        atlas_height: atlas_h,
        placed_count,
        unplaced_count,
    }
}

/// Return the number of rectangles in a pack result.
#[allow(dead_code)]
pub fn atlas_pack_rect_count(result: &AtlasPackResult) -> usize {
    result.rects.len()
}

/// Return the fraction of atlas area occupied by placed rectangles.
#[allow(dead_code)]
pub fn atlas_pack_utilization(result: &AtlasPackResult) -> f32 {
    let atlas_area = result.atlas_width as f64 * result.atlas_height as f64;
    if atlas_area < 1.0 {
        return 0.0;
    }
    let used: f64 = result
        .rects
        .iter()
        .filter(|r| r.placed)
        .map(|r| r.width as f64 * r.height as f64)
        .sum();
    (used / atlas_area) as f32
}

/// Return `true` if the rectangle at `index` was successfully placed.
#[allow(dead_code)]
pub fn atlas_rect_placed(result: &AtlasPackResult, index: usize) -> bool {
    result.rects.get(index).is_some_and(|r| r.placed)
}

/// Return the total area covered by all placed rectangles.
#[allow(dead_code)]
pub fn atlas_total_area(result: &AtlasPackResult) -> u64 {
    result
        .rects
        .iter()
        .filter(|r| r.placed)
        .map(|r| r.width as u64 * r.height as u64)
        .sum()
}

/// Serialise the pack result to a JSON string.
#[allow(dead_code)]
pub fn atlas_pack_to_json(result: &AtlasPackResult) -> String {
    format!(
        "{{\"atlas_width\":{},\"atlas_height\":{},\"placed\":{},\"unplaced\":{}}}",
        result.atlas_width, result.atlas_height, result.placed_count, result.unplaced_count,
    )
}

/// Reset all rectangles in a result to un-placed state.
#[allow(dead_code)]
pub fn atlas_pack_reset(result: &mut AtlasPackResult) {
    for r in result.rects.iter_mut() {
        r.placed = false;
        r.placed_x = 0;
        r.placed_y = 0;
        r.rotated = false;
    }
    result.placed_count = 0;
    result.unplaced_count = result.rects.len() as u32;
}

/// Return `true` if a single rectangle of `(w, h)` would fit in the atlas.
#[allow(dead_code)]
pub fn atlas_pack_fits(config: &AtlasPackConfig, w: u32, h: u32) -> bool {
    w <= config.atlas_width && h <= config.atlas_height
}

/// Return the atlas width from a config.
#[allow(dead_code)]
pub fn atlas_pack_width(config: &AtlasPackConfig) -> u32 {
    config.atlas_width
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rect(id: u32, w: u32, h: u32) -> AtlasRect {
        AtlasRect { id, width: w, height: h, placed_x: 0, placed_y: 0, placed: false, rotated: false }
    }

    #[test]
    fn test_default_config_positive_dimensions() {
        let cfg = default_atlas_pack_config();
        assert!(cfg.atlas_width > 0);
        assert!(cfg.atlas_height > 0);
    }

    #[test]
    fn test_pack_single_rect_placed() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 100, 100)];
        let result = pack_atlas_rects(&rects, &cfg);
        assert_eq!(result.placed_count, 1);
        assert_eq!(result.unplaced_count, 0);
    }

    #[test]
    fn test_pack_oversized_rect_not_placed() {
        let cfg = AtlasPackConfig { atlas_width: 64, atlas_height: 64, ..default_atlas_pack_config() };
        let rects = vec![make_rect(0, 200, 200)];
        let result = pack_atlas_rects(&rects, &cfg);
        assert_eq!(result.placed_count, 0);
        assert_eq!(result.unplaced_count, 1);
    }

    #[test]
    fn test_pack_multiple_rects() {
        let cfg = default_atlas_pack_config();
        let rects = vec![
            make_rect(0, 100, 100),
            make_rect(1, 200, 50),
            make_rect(2, 150, 150),
        ];
        let result = pack_atlas_rects(&rects, &cfg);
        assert_eq!(result.placed_count + result.unplaced_count, rects.len() as u32);
        assert!(result.placed_count > 0);
    }

    #[test]
    fn test_atlas_pack_rect_count() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 10, 10), make_rect(1, 20, 20)];
        let result = pack_atlas_rects(&rects, &cfg);
        assert_eq!(atlas_pack_rect_count(&result), 2);
    }

    #[test]
    fn test_atlas_pack_utilization_range() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 100, 100)];
        let result = pack_atlas_rects(&rects, &cfg);
        let util = atlas_pack_utilization(&result);
        assert!((0.0..=1.0).contains(&util));
    }

    #[test]
    fn test_atlas_rect_placed_flag() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 100, 100)];
        let result = pack_atlas_rects(&rects, &cfg);
        assert!(atlas_rect_placed(&result, 0));
    }

    #[test]
    fn test_atlas_total_area() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 100, 100)];
        let result = pack_atlas_rects(&rects, &cfg);
        assert_eq!(atlas_total_area(&result), 10_000);
    }

    #[test]
    fn test_atlas_pack_to_json() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 50, 50)];
        let result = pack_atlas_rects(&rects, &cfg);
        let json = atlas_pack_to_json(&result);
        assert!(json.contains("atlas_width"));
        assert!(json.contains("placed"));
    }

    #[test]
    fn test_atlas_pack_reset() {
        let cfg = default_atlas_pack_config();
        let rects = vec![make_rect(0, 50, 50)];
        let mut result = pack_atlas_rects(&rects, &cfg);
        atlas_pack_reset(&mut result);
        assert_eq!(result.placed_count, 0);
        assert!(result.rects.iter().all(|r| !r.placed));
    }

    #[test]
    fn test_atlas_pack_fits_small() {
        let cfg = default_atlas_pack_config();
        assert!(atlas_pack_fits(&cfg, 100, 100));
    }

    #[test]
    fn test_atlas_pack_fits_oversized() {
        let cfg = AtlasPackConfig { atlas_width: 32, atlas_height: 32, ..default_atlas_pack_config() };
        assert!(!atlas_pack_fits(&cfg, 64, 64));
    }

    #[test]
    fn test_atlas_pack_width() {
        let cfg = AtlasPackConfig { atlas_width: 512, ..default_atlas_pack_config() };
        assert_eq!(atlas_pack_width(&cfg), 512);
    }

    #[test]
    fn test_empty_rect_list() {
        let cfg = default_atlas_pack_config();
        let result = pack_atlas_rects(&[], &cfg);
        assert_eq!(result.placed_count, 0);
        assert_eq!(result.unplaced_count, 0);
    }
}
