// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Isolines on mesh surface (elevation contours).

#![allow(dead_code)]

/// Configuration for contour line rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContourLineConfig {
    /// Number of contour levels.
    pub num_levels: u32,
    /// Minimum value for contour range.
    pub value_min: f32,
    /// Maximum value for contour range.
    pub value_max: f32,
    /// Line width in pixels.
    pub line_width: f32,
    /// Color of contour lines.
    pub color: [f32; 4],
    /// Show major/minor lines.
    pub major_every: u32,
}

#[allow(dead_code)]
impl Default for ContourLineConfig {
    fn default() -> Self {
        Self {
            num_levels: 10,
            value_min: 0.0,
            value_max: 1.0,
            line_width: 1.0,
            color: [0.0, 0.0, 0.0, 1.0],
            major_every: 5,
        }
    }
}

/// A contour level.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContourLevel {
    pub value: f32,
    pub is_major: bool,
}

/// Create default contour line config.
#[allow(dead_code)]
pub fn new_contour_line_config() -> ContourLineConfig {
    ContourLineConfig::default()
}

/// Generate all contour level values.
#[allow(dead_code)]
pub fn cl_generate_levels(cfg: &ContourLineConfig) -> Vec<ContourLevel> {
    if cfg.num_levels == 0 {
        return Vec::new();
    }
    let step = (cfg.value_max - cfg.value_min) / cfg.num_levels as f32;
    (0..cfg.num_levels)
        .map(|i| {
            let value = cfg.value_min + (i as f32 + 0.5) * step;
            let is_major = cfg.major_every > 0 && i % cfg.major_every == 0;
            ContourLevel { value, is_major }
        })
        .collect()
}

/// Check if a vertex value is near a contour level.
#[allow(dead_code)]
pub fn cl_is_on_contour(value: f32, level_value: f32, tolerance: f32) -> bool {
    (value - level_value).abs() < tolerance
}

/// Interpolate a contour crossing position between two vertices.
#[allow(dead_code)]
pub fn cl_interpolate_crossing(v0: f32, v1: f32, isovalue: f32) -> f32 {
    let dv = v1 - v0;
    if dv.abs() < 1e-10 {
        return 0.5;
    }
    ((isovalue - v0) / dv).clamp(0.0, 1.0)
}

/// Set number of levels.
#[allow(dead_code)]
pub fn cl_set_levels(cfg: &mut ContourLineConfig, n: u32) {
    cfg.num_levels = n;
}

/// Set value range.
#[allow(dead_code)]
pub fn cl_set_range(cfg: &mut ContourLineConfig, min: f32, max: f32) {
    cfg.value_min = min;
    cfg.value_max = max.max(min + 1e-6);
}

/// Set line width.
#[allow(dead_code)]
pub fn cl_set_line_width(cfg: &mut ContourLineConfig, w: f32) {
    cfg.line_width = w.max(0.0);
}

/// Spacing between contour levels.
#[allow(dead_code)]
pub fn cl_level_spacing(cfg: &ContourLineConfig) -> f32 {
    if cfg.num_levels == 0 {
        return 0.0;
    }
    (cfg.value_max - cfg.value_min) / cfg.num_levels as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn contour_line_to_json(cfg: &ContourLineConfig) -> String {
    format!(
        r#"{{"num_levels":{},"value_min":{:.4},"value_max":{:.4},"line_width":{:.4}}}"#,
        cfg.num_levels, cfg.value_min, cfg.value_max, cfg.line_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = ContourLineConfig::default();
        assert_eq!(c.num_levels, 10);
        assert!((c.line_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_levels_count() {
        let c = ContourLineConfig::default();
        let levels = cl_generate_levels(&c);
        assert_eq!(levels.len(), 10);
    }

    #[test]
    fn test_generate_levels_empty() {
        let c = ContourLineConfig {
            num_levels: 0,
            ..Default::default()
        };
        let levels = cl_generate_levels(&c);
        assert!(levels.is_empty());
    }

    #[test]
    fn test_is_on_contour() {
        assert!(cl_is_on_contour(0.501, 0.5, 0.01));
        assert!(!cl_is_on_contour(0.6, 0.5, 0.01));
    }

    #[test]
    fn test_interpolate_crossing_mid() {
        let t = cl_interpolate_crossing(0.0, 1.0, 0.5);
        assert!((t - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_level_spacing() {
        let c = ContourLineConfig::default();
        let s = cl_level_spacing(&c);
        assert!((s - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_set_range() {
        let mut c = ContourLineConfig::default();
        cl_set_range(&mut c, -1.0, 1.0);
        assert!((c.value_min + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_major_levels() {
        let c = ContourLineConfig {
            num_levels: 10,
            major_every: 5,
            ..Default::default()
        };
        let levels = cl_generate_levels(&c);
        assert!(levels[0].is_major);
        assert!(levels[5].is_major);
        assert!(!levels[1].is_major);
    }

    #[test]
    fn test_to_json() {
        let j = contour_line_to_json(&ContourLineConfig::default());
        assert!(j.contains("num_levels"));
    }
}
