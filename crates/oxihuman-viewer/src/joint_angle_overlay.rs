// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Joint angle annotation overlay data.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Config for joint angle overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointAngleOverlayConfig {
    /// Show angles in degrees (true) or radians (false).
    pub show_degrees: bool,
    /// Font size for labels.
    pub font_size: f32,
    /// Arc radius for angle visualization.
    pub arc_radius: f32,
    /// Color of the arc.
    pub arc_color: [f32; 4],
    /// Only show joints with deviation above this threshold (radians).
    pub min_display_angle: f32,
}

#[allow(dead_code)]
impl Default for JointAngleOverlayConfig {
    fn default() -> Self {
        Self {
            show_degrees: true,
            font_size: 12.0,
            arc_radius: 0.05,
            arc_color: [1.0, 0.8, 0.0, 1.0],
            min_display_angle: 0.0,
        }
    }
}

/// Joint angle annotation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointAngleAnnotation {
    pub joint_name: String,
    pub angle_rad: f32,
    pub position: [f32; 3],
}

/// Create default config.
#[allow(dead_code)]
pub fn new_joint_angle_overlay_config() -> JointAngleOverlayConfig {
    JointAngleOverlayConfig::default()
}

/// Convert radians to degrees.
#[allow(dead_code)]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

/// Convert degrees to radians.
#[allow(dead_code)]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

/// Format angle for display.
#[allow(dead_code)]
pub fn format_angle(angle_rad: f32, show_degrees: bool) -> String {
    if show_degrees {
        format!("{:.1}°", rad_to_deg(angle_rad))
    } else {
        format!("{:.3}rad", angle_rad)
    }
}

/// Filter annotations by minimum display angle.
#[allow(dead_code)]
pub fn filter_annotations<'a>(
    annotations: &'a [JointAngleAnnotation],
    cfg: &JointAngleOverlayConfig,
) -> Vec<&'a JointAngleAnnotation> {
    annotations
        .iter()
        .filter(|a| a.angle_rad.abs() >= cfg.min_display_angle)
        .collect()
}

/// Set font size.
#[allow(dead_code)]
pub fn jao_set_font_size(cfg: &mut JointAngleOverlayConfig, size: f32) {
    cfg.font_size = size.max(1.0);
}

/// Set arc radius.
#[allow(dead_code)]
pub fn jao_set_arc_radius(cfg: &mut JointAngleOverlayConfig, r: f32) {
    cfg.arc_radius = r.max(0.0);
}

/// Toggle degree/radian display.
#[allow(dead_code)]
pub fn jao_toggle_units(cfg: &mut JointAngleOverlayConfig) {
    cfg.show_degrees = !cfg.show_degrees;
}

/// Generate arc points for a given angle.
#[allow(dead_code)]
pub fn arc_points(center: [f32; 3], radius: f32, angle_rad: f32, steps: u32) -> Vec<[f32; 3]> {
    if steps == 0 {
        return Vec::new();
    }
    (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32 * angle_rad;
            [
                center[0] + radius * t.cos(),
                center[1] + radius * t.sin(),
                center[2],
            ]
        })
        .collect()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn joint_angle_overlay_to_json(cfg: &JointAngleOverlayConfig) -> String {
    format!(
        r#"{{"show_degrees":{},"font_size":{:.1},"arc_radius":{:.4}}}"#,
        cfg.show_degrees, cfg.font_size, cfg.arc_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = JointAngleOverlayConfig::default();
        assert!(c.show_degrees);
        assert!((c.font_size - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_rad_to_deg() {
        let d = rad_to_deg(PI);
        assert!((d - 180.0).abs() < 1e-3);
    }

    #[test]
    fn test_deg_to_rad() {
        let r = deg_to_rad(180.0);
        assert!((r - PI).abs() < 1e-5);
    }

    #[test]
    fn test_format_angle_degrees() {
        let s = format_angle(PI / 2.0, true);
        assert!(s.contains("90"));
    }

    #[test]
    fn test_format_angle_radians() {
        let s = format_angle(1.0, false);
        assert!(s.contains("rad"));
    }

    #[test]
    fn test_filter_annotations() {
        let anns = vec![
            JointAngleAnnotation {
                joint_name: "a".to_string(),
                angle_rad: 0.1,
                position: [0.0, 0.0, 0.0],
            },
            JointAngleAnnotation {
                joint_name: "b".to_string(),
                angle_rad: 0.5,
                position: [0.0, 0.0, 0.0],
            },
        ];
        let cfg = JointAngleOverlayConfig {
            min_display_angle: 0.3,
            ..Default::default()
        };
        let filtered = filter_annotations(&anns, &cfg);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_arc_points_count() {
        let pts = arc_points([0.0, 0.0, 0.0], 1.0, PI / 2.0, 8);
        assert_eq!(pts.len(), 9);
    }

    #[test]
    fn test_toggle_units() {
        let mut c = JointAngleOverlayConfig::default();
        jao_toggle_units(&mut c);
        assert!(!c.show_degrees);
    }

    #[test]
    fn test_to_json() {
        let j = joint_angle_overlay_to_json(&JointAngleOverlayConfig::default());
        assert!(j.contains("show_degrees"));
    }
}
