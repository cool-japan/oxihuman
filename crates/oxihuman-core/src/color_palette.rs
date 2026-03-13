// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Color palette management.

#![allow(dead_code)]

/// A single named color entry (RGBA in [0, 1]).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorEntry {
    pub name: String,
    pub rgba: [f32; 4],
}

/// A named collection of color entries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub name: String,
    pub colors: Vec<ColorEntry>,
}

/// Creates a new empty color palette with the given name.
#[allow(dead_code)]
pub fn new_color_palette(name: &str) -> ColorPalette {
    ColorPalette {
        name: name.to_string(),
        colors: Vec::new(),
    }
}

/// Adds a color entry to a palette.
#[allow(dead_code)]
pub fn add_color(pal: &mut ColorPalette, name: &str, rgba: [f32; 4]) {
    pal.colors.push(ColorEntry {
        name: name.to_string(),
        rgba,
    });
}

/// Looks up a color by name. Returns `None` if not found.
#[allow(dead_code)]
pub fn get_color(pal: &ColorPalette, name: &str) -> Option<[f32; 4]> {
    pal.colors.iter().find(|e| e.name == name).map(|e| e.rgba)
}

/// Returns the number of colors in a palette.
#[allow(dead_code)]
pub fn palette_size(pal: &ColorPalette) -> usize {
    pal.colors.len()
}

/// Linearly blends two RGBA colors by `t` in [0, 1].
#[allow(dead_code)]
pub fn blend_palette_colors(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_new_palette_empty() {
        let p = new_color_palette("test");
        assert_eq!(p.name, "test");
        assert_eq!(palette_size(&p), 0);
    }

    #[test]
    fn test_add_color() {
        let mut p = new_color_palette("pal");
        add_color(&mut p, "red", [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(palette_size(&p), 1);
    }

    #[test]
    fn test_get_color_found() {
        let mut p = new_color_palette("pal");
        add_color(&mut p, "blue", [0.0, 0.0, 1.0, 1.0]);
        let c = get_color(&p, "blue").expect("should succeed");
        assert!((c[2] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_get_color_missing() {
        let p = new_color_palette("pal");
        assert!(get_color(&p, "missing").is_none());
    }

    #[test]
    fn test_palette_size_multiple() {
        let mut p = new_color_palette("pal");
        add_color(&mut p, "a", [1.0, 0.0, 0.0, 1.0]);
        add_color(&mut p, "b", [0.0, 1.0, 0.0, 1.0]);
        add_color(&mut p, "c", [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(palette_size(&p), 3);
    }

    #[test]
    fn test_blend_t0() {
        let a = [1.0, 0.0, 0.0, 1.0];
        let b = [0.0, 1.0, 0.0, 1.0];
        let r = blend_palette_colors(a, b, 0.0);
        assert!((r[0] - 1.0).abs() < EPS);
        assert!((r[1]).abs() < EPS);
    }

    #[test]
    fn test_blend_t1() {
        let a = [1.0, 0.0, 0.0, 1.0];
        let b = [0.0, 1.0, 0.0, 1.0];
        let r = blend_palette_colors(a, b, 1.0);
        assert!((r[0]).abs() < EPS);
        assert!((r[1] - 1.0).abs() < EPS);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = [0.0, 0.0, 0.0, 0.0];
        let b = [1.0, 1.0, 1.0, 1.0];
        let r = blend_palette_colors(a, b, 0.5);
        for ch in r {
            assert!((ch - 0.5).abs() < EPS);
        }
    }

    #[test]
    fn test_blend_clamp() {
        let a = [1.0, 0.0, 0.0, 1.0];
        let b = [0.0, 1.0, 0.0, 1.0];
        let r = blend_palette_colors(a, b, 2.0);
        // Should be clamped to t=1.0
        assert!((r[0]).abs() < EPS);
    }
}
