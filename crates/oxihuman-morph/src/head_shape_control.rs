// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Head overall shape morph controls: cranium width, height, depth.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum HeadShapeCategory {
    Dolichocephalic,
    Mesocephalic,
    Brachycephalic,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeadShapeConfig {
    pub width_range: f32,
    pub height_range: f32,
    pub depth_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeadShapeState {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub top_flatness: f32,
    pub category: HeadShapeCategory,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeadShapeMorphWeights {
    pub wide: f32,
    pub narrow: f32,
    pub tall: f32,
    pub short: f32,
    pub deep: f32,
    pub shallow: f32,
}

#[allow(dead_code)]
pub fn default_head_shape_config() -> HeadShapeConfig {
    HeadShapeConfig {
        width_range: 0.6,
        height_range: 0.5,
        depth_range: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_head_shape_state() -> HeadShapeState {
    HeadShapeState {
        width: 0.5,
        height: 0.5,
        depth: 0.5,
        top_flatness: 0.0,
        category: HeadShapeCategory::Mesocephalic,
    }
}

#[allow(dead_code)]
pub fn classify_head_shape(width: f32, depth: f32) -> HeadShapeCategory {
    let ratio = if depth > 0.001 { width / depth } else { 1.0 };
    if ratio > 1.2 {
        HeadShapeCategory::Brachycephalic
    } else if ratio < 0.8 {
        HeadShapeCategory::Dolichocephalic
    } else {
        HeadShapeCategory::Mesocephalic
    }
}

#[allow(dead_code)]
pub fn set_head_width(state: &mut HeadShapeState, value: f32) {
    state.width = value.clamp(0.0, 1.0);
    state.category = classify_head_shape(state.width, state.depth);
}

#[allow(dead_code)]
pub fn set_head_height(state: &mut HeadShapeState, value: f32) {
    state.height = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_head_depth(state: &mut HeadShapeState, value: f32) {
    state.depth = value.clamp(0.0, 1.0);
    state.category = classify_head_shape(state.width, state.depth);
}

#[allow(dead_code)]
pub fn set_top_flatness(state: &mut HeadShapeState, value: f32) {
    state.top_flatness = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_head_shape_weights(state: &HeadShapeState, cfg: &HeadShapeConfig) -> HeadShapeMorphWeights {
    let w = (state.width - 0.5) * 2.0 * cfg.width_range;
    let h = (state.height - 0.5) * 2.0 * cfg.height_range;
    let d = (state.depth - 0.5) * 2.0 * cfg.depth_range;
    HeadShapeMorphWeights {
        wide: w.max(0.0).clamp(0.0, 1.0),
        narrow: (-w).max(0.0).clamp(0.0, 1.0),
        tall: h.max(0.0).clamp(0.0, 1.0),
        short: (-h).max(0.0).clamp(0.0, 1.0),
        deep: d.max(0.0).clamp(0.0, 1.0),
        shallow: (-d).max(0.0).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn head_shape_to_json(state: &HeadShapeState) -> String {
    let cat = match &state.category {
        HeadShapeCategory::Dolichocephalic => "dolichocephalic",
        HeadShapeCategory::Mesocephalic => "mesocephalic",
        HeadShapeCategory::Brachycephalic => "brachycephalic",
    };
    format!(
        r#"{{"width":{},"height":{},"depth":{},"flatness":{},"category":"{}"}}"#,
        state.width, state.height, state.depth, state.top_flatness, cat
    )
}

#[allow(dead_code)]
pub fn blend_head_shape_states(a: &HeadShapeState, b: &HeadShapeState, t: f32) -> HeadShapeState {
    let t = t.clamp(0.0, 1.0);
    let w = a.width + (b.width - a.width) * t;
    let d = a.depth + (b.depth - a.depth) * t;
    HeadShapeState {
        width: w,
        height: a.height + (b.height - a.height) * t,
        depth: d,
        top_flatness: a.top_flatness + (b.top_flatness - a.top_flatness) * t,
        category: classify_head_shape(w, d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_head_shape_config();
        assert!((0.0..=1.0).contains(&c.width_range));
    }

    #[test]
    fn test_new_state() {
        let s = new_head_shape_state();
        assert_eq!(s.category, HeadShapeCategory::Mesocephalic);
    }

    #[test]
    fn test_classify_brachy() {
        assert_eq!(classify_head_shape(0.9, 0.5), HeadShapeCategory::Brachycephalic);
    }

    #[test]
    fn test_classify_dolicho() {
        assert_eq!(classify_head_shape(0.3, 0.8), HeadShapeCategory::Dolichocephalic);
    }

    #[test]
    fn test_set_width() {
        let mut s = new_head_shape_state();
        set_head_width(&mut s, 0.8);
        assert!((s.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        let mut s = new_head_shape_state();
        set_head_height(&mut s, 3.0);
        assert!((s.height - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_weights_neutral() {
        let s = new_head_shape_state();
        let cfg = default_head_shape_config();
        let w = compute_head_shape_weights(&s, &cfg);
        assert!(w.wide.abs() < 1e-6);
        assert!(w.narrow.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_head_shape_state();
        let j = head_shape_to_json(&s);
        assert!(j.contains("mesocephalic"));
    }

    #[test]
    fn test_blend() {
        let a = new_head_shape_state();
        let mut b = new_head_shape_state();
        b.width = 1.0;
        let mid = blend_head_shape_states(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth() {
        let mut s = new_head_shape_state();
        set_head_depth(&mut s, 0.3);
        assert!((s.depth - 0.3).abs() < 1e-6);
    }
}
