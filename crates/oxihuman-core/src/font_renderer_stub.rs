// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Font rendering stub providing glyph metrics, text layout, and bounding box utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub size_px: u32,
    pub bold: bool,
    pub italic: bool,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextLayout {
    pub glyphs: Vec<GlyphMetrics>,
    pub total_width: f32,
    pub line_height: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FontRendererStub {
    pub config: FontConfig,
    pub glyph_cache_size: usize,
}

#[allow(dead_code)]
pub fn default_font_config() -> FontConfig {
    FontConfig {
        size_px: 16,
        bold: false,
        italic: false,
        name: String::from("sans-serif"),
    }
}

#[allow(dead_code)]
pub fn new_font_renderer_stub(cfg: FontConfig) -> FontRendererStub {
    FontRendererStub {
        config: cfg,
        glyph_cache_size: 256,
    }
}

#[allow(dead_code)]
pub fn measure_glyph(renderer: &FontRendererStub, ch: char) -> GlyphMetrics {
    let size = renderer.config.size_px as f32;
    let bold_scale = if renderer.config.bold { 1.1 } else { 1.0 };
    // Stub: approximate advance from character code for determinism.
    let char_scale = if ch.is_ascii_alphabetic() { 0.6 } else { 0.5 };
    let advance = size * char_scale * bold_scale;
    GlyphMetrics {
        advance,
        bearing_x: 0.0,
        bearing_y: size * 0.8,
        width: (advance as u32).max(1),
        height: size as u32,
    }
}

#[allow(dead_code)]
pub fn layout_text(renderer: &FontRendererStub, text: &str) -> TextLayout {
    let mut glyphs = Vec::with_capacity(text.len());
    let mut total_width = 0.0f32;
    for ch in text.chars() {
        let g = measure_glyph(renderer, ch);
        total_width += g.advance;
        glyphs.push(g);
    }
    let line_height = renderer.config.size_px as f32 * 1.2;
    TextLayout {
        glyphs,
        total_width,
        line_height,
    }
}

#[allow(dead_code)]
pub fn glyph_count(layout: &TextLayout) -> usize {
    layout.glyphs.len()
}

#[allow(dead_code)]
pub fn font_config_to_json(cfg: &FontConfig) -> String {
    format!(
        "{{\"size_px\":{},\"bold\":{},\"italic\":{},\"name\":\"{}\"}}",
        cfg.size_px, cfg.bold, cfg.italic, cfg.name
    )
}

/// Returns `[x, y, w, h]` bounding box for the laid-out text.
#[allow(dead_code)]
pub fn text_bounding_box(layout: &TextLayout) -> [f32; 4] {
    [0.0, 0.0, layout.total_width, layout.line_height]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_font_config() {
        let cfg = default_font_config();
        assert_eq!(cfg.size_px, 16);
        assert!(!cfg.bold);
        assert!(!cfg.italic);
        assert!(!cfg.name.is_empty());
    }

    #[test]
    fn test_new_font_renderer_stub() {
        let cfg = default_font_config();
        let renderer = new_font_renderer_stub(cfg);
        assert_eq!(renderer.glyph_cache_size, 256);
        assert_eq!(renderer.config.size_px, 16);
    }

    #[test]
    fn test_measure_glyph_nonzero() {
        let renderer = new_font_renderer_stub(default_font_config());
        let g = measure_glyph(&renderer, 'A');
        assert!(g.advance > 0.0);
        assert!(g.height > 0);
    }

    #[test]
    fn test_layout_text_empty() {
        let renderer = new_font_renderer_stub(default_font_config());
        let layout = layout_text(&renderer, "");
        assert_eq!(glyph_count(&layout), 0);
        assert!((layout.total_width - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_layout_text_nonempty() {
        let renderer = new_font_renderer_stub(default_font_config());
        let layout = layout_text(&renderer, "Hi");
        assert_eq!(glyph_count(&layout), 2);
        assert!(layout.total_width > 0.0);
    }

    #[test]
    fn test_glyph_count() {
        let renderer = new_font_renderer_stub(default_font_config());
        let layout = layout_text(&renderer, "abc");
        assert_eq!(glyph_count(&layout), 3);
    }

    #[test]
    fn test_font_config_to_json() {
        let cfg = default_font_config();
        let json = font_config_to_json(&cfg);
        assert!(json.contains("size_px"));
        assert!(json.contains("sans-serif"));
    }

    #[test]
    fn test_text_bounding_box() {
        let renderer = new_font_renderer_stub(default_font_config());
        let layout = layout_text(&renderer, "Test");
        let bb = text_bounding_box(&layout);
        assert!((bb[0] - 0.0).abs() < 1e-6);
        assert!((bb[1] - 0.0).abs() < 1e-6);
        assert!(bb[2] > 0.0);
        assert!(bb[3] > 0.0);
    }

    #[test]
    fn test_bold_advance_larger() {
        let cfg_normal = default_font_config();
        let mut cfg_bold = default_font_config();
        cfg_bold.bold = true;
        let r_normal = new_font_renderer_stub(cfg_normal);
        let r_bold = new_font_renderer_stub(cfg_bold);
        let g_normal = measure_glyph(&r_normal, 'A');
        let g_bold = measure_glyph(&r_bold, 'A');
        assert!(g_bold.advance > g_normal.advance);
    }
}
