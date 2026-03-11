// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Texture atlas packing utilities.

#[allow(dead_code)]
pub struct TextureRect {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub struct PackInput {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA
}

#[allow(dead_code)]
pub struct PackResult {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub placements: Vec<TextureRect>,
    pub atlas_pixels: Vec<u8>,
}

#[allow(dead_code)]
pub struct PackConfig {
    pub padding: u32,
    pub power_of_two: bool,
    pub max_size: u32,
}

#[allow(dead_code)]
pub fn default_pack_config() -> PackConfig {
    PackConfig {
        padding: 1,
        power_of_two: true,
        max_size: 4096,
    }
}

#[allow(dead_code)]
pub fn next_power_of_two(n: u32) -> u32 {
    if n == 0 {
        return 1;
    }
    let mut v = n;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

/// Shelf-first packing algorithm.
#[allow(dead_code)]
pub fn pack_textures(inputs: &[PackInput], cfg: &PackConfig) -> PackResult {
    if inputs.is_empty() {
        return PackResult {
            atlas_width: 0,
            atlas_height: 0,
            placements: Vec::new(),
            atlas_pixels: Vec::new(),
        };
    }

    // Sort by height descending for better packing
    let mut order: Vec<usize> = (0..inputs.len()).collect();
    order.sort_by(|&a, &b| inputs[b].height.cmp(&inputs[a].height));

    let max_w = cfg.max_size;
    let pad = cfg.padding;

    let mut placements: Vec<TextureRect> = Vec::new();
    let mut shelf_x = 0u32;
    let mut shelf_y = 0u32;
    let mut shelf_h = 0u32;
    let mut atlas_w = 0u32;
    let mut atlas_h = 0u32;

    for &idx in &order {
        let inp = &inputs[idx];
        let needed_w = inp.width + pad;
        let needed_h = inp.height + pad;

        if shelf_x + needed_w > max_w {
            // New shelf
            shelf_y += shelf_h;
            shelf_x = 0;
            shelf_h = 0;
        }

        let x = shelf_x;
        let y = shelf_y;

        shelf_x += needed_w;
        if needed_h > shelf_h {
            shelf_h = needed_h;
        }

        let right = x + inp.width;
        let bottom = y + inp.height;
        if right > atlas_w {
            atlas_w = right;
        }
        if bottom + pad > atlas_h {
            atlas_h = bottom + pad;
        }

        placements.push(TextureRect {
            id: inp.id,
            x,
            y,
            width: inp.width,
            height: inp.height,
        });
    }

    // Finalise atlas dimensions
    atlas_h += shelf_h.saturating_sub(pad);

    if cfg.power_of_two {
        atlas_w = next_power_of_two(atlas_w).min(cfg.max_size);
        atlas_h = next_power_of_two(atlas_h).min(cfg.max_size);
    } else {
        atlas_w = atlas_w.min(cfg.max_size);
        atlas_h = atlas_h.min(cfg.max_size);
    }

    let mut atlas_pixels = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    for (placement, &orig_idx) in placements.iter().zip(order.iter()) {
        let src = &inputs[orig_idx];
        blit_texture(
            &mut atlas_pixels,
            atlas_w,
            &src.pixels,
            src.width,
            src.height,
            placement.x,
            placement.y,
        );
    }

    PackResult {
        atlas_width: atlas_w,
        atlas_height: atlas_h,
        placements,
        atlas_pixels,
    }
}

/// Blit RGBA pixels from src into atlas at (dst_x, dst_y).
#[allow(dead_code)]
pub fn blit_texture(
    atlas: &mut [u8],
    atlas_w: u32,
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dst_x: u32,
    dst_y: u32,
) {
    for row in 0..src_h {
        for col in 0..src_w {
            let src_idx = ((row * src_w + col) * 4) as usize;
            let dst_idx = (((dst_y + row) * atlas_w + (dst_x + col)) * 4) as usize;
            if src_idx + 3 < src.len() && dst_idx + 3 < atlas.len() {
                atlas[dst_idx] = src[src_idx];
                atlas[dst_idx + 1] = src[src_idx + 1];
                atlas[dst_idx + 2] = src[src_idx + 2];
                atlas[dst_idx + 3] = src[src_idx + 3];
            }
        }
    }
}

#[allow(dead_code)]
pub fn pack_single(input: &PackInput, cfg: &PackConfig) -> PackResult {
    pack_textures(std::slice::from_ref(input), cfg)
}

/// Returns (offset `[u,v]`, scale `[u,v]`) for remapping UVs from a sub-texture to atlas space.
#[allow(dead_code)]
pub fn uv_transform_for_rect(
    rect: &TextureRect,
    atlas_w: u32,
    atlas_h: u32,
) -> ([f32; 2], [f32; 2]) {
    let offset = [
        rect.x as f32 / atlas_w as f32,
        rect.y as f32 / atlas_h as f32,
    ];
    let scale = [
        rect.width as f32 / atlas_w as f32,
        rect.height as f32 / atlas_h as f32,
    ];
    (offset, scale)
}

#[allow(dead_code)]
pub fn find_placement(result: &PackResult, id: u32) -> Option<&TextureRect> {
    result.placements.iter().find(|r| r.id == id)
}

#[allow(dead_code)]
pub fn generate_solid_color_texture(width: u32, height: u32, color: [u8; 4]) -> PackInput {
    let pixel_count = (width * height * 4) as usize;
    let mut pixels = Vec::with_capacity(pixel_count);
    for _ in 0..(width * height) {
        pixels.extend_from_slice(&color);
    }
    PackInput {
        id: 0,
        width,
        height,
        pixels,
    }
}

#[allow(dead_code)]
pub fn pack_config_max_size(cfg: &PackConfig) -> u32 {
    cfg.max_size
}

#[allow(dead_code)]
pub fn atlas_pixel_count(result: &PackResult) -> usize {
    result.atlas_pixels.len()
}

#[allow(dead_code)]
pub fn atlas_utilization(result: &PackResult) -> f32 {
    let total = result.atlas_width * result.atlas_height;
    if total == 0 {
        return 0.0;
    }
    let packed: u32 = result.placements.iter().map(|r| r.width * r.height).sum();
    packed as f32 / total as f32
}

#[allow(dead_code)]
pub fn rects_overlap(a: &TextureRect, b: &TextureRect) -> bool {
    !(a.x + a.width <= b.x
        || b.x + b.width <= a.x
        || a.y + a.height <= b.y
        || b.y + b.height <= a.y)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_power_of_two_zero() {
        assert_eq!(next_power_of_two(0), 1);
    }

    #[test]
    fn test_next_power_of_two_exact() {
        assert_eq!(next_power_of_two(8), 8);
    }

    #[test]
    fn test_next_power_of_two_round_up() {
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(100), 128);
        assert_eq!(next_power_of_two(257), 512);
    }

    #[test]
    fn test_pack_single_texture() {
        let cfg = default_pack_config();
        let inp = generate_solid_color_texture(64, 64, [255, 0, 0, 255]);
        let result = pack_single(&inp, &cfg);
        assert_eq!(result.placements.len(), 1);
        assert!(result.atlas_width >= 64);
        assert!(result.atlas_height >= 64);
    }

    #[test]
    fn test_pack_multiple_textures() {
        let cfg = default_pack_config();
        let inputs: Vec<PackInput> = (0..4u32)
            .map(|id| {
                let mut t = generate_solid_color_texture(32, 32, [id as u8 * 60, 0, 0, 255]);
                t.id = id;
                t
            })
            .collect();
        let result = pack_textures(&inputs, &cfg);
        assert_eq!(result.placements.len(), 4);
    }

    #[test]
    fn test_no_overlaps_in_result() {
        let cfg = default_pack_config();
        let inputs: Vec<PackInput> = (0..6u32)
            .map(|id| {
                let mut t = generate_solid_color_texture(20, 20, [255, 255, 255, 255]);
                t.id = id;
                t
            })
            .collect();
        let result = pack_textures(&inputs, &cfg);
        let n = result.placements.len();
        for i in 0..n {
            for j in (i + 1)..n {
                assert!(
                    !rects_overlap(&result.placements[i], &result.placements[j]),
                    "Rects {} and {} overlap",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_blit_texture_correct_size() {
        let src = vec![255u8, 0, 0, 255, 0, 255, 0, 255]; // 2 RGBA pixels
        let mut atlas = vec![0u8; 16 * 16 * 4];
        blit_texture(&mut atlas, 16, &src, 2, 1, 0, 0);
        // First pixel should be red
        assert_eq!(atlas[0], 255);
        assert_eq!(atlas[1], 0);
        assert_eq!(atlas[2], 0);
        assert_eq!(atlas[3], 255);
    }

    #[test]
    fn test_utilization_in_valid_range() {
        let cfg = default_pack_config();
        let inputs: Vec<PackInput> = (0..3u32)
            .map(|id| {
                let mut t = generate_solid_color_texture(16, 16, [100, 100, 100, 255]);
                t.id = id;
                t
            })
            .collect();
        let result = pack_textures(&inputs, &cfg);
        let u = atlas_utilization(&result);
        assert!(u > 0.0);
        assert!(u <= 1.0);
    }

    #[test]
    fn test_solid_color_texture_pixel_count() {
        let t = generate_solid_color_texture(4, 4, [0, 0, 255, 255]);
        assert_eq!(t.pixels.len(), 4 * 4 * 4);
    }

    #[test]
    fn test_solid_color_texture_values() {
        let t = generate_solid_color_texture(2, 2, [10, 20, 30, 255]);
        assert_eq!(t.pixels[0], 10);
        assert_eq!(t.pixels[1], 20);
        assert_eq!(t.pixels[2], 30);
        assert_eq!(t.pixels[3], 255);
    }

    #[test]
    fn test_uv_transform() {
        let rect = TextureRect {
            id: 0,
            x: 0,
            y: 0,
            width: 64,
            height: 64,
        };
        let (offset, scale) = uv_transform_for_rect(&rect, 128, 128);
        assert_eq!(offset, [0.0, 0.0]);
        assert!((scale[0] - 0.5).abs() < 1e-5);
        assert!((scale[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_uv_transform_offset() {
        let rect = TextureRect {
            id: 1,
            x: 64,
            y: 0,
            width: 64,
            height: 64,
        };
        let (offset, _scale) = uv_transform_for_rect(&rect, 128, 128);
        assert!((offset[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_find_placement() {
        let cfg = default_pack_config();
        let mut inp = generate_solid_color_texture(10, 10, [0, 0, 0, 255]);
        inp.id = 42;
        let result = pack_single(&inp, &cfg);
        let found = find_placement(&result, 42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 42);
    }

    #[test]
    fn test_find_placement_missing() {
        let cfg = default_pack_config();
        let inp = generate_solid_color_texture(10, 10, [0, 0, 0, 255]);
        let result = pack_single(&inp, &cfg);
        assert!(find_placement(&result, 99).is_none());
    }

    #[test]
    fn test_atlas_pixel_count() {
        let cfg = default_pack_config();
        let inp = generate_solid_color_texture(8, 8, [0, 0, 0, 255]);
        let result = pack_single(&inp, &cfg);
        // Atlas pixels = atlas_w * atlas_h * 4
        assert_eq!(
            atlas_pixel_count(&result),
            (result.atlas_width * result.atlas_height * 4) as usize
        );
    }

    #[test]
    fn test_rects_no_overlap() {
        let a = TextureRect {
            id: 0,
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        let b = TextureRect {
            id: 1,
            x: 10,
            y: 0,
            width: 10,
            height: 10,
        };
        assert!(!rects_overlap(&a, &b));
    }

    #[test]
    fn test_rects_do_overlap() {
        let a = TextureRect {
            id: 0,
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        let b = TextureRect {
            id: 1,
            x: 5,
            y: 5,
            width: 10,
            height: 10,
        };
        assert!(rects_overlap(&a, &b));
    }
}
