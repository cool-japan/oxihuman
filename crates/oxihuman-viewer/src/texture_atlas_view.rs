// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Texture atlas debug view — visualizes atlas packing, utilization, and sub-regions.

/// Texture atlas view configuration.
#[derive(Debug, Clone)]
pub struct TextureAtlasView {
    pub enabled: bool,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub region_count: u32,
    pub show_borders: bool,
}

impl TextureAtlasView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            atlas_width: 2048,
            atlas_height: 2048,
            region_count: 0,
            show_borders: true,
        }
    }
}

impl Default for TextureAtlasView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new texture atlas view.
pub fn new_texture_atlas_view() -> TextureAtlasView {
    TextureAtlasView::new()
}

/// Enable or disable texture atlas debug view.
pub fn tav_set_enabled(v: &mut TextureAtlasView, enabled: bool) {
    v.enabled = enabled;
}

/// Set atlas dimensions.
pub fn tav_set_dimensions(v: &mut TextureAtlasView, width: u32, height: u32) {
    v.atlas_width = width.max(1);
    v.atlas_height = height.max(1);
}

/// Set the number of packed sub-regions.
pub fn tav_set_region_count(v: &mut TextureAtlasView, count: u32) {
    v.region_count = count;
}

/// Toggle sub-region border rendering.
pub fn tav_set_show_borders(v: &mut TextureAtlasView, show: bool) {
    v.show_borders = show;
}

/// Compute total atlas texel count.
pub fn tav_total_texels(v: &TextureAtlasView) -> u64 {
    v.atlas_width as u64 * v.atlas_height as u64
}

/// Serialize to JSON-like string.
pub fn texture_atlas_view_to_json(v: &TextureAtlasView) -> String {
    format!(
        r#"{{"enabled":{},"atlas_width":{},"atlas_height":{},"region_count":{},"show_borders":{}}}"#,
        v.enabled, v.atlas_width, v.atlas_height, v.region_count, v.show_borders
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_texture_atlas_view();
        assert!(!v.enabled);
        assert_eq!(v.atlas_width, 2048);
    }

    #[test]
    fn test_enable() {
        let mut v = new_texture_atlas_view();
        tav_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_dimensions() {
        let mut v = new_texture_atlas_view();
        tav_set_dimensions(&mut v, 4096, 4096);
        assert_eq!(v.atlas_width, 4096);
        assert_eq!(v.atlas_height, 4096);
    }

    #[test]
    fn test_dimensions_min_one() {
        let mut v = new_texture_atlas_view();
        tav_set_dimensions(&mut v, 0, 0);
        assert_eq!(v.atlas_width, 1);
        assert_eq!(v.atlas_height, 1);
    }

    #[test]
    fn test_region_count_set() {
        let mut v = new_texture_atlas_view();
        tav_set_region_count(&mut v, 32);
        assert_eq!(v.region_count, 32);
    }

    #[test]
    fn test_show_borders_off() {
        let mut v = new_texture_atlas_view();
        tav_set_show_borders(&mut v, false);
        assert!(!v.show_borders);
    }

    #[test]
    fn test_total_texels_default() {
        let v = new_texture_atlas_view();
        assert_eq!(tav_total_texels(&v), 2048 * 2048);
    }

    #[test]
    fn test_total_texels_custom() {
        let mut v = new_texture_atlas_view();
        tav_set_dimensions(&mut v, 512, 1024);
        assert_eq!(tav_total_texels(&v), 512 * 1024);
    }

    #[test]
    fn test_json_keys() {
        let v = new_texture_atlas_view();
        let s = texture_atlas_view_to_json(&v);
        assert!(s.contains("region_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_texture_atlas_view();
        let v2 = v.clone();
        assert_eq!(v2.atlas_width, v.atlas_width);
    }
}
