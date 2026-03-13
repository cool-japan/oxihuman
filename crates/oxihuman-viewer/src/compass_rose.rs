// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compass rose orientation indicator (N/S/E/W).

use std::f32::consts::PI;

/// Compass rose configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompassRoseConfig {
    /// Current heading in degrees (0 = North, 90 = East).
    pub heading_deg: f32,
    /// Rose size in screen pixels.
    pub size_px: f32,
    /// Screen position [x, y] in normalized coords (0..=1).
    pub screen_pos: [f32; 2],
    /// Color for cardinal directions [R, G, B, A].
    pub cardinal_color: [f32; 4],
    /// Color for ordinal directions [R, G, B, A].
    pub ordinal_color: [f32; 4],
    /// Show ordinal (NE, NW, SE, SW) indicators.
    pub show_ordinals: bool,
}

impl Default for CompassRoseConfig {
    fn default() -> Self {
        Self {
            heading_deg: 0.0,
            size_px: 64.0,
            screen_pos: [0.92, 0.08],
            cardinal_color: [1.0, 0.9, 0.7, 1.0],
            ordinal_color: [0.7, 0.7, 0.7, 0.8],
            show_ordinals: true,
        }
    }
}

/// A compass point entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompassPoint {
    pub label: &'static str,
    pub angle_deg: f32,
    pub direction: [f32; 2],
    pub is_cardinal: bool,
}

/// Create default config.
#[allow(dead_code)]
pub fn new_compass_rose_config() -> CompassRoseConfig {
    CompassRoseConfig::default()
}

/// Generate compass points relative to current heading.
#[allow(dead_code)]
pub fn generate_compass_points(cfg: &CompassRoseConfig) -> Vec<CompassPoint> {
    let points_def: &[(&str, f32, bool)] = &[
        ("N", 0.0, true),
        ("NE", 45.0, false),
        ("E", 90.0, true),
        ("SE", 135.0, false),
        ("S", 180.0, true),
        ("SW", 225.0, false),
        ("W", 270.0, true),
        ("NW", 315.0, false),
    ];

    points_def
        .iter()
        .filter(|&&(_, _, is_card)| is_card || cfg.show_ordinals)
        .map(|&(label, angle, is_card)| {
            let rel_deg = angle - cfg.heading_deg;
            let rad = rel_deg * PI / 180.0;
            CompassPoint {
                label,
                angle_deg: rel_deg,
                direction: [rad.sin(), rad.cos()],
                is_cardinal: is_card,
            }
        })
        .collect()
}

/// Set heading.
#[allow(dead_code)]
pub fn cr_set_heading(cfg: &mut CompassRoseConfig, heading_deg: f32) {
    cfg.heading_deg = heading_deg.rem_euclid(360.0);
}

/// Set size.
#[allow(dead_code)]
pub fn cr_set_size(cfg: &mut CompassRoseConfig, size_px: f32) {
    cfg.size_px = size_px.max(16.0);
}

/// Get heading as radians.
#[allow(dead_code)]
pub fn cr_heading_rad(cfg: &CompassRoseConfig) -> f32 {
    cfg.heading_deg * PI / 180.0
}

/// Cardinal point count.
#[allow(dead_code)]
pub fn cr_cardinal_count() -> usize {
    4
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn compass_rose_to_json(cfg: &CompassRoseConfig) -> String {
    format!(
        r#"{{"heading_deg":{:.2},"size_px":{:.1},"show_ordinals":{}}}"#,
        cfg.heading_deg, cfg.size_px, cfg.show_ordinals
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default() {
        let c = CompassRoseConfig::default();
        assert!(c.heading_deg.abs() < 1e-6);
    }

    #[test]
    fn test_generate_all() {
        let c = CompassRoseConfig::default();
        let pts = generate_compass_points(&c);
        assert_eq!(pts.len(), 8);
    }

    #[test]
    fn test_generate_cardinals_only() {
        let c = CompassRoseConfig {
            show_ordinals: false,
            ..Default::default()
        };
        let pts = generate_compass_points(&c);
        assert_eq!(pts.len(), 4);
        assert!(pts.iter().all(|p| p.is_cardinal));
    }

    #[test]
    fn test_set_heading_wraps() {
        let mut c = CompassRoseConfig::default();
        cr_set_heading(&mut c, 370.0);
        assert!((c.heading_deg - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_size_min() {
        let mut c = CompassRoseConfig::default();
        cr_set_size(&mut c, 4.0);
        assert!(c.size_px >= 16.0);
    }

    #[test]
    fn test_heading_rad() {
        let c = CompassRoseConfig {
            heading_deg: 90.0,
            ..Default::default()
        };
        let r = cr_heading_rad(&c);
        assert!((r - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_cardinal_count() {
        assert_eq!(cr_cardinal_count(), 4);
    }

    #[test]
    fn test_north_direction_at_zero() {
        let c = CompassRoseConfig::default();
        let pts = generate_compass_points(&c);
        let north = pts.iter().find(|p| p.label == "N").expect("should succeed");
        assert!(north.direction[0].abs() < 1e-5);
        assert!((north.direction[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pi_used() {
        let _pi = PI;
        let c = CompassRoseConfig {
            heading_deg: 180.0,
            ..Default::default()
        };
        let r = cr_heading_rad(&c);
        assert!((r - PI).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = compass_rose_to_json(&CompassRoseConfig::default());
        assert!(j.contains("heading_deg"));
        assert!(j.contains("show_ordinals"));
    }
}
