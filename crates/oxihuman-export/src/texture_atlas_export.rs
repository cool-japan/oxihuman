// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Texture atlas pipeline for export.
//!
//! Packs multiple RGBA textures into a single atlas using a shelf-packing algorithm.
//! This module operates on CPU-side RGBA pixel data; no GPU or image-codec dependencies.

// ── Types ─────────────────────────────────────────────────────────────────────

/// A rectangular sub-region inside a `TextureAtlas`.
#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

/// A packed texture atlas containing RGBA pixel data and region metadata.
#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
    pub regions: Vec<AtlasRegion>,
    pub padding: u32,
}

/// Input texture to be packed into an atlas.
#[derive(Debug, Clone)]
pub struct AtlasInput {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}

// ── Shelf state (private) ─────────────────────────────────────────────────────

/// Tracks the current shelf cursor used by `find_free_space`.
#[derive(Debug, Clone, Default)]
struct ShelfCursor {
    /// X offset of the next position on the current shelf.
    cursor_x: u32,
    /// Y offset where the current shelf starts.
    shelf_y: u32,
    /// Height of the tallest item placed on the current shelf.
    shelf_h: u32,
}

// We store the cursor inside the atlas via a simple linear search over existing regions.
// For simplicity we recompute the shelf state from the regions list each time.

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a new empty `TextureAtlas` with the given dimensions and inter-texture padding.
#[allow(dead_code)]
pub fn new_texture_atlas(width: u32, height: u32, padding: u32) -> TextureAtlas {
    let n = (width as usize) * (height as usize);
    TextureAtlas {
        width,
        height,
        pixels: vec![[0, 0, 0, 0]; n],
        regions: Vec::new(),
        padding,
    }
}

/// Pack multiple textures into a single atlas using greedy row (shelf) packing.
/// Returns an atlas with all successfully placed textures.
#[allow(dead_code)]
pub fn pack_textures(inputs: Vec<AtlasInput>, atlas_size: u32, padding: u32) -> TextureAtlas {
    let mut atlas = new_texture_atlas(atlas_size, atlas_size, padding);
    for input in &inputs {
        add_region(&mut atlas, input);
    }
    atlas
}

/// Find space for a texture and blit it into the atlas.
/// Returns the placed `AtlasRegion` on success, `None` if it does not fit.
#[allow(dead_code)]
pub fn add_region(atlas: &mut TextureAtlas, input: &AtlasInput) -> Option<AtlasRegion> {
    let padded_w = input.width + atlas.padding;
    let padded_h = input.height + atlas.padding;
    let (x, y) = find_free_space(atlas, padded_w, padded_h)?;
    // Actual blit uses unpadded dimensions
    blit_to_atlas(atlas, input, x, y);
    let region = AtlasRegion {
        id: input.id,
        x,
        y,
        width: input.width,
        height: input.height,
        uv_min: [
            x as f32 / atlas.width as f32,
            y as f32 / atlas.height as f32,
        ],
        uv_max: [
            (x + input.width) as f32 / atlas.width as f32,
            (y + input.height) as f32 / atlas.height as f32,
        ],
    };
    atlas.regions.push(region.clone());
    Some(region)
}

/// Copy `src` pixel data into `atlas` at position `(x, y)`.
#[allow(dead_code)]
pub fn blit_to_atlas(atlas: &mut TextureAtlas, src: &AtlasInput, x: u32, y: u32) {
    let aw = atlas.width as usize;
    let sw = src.width as usize;
    let sh = src.height as usize;
    for row in 0..sh {
        let dst_y = y as usize + row;
        if dst_y >= atlas.height as usize {
            break;
        }
        for col in 0..sw {
            let dst_x = x as usize + col;
            if dst_x >= aw {
                break;
            }
            let src_idx = row * sw + col;
            let dst_idx = dst_y * aw + dst_x;
            if src_idx < src.pixels.len() && dst_idx < atlas.pixels.len() {
                atlas.pixels[dst_idx] = src.pixels[src_idx];
            }
        }
    }
}

/// Find a free (x, y) position for a texture of size `(w, h)` using shelf packing.
/// Returns `None` if there is no space.
#[allow(dead_code)]
pub fn find_free_space(atlas: &TextureAtlas, w: u32, h: u32) -> Option<(u32, u32)> {
    // Recompute shelf state from existing regions.
    let cursor = compute_shelf_cursor(atlas);
    let aw = atlas.width;
    let ah = atlas.height;

    // Try to place on current shelf.
    if cursor.cursor_x + w <= aw && cursor.shelf_y + h <= ah {
        return Some((cursor.cursor_x, cursor.shelf_y));
    }

    // Try next shelf.
    let next_shelf_y = cursor.shelf_y + cursor.shelf_h + atlas.padding;
    if w <= aw && next_shelf_y + h <= ah {
        return Some((0, next_shelf_y));
    }

    None
}

/// Compute the current shelf cursor from existing atlas regions.
fn compute_shelf_cursor(atlas: &TextureAtlas) -> ShelfCursor {
    if atlas.regions.is_empty() {
        return ShelfCursor {
            cursor_x: 0,
            shelf_y: 0,
            shelf_h: 0,
        };
    }

    // Find the highest shelf_y used.
    let max_y = atlas.regions.iter().map(|r| r.y).max().unwrap_or(0);

    // All regions on the last shelf (same y).
    let last_shelf_regions: Vec<&AtlasRegion> =
        atlas.regions.iter().filter(|r| r.y == max_y).collect();

    let max_x_end = last_shelf_regions
        .iter()
        .map(|r| r.x + r.width + atlas.padding)
        .max()
        .unwrap_or(0);

    let shelf_h = last_shelf_regions
        .iter()
        .map(|r| r.height)
        .max()
        .unwrap_or(0);

    ShelfCursor {
        cursor_x: max_x_end,
        shelf_y: max_y,
        shelf_h,
    }
}

/// Compute the fraction of atlas area used by placed textures.
#[allow(dead_code)]
pub fn atlas_utilization(atlas: &TextureAtlas) -> f32 {
    let total = (atlas.width as u64) * (atlas.height as u64);
    if total == 0 {
        return 0.0;
    }
    let used: u64 = atlas
        .regions
        .iter()
        .map(|r| (r.width as u64) * (r.height as u64))
        .sum();
    used as f32 / total as f32
}

/// Produce a minimal stub byte sequence that represents this atlas.
/// Not a real PNG — just a header placeholder followed by raw RGBA row data.
/// Useful for size estimation and testing pipelines that need a byte blob.
#[allow(dead_code)]
pub fn atlas_to_png_stub(atlas: &TextureAtlas) -> Vec<u8> {
    // Stub: 8-byte magic + 4-byte width + 4-byte height + raw RGBA pixels.
    let mut out = Vec::new();
    // PNG-like magic (not real PNG)
    out.extend_from_slice(b"\x89OXA\r\n\x1a\n");
    out.extend_from_slice(&atlas.width.to_le_bytes());
    out.extend_from_slice(&atlas.height.to_le_bytes());
    for px in &atlas.pixels {
        out.extend_from_slice(px);
    }
    out
}

/// Find the region with the given id.
#[allow(dead_code)]
pub fn atlas_region_for_id(atlas: &TextureAtlas, id: u32) -> Option<&AtlasRegion> {
    atlas.regions.iter().find(|r| r.id == id)
}

/// Sample the atlas at normalized UV coordinates (bilinear).
#[allow(dead_code)]
pub fn sample_atlas(atlas: &TextureAtlas, u: f32, v: f32) -> [u8; 4] {
    if atlas.width == 0 || atlas.height == 0 || atlas.pixels.is_empty() {
        return [0, 0, 0, 0];
    }
    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let px = u * (atlas.width as f32 - 1.0);
    let py = v * (atlas.height as f32 - 1.0);
    let x0 = px as usize;
    let y0 = py as usize;
    let x1 = (x0 + 1).min(atlas.width as usize - 1);
    let y1 = (y0 + 1).min(atlas.height as usize - 1);
    let fx = px - x0 as f32;
    let fy = py - y0 as f32;

    let aw = atlas.width as usize;
    let c00 = atlas.pixels[y0 * aw + x0];
    let c10 = atlas.pixels[y0 * aw + x1];
    let c01 = atlas.pixels[y1 * aw + x0];
    let c11 = atlas.pixels[y1 * aw + x1];

    let mut out = [0u8; 4];
    for i in 0..4 {
        let v00 = c00[i] as f32;
        let v10 = c10[i] as f32;
        let v01 = c01[i] as f32;
        let v11 = c11[i] as f32;
        let val = v00 * (1.0 - fx) * (1.0 - fy)
            + v10 * fx * (1.0 - fy)
            + v01 * (1.0 - fx) * fy
            + v11 * fx * fy;
        out[i] = val.round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Split a large atlas into multiple smaller atlases, each containing at most
/// `max_regions_per_atlas` regions.
#[allow(dead_code)]
pub fn split_atlas(atlas: &TextureAtlas, max_regions_per_atlas: usize) -> Vec<TextureAtlas> {
    if max_regions_per_atlas == 0 || atlas.regions.is_empty() {
        return vec![atlas.clone()];
    }
    let chunks: Vec<&[AtlasRegion]> = atlas.regions.chunks(max_regions_per_atlas).collect();
    chunks
        .into_iter()
        .map(|chunk| {
            let mut sub = new_texture_atlas(atlas.width, atlas.height, atlas.padding);
            for region in chunk {
                // Blit region pixels from the parent atlas.
                let aw = atlas.width as usize;
                let sw = region.width as usize;
                let sh = region.height as usize;
                let mut src_pixels = Vec::with_capacity(sw * sh);
                for row in 0..sh {
                    let src_y = region.y as usize + row;
                    for col in 0..sw {
                        let src_x = region.x as usize + col;
                        src_pixels.push(atlas.pixels[src_y * aw + src_x]);
                    }
                }
                let input = AtlasInput {
                    id: region.id,
                    width: region.width,
                    height: region.height,
                    pixels: src_pixels,
                };
                add_region(&mut sub, &input);
            }
            sub
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn red_input(id: u32, w: u32, h: u32) -> AtlasInput {
        AtlasInput {
            id,
            width: w,
            height: h,
            pixels: vec![[255, 0, 0, 255]; (w * h) as usize],
        }
    }

    #[test]
    fn new_texture_atlas_dimensions() {
        let atlas = new_texture_atlas(256, 128, 2);
        assert_eq!(atlas.width, 256);
        assert_eq!(atlas.height, 128);
        assert_eq!(atlas.pixels.len(), 256 * 128);
        assert_eq!(atlas.padding, 2);
        assert!(atlas.regions.is_empty());
    }

    #[test]
    fn pack_textures_places_single_texture() {
        let inputs = vec![red_input(1, 32, 32)];
        let atlas = pack_textures(inputs, 128, 0);
        assert_eq!(atlas.regions.len(), 1);
        assert_eq!(atlas.regions[0].id, 1);
    }

    #[test]
    fn pack_textures_places_multiple_textures() {
        let inputs = vec![red_input(1, 32, 32), red_input(2, 32, 32)];
        let atlas = pack_textures(inputs, 128, 0);
        assert_eq!(atlas.regions.len(), 2);
    }

    #[test]
    fn add_region_returns_some_when_space_available() {
        let mut atlas = new_texture_atlas(64, 64, 0);
        let inp = red_input(42, 16, 16);
        let result = add_region(&mut atlas, &inp);
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 42);
    }

    #[test]
    fn add_region_returns_none_when_no_space() {
        let mut atlas = new_texture_atlas(4, 4, 0);
        let inp = red_input(1, 8, 8);
        let result = add_region(&mut atlas, &inp);
        assert!(result.is_none());
    }

    #[test]
    fn blit_to_atlas_sets_pixels() {
        let mut atlas = new_texture_atlas(8, 8, 0);
        let inp = red_input(1, 2, 2);
        blit_to_atlas(&mut atlas, &inp, 0, 0);
        // Top-left 2x2 pixels should be red.
        assert_eq!(atlas.pixels[0], [255, 0, 0, 255]);
        assert_eq!(atlas.pixels[1], [255, 0, 0, 255]);
    }

    #[test]
    fn find_free_space_returns_origin_for_empty_atlas() {
        let atlas = new_texture_atlas(64, 64, 0);
        let pos = find_free_space(&atlas, 16, 16);
        assert_eq!(pos, Some((0, 0)));
    }

    #[test]
    fn find_free_space_returns_none_when_full() {
        let atlas = new_texture_atlas(8, 8, 0);
        let pos = find_free_space(&atlas, 16, 16);
        assert!(pos.is_none());
    }

    #[test]
    fn atlas_utilization_empty() {
        let atlas = new_texture_atlas(64, 64, 0);
        assert!((atlas_utilization(&atlas) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn atlas_utilization_full_region() {
        let inputs = vec![red_input(1, 64, 64)];
        let atlas = pack_textures(inputs, 64, 0);
        let u = atlas_utilization(&atlas);
        assert!(
            (u - 1.0).abs() < 1e-6,
            "utilization should be 1.0, got {}",
            u
        );
    }

    #[test]
    fn atlas_to_png_stub_starts_with_magic() {
        let atlas = new_texture_atlas(4, 4, 0);
        let bytes = atlas_to_png_stub(&atlas);
        assert!(bytes.starts_with(b"\x89OXA"));
    }

    #[test]
    fn atlas_region_for_id_found() {
        let mut atlas = new_texture_atlas(64, 64, 0);
        let inp = red_input(99, 8, 8);
        add_region(&mut atlas, &inp);
        let r = atlas_region_for_id(&atlas, 99);
        assert!(r.is_some());
    }

    #[test]
    fn atlas_region_for_id_not_found() {
        let atlas = new_texture_atlas(64, 64, 0);
        assert!(atlas_region_for_id(&atlas, 99).is_none());
    }

    #[test]
    fn sample_atlas_returns_pixel() {
        let mut atlas = new_texture_atlas(4, 4, 0);
        // Fill all pixels with green.
        for px in &mut atlas.pixels {
            *px = [0, 255, 0, 255];
        }
        let c = sample_atlas(&atlas, 0.5, 0.5);
        assert_eq!(c[1], 255); // green channel
    }

    #[test]
    fn split_atlas_single_region() {
        let inputs = vec![red_input(1, 16, 16), red_input(2, 16, 16)];
        let atlas = pack_textures(inputs, 128, 0);
        let parts = split_atlas(&atlas, 1);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].regions.len(), 1);
        assert_eq!(parts[1].regions.len(), 1);
    }
}
