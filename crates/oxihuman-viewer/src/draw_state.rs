// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Draw state: tracks the current GPU pipeline state to minimize redundant state changes.

/// Cull mode for rasterization.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CullFace {
    None,
    Front,
    Back,
}

/// Depth test mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DepthFunc {
    Always,
    Less,
    LessEqual,
    Greater,
    Equal,
}

/// Current draw state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawState {
    pub cull_face: CullFace,
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_func: DepthFunc,
    pub blend_enabled: bool,
    pub scissor_enabled: bool,
    pub state_changes: u32,
}

#[allow(dead_code)]
pub fn default_draw_state() -> DrawState {
    DrawState {
        cull_face: CullFace::Back,
        depth_test: true,
        depth_write: true,
        depth_func: DepthFunc::Less,
        blend_enabled: false,
        scissor_enabled: false,
        state_changes: 0,
    }
}

#[allow(dead_code)]
pub fn set_cull_face(state: &mut DrawState, cull: CullFace) {
    if state.cull_face != cull {
        state.cull_face = cull;
        state.state_changes += 1;
    }
}

#[allow(dead_code)]
pub fn set_depth_test(state: &mut DrawState, enabled: bool) {
    if state.depth_test != enabled {
        state.depth_test = enabled;
        state.state_changes += 1;
    }
}

#[allow(dead_code)]
pub fn set_depth_write(state: &mut DrawState, enabled: bool) {
    if state.depth_write != enabled {
        state.depth_write = enabled;
        state.state_changes += 1;
    }
}

#[allow(dead_code)]
pub fn set_depth_func(state: &mut DrawState, func: DepthFunc) {
    if state.depth_func != func {
        state.depth_func = func;
        state.state_changes += 1;
    }
}

#[allow(dead_code)]
pub fn set_blend_enabled(state: &mut DrawState, enabled: bool) {
    if state.blend_enabled != enabled {
        state.blend_enabled = enabled;
        state.state_changes += 1;
    }
}

#[allow(dead_code)]
pub fn reset_draw_state(state: &mut DrawState) {
    *state = default_draw_state();
}

#[allow(dead_code)]
pub fn draw_state_to_json(state: &DrawState) -> String {
    let cull = match state.cull_face {
        CullFace::None => "none",
        CullFace::Front => "front",
        CullFace::Back => "back",
    };
    format!(
        r#"{{"cull":"{}","depth_test":{},"blend":{},"state_changes":{}}}"#,
        cull, state.depth_test, state.blend_enabled, state.state_changes
    )
}

#[allow(dead_code)]
pub fn state_change_count(state: &DrawState) -> u32 {
    state.state_changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_draw_state() {
        let s = default_draw_state();
        assert_eq!(s.cull_face, CullFace::Back);
        assert!(s.depth_test);
        assert!(s.depth_write);
    }

    #[test]
    fn test_set_cull_face_changes_count() {
        let mut s = default_draw_state();
        set_cull_face(&mut s, CullFace::None);
        assert_eq!(s.state_changes, 1);
    }

    #[test]
    fn test_set_cull_face_no_change() {
        let mut s = default_draw_state();
        set_cull_face(&mut s, CullFace::Back);
        assert_eq!(s.state_changes, 0);
    }

    #[test]
    fn test_set_depth_test() {
        let mut s = default_draw_state();
        set_depth_test(&mut s, false);
        assert!(!s.depth_test);
        assert_eq!(s.state_changes, 1);
    }

    #[test]
    fn test_set_blend() {
        let mut s = default_draw_state();
        set_blend_enabled(&mut s, true);
        assert!(s.blend_enabled);
    }

    #[test]
    fn test_reset() {
        let mut s = default_draw_state();
        set_cull_face(&mut s, CullFace::Front);
        set_blend_enabled(&mut s, true);
        reset_draw_state(&mut s);
        assert_eq!(s.cull_face, CullFace::Back);
        assert!(!s.blend_enabled);
    }

    #[test]
    fn test_draw_state_to_json() {
        let s = default_draw_state();
        let j = draw_state_to_json(&s);
        assert!(j.contains("\"cull\":\"back\""));
    }

    #[test]
    fn test_state_change_count() {
        let mut s = default_draw_state();
        set_cull_face(&mut s, CullFace::None);
        set_depth_test(&mut s, false);
        assert_eq!(state_change_count(&s), 2);
    }

    #[test]
    fn test_set_depth_func() {
        let mut s = default_draw_state();
        set_depth_func(&mut s, DepthFunc::LessEqual);
        assert_eq!(s.depth_func, DepthFunc::LessEqual);
    }

    #[test]
    fn test_set_depth_write() {
        let mut s = default_draw_state();
        set_depth_write(&mut s, false);
        assert!(!s.depth_write);
    }
}
