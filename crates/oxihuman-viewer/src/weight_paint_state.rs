#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Weight paint mode state.
#[derive(Debug, Clone)]
pub struct WeightPaintState {
    pub active_group: String,
    pub weight: f32,
    pub radius: f32,
    pub strength: f32,
    pub auto_normalize: bool,
    pub show_zero_weights: bool,
}

#[allow(dead_code)]
pub fn new_weight_paint_state() -> WeightPaintState {
    WeightPaintState {
        active_group: "Group".to_string(),
        weight: 1.0,
        radius: 0.1,
        strength: 0.5,
        auto_normalize: true,
        show_zero_weights: false,
    }
}

#[allow(dead_code)]
pub fn set_active_group(state: &mut WeightPaintState, name: &str) {
    state.active_group = name.to_string();
}

#[allow(dead_code)]
pub fn set_weight(state: &mut WeightPaintState, w: f32) {
    state.weight = w.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_radius(state: &mut WeightPaintState, r: f32) {
    state.radius = r.max(0.0);
}

#[allow(dead_code)]
pub fn toggle_normalize(state: &mut WeightPaintState) {
    state.auto_normalize = !state.auto_normalize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let s = new_weight_paint_state();
        assert_eq!(s.active_group, "Group");
        assert!((s.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_active_group() {
        let mut s = new_weight_paint_state();
        set_active_group(&mut s, "Arm");
        assert_eq!(s.active_group, "Arm");
    }

    #[test]
    fn test_set_weight() {
        let mut s = new_weight_paint_state();
        set_weight(&mut s, 0.7);
        assert!((s.weight - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_clamps_high() {
        let mut s = new_weight_paint_state();
        set_weight(&mut s, 2.0);
        assert!((s.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_clamps_low() {
        let mut s = new_weight_paint_state();
        set_weight(&mut s, -0.5);
        assert!((s.weight).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius() {
        let mut s = new_weight_paint_state();
        set_radius(&mut s, 0.25);
        assert!((s.radius - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius_clamps_negative() {
        let mut s = new_weight_paint_state();
        set_radius(&mut s, -1.0);
        assert!((s.radius).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_normalize() {
        let mut s = new_weight_paint_state();
        assert!(s.auto_normalize);
        toggle_normalize(&mut s);
        assert!(!s.auto_normalize);
        toggle_normalize(&mut s);
        assert!(s.auto_normalize);
    }

    #[test]
    fn test_show_zero_weights_default_false() {
        let s = new_weight_paint_state();
        assert!(!s.show_zero_weights);
    }
}
