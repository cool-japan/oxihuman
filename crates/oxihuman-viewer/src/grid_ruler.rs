// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! On-screen ruler overlay that shows world-space grid spacing labels.

#![allow(dead_code)]

/// Unit system for the ruler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerUnit {
    Meters,
    Centimeters,
    Millimeters,
    Inches,
    Feet,
}

/// Configuration for the grid ruler.
#[derive(Debug, Clone)]
pub struct GridRulerConfig {
    /// World-space grid spacing in current units.
    pub grid_spacing: f32,
    /// Active unit system.
    pub unit: RulerUnit,
    /// Number of major ticks to generate.
    pub tick_count: usize,
    /// Whether to show sub-ticks between major ticks.
    pub show_sub_ticks: bool,
}

/// A single ruler tick mark.
#[derive(Debug, Clone)]
pub struct RulerTick {
    /// World-space position along the ruler axis.
    pub world_pos: f32,
    /// Screen-space pixel position.
    pub screen_px: f32,
    /// Label string (e.g. "1.0 m").
    pub label: String,
    /// True if this is a major tick.
    pub is_major: bool,
}

/// Runtime state of the ruler.
#[derive(Debug, Clone)]
pub struct GridRulerState {
    pub config: GridRulerConfig,
    /// Generated ticks.
    pub ticks: Vec<RulerTick>,
}

/// Returns the default [`GridRulerConfig`].
#[allow(dead_code)]
pub fn default_grid_ruler_config() -> GridRulerConfig {
    GridRulerConfig {
        grid_spacing: 1.0,
        unit: RulerUnit::Meters,
        tick_count: 10,
        show_sub_ticks: false,
    }
}

/// Creates a new [`GridRulerState`] with no ticks.
#[allow(dead_code)]
pub fn new_grid_ruler(cfg: GridRulerConfig) -> GridRulerState {
    GridRulerState {
        config: cfg,
        ticks: Vec::new(),
    }
}

/// Generates ticks based on `pixels_per_unit` (screen pixels per world unit).
#[allow(dead_code)]
pub fn ruler_generate_ticks(state: &mut GridRulerState, pixels_per_unit: f32) {
    state.ticks.clear();
    let unit_name = ruler_unit_name(state);
    for i in 0..state.config.tick_count {
        let world = i as f32 * state.config.grid_spacing;
        let screen = world * pixels_per_unit;
        let label = format!("{:.2} {}", world, unit_name);
        state.ticks.push(RulerTick {
            world_pos: world,
            screen_px: screen,
            label,
            is_major: true,
        });
    }
}

/// Returns the number of ticks currently stored.
#[allow(dead_code)]
pub fn ruler_tick_count(state: &GridRulerState) -> usize {
    state.ticks.len()
}

/// Returns the label for tick at `index`, or empty string if out of range.
#[allow(dead_code)]
pub fn ruler_tick_label(state: &GridRulerState, index: usize) -> String {
    state
        .ticks
        .get(index)
        .map(|t| t.label.clone())
        .unwrap_or_default()
}

/// Sets the active unit system.
#[allow(dead_code)]
pub fn ruler_set_unit(state: &mut GridRulerState, unit: RulerUnit) {
    state.config.unit = unit;
}

/// Returns the display name of the current unit.
#[allow(dead_code)]
pub fn ruler_unit_name(state: &GridRulerState) -> &'static str {
    match state.config.unit {
        RulerUnit::Meters => "m",
        RulerUnit::Centimeters => "cm",
        RulerUnit::Millimeters => "mm",
        RulerUnit::Inches => "in",
        RulerUnit::Feet => "ft",
    }
}

/// Serialises the ruler state to a JSON string.
#[allow(dead_code)]
pub fn ruler_to_json(state: &GridRulerState) -> String {
    format!(
        "{{\"spacing\":{:.4},\"unit\":\"{}\",\"tick_count\":{}}}",
        state.config.grid_spacing,
        ruler_unit_name(state),
        state.ticks.len(),
    )
}

/// Resets ticks and restores default spacing / unit.
#[allow(dead_code)]
pub fn ruler_reset(state: &mut GridRulerState) {
    state.ticks.clear();
    state.config.grid_spacing = 1.0;
    state.config.unit = RulerUnit::Meters;
}

/// Sets the world-space grid spacing.
#[allow(dead_code)]
pub fn ruler_set_spacing(state: &mut GridRulerState, spacing: f32) {
    state.config.grid_spacing = spacing.max(1e-6);
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_grid_ruler_config();
        assert!((cfg.grid_spacing - 1.0).abs() < 1e-5);
        assert_eq!(cfg.tick_count, 10);
        assert_eq!(cfg.unit, RulerUnit::Meters);
    }

    #[test]
    fn new_ruler_has_no_ticks() {
        let s = new_grid_ruler(default_grid_ruler_config());
        assert_eq!(ruler_tick_count(&s), 0);
    }

    #[test]
    fn generate_ticks_correct_count() {
        let mut s = new_grid_ruler(default_grid_ruler_config());
        ruler_generate_ticks(&mut s, 100.0);
        assert_eq!(ruler_tick_count(&s), 10);
    }

    #[test]
    fn tick_labels_contain_unit() {
        let mut s = new_grid_ruler(default_grid_ruler_config());
        ruler_generate_ticks(&mut s, 100.0);
        let label = ruler_tick_label(&s, 1);
        assert!(label.contains(" m"), "label='{}' should contain ' m'", label);
    }

    #[test]
    fn set_unit_changes_name() {
        let mut s = new_grid_ruler(default_grid_ruler_config());
        ruler_set_unit(&mut s, RulerUnit::Centimeters);
        assert_eq!(ruler_unit_name(&s), "cm");
    }

    #[test]
    fn ruler_reset_clears_ticks() {
        let mut s = new_grid_ruler(default_grid_ruler_config());
        ruler_generate_ticks(&mut s, 50.0);
        ruler_reset(&mut s);
        assert_eq!(ruler_tick_count(&s), 0);
        assert!((s.config.grid_spacing - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_spacing_positive() {
        let mut s = new_grid_ruler(default_grid_ruler_config());
        ruler_set_spacing(&mut s, 0.5);
        assert!((s.config.grid_spacing - 0.5).abs() < 1e-5);
    }

    #[test]
    fn set_spacing_zero_clamped() {
        let mut s = new_grid_ruler(default_grid_ruler_config());
        ruler_set_spacing(&mut s, 0.0);
        assert!(s.config.grid_spacing > 0.0);
    }

    #[test]
    fn to_json_contains_spacing() {
        let s = new_grid_ruler(default_grid_ruler_config());
        let json = ruler_to_json(&s);
        assert!(json.contains("\"spacing\""));
        assert!(json.contains("\"unit\""));
    }

    #[test]
    fn tick_label_out_of_range_is_empty() {
        let s = new_grid_ruler(default_grid_ruler_config());
        assert_eq!(ruler_tick_label(&s, 999), "");
    }
}
