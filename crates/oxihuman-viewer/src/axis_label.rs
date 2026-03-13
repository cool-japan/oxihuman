// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Axis label rendering at world-space positions.

/// Axis label configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AxisLabelConfig {
    /// Font size in pixels.
    pub font_size: f32,
    /// Label offset from axis end in world units.
    pub offset: f32,
    /// Show X axis label.
    pub show_x: bool,
    /// Show Y axis label.
    pub show_y: bool,
    /// Show Z axis label.
    pub show_z: bool,
    /// X axis color [R, G, B, A].
    pub color_x: [f32; 4],
    /// Y axis color [R, G, B, A].
    pub color_y: [f32; 4],
    /// Z axis color [R, G, B, A].
    pub color_z: [f32; 4],
}

impl Default for AxisLabelConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            offset: 0.15,
            show_x: true,
            show_y: true,
            show_z: true,
            color_x: [1.0, 0.2, 0.2, 1.0],
            color_y: [0.2, 1.0, 0.2, 1.0],
            color_z: [0.2, 0.4, 1.0, 1.0],
        }
    }
}

/// An axis label entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AxisLabelEntry {
    pub label: String,
    pub world_position: [f32; 3],
    pub color: [f32; 4],
}

/// Create default config.
#[allow(dead_code)]
pub fn new_axis_label_config() -> AxisLabelConfig {
    AxisLabelConfig::default()
}

/// Generate label entries for given axis length.
#[allow(dead_code)]
pub fn generate_axis_labels(cfg: &AxisLabelConfig, axis_length: f32) -> Vec<AxisLabelEntry> {
    let mut labels = Vec::new();
    let l = axis_length + cfg.offset;
    if cfg.show_x {
        labels.push(AxisLabelEntry {
            label: "X".to_string(),
            world_position: [l, 0.0, 0.0],
            color: cfg.color_x,
        });
    }
    if cfg.show_y {
        labels.push(AxisLabelEntry {
            label: "Y".to_string(),
            world_position: [0.0, l, 0.0],
            color: cfg.color_y,
        });
    }
    if cfg.show_z {
        labels.push(AxisLabelEntry {
            label: "Z".to_string(),
            world_position: [0.0, 0.0, l],
            color: cfg.color_z,
        });
    }
    labels
}

/// Set font size.
#[allow(dead_code)]
pub fn al_set_font_size(cfg: &mut AxisLabelConfig, size: f32) {
    cfg.font_size = size.max(4.0);
}

/// Toggle axis visibility.
#[allow(dead_code)]
pub fn al_toggle_axis(cfg: &mut AxisLabelConfig, axis: u8, visible: bool) {
    match axis {
        0 => cfg.show_x = visible,
        1 => cfg.show_y = visible,
        2 => cfg.show_z = visible,
        _ => {}
    }
}

/// Visible label count.
#[allow(dead_code)]
pub fn al_visible_count(cfg: &AxisLabelConfig) -> usize {
    [cfg.show_x, cfg.show_y, cfg.show_z]
        .iter()
        .filter(|&&v| v)
        .count()
}

/// Get label text for axis index.
#[allow(dead_code)]
pub fn al_label_text(axis: u8) -> &'static str {
    match axis {
        0 => "X",
        1 => "Y",
        2 => "Z",
        _ => "?",
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn axis_label_to_json(cfg: &AxisLabelConfig) -> String {
    format!(
        r#"{{"font_size":{:.1},"offset":{:.4},"show_x":{},"show_y":{},"show_z":{}}}"#,
        cfg.font_size, cfg.offset, cfg.show_x, cfg.show_y, cfg.show_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = AxisLabelConfig::default();
        assert!(c.show_x && c.show_y && c.show_z);
    }

    #[test]
    fn test_generate_all() {
        let c = AxisLabelConfig::default();
        let labels = generate_axis_labels(&c, 1.0);
        assert_eq!(labels.len(), 3);
    }

    #[test]
    fn test_generate_x_only() {
        let c = AxisLabelConfig {
            show_y: false,
            show_z: false,
            ..Default::default()
        };
        let labels = generate_axis_labels(&c, 1.0);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].label, "X");
    }

    #[test]
    fn test_set_font_size_min() {
        let mut c = AxisLabelConfig::default();
        al_set_font_size(&mut c, -5.0);
        assert!(c.font_size >= 4.0);
    }

    #[test]
    fn test_toggle_axis() {
        let mut c = AxisLabelConfig::default();
        al_toggle_axis(&mut c, 0, false);
        assert!(!c.show_x);
    }

    #[test]
    fn test_visible_count() {
        let c = AxisLabelConfig::default();
        assert_eq!(al_visible_count(&c), 3);
    }

    #[test]
    fn test_visible_count_partial() {
        let c = AxisLabelConfig {
            show_z: false,
            ..Default::default()
        };
        assert_eq!(al_visible_count(&c), 2);
    }

    #[test]
    fn test_label_text() {
        assert_eq!(al_label_text(0), "X");
        assert_eq!(al_label_text(1), "Y");
        assert_eq!(al_label_text(2), "Z");
    }

    #[test]
    fn test_label_position_offset() {
        let c = AxisLabelConfig {
            offset: 0.5,
            ..Default::default()
        };
        let labels = generate_axis_labels(&c, 1.0);
        let x_label = &labels[0];
        assert!((x_label.world_position[0] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = axis_label_to_json(&AxisLabelConfig::default());
        assert!(j.contains("font_size"));
        assert!(j.contains("show_x"));
    }
}
