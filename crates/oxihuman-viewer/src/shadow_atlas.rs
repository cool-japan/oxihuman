// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shadow atlas texture packing.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowAtlasRegion {
    pub x: u32,
    pub y: u32,
    pub size: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowAtlas {
    pub width: u32,
    pub height: u32,
    pub regions: Vec<ShadowAtlasRegion>,
}

#[allow(dead_code)]
pub fn new_shadow_atlas(width: u32, height: u32) -> ShadowAtlas {
    ShadowAtlas { width, height, regions: Vec::new() }
}

#[allow(dead_code)]
pub fn sa_allocate(atlas: &mut ShadowAtlas, size: u32) -> Option<ShadowAtlasRegion> {
    /* simple row packing: fill current row, then start new row */
    let mut x_cursor = 0u32;
    let mut y_cursor = 0u32;
    let mut row_h = 0u32;
    for r in &atlas.regions {
        if x_cursor + r.size > atlas.width {
            y_cursor += row_h;
            x_cursor = 0;
            row_h = 0;
        }
        x_cursor += r.size;
        if r.size > row_h {
            row_h = r.size;
        }
    }
    if x_cursor + size > atlas.width {
        y_cursor += row_h;
        x_cursor = 0;
    }
    if y_cursor + size > atlas.height {
        return None;
    }
    let region = ShadowAtlasRegion { x: x_cursor, y: y_cursor, size };
    atlas.regions.push(region.clone());
    Some(region)
}

#[allow(dead_code)]
pub fn sa_region_count(atlas: &ShadowAtlas) -> usize {
    atlas.regions.len()
}

#[allow(dead_code)]
pub fn sa_used_area(atlas: &ShadowAtlas) -> u32 {
    atlas.regions.iter().map(|r| r.size * r.size).sum()
}

#[allow(dead_code)]
pub fn sa_utilization(atlas: &ShadowAtlas) -> f32 {
    let total = atlas.width * atlas.height;
    if total == 0 { return 0.0; }
    sa_used_area(atlas) as f32 / total as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_returns_some() {
        let mut a = new_shadow_atlas(512, 512);
        let r = sa_allocate(&mut a, 128);
        assert!(r.is_some());
    }

    #[test]
    fn test_region_count_after_allocate() {
        let mut a = new_shadow_atlas(512, 512);
        sa_allocate(&mut a, 128);
        assert_eq!(sa_region_count(&a), 1);
    }

    #[test]
    fn test_used_area_single_region() {
        let mut a = new_shadow_atlas(512, 512);
        sa_allocate(&mut a, 128);
        assert_eq!(sa_used_area(&a), 128 * 128);
    }

    #[test]
    fn test_utilization_in_range() {
        let mut a = new_shadow_atlas(512, 512);
        sa_allocate(&mut a, 128);
        let u = sa_utilization(&a);
        assert!((0.0..=1.0).contains(&u));
    }

    #[test]
    fn test_utilization_zero_empty() {
        let a = new_shadow_atlas(512, 512);
        assert_eq!(sa_utilization(&a), 0.0);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut a = new_shadow_atlas(512, 512);
        sa_allocate(&mut a, 128);
        sa_allocate(&mut a, 64);
        assert_eq!(sa_region_count(&a), 2);
    }

    #[test]
    fn test_allocate_too_large_returns_none() {
        let mut a = new_shadow_atlas(64, 64);
        let r = sa_allocate(&mut a, 128);
        assert!(r.is_none());
    }

    #[test]
    fn test_region_starts_at_origin() {
        let mut a = new_shadow_atlas(512, 512);
        let r = sa_allocate(&mut a, 128).unwrap();
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
    }
}
