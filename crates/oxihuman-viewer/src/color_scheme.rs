// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Color scheme and theme manager for the OxiHuman viewer UI.
//!
//! Provides named color roles, dark/light presets, blending, and WCAG-style
//! contrast ratio computation.

#![allow(dead_code)]

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Semantic color role within a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeColor {
    /// Window / viewport background.
    Background,
    /// Default text and icon foreground.
    Foreground,
    /// Primary accent / highlight color.
    Accent,
    /// Non-critical warning indicator.
    Warning,
    /// Error or destructive action indicator.
    Error,
    /// Success or confirmation indicator.
    Success,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A theme color entry stored as `[r, g, b, a]` in `0.0..=1.0` linear space.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorEntry {
    /// RGBA components in linear `[0, 1]` range.
    pub rgba: [f32; 4],
}

impl ColorEntry {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { rgba: [r, g, b, a] }
    }
}

/// A complete color scheme holding one [`ColorEntry`] per [`ThemeColor`] role.
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Human-readable scheme name.
    pub name: String,
    background: ColorEntry,
    foreground: ColorEntry,
    accent: ColorEntry,
    warning: ColorEntry,
    error: ColorEntry,
    success: ColorEntry,
}

/// Configuration for constructing a [`ColorScheme`].
#[derive(Debug, Clone)]
pub struct ColorSchemeConfig {
    /// Scheme name. Default: `"default"`.
    pub name: String,
    /// Background RGBA. Default: dark grey `[0.12, 0.12, 0.12, 1.0]`.
    pub background: [f32; 4],
    /// Foreground RGBA. Default: near-white `[0.9, 0.9, 0.9, 1.0]`.
    pub foreground: [f32; 4],
    /// Accent RGBA. Default: teal `[0.0, 0.6, 0.8, 1.0]`.
    pub accent: [f32; 4],
    /// Warning RGBA. Default: amber `[1.0, 0.75, 0.0, 1.0]`.
    pub warning: [f32; 4],
    /// Error RGBA. Default: red `[0.9, 0.1, 0.1, 1.0]`.
    pub error: [f32; 4],
    /// Success RGBA. Default: green `[0.1, 0.8, 0.2, 1.0]`.
    pub success: [f32; 4],
}

impl Default for ColorSchemeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            background: [0.12, 0.12, 0.12, 1.0],
            foreground: [0.90, 0.90, 0.90, 1.0],
            accent: [0.00, 0.60, 0.80, 1.0],
            warning: [1.00, 0.75, 0.00, 1.0],
            error: [0.90, 0.10, 0.10, 1.0],
            success: [0.10, 0.80, 0.20, 1.0],
        }
    }
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// An RGBA color as `[r, g, b, a]` in `0.0..=1.0` linear space.
pub type Rgba = [f32; 4];

/// Luminance-based contrast ratio as defined by WCAG 2.1.
pub type ContrastRatio = f32;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default dark [`ColorScheme`] built from [`ColorSchemeConfig::default`].
#[allow(dead_code)]
pub fn default_color_scheme() -> ColorScheme {
    new_color_scheme(&ColorSchemeConfig::default())
}

/// Construct a [`ColorScheme`] from a [`ColorSchemeConfig`].
#[allow(dead_code)]
pub fn new_color_scheme(cfg: &ColorSchemeConfig) -> ColorScheme {
    ColorScheme {
        name: cfg.name.clone(),
        background: ColorEntry::new(cfg.background[0], cfg.background[1], cfg.background[2], cfg.background[3]),
        foreground: ColorEntry::new(cfg.foreground[0], cfg.foreground[1], cfg.foreground[2], cfg.foreground[3]),
        accent: ColorEntry::new(cfg.accent[0], cfg.accent[1], cfg.accent[2], cfg.accent[3]),
        warning: ColorEntry::new(cfg.warning[0], cfg.warning[1], cfg.warning[2], cfg.warning[3]),
        error: ColorEntry::new(cfg.error[0], cfg.error[1], cfg.error[2], cfg.error[3]),
        success: ColorEntry::new(cfg.success[0], cfg.success[1], cfg.success[2], cfg.success[3]),
    }
}

/// Return the RGBA value for `role` in `scheme`.
#[allow(dead_code)]
pub fn get_theme_color(scheme: &ColorScheme, role: ThemeColor) -> Rgba {
    match role {
        ThemeColor::Background => scheme.background.rgba,
        ThemeColor::Foreground => scheme.foreground.rgba,
        ThemeColor::Accent => scheme.accent.rgba,
        ThemeColor::Warning => scheme.warning.rgba,
        ThemeColor::Error => scheme.error.rgba,
        ThemeColor::Success => scheme.success.rgba,
    }
}

/// Set the color for `role` in `scheme` to `rgba`.
#[allow(dead_code)]
pub fn set_theme_color(scheme: &mut ColorScheme, role: ThemeColor, rgba: Rgba) {
    let entry = ColorEntry::new(rgba[0], rgba[1], rgba[2], rgba[3]);
    match role {
        ThemeColor::Background => scheme.background = entry,
        ThemeColor::Foreground => scheme.foreground = entry,
        ThemeColor::Accent => scheme.accent = entry,
        ThemeColor::Warning => scheme.warning = entry,
        ThemeColor::Error => scheme.error = entry,
        ThemeColor::Success => scheme.success = entry,
    }
}

/// Return a human-readable name for a [`ThemeColor`] role.
#[allow(dead_code)]
pub fn theme_color_name(role: ThemeColor) -> &'static str {
    match role {
        ThemeColor::Background => "background",
        ThemeColor::Foreground => "foreground",
        ThemeColor::Accent => "accent",
        ThemeColor::Warning => "warning",
        ThemeColor::Error => "error",
        ThemeColor::Success => "success",
    }
}

/// Return the name of `scheme`.
#[allow(dead_code)]
pub fn scheme_name(scheme: &ColorScheme) -> &str {
    &scheme.name
}

/// Set the name of `scheme`.
#[allow(dead_code)]
pub fn set_scheme_name(scheme: &mut ColorScheme, name: &str) {
    scheme.name = name.to_string();
}

/// Serialize `scheme` to a compact JSON string.
#[allow(dead_code)]
pub fn scheme_to_json(scheme: &ColorScheme) -> String {
    let roles = [
        ThemeColor::Background,
        ThemeColor::Foreground,
        ThemeColor::Accent,
        ThemeColor::Warning,
        ThemeColor::Error,
        ThemeColor::Success,
    ];
    let entries: Vec<String> = roles
        .iter()
        .map(|&r| {
            let c = get_theme_color(scheme, r);
            format!(
                r#""{}":[{},{},{},{}]"#,
                theme_color_name(r),
                c[0], c[1], c[2], c[3]
            )
        })
        .collect();
    format!(r#"{{"name":"{}","colors":{{{}}}}}"#, scheme.name, entries.join(","))
}

/// Return a pre-built dark theme [`ColorScheme`].
#[allow(dead_code)]
pub fn dark_theme() -> ColorScheme {
    new_color_scheme(&ColorSchemeConfig {
        name: "dark".to_string(),
        background: [0.08, 0.08, 0.08, 1.0],
        foreground: [0.92, 0.92, 0.92, 1.0],
        accent: [0.20, 0.60, 1.00, 1.0],
        warning: [1.00, 0.70, 0.00, 1.0],
        error: [1.00, 0.20, 0.20, 1.0],
        success: [0.20, 0.85, 0.30, 1.0],
    })
}

/// Return a pre-built light theme [`ColorScheme`].
#[allow(dead_code)]
pub fn light_theme() -> ColorScheme {
    new_color_scheme(&ColorSchemeConfig {
        name: "light".to_string(),
        background: [0.96, 0.96, 0.96, 1.0],
        foreground: [0.10, 0.10, 0.10, 1.0],
        accent: [0.00, 0.45, 0.80, 1.0],
        warning: [0.80, 0.55, 0.00, 1.0],
        error: [0.75, 0.00, 0.00, 1.0],
        success: [0.00, 0.55, 0.10, 1.0],
    })
}

/// Linearly blend `a` and `b` by `t` (0.0 = full `a`, 1.0 = full `b`).
#[allow(dead_code)]
pub fn blend_schemes(a: &ColorScheme, b: &ColorScheme, t: f32) -> ColorScheme {
    let blend_rgba = |ca: Rgba, cb: Rgba| -> [f32; 4] {
        [
            ca[0] + (cb[0] - ca[0]) * t,
            ca[1] + (cb[1] - ca[1]) * t,
            ca[2] + (cb[2] - ca[2]) * t,
            ca[3] + (cb[3] - ca[3]) * t,
        ]
    };
    let roles = [
        ThemeColor::Background,
        ThemeColor::Foreground,
        ThemeColor::Accent,
        ThemeColor::Warning,
        ThemeColor::Error,
        ThemeColor::Success,
    ];
    let mut result = a.clone();
    result.name = format!("blend({},{})", a.name, b.name);
    for &role in &roles {
        let ca = get_theme_color(a, role);
        let cb = get_theme_color(b, role);
        set_theme_color(&mut result, role, blend_rgba(ca, cb));
    }
    result
}

/// Compute the relative luminance of an sRGB color per WCAG 2.1.
fn relative_luminance(rgba: Rgba) -> f32 {
    let linearize = |c: f32| -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let r = linearize(rgba[0]);
    let g = linearize(rgba[1]);
    let b = linearize(rgba[2]);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Compute the WCAG 2.1 contrast ratio between two colors.
#[allow(dead_code)]
pub fn contrast_ratio(fg: Rgba, bg: Rgba) -> ContrastRatio {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Return `(foreground, background)` such that the pair meets a minimum contrast ratio.
///
/// Tries the scheme's foreground/background first; if the ratio is below `min_ratio`,
/// swaps them. Returns the pair that has the higher ratio.
#[allow(dead_code)]
pub fn accessible_color_pair(scheme: &ColorScheme, min_ratio: f32) -> (Rgba, Rgba) {
    let fg = get_theme_color(scheme, ThemeColor::Foreground);
    let bg = get_theme_color(scheme, ThemeColor::Background);
    let ratio = contrast_ratio(fg, bg);
    if ratio >= min_ratio {
        (fg, bg)
    } else {
        // swap and try; return whichever is higher
        let swapped_ratio = contrast_ratio(bg, fg);
        if swapped_ratio >= ratio { (bg, fg) } else { (fg, bg) }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_scheme_name() {
        let s = default_color_scheme();
        assert_eq!(s.name, "default");
    }

    #[test]
    fn test_new_color_scheme_from_config() {
        let cfg = ColorSchemeConfig::default();
        let s = new_color_scheme(&cfg);
        assert_eq!(s.name, "default");
    }

    #[test]
    fn test_get_theme_color_background() {
        let s = default_color_scheme();
        let bg = get_theme_color(&s, ThemeColor::Background);
        // dark background should have low luminance
        assert!(bg[0] < 0.5);
    }

    #[test]
    fn test_set_theme_color() {
        let mut s = default_color_scheme();
        set_theme_color(&mut s, ThemeColor::Accent, [1.0, 0.0, 0.0, 1.0]);
        let accent = get_theme_color(&s, ThemeColor::Accent);
        assert_eq!(accent, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_theme_color_name_all_variants() {
        assert_eq!(theme_color_name(ThemeColor::Background), "background");
        assert_eq!(theme_color_name(ThemeColor::Foreground), "foreground");
        assert_eq!(theme_color_name(ThemeColor::Accent), "accent");
        assert_eq!(theme_color_name(ThemeColor::Warning), "warning");
        assert_eq!(theme_color_name(ThemeColor::Error), "error");
        assert_eq!(theme_color_name(ThemeColor::Success), "success");
    }

    #[test]
    fn test_scheme_name_get_set() {
        let mut s = default_color_scheme();
        assert_eq!(scheme_name(&s), "default");
        set_scheme_name(&mut s, "custom");
        assert_eq!(scheme_name(&s), "custom");
    }

    #[test]
    fn test_scheme_to_json_contains_name() {
        let s = default_color_scheme();
        let json = scheme_to_json(&s);
        assert!(json.contains("\"name\":\"default\""));
    }

    #[test]
    fn test_scheme_to_json_contains_all_roles() {
        let s = default_color_scheme();
        let json = scheme_to_json(&s);
        for role in ["background", "foreground", "accent", "warning", "error", "success"] {
            assert!(json.contains(role), "missing role: {}", role);
        }
    }

    #[test]
    fn test_dark_theme_name() {
        let d = dark_theme();
        assert_eq!(d.name, "dark");
    }

    #[test]
    fn test_light_theme_name() {
        let l = light_theme();
        assert_eq!(l.name, "light");
    }

    #[test]
    fn test_dark_background_darker_than_light() {
        let dark = dark_theme();
        let light = light_theme();
        let dark_bg = get_theme_color(&dark, ThemeColor::Background);
        let light_bg = get_theme_color(&light, ThemeColor::Background);
        assert!(dark_bg[0] < light_bg[0]);
    }

    #[test]
    fn test_blend_schemes_midpoint() {
        let a = dark_theme();
        let b = light_theme();
        let blended = blend_schemes(&a, &b, 0.5);
        let a_bg = get_theme_color(&a, ThemeColor::Background);
        let b_bg = get_theme_color(&b, ThemeColor::Background);
        let blend_bg = get_theme_color(&blended, ThemeColor::Background);
        let expected = (a_bg[0] + b_bg[0]) / 2.0;
        assert!((blend_bg[0] - expected).abs() < 1e-5);
    }

    #[test]
    fn test_blend_schemes_name() {
        let blended = blend_schemes(&dark_theme(), &light_theme(), 0.5);
        assert!(blended.name.contains("dark"));
        assert!(blended.name.contains("light"));
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let white = [1.0f32, 1.0, 1.0, 1.0];
        let black = [0.0f32, 0.0, 0.0, 1.0];
        let ratio = contrast_ratio(white, black);
        assert!((ratio - 21.0).abs() < 0.1, "expected ~21, got {}", ratio);
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let grey = [0.5f32, 0.5, 0.5, 1.0];
        let ratio = contrast_ratio(grey, grey);
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_accessible_color_pair_dark_theme() {
        let s = dark_theme();
        let (fg, bg) = accessible_color_pair(&s, 4.5);
        let ratio = contrast_ratio(fg, bg);
        // dark theme should have good contrast
        assert!(ratio > 1.0);
    }
}
