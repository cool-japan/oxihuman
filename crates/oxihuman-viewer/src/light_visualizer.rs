// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Light cone/area visualization helper.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Light visualization type.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum LightVisType {
    Point,
    Spot,
    Area,
    Sun,
}

/// Configuration for light visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightVisConfig {
    pub vis_type: LightVisType,
    pub color: [f32; 4],
    pub size: f32,
    pub show_cone: bool,
}

/// Runtime state for light visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightVisState {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub config: LightVisConfig,
}

#[allow(dead_code)]
pub fn default_light_vis_config() -> LightVisConfig {
    LightVisConfig {
        vis_type: LightVisType::Point,
        color: [1.0, 1.0, 0.8, 1.0],
        size: 1.0,
        show_cone: true,
    }
}

#[allow(dead_code)]
pub fn new_light_vis_state() -> LightVisState {
    LightVisState {
        position: [0.0, 0.0, 0.0],
        direction: [0.0, -1.0, 0.0],
        config: default_light_vis_config(),
    }
}

#[allow(dead_code)]
pub fn lv_set_position(state: &mut LightVisState, pos: [f32; 3]) {
    state.position = pos;
}

#[allow(dead_code)]
pub fn lv_set_direction(state: &mut LightVisState, dir: [f32; 3]) {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len > 1e-9 {
        state.direction = [dir[0] / len, dir[1] / len, dir[2] / len];
    }
}

#[allow(dead_code)]
pub fn lv_cone_angle(state: &LightVisState) -> f32 {
    match state.config.vis_type {
        LightVisType::Spot => PI / 6.0,
        _ => 0.0,
    }
}

#[allow(dead_code)]
pub fn lv_to_json(state: &LightVisState) -> String {
    let p = &state.position;
    let d = &state.direction;
    let c = &state.config.color;
    format!(
        r#"{{"type":"{}","position":[{:.4},{:.4},{:.4}],"direction":[{:.4},{:.4},{:.4}],"color":[{:.4},{:.4},{:.4},{:.4}],"size":{:.4},"show_cone":{}}}"#,
        lv_type_name(state),
        p[0], p[1], p[2],
        d[0], d[1], d[2],
        c[0], c[1], c[2], c[3],
        state.config.size,
        state.config.show_cone
    )
}

#[allow(dead_code)]
pub fn lv_type_name(state: &LightVisState) -> &'static str {
    match state.config.vis_type {
        LightVisType::Point => "point",
        LightVisType::Spot => "spot",
        LightVisType::Area => "area",
        LightVisType::Sun => "sun",
    }
}

#[allow(dead_code)]
pub fn lv_reset(state: &mut LightVisState) {
    *state = new_light_vis_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_light_vis_config();
        assert_eq!(cfg.vis_type, LightVisType::Point);
        assert!((cfg.size - 1.0).abs() < 1e-6);
        assert!(cfg.show_cone);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_light_vis_state();
        assert_eq!(s.position, [0.0, 0.0, 0.0]);
        assert!((s.direction[1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_position() {
        let mut s = new_light_vis_state();
        lv_set_position(&mut s, [1.0, 2.0, 3.0]);
        assert!((s.position[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_direction_normalizes() {
        let mut s = new_light_vis_state();
        lv_set_direction(&mut s, [3.0, 0.0, 0.0]);
        assert!((s.direction[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cone_angle_spot() {
        let mut s = new_light_vis_state();
        s.config.vis_type = LightVisType::Spot;
        assert!(lv_cone_angle(&s) > 0.0);
    }

    #[test]
    fn test_cone_angle_point_zero() {
        let s = new_light_vis_state();
        assert!((lv_cone_angle(&s)).abs() < 1e-6);
    }

    #[test]
    fn test_type_name() {
        let s = new_light_vis_state();
        assert_eq!(lv_type_name(&s), "point");
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_light_vis_state();
        let j = lv_to_json(&s);
        assert!(j.contains("type"));
        assert!(j.contains("position"));
        assert!(j.contains("direction"));
    }
}
