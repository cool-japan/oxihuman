// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Glyph renderer for font glyph atlas rendering.

/// A single glyph metric.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlyphMetric {
    pub codepoint: u32,
    pub advance_x: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub width: f32,
    pub height: f32,
}

/// Glyph atlas configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlyphAtlasConfig {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub font_size: f32,
    pub padding: u32,
}

#[allow(dead_code)]
pub fn default_glyph_atlas_config() -> GlyphAtlasConfig {
    GlyphAtlasConfig { atlas_width: 512, atlas_height: 512, font_size: 16.0, padding: 2 }
}

#[allow(dead_code)]
pub fn new_glyph_metric(codepoint: u32, advance_x: f32, w: f32, h: f32) -> GlyphMetric {
    GlyphMetric { codepoint, advance_x, bearing_x: 0.0, bearing_y: h, width: w, height: h }
}

#[allow(dead_code)]
pub fn set_font_size(cfg: &mut GlyphAtlasConfig, size: f32) {
    cfg.font_size = size.clamp(4.0, 256.0);
}

#[allow(dead_code)]
pub fn set_atlas_size(cfg: &mut GlyphAtlasConfig, width: u32, height: u32) {
    cfg.atlas_width = width.clamp(64, 4096);
    cfg.atlas_height = height.clamp(64, 4096);
}

#[allow(dead_code)]
pub fn measure_text_width(glyphs: &[GlyphMetric], text: &str) -> f32 {
    text.chars()
        .filter_map(|c| glyphs.iter().find(|g| g.codepoint == c as u32))
        .map(|g| g.advance_x)
        .sum()
}

#[allow(dead_code)]
pub fn atlas_capacity(cfg: &GlyphAtlasConfig, glyph_size: u32) -> u32 {
    if glyph_size == 0 { return 0; }
    let cell = glyph_size + cfg.padding;
    let cols = cfg.atlas_width / cell;
    let rows = cfg.atlas_height / cell;
    cols * rows
}

#[allow(dead_code)]
pub fn glyph_area(metric: &GlyphMetric) -> f32 {
    metric.width * metric.height
}

#[allow(dead_code)]
pub fn scale_glyph(metric: &mut GlyphMetric, scale: f32) {
    metric.width *= scale;
    metric.height *= scale;
    metric.advance_x *= scale;
    metric.bearing_x *= scale;
    metric.bearing_y *= scale;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_atlas() {
        let cfg = default_glyph_atlas_config();
        assert_eq!(cfg.atlas_width, 512);
    }

    #[test]
    fn test_new_glyph_metric() {
        let g = new_glyph_metric(65, 10.0, 8.0, 12.0);
        assert_eq!(g.codepoint, 65);
    }

    #[test]
    fn test_set_font_size_clamp() {
        let mut cfg = default_glyph_atlas_config();
        set_font_size(&mut cfg, 1.0);
        assert!((cfg.font_size - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_atlas_size() {
        let mut cfg = default_glyph_atlas_config();
        set_atlas_size(&mut cfg, 1024, 1024);
        assert_eq!(cfg.atlas_width, 1024);
    }

    #[test]
    fn test_measure_text_width() {
        let glyphs = vec![
            new_glyph_metric(72, 8.0, 7.0, 10.0), // H
            new_glyph_metric(105, 4.0, 3.0, 10.0), // i
        ];
        let w = measure_text_width(&glyphs, "Hi");
        assert!((w - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_atlas_capacity() {
        let cfg = default_glyph_atlas_config();
        let cap = atlas_capacity(&cfg, 16);
        assert!(cap > 0);
    }

    #[test]
    fn test_atlas_capacity_zero_glyph() {
        let cfg = default_glyph_atlas_config();
        assert_eq!(atlas_capacity(&cfg, 0), 0);
    }

    #[test]
    fn test_glyph_area() {
        let g = new_glyph_metric(65, 10.0, 8.0, 12.0);
        assert!((glyph_area(&g) - 96.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_glyph() {
        let mut g = new_glyph_metric(65, 10.0, 8.0, 12.0);
        scale_glyph(&mut g, 2.0);
        assert!((g.width - 16.0).abs() < 1e-6);
        assert!((g.advance_x - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_measure_empty_text() {
        let glyphs = vec![new_glyph_metric(65, 10.0, 8.0, 12.0)];
        let w = measure_text_width(&glyphs, "");
        assert!(w.abs() < 1e-6);
    }
}
