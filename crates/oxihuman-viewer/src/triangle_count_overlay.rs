// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-object triangle count overlay.

/// Triangle count overlay configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriangleCountOverlayConfig {
    /// Enable overlay.
    pub enabled: bool,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color [R, G, B, A].
    pub text_color: [f32; 4],
    /// Highlight objects above this count.
    pub highlight_threshold: u32,
    /// Highlight color [R, G, B, A].
    pub highlight_color: [f32; 4],
    /// Show total triangle count.
    pub show_total: bool,
}

impl Default for TriangleCountOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            font_size: 12.0,
            text_color: [1.0, 1.0, 1.0, 0.9],
            highlight_threshold: 100_000,
            highlight_color: [1.0, 0.3, 0.3, 1.0],
            show_total: true,
        }
    }
}

/// A triangle count entry for one object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriangleCountEntry {
    pub object_id: u32,
    pub triangle_count: u32,
    pub is_highlighted: bool,
    pub label: String,
}

/// Create default config.
#[allow(dead_code)]
pub fn new_triangle_count_overlay_config() -> TriangleCountOverlayConfig {
    TriangleCountOverlayConfig::default()
}

/// Build entry for an object.
#[allow(dead_code)]
pub fn build_triangle_entry(
    id: u32,
    count: u32,
    cfg: &TriangleCountOverlayConfig,
) -> TriangleCountEntry {
    let highlighted = count >= cfg.highlight_threshold;
    TriangleCountEntry {
        object_id: id,
        triangle_count: count,
        is_highlighted: highlighted,
        label: format_triangle_count(count),
    }
}

/// Format triangle count as human-readable string.
#[allow(dead_code)]
pub fn format_triangle_count(count: u32) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f32 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f32 / 1_000.0)
    } else {
        format!("{}", count)
    }
}

/// Sum total triangles from entries.
#[allow(dead_code)]
pub fn total_triangle_count(entries: &[TriangleCountEntry]) -> u64 {
    entries.iter().map(|e| e.triangle_count as u64).sum()
}

/// Enable overlay.
#[allow(dead_code)]
pub fn tco_enable(cfg: &mut TriangleCountOverlayConfig) {
    cfg.enabled = true;
}

/// Disable overlay.
#[allow(dead_code)]
pub fn tco_disable(cfg: &mut TriangleCountOverlayConfig) {
    cfg.enabled = false;
}

/// Set highlight threshold.
#[allow(dead_code)]
pub fn tco_set_threshold(cfg: &mut TriangleCountOverlayConfig, threshold: u32) {
    cfg.highlight_threshold = threshold;
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn triangle_count_overlay_to_json(cfg: &TriangleCountOverlayConfig) -> String {
    format!(
        r#"{{"enabled":{},"highlight_threshold":{},"show_total":{},"font_size":{:.1}}}"#,
        cfg.enabled, cfg.highlight_threshold, cfg.show_total, cfg.font_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = TriangleCountOverlayConfig::default();
        assert!(!c.enabled);
        assert!(c.show_total);
    }

    #[test]
    fn test_format_millions() {
        let s = format_triangle_count(2_500_000);
        assert!(s.contains('M'));
    }

    #[test]
    fn test_format_thousands() {
        let s = format_triangle_count(5_000);
        assert!(s.contains('K'));
    }

    #[test]
    fn test_format_small() {
        let s = format_triangle_count(500);
        assert_eq!(s, "500");
    }

    #[test]
    fn test_build_entry_highlighted() {
        let cfg = TriangleCountOverlayConfig {
            highlight_threshold: 100,
            ..Default::default()
        };
        let e = build_triangle_entry(1, 500, &cfg);
        assert!(e.is_highlighted);
    }

    #[test]
    fn test_build_entry_not_highlighted() {
        let cfg = TriangleCountOverlayConfig::default();
        let e = build_triangle_entry(1, 100, &cfg);
        assert!(!e.is_highlighted);
    }

    #[test]
    fn test_total_count() {
        let cfg = TriangleCountOverlayConfig::default();
        let entries = vec![
            build_triangle_entry(0, 1000, &cfg),
            build_triangle_entry(1, 2000, &cfg),
        ];
        assert_eq!(total_triangle_count(&entries), 3000);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = TriangleCountOverlayConfig::default();
        tco_enable(&mut c);
        assert!(c.enabled);
        tco_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_threshold() {
        let mut c = TriangleCountOverlayConfig::default();
        tco_set_threshold(&mut c, 50_000);
        assert_eq!(c.highlight_threshold, 50_000);
    }

    #[test]
    fn test_to_json() {
        let j = triangle_count_overlay_to_json(&TriangleCountOverlayConfig::default());
        assert!(j.contains("highlight_threshold"));
        assert!(j.contains("show_total"));
    }
}
