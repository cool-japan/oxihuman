// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Viewport shading mode state.

#![allow(dead_code)]

/// Available viewport shading types.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ShadingType {
    Wireframe,
    Solid,
    Material,
    Rendered,
}

/// Viewport shading state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadingState {
    pub mode: ShadingType,
    pub show_specular: bool,
    pub cavity: f32,
    pub shadow_intensity: f32,
    pub matcap_id: u32,
}

/// Returns the default shading state (Solid mode).
#[allow(dead_code)]
pub fn default_shading_state() -> ShadingState {
    ShadingState {
        mode: ShadingType::Solid,
        show_specular: true,
        cavity: 0.0,
        shadow_intensity: 0.5,
        matcap_id: 0,
    }
}

/// Advances to the next shading mode (cycles through all four).
#[allow(dead_code)]
pub fn next_shading_mode(state: &mut ShadingState) {
    state.mode = match state.mode {
        ShadingType::Wireframe => ShadingType::Solid,
        ShadingType::Solid => ShadingType::Material,
        ShadingType::Material => ShadingType::Rendered,
        ShadingType::Rendered => ShadingType::Wireframe,
    };
}

/// Returns the name of the current shading mode.
#[allow(dead_code)]
pub fn shading_mode_name(state: &ShadingState) -> &'static str {
    match state.mode {
        ShadingType::Wireframe => "Wireframe",
        ShadingType::Solid => "Solid",
        ShadingType::Material => "Material",
        ShadingType::Rendered => "Rendered",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_solid() {
        let s = default_shading_state();
        assert_eq!(s.mode, ShadingType::Solid);
    }

    #[test]
    fn test_default_values() {
        let s = default_shading_state();
        assert!(s.show_specular);
        assert!((s.shadow_intensity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_next_from_solid() {
        let mut s = default_shading_state();
        next_shading_mode(&mut s);
        assert_eq!(s.mode, ShadingType::Material);
    }

    #[test]
    fn test_next_from_material() {
        let mut s = default_shading_state();
        s.mode = ShadingType::Material;
        next_shading_mode(&mut s);
        assert_eq!(s.mode, ShadingType::Rendered);
    }

    #[test]
    fn test_next_from_rendered_wraps() {
        let mut s = default_shading_state();
        s.mode = ShadingType::Rendered;
        next_shading_mode(&mut s);
        assert_eq!(s.mode, ShadingType::Wireframe);
    }

    #[test]
    fn test_next_full_cycle() {
        let mut s = default_shading_state();
        s.mode = ShadingType::Wireframe;
        for _ in 0..4 {
            next_shading_mode(&mut s);
        }
        assert_eq!(s.mode, ShadingType::Wireframe);
    }

    #[test]
    fn test_shading_mode_name_solid() {
        let s = default_shading_state();
        assert_eq!(shading_mode_name(&s), "Solid");
    }

    #[test]
    fn test_shading_mode_name_wireframe() {
        let mut s = default_shading_state();
        s.mode = ShadingType::Wireframe;
        assert_eq!(shading_mode_name(&s), "Wireframe");
    }

    #[test]
    fn test_shading_mode_name_rendered() {
        let mut s = default_shading_state();
        s.mode = ShadingType::Rendered;
        assert_eq!(shading_mode_name(&s), "Rendered");
    }
}
