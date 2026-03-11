// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Weight heat map on mesh view stub.

/// Color ramp preset for heat map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatMapRamp {
    BlueRed,
    Inferno,
    Viridis,
    Grayscale,
}

/// Weight heat map view configuration.
#[derive(Debug, Clone)]
pub struct WeightHeatMapView {
    pub ramp: HeatMapRamp,
    pub bone_index: usize,
    pub threshold_low: f32,
    pub threshold_high: f32,
    pub enabled: bool,
}

impl WeightHeatMapView {
    pub fn new() -> Self {
        WeightHeatMapView {
            ramp: HeatMapRamp::BlueRed,
            bone_index: 0,
            threshold_low: 0.0,
            threshold_high: 1.0,
            enabled: true,
        }
    }
}

impl Default for WeightHeatMapView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new weight heat map view.
pub fn new_weight_heat_map_view() -> WeightHeatMapView {
    WeightHeatMapView::new()
}

/// Set color ramp preset.
pub fn whm_set_ramp(view: &mut WeightHeatMapView, ramp: HeatMapRamp) {
    view.ramp = ramp;
}

/// Set the bone index whose weights are displayed.
pub fn whm_set_bone_index(view: &mut WeightHeatMapView, bone_index: usize) {
    view.bone_index = bone_index;
}

/// Set display threshold range.
pub fn whm_set_thresholds(view: &mut WeightHeatMapView, low: f32, high: f32) {
    view.threshold_low = low.clamp(0.0, 1.0);
    view.threshold_high = high.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn whm_set_enabled(view: &mut WeightHeatMapView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn whm_to_json(view: &WeightHeatMapView) -> String {
    let ramp = match view.ramp {
        HeatMapRamp::BlueRed => "blue_red",
        HeatMapRamp::Inferno => "inferno",
        HeatMapRamp::Viridis => "viridis",
        HeatMapRamp::Grayscale => "grayscale",
    };
    format!(
        r#"{{"ramp":"{}","bone_index":{},"threshold_low":{},"threshold_high":{},"enabled":{}}}"#,
        ramp, view.bone_index, view.threshold_low, view.threshold_high, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ramp() {
        let v = new_weight_heat_map_view();
        assert_eq!(
            v.ramp,
            HeatMapRamp::BlueRed /* default ramp must be BlueRed */
        );
    }

    #[test]
    fn test_set_ramp() {
        let mut v = new_weight_heat_map_view();
        whm_set_ramp(&mut v, HeatMapRamp::Viridis);
        assert_eq!(v.ramp, HeatMapRamp::Viridis /* ramp must be set */);
    }

    #[test]
    fn test_set_bone_index() {
        let mut v = new_weight_heat_map_view();
        whm_set_bone_index(&mut v, 5);
        assert_eq!(v.bone_index, 5 /* bone_index must be set */);
    }

    #[test]
    fn test_threshold_clamp() {
        let mut v = new_weight_heat_map_view();
        whm_set_thresholds(&mut v, -0.5, 1.5);
        assert!(v.threshold_low.abs() < 1e-6 /* low clamped to 0.0 */);
        assert!((v.threshold_high - 1.0).abs() < 1e-6 /* high clamped to 1.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_weight_heat_map_view();
        whm_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_ramp() {
        let v = new_weight_heat_map_view();
        let j = whm_to_json(&v);
        assert!(j.contains("\"ramp\"") /* JSON must have ramp */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_weight_heat_map_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_bone_index() {
        let v = new_weight_heat_map_view();
        assert_eq!(v.bone_index, 0 /* default bone_index must be 0 */);
    }

    #[test]
    fn test_default_thresholds() {
        let v = new_weight_heat_map_view();
        assert!(v.threshold_low.abs() < 1e-6 /* default low must be 0.0 */);
        assert!((v.threshold_high - 1.0).abs() < 1e-6 /* default high must be 1.0 */);
    }
}
