// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 3D scale ruler overlay with tick marks.

/// Scale ruler configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScaleRuler3DConfig {
    /// Total ruler length in world units.
    pub length: f32,
    /// Major tick spacing in world units.
    pub major_tick_spacing: f32,
    /// Minor ticks per major tick interval.
    pub minor_ticks_per_major: u32,
    /// Tick height (major) in world units.
    pub major_tick_height: f32,
    /// Tick height (minor) in world units.
    pub minor_tick_height: f32,
    /// Ruler color [R, G, B, A].
    pub color: [f32; 4],
    /// Label unit string.
    pub unit_label: &'static str,
    /// Show numbers at major ticks.
    pub show_numbers: bool,
}

impl Default for ScaleRuler3DConfig {
    fn default() -> Self {
        Self {
            length: 2.0,
            major_tick_spacing: 0.5,
            minor_ticks_per_major: 5,
            major_tick_height: 0.05,
            minor_tick_height: 0.025,
            color: [1.0, 1.0, 1.0, 0.9],
            unit_label: "m",
            show_numbers: true,
        }
    }
}

/// A tick mark entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TickMark3D {
    pub position: f32,
    pub height: f32,
    pub is_major: bool,
    pub label: Option<String>,
}

/// Create default config.
#[allow(dead_code)]
pub fn new_scale_ruler_3d_config() -> ScaleRuler3DConfig {
    ScaleRuler3DConfig::default()
}

/// Generate tick marks for the ruler.
#[allow(dead_code)]
pub fn generate_tick_marks(cfg: &ScaleRuler3DConfig) -> Vec<TickMark3D> {
    let mut ticks = Vec::new();
    let n_major = (cfg.length / cfg.major_tick_spacing).ceil() as u32;
    let minor_step = if cfg.minor_ticks_per_major > 0 {
        cfg.major_tick_spacing / cfg.minor_ticks_per_major as f32
    } else {
        cfg.major_tick_spacing
    };

    let n_minor_total = (cfg.length / minor_step).ceil() as u32 + 1;
    let mut pos;
    for i in 0..n_minor_total {
        pos = i as f32 * minor_step;
        if pos > cfg.length + 1e-5 {
            break;
        }
        let major_idx = (pos / cfg.major_tick_spacing).round() as u32;
        let is_major = (pos - major_idx as f32 * cfg.major_tick_spacing).abs() < 1e-5;
        let label = if is_major && cfg.show_numbers {
            Some(format!("{:.1}{}", pos, cfg.unit_label))
        } else {
            None
        };
        ticks.push(TickMark3D {
            position: pos,
            height: if is_major {
                cfg.major_tick_height
            } else {
                cfg.minor_tick_height
            },
            is_major,
            label,
        });
    }
    let _ = n_major;
    ticks
}

/// Set ruler length.
#[allow(dead_code)]
pub fn sr_set_length(cfg: &mut ScaleRuler3DConfig, length: f32) {
    cfg.length = length.max(0.01);
}

/// Set major tick spacing.
#[allow(dead_code)]
pub fn sr_set_major_spacing(cfg: &mut ScaleRuler3DConfig, spacing: f32) {
    cfg.major_tick_spacing = spacing.max(0.001);
}

/// Count of major ticks.
#[allow(dead_code)]
pub fn sr_major_tick_count(cfg: &ScaleRuler3DConfig) -> u32 {
    (cfg.length / cfg.major_tick_spacing).floor() as u32 + 1
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn scale_ruler_3d_to_json(cfg: &ScaleRuler3DConfig) -> String {
    format!(
        r#"{{"length":{:.4},"major_spacing":{:.4},"minor_per_major":{},"unit":"{}"}}"#,
        cfg.length, cfg.major_tick_spacing, cfg.minor_ticks_per_major, cfg.unit_label
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = ScaleRuler3DConfig::default();
        assert!((c.length - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_ticks_not_empty() {
        let c = ScaleRuler3DConfig::default();
        let ticks = generate_tick_marks(&c);
        assert!(!ticks.is_empty());
    }

    #[test]
    fn test_generate_has_major() {
        let c = ScaleRuler3DConfig::default();
        let ticks = generate_tick_marks(&c);
        assert!(ticks.iter().any(|t| t.is_major));
    }

    #[test]
    fn test_generate_major_has_label() {
        let c = ScaleRuler3DConfig::default();
        let ticks = generate_tick_marks(&c);
        let majors: Vec<_> = ticks.iter().filter(|t| t.is_major).collect();
        assert!(majors[0].label.is_some());
    }

    #[test]
    fn test_set_length() {
        let mut c = ScaleRuler3DConfig::default();
        sr_set_length(&mut c, 5.0);
        assert!((c.length - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_length_min() {
        let mut c = ScaleRuler3DConfig::default();
        sr_set_length(&mut c, -1.0);
        assert!(c.length > 0.0);
    }

    #[test]
    fn test_set_major_spacing() {
        let mut c = ScaleRuler3DConfig::default();
        sr_set_major_spacing(&mut c, 0.1);
        assert!((c.major_tick_spacing - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_major_tick_count() {
        let c = ScaleRuler3DConfig {
            length: 2.0,
            major_tick_spacing: 0.5,
            ..Default::default()
        };
        assert_eq!(sr_major_tick_count(&c), 5);
    }

    #[test]
    fn test_to_json() {
        let j = scale_ruler_3d_to_json(&ScaleRuler3DConfig::default());
        assert!(j.contains("length"));
        assert!(j.contains("unit"));
    }

    #[test]
    fn test_minor_shorter() {
        let c = ScaleRuler3DConfig::default();
        assert!(c.minor_tick_height < c.major_tick_height);
    }
}
