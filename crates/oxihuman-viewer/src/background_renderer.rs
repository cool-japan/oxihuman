// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Background / environment rendering: gradient, solid colour, checkerboard, and cubemap stub.

// ── Enums / Structs ──────────────────────────────────────────────────────────

/// Supported background rendering modes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundType {
    /// Flat single colour.
    SolidColor,
    /// Top-to-bottom gradient.
    VerticalGradient,
    /// Left-to-right gradient.
    HorizontalGradient,
    /// Checkerboard pattern (useful for transparency preview).
    Checkerboard,
    /// Placeholder for future cubemap support.
    CubemapStub,
}

/// Configuration for background rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BackgroundConfig {
    /// Which background mode is active.
    pub bg_type: BackgroundType,
    /// Primary / top / fill colour (linear RGBA).
    pub top_color: [f32; 4],
    /// Secondary / bottom colour (linear RGBA).
    pub bottom_color: [f32; 4],
    /// Size of each checker square in pixels.
    pub checker_size: u32,
}

/// A sampled background colour at a UV coordinate.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundSample {
    /// Sampled RGBA colour (linear, [0, 1]).
    pub color: [f32; 4],
    /// U coordinate that was sampled.
    pub u: f32,
    /// V coordinate that was sampled.
    pub v: f32,
}

// ── Constructors ─────────────────────────────────────────────────────────────

/// Create a default background config (dark grey solid colour).
#[allow(dead_code)]
pub fn default_background_config() -> BackgroundConfig {
    BackgroundConfig {
        bg_type: BackgroundType::SolidColor,
        top_color: [0.15, 0.15, 0.18, 1.0],
        bottom_color: [0.05, 0.05, 0.07, 1.0],
        checker_size: 16,
    }
}

/// Create a new background config with the given type.
#[allow(dead_code)]
pub fn new_background_config(bg_type: BackgroundType) -> BackgroundConfig {
    BackgroundConfig {
        bg_type,
        ..default_background_config()
    }
}

// ── Sampling ─────────────────────────────────────────────────────────────────

/// Sample the background colour at normalised UV coordinates `(u, v)` in [0, 1].
///
/// * `u` = horizontal (0 = left, 1 = right)
/// * `v` = vertical   (0 = top,  1 = bottom)
#[allow(dead_code)]
pub fn sample_background(config: &BackgroundConfig, u: f32, v: f32) -> BackgroundSample {
    let color = match config.bg_type {
        BackgroundType::SolidColor => config.top_color,
        BackgroundType::VerticalGradient => {
            gradient_color(&config.top_color, &config.bottom_color, v)
        }
        BackgroundType::HorizontalGradient => {
            gradient_color(&config.top_color, &config.bottom_color, u)
        }
        BackgroundType::Checkerboard => checkerboard_color(config, u, v),
        BackgroundType::CubemapStub => config.top_color, // stub: just return top colour
    };
    BackgroundSample { color, u, v }
}

/// Linearly interpolate between two RGBA colours by `t` in [0, 1].
#[allow(dead_code)]
pub fn gradient_color(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Compute a checkerboard colour at `(u, v)` using the config's checker size
/// and top/bottom colours.
#[allow(dead_code)]
pub fn checkerboard_color(config: &BackgroundConfig, u: f32, v: f32) -> [f32; 4] {
    let size = config.checker_size.max(1) as f32;
    // Scale UV into pixel-grid space (assume a virtual 1024-pixel canvas)
    let px = (u * 1024.0) / size;
    let py = (v * 1024.0) / size;
    let checker = ((px as i32) + (py as i32)) % 2 == 0;
    if checker {
        config.top_color
    } else {
        config.bottom_color
    }
}

// ── Setters ──────────────────────────────────────────────────────────────────

/// Set the background type.
#[allow(dead_code)]
pub fn set_background_type(config: &mut BackgroundConfig, bg_type: BackgroundType) {
    config.bg_type = bg_type;
}

/// Set the top / primary colour.
#[allow(dead_code)]
pub fn set_top_color(config: &mut BackgroundConfig, color: [f32; 4]) {
    config.top_color = color;
}

/// Set the bottom / secondary colour.
#[allow(dead_code)]
pub fn set_bottom_color(config: &mut BackgroundConfig, color: [f32; 4]) {
    config.bottom_color = color;
}

/// Set the checkerboard square size (in pixels).
#[allow(dead_code)]
pub fn set_checker_size(config: &mut BackgroundConfig, size: u32) {
    config.checker_size = size.max(1);
}

// ── Query / utility ──────────────────────────────────────────────────────────

/// Return a human-readable name for the background type.
#[allow(dead_code)]
pub fn background_type_name(bg_type: BackgroundType) -> &'static str {
    match bg_type {
        BackgroundType::SolidColor => "Solid Color",
        BackgroundType::VerticalGradient => "Vertical Gradient",
        BackgroundType::HorizontalGradient => "Horizontal Gradient",
        BackgroundType::Checkerboard => "Checkerboard",
        BackgroundType::CubemapStub => "Cubemap (stub)",
    }
}

/// Return `true` if the background type produces a pattern that varies across
/// the viewport (i.e. is not a single flat colour).
#[allow(dead_code)]
pub fn is_dynamic_background(bg_type: BackgroundType) -> bool {
    !matches!(bg_type, BackgroundType::SolidColor)
}

/// Serialise a background config to a JSON string.
#[allow(dead_code)]
pub fn background_to_json(config: &BackgroundConfig) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"type\": \"{}\",\n",
            "  \"top_color\": [{:.6}, {:.6}, {:.6}, {:.6}],\n",
            "  \"bottom_color\": [{:.6}, {:.6}, {:.6}, {:.6}],\n",
            "  \"checker_size\": {}\n",
            "}}"
        ),
        background_type_name(config.bg_type),
        config.top_color[0],
        config.top_color[1],
        config.top_color[2],
        config.top_color[3],
        config.bottom_color[0],
        config.bottom_color[1],
        config.bottom_color[2],
        config.bottom_color[3],
        config.checker_size,
    )
}

/// Generate a flat pixel grid of background samples.
///
/// Returns a `Vec<BackgroundSample>` with `width * height` entries in row-major
/// order (top-left to bottom-right).
#[allow(dead_code)]
pub fn background_pixel_grid(
    config: &BackgroundConfig,
    width: u32,
    height: u32,
) -> Vec<BackgroundSample> {
    let w = width.max(1);
    let h = height.max(1);
    let mut out = Vec::with_capacity((w * h) as usize);
    for row in 0..h {
        let v = if h > 1 {
            row as f32 / (h - 1) as f32
        } else {
            0.0
        };
        for col in 0..w {
            let u = if w > 1 {
                col as f32 / (w - 1) as f32
            } else {
                0.0
            };
            out.push(sample_background(config, u, v));
        }
    }
    out
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_solid() {
        let cfg = default_background_config();
        assert_eq!(cfg.bg_type, BackgroundType::SolidColor);
    }

    #[test]
    fn new_background_config_sets_type() {
        let cfg = new_background_config(BackgroundType::Checkerboard);
        assert_eq!(cfg.bg_type, BackgroundType::Checkerboard);
    }

    #[test]
    fn sample_solid_returns_top_color() {
        let cfg = default_background_config();
        let s = sample_background(&cfg, 0.5, 0.5);
        assert_eq!(s.color, cfg.top_color);
    }

    #[test]
    fn gradient_color_at_zero() {
        let a = [1.0, 0.0, 0.0, 1.0];
        let b = [0.0, 1.0, 0.0, 1.0];
        let c = gradient_color(&a, &b, 0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn gradient_color_at_one() {
        let a = [1.0, 0.0, 0.0, 1.0];
        let b = [0.0, 1.0, 0.0, 1.0];
        let c = gradient_color(&a, &b, 1.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gradient_color_midpoint() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [1.0, 1.0, 1.0, 1.0];
        let c = gradient_color(&a, &b, 0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn vertical_gradient_top_is_top_color() {
        let mut cfg = default_background_config();
        set_background_type(&mut cfg, BackgroundType::VerticalGradient);
        set_top_color(&mut cfg, [1.0, 0.0, 0.0, 1.0]);
        set_bottom_color(&mut cfg, [0.0, 0.0, 1.0, 1.0]);
        let s = sample_background(&cfg, 0.5, 0.0);
        assert!((s.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn horizontal_gradient_varies_by_u() {
        let mut cfg = new_background_config(BackgroundType::HorizontalGradient);
        cfg.top_color = [0.0, 0.0, 0.0, 1.0];
        cfg.bottom_color = [1.0, 1.0, 1.0, 1.0];
        let left = sample_background(&cfg, 0.0, 0.5);
        let right = sample_background(&cfg, 1.0, 0.5);
        assert!((left.color[0] - 0.0).abs() < 1e-6);
        assert!((right.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn checkerboard_alternates() {
        let mut cfg = new_background_config(BackgroundType::Checkerboard);
        cfg.top_color = [1.0, 1.0, 1.0, 1.0];
        cfg.bottom_color = [0.0, 0.0, 0.0, 1.0];
        cfg.checker_size = 16;
        let s1 = sample_background(&cfg, 0.0, 0.0);
        let s2 = sample_background(&cfg, 16.0 / 1024.0, 0.0);
        // They should differ (one white, one black)
        assert_ne!(s1.color, s2.color);
    }

    #[test]
    fn set_checker_size_clamps_min() {
        let mut cfg = default_background_config();
        set_checker_size(&mut cfg, 0);
        assert_eq!(cfg.checker_size, 1);
    }

    #[test]
    fn background_type_name_all_variants() {
        assert_eq!(
            background_type_name(BackgroundType::SolidColor),
            "Solid Color"
        );
        assert_eq!(
            background_type_name(BackgroundType::VerticalGradient),
            "Vertical Gradient"
        );
        assert_eq!(
            background_type_name(BackgroundType::HorizontalGradient),
            "Horizontal Gradient"
        );
        assert_eq!(
            background_type_name(BackgroundType::Checkerboard),
            "Checkerboard"
        );
        assert_eq!(
            background_type_name(BackgroundType::CubemapStub),
            "Cubemap (stub)"
        );
    }

    #[test]
    fn is_dynamic_background_correct() {
        assert!(!is_dynamic_background(BackgroundType::SolidColor));
        assert!(is_dynamic_background(BackgroundType::VerticalGradient));
        assert!(is_dynamic_background(BackgroundType::HorizontalGradient));
        assert!(is_dynamic_background(BackgroundType::Checkerboard));
        assert!(is_dynamic_background(BackgroundType::CubemapStub));
    }

    #[test]
    fn background_to_json_contains_type() {
        let cfg = default_background_config();
        let json = background_to_json(&cfg);
        assert!(json.contains("Solid Color"));
        assert!(json.contains("top_color"));
        assert!(json.contains("bottom_color"));
    }

    #[test]
    fn background_pixel_grid_correct_count() {
        let cfg = default_background_config();
        let grid = background_pixel_grid(&cfg, 4, 3);
        assert_eq!(grid.len(), 12);
    }

    #[test]
    fn background_pixel_grid_single_pixel() {
        let cfg = default_background_config();
        let grid = background_pixel_grid(&cfg, 1, 1);
        assert_eq!(grid.len(), 1);
    }

    #[test]
    fn cubemap_stub_returns_top_color() {
        let cfg = new_background_config(BackgroundType::CubemapStub);
        let s = sample_background(&cfg, 0.5, 0.5);
        assert_eq!(s.color, cfg.top_color);
    }
}
