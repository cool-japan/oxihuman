#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A sculpt brush configuration.
#[derive(Debug, Clone)]
pub struct SculptBrush {
    pub radius: f32,
    pub strength: f32,
    pub hardness: f32,
    pub brush_type: u8,
    pub smooth: bool,
}

/// Sculpt mode state.
#[derive(Debug, Clone)]
pub struct SculptModeState {
    pub active_brush: SculptBrush,
    pub symmetry: u8,
    pub multires_level: u32,
    pub show_mask: bool,
}

#[allow(dead_code)]
pub fn new_sculpt_mode_state() -> SculptModeState {
    SculptModeState {
        active_brush: SculptBrush {
            radius: 0.1,
            strength: 0.5,
            hardness: 0.5,
            brush_type: 0,
            smooth: false,
        },
        symmetry: 0,
        multires_level: 0,
        show_mask: false,
    }
}

#[allow(dead_code)]
pub fn set_brush_radius(state: &mut SculptModeState, r: f32) {
    state.active_brush.radius = r.max(0.0);
}

#[allow(dead_code)]
pub fn set_brush_strength(state: &mut SculptModeState, s: f32) {
    state.active_brush.strength = s.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sculpt_brush_name(brush_type: u8) -> &'static str {
    match brush_type {
        0 => "Draw",
        1 => "Smooth",
        2 => "Flatten",
        3 => "Inflate",
        4 => "Grab",
        _ => "Custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_defaults() {
        let s = new_sculpt_mode_state();
        assert!((s.active_brush.radius - 0.1).abs() < 1e-6);
        assert!((s.active_brush.strength - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_brush_radius() {
        let mut s = new_sculpt_mode_state();
        set_brush_radius(&mut s, 0.3);
        assert!((s.active_brush.radius - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_brush_radius_clamps_negative() {
        let mut s = new_sculpt_mode_state();
        set_brush_radius(&mut s, -1.0);
        assert!((s.active_brush.radius).abs() < 1e-6);
    }

    #[test]
    fn test_set_brush_strength() {
        let mut s = new_sculpt_mode_state();
        set_brush_strength(&mut s, 0.9);
        assert!((s.active_brush.strength - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_brush_strength_clamps() {
        let mut s = new_sculpt_mode_state();
        set_brush_strength(&mut s, 1.5);
        assert!((s.active_brush.strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_brush_name_draw() {
        assert_eq!(sculpt_brush_name(0), "Draw");
    }

    #[test]
    fn test_brush_name_smooth() {
        assert_eq!(sculpt_brush_name(1), "Smooth");
    }

    #[test]
    fn test_brush_name_custom() {
        assert_eq!(sculpt_brush_name(99), "Custom");
    }

    #[test]
    fn test_show_mask_default_false() {
        let s = new_sculpt_mode_state();
        assert!(!s.show_mask);
    }
}
