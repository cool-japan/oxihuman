// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Checkerboard debug texture pattern.

/// Checkerboard configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheckerboardConfig {
    pub cell_size: u32,
    pub color_a: [f32; 4],
    pub color_b: [f32; 4],
    pub enabled: bool,
}

impl Default for CheckerboardConfig {
    fn default() -> Self {
        CheckerboardConfig {
            cell_size: 8,
            color_a: [0.8, 0.8, 0.8, 1.0],
            color_b: [0.2, 0.2, 0.2, 1.0],
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_checkerboard_config() -> CheckerboardConfig {
    CheckerboardConfig::default()
}

#[allow(dead_code)]
pub fn cb_set_cell_size(cfg: &mut CheckerboardConfig, size: u32) {
    cfg.cell_size = size.max(1);
}

#[allow(dead_code)]
pub fn cb_set_color_a(cfg: &mut CheckerboardConfig, rgba: [f32; 4]) {
    cfg.color_a = rgba;
}

#[allow(dead_code)]
pub fn cb_set_color_b(cfg: &mut CheckerboardConfig, rgba: [f32; 4]) {
    cfg.color_b = rgba;
}

#[allow(dead_code)]
pub fn cb_enable(cfg: &mut CheckerboardConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn cb_disable(cfg: &mut CheckerboardConfig) {
    cfg.enabled = false;
}

/// Sample the checkerboard color at pixel (x, y).
#[allow(dead_code)]
pub fn cb_sample(cfg: &CheckerboardConfig, x: u32, y: u32) -> [f32; 4] {
    let cx = (x / cfg.cell_size) % 2;
    let cy = (y / cfg.cell_size) % 2;
    if (cx + cy).is_multiple_of(2) {
        cfg.color_a
    } else {
        cfg.color_b
    }
}

#[allow(dead_code)]
pub fn cb_is_color_a(cfg: &CheckerboardConfig, x: u32, y: u32) -> bool {
    let cx = (x / cfg.cell_size) % 2;
    let cy = (y / cfg.cell_size) % 2;
    (cx + cy).is_multiple_of(2)
}

/// Generate a flat row of pixel colors.
#[allow(dead_code)]
pub fn cb_generate_row(cfg: &CheckerboardConfig, y: u32, width: u32) -> Vec<[f32; 4]> {
    (0..width).map(|x| cb_sample(cfg, x, y)).collect()
}

#[allow(dead_code)]
pub fn cb_to_json(cfg: &CheckerboardConfig) -> String {
    format!(
        r#"{{"cell_size":{},"enabled":{}}}"#,
        cfg.cell_size, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_checkerboard_config().enabled);
    }

    #[test]
    fn default_cell_size() {
        assert_eq!(default_checkerboard_config().cell_size, 8);
    }

    #[test]
    fn cb_sample_origin_is_color_a() {
        let cfg = default_checkerboard_config();
        let c = cb_sample(&cfg, 0, 0);
        assert!((c[0] - cfg.color_a[0]).abs() < 1e-6);
    }

    #[test]
    fn cb_sample_alternates() {
        let cfg = default_checkerboard_config();
        let a = cb_is_color_a(&cfg, 0, 0);
        let b = cb_is_color_a(&cfg, 8, 0);
        assert_ne!(a, b);
    }

    #[test]
    fn cell_size_min_one() {
        let mut cfg = default_checkerboard_config();
        cb_set_cell_size(&mut cfg, 0);
        assert_eq!(cfg.cell_size, 1);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_checkerboard_config();
        cb_enable(&mut cfg);
        assert!(cfg.enabled);
        cb_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn generate_row_length() {
        let cfg = default_checkerboard_config();
        let row = cb_generate_row(&cfg, 0, 16);
        assert_eq!(row.len(), 16);
    }

    #[test]
    fn generate_row_alternates_cells() {
        let cfg = default_checkerboard_config();
        let row = cb_generate_row(&cfg, 0, 16);
        let first = row[0];
        let ninth = row[8];
        assert!((first[0] - ninth[0]).abs() > 0.01);
    }

    #[test]
    fn to_json_has_cell_size() {
        assert!(cb_to_json(&default_checkerboard_config()).contains("cell_size"));
    }
}
