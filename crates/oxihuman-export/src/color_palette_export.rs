// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export a named color palette as JSON or CSS custom-properties.

#![allow(dead_code)]

/// Configuration for color-palette export.
#[derive(Debug, Clone)]
pub struct ColorPaletteExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// CSS variable prefix (e.g. `"--color"`).
    pub css_prefix: String,
}

/// A single named color entry.
#[derive(Debug, Clone)]
pub struct PaletteColor {
    /// Unique name for the color.
    pub name: String,
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
    /// Alpha channel (0–255).
    pub a: u8,
}

/// Container holding the color palette for export.
#[derive(Debug, Clone)]
pub struct ColorPaletteExport {
    /// All palette colors.
    pub colors: Vec<PaletteColor>,
    /// Byte count of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`ColorPaletteExportConfig`].
pub fn default_color_palette_export_config() -> ColorPaletteExportConfig {
    ColorPaletteExportConfig {
        pretty: true,
        css_prefix: "--color".to_string(),
    }
}

/// Creates a new, empty [`ColorPaletteExport`].
pub fn new_color_palette_export() -> ColorPaletteExport {
    ColorPaletteExport {
        colors: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds (or replaces by name) a color entry.
pub fn cpe_add_color(export: &mut ColorPaletteExport, color: PaletteColor) {
    if let Some(existing) = export.colors.iter_mut().find(|c| c.name == color.name) {
        *existing = color;
    } else {
        export.colors.push(color);
    }
}

/// Serialises the palette as JSON.
pub fn cpe_to_json(export: &mut ColorPaletteExport, cfg: &ColorPaletteExportConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };

    let mut out = format!("{{{nl}{indent}\"colors\":[{nl}");
    let len = export.colors.len();
    for (i, c) in export.colors.iter().enumerate() {
        let comma = if i + 1 < len { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"name\":\"{}\",\"r\":{},\"g\":{},\"b\":{},\"a\":{},\
             \"hex\":\"#{:02X}{:02X}{:02X}{:02X}\"}}{comma}{nl}",
            c.name, c.r, c.g, c.b, c.a, c.r, c.g, c.b, c.a
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));
    export.total_bytes = out.len();
    out
}

/// Serialises the palette as CSS custom properties block.
pub fn cpe_to_css(export: &mut ColorPaletteExport, cfg: &ColorPaletteExportConfig) -> String {
    let mut out = String::from(":root {\n");
    for c in &export.colors {
        let slug = c.name.to_lowercase().replace(' ', "-");
        out.push_str(&format!(
            "  {}-{}: rgba({},{},{},{:.4});\n",
            cfg.css_prefix,
            slug,
            c.r,
            c.g,
            c.b,
            c.a as f32 / 255.0
        ));
    }
    out.push_str("}\n");
    export.total_bytes = out.len();
    out
}

/// Returns the number of colors stored.
pub fn cpe_color_count(export: &ColorPaletteExport) -> usize {
    export.colors.len()
}

/// Finds a color by name, returning a reference or `None`.
pub fn cpe_find_by_name<'a>(
    export: &'a ColorPaletteExport,
    name: &str,
) -> Option<&'a PaletteColor> {
    export.colors.iter().find(|c| c.name == name)
}

/// Writes JSON to a file path (stub — returns byte count).
pub fn cpe_write_to_file(
    export: &mut ColorPaletteExport,
    cfg: &ColorPaletteExportConfig,
    _path: &str,
) -> usize {
    let json = cpe_to_json(export, cfg);
    export.total_bytes = json.len();
    export.total_bytes
}

/// Clears all colors and resets state.
pub fn cpe_clear(export: &mut ColorPaletteExport) {
    export.colors.clear();
    export.total_bytes = 0;
}

/// Returns the byte count of the last serialised output.
pub fn cpe_total_bytes(export: &ColorPaletteExport) -> usize {
    export.total_bytes
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_color(name: &str, r: u8, g: u8, b: u8, a: u8) -> PaletteColor {
    PaletteColor { name: name.to_string(), r, g, b, a }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_color_palette_export_config();
        assert!(cfg.pretty);
        assert_eq!(cfg.css_prefix, "--color");
    }

    #[test]
    fn new_export_is_empty() {
        let e = new_color_palette_export();
        assert_eq!(cpe_color_count(&e), 0);
        assert_eq!(cpe_total_bytes(&e), 0);
    }

    #[test]
    fn add_color_increments_count() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("primary", 255, 0, 0, 255));
        assert_eq!(cpe_color_count(&e), 1);
    }

    #[test]
    fn duplicate_name_overwrites() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("accent", 0, 255, 0, 255));
        cpe_add_color(&mut e, make_color("accent", 0, 0, 255, 255));
        assert_eq!(cpe_color_count(&e), 1);
        assert_eq!(e.colors[0].b, 255);
        assert_eq!(e.colors[0].g, 0);
    }

    #[test]
    fn json_contains_colors_key() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("red", 255, 0, 0, 255));
        let cfg = default_color_palette_export_config();
        let json = cpe_to_json(&mut e, &cfg);
        assert!(json.contains("\"colors\""));
        assert!(json.contains("\"red\""));
        assert!(json.contains("\"hex\""));
    }

    #[test]
    fn json_hex_value_correct() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("white", 255, 255, 255, 255));
        let cfg = default_color_palette_export_config();
        let json = cpe_to_json(&mut e, &cfg);
        assert!(json.contains("FFFFFFFF"));
    }

    #[test]
    fn css_contains_root_block() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("Primary Blue", 0, 100, 200, 255));
        let cfg = default_color_palette_export_config();
        let css = cpe_to_css(&mut e, &cfg);
        assert!(css.contains(":root"));
        assert!(css.contains("--color-primary-blue"));
    }

    #[test]
    fn find_by_name_returns_correct_entry() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("sky", 135, 206, 235, 255));
        let found = cpe_find_by_name(&e, "sky");
        assert!(found.is_some());
        assert_eq!(found.unwrap().r, 135);
    }

    #[test]
    fn find_by_name_missing_returns_none() {
        let e = new_color_palette_export();
        assert!(cpe_find_by_name(&e, "nonexistent").is_none());
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("green", 0, 255, 0, 255));
        let cfg = default_color_palette_export_config();
        let n = cpe_write_to_file(&mut e, &cfg, "/tmp/palette.json");
        assert!(n > 0);
        assert_eq!(cpe_total_bytes(&e), n);
    }

    #[test]
    fn clear_resets_state() {
        let mut e = new_color_palette_export();
        cpe_add_color(&mut e, make_color("blue", 0, 0, 255, 255));
        let cfg = default_color_palette_export_config();
        cpe_write_to_file(&mut e, &cfg, "/tmp/palette.json");
        cpe_clear(&mut e);
        assert_eq!(cpe_color_count(&e), 0);
        assert_eq!(cpe_total_bytes(&e), 0);
    }
}
