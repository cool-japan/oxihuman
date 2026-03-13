// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Clapperboard animation view for syncing audio and video.

/// Clapperboard state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClapperState {
    Open,
    Clappping,
    Closed,
}

/// Clapper view configuration.
#[derive(Debug, Clone)]
pub struct ClapperView {
    pub state: ClapperState,
    pub clap_angle: f32,
    pub scene: String,
    pub take: u32,
    pub animation_speed: f32,
    pub enabled: bool,
}

impl ClapperView {
    pub fn new() -> Self {
        Self {
            state: ClapperState::Open,
            clap_angle: 45.0,
            scene: String::from("1"),
            take: 1,
            animation_speed: 1.0,
            enabled: false,
        }
    }
}

impl Default for ClapperView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new clapper view.
pub fn new_clapper_view() -> ClapperView {
    ClapperView::new()
}

/// Set clapper open/close state.
pub fn clv_set_state(view: &mut ClapperView, state: ClapperState) {
    view.state = state;
    view.clap_angle = match state {
        ClapperState::Open => 45.0,
        ClapperState::Clappping => 10.0,
        ClapperState::Closed => 0.0,
    };
}

/// Set scene label.
pub fn clv_set_scene(view: &mut ClapperView, scene: &str) {
    view.scene = scene.to_string();
}

/// Set take number.
pub fn clv_set_take(view: &mut ClapperView, take: u32) {
    view.take = take.max(1);
}

/// Set animation speed multiplier.
pub fn clv_set_animation_speed(view: &mut ClapperView, speed: f32) {
    view.animation_speed = speed.clamp(0.1, 10.0);
}

/// Simulate one animation tick; returns true when clap is complete.
pub fn clv_animate_tick(view: &mut ClapperView, delta: f32) -> bool {
    if view.state == ClapperState::Closed {
        return true;
    }
    view.clap_angle = (view.clap_angle - delta * view.animation_speed * 60.0).max(0.0);
    if view.clap_angle <= 0.0 {
        view.state = ClapperState::Closed;
        true
    } else {
        false
    }
}

/// Serialize to JSON-like string.
pub fn clapper_view_to_json(view: &ClapperView) -> String {
    let state_str = match view.state {
        ClapperState::Open => "open",
        ClapperState::Clappping => "clapping",
        ClapperState::Closed => "closed",
    };
    format!(
        r#"{{"state":"{state_str}","scene":"{}","take":{},"clap_angle":{:.2}}}"#,
        view.scene, view.take, view.clap_angle
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_clapper_view();
        assert_eq!(v.state, ClapperState::Open);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_state_closed() {
        let mut v = new_clapper_view();
        clv_set_state(&mut v, ClapperState::Closed);
        assert_eq!(v.clap_angle, 0.0);
    }

    #[test]
    fn test_set_state_open() {
        let mut v = new_clapper_view();
        clv_set_state(&mut v, ClapperState::Open);
        assert!((v.clap_angle - 45.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_scene() {
        let mut v = new_clapper_view();
        clv_set_scene(&mut v, "5B");
        assert_eq!(v.scene, "5B");
    }

    #[test]
    fn test_set_take() {
        let mut v = new_clapper_view();
        clv_set_take(&mut v, 3);
        assert_eq!(v.take, 3);
    }

    #[test]
    fn test_animation_speed_clamp() {
        let mut v = new_clapper_view();
        clv_set_animation_speed(&mut v, 0.0);
        assert!((v.animation_speed - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_animate_tick_completes() {
        let mut v = new_clapper_view();
        let done = clv_animate_tick(&mut v, 1.0); /* big delta closes it */
        assert!(done);
        assert_eq!(v.state, ClapperState::Closed);
    }

    #[test]
    fn test_json() {
        let v = new_clapper_view();
        let s = clapper_view_to_json(&v);
        assert!(s.contains("open"));
    }

    #[test]
    fn test_clone() {
        let v = new_clapper_view();
        let v2 = v.clone();
        assert_eq!(v2.state, v.state);
    }
}
