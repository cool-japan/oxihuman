// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Carpal bone arrangement morph — wrist thickness and intercarpal spacing.

/// Carpals morph configuration.
#[derive(Debug, Clone)]
pub struct CarpalsMorph {
    pub width: f32,
    pub height: f32,
    pub spacing: f32,
    pub tunnel_depth: f32,
}

impl CarpalsMorph {
    pub fn new() -> Self {
        Self {
            width: 0.5,
            height: 0.5,
            spacing: 0.5,
            tunnel_depth: 0.5,
        }
    }
}

impl Default for CarpalsMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new carpals morph.
pub fn new_carpals_morph() -> CarpalsMorph {
    CarpalsMorph::new()
}

/// Set carpal arch width.
pub fn carp_set_width(m: &mut CarpalsMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set carpal arch height.
pub fn carp_set_height(m: &mut CarpalsMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

/// Set intercarpal spacing.
pub fn carp_set_spacing(m: &mut CarpalsMorph, v: f32) {
    m.spacing = v.clamp(0.0, 1.0);
}

/// Set carpal tunnel depth.
pub fn carp_set_tunnel_depth(m: &mut CarpalsMorph, v: f32) {
    m.tunnel_depth = v.clamp(0.0, 1.0);
}

/// Carpal tunnel cross-sectional area estimate.
pub fn carp_tunnel_area(m: &CarpalsMorph) -> f32 {
    m.width * m.tunnel_depth
}

/// Serialize to JSON-like string.
pub fn carpals_morph_to_json(m: &CarpalsMorph) -> String {
    format!(
        r#"{{"width":{:.4},"height":{:.4},"spacing":{:.4},"tunnel_depth":{:.4}}}"#,
        m.width, m.height, m.spacing, m.tunnel_depth
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_carpals_morph();
        assert!((m.width - 0.5).abs() < 1e-6);
        assert!((m.tunnel_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_width_clamp() {
        let mut m = new_carpals_morph();
        carp_set_width(&mut m, 5.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_height_set() {
        let mut m = new_carpals_morph();
        carp_set_height(&mut m, 0.7);
        assert!((m.height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_spacing_set() {
        let mut m = new_carpals_morph();
        carp_set_spacing(&mut m, 0.3);
        assert!((m.spacing - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_tunnel_depth_clamp() {
        let mut m = new_carpals_morph();
        carp_set_tunnel_depth(&mut m, -1.0);
        assert_eq!(m.tunnel_depth, 0.0);
    }

    #[test]
    fn test_tunnel_area_positive() {
        let m = new_carpals_morph();
        assert!(carp_tunnel_area(&m) > 0.0);
    }

    #[test]
    fn test_tunnel_area_zero() {
        let mut m = new_carpals_morph();
        carp_set_width(&mut m, 0.0);
        assert_eq!(carp_tunnel_area(&m), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_carpals_morph();
        let s = carpals_morph_to_json(&m);
        assert!(s.contains("tunnel_depth"));
    }

    #[test]
    fn test_clone() {
        let m = new_carpals_morph();
        let m2 = m.clone();
        assert!((m2.spacing - m.spacing).abs() < 1e-6);
    }
}
