// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Visualize mesh poles (vertices with 3 or 5+ edge connections).
#[derive(Debug, Clone)]
pub struct PoleVisualizer {
    pub enabled: bool,
    /// Radius of the pole indicator glyph in UV space (0.0 … 1.0).
    pub glyph_radius: f32,
    /// Colour for n-poles (3 edges).
    pub n_pole_color: [f32; 3],
    /// Colour for e-poles (5+ edges).
    pub e_pole_color: [f32; 3],
}

pub fn new_pole_visualizer() -> PoleVisualizer {
    PoleVisualizer {
        enabled: false,
        glyph_radius: 0.01,
        n_pole_color: [1.0, 0.3, 0.0],
        e_pole_color: [0.2, 0.4, 1.0],
    }
}

pub fn pv_enable(v: &mut PoleVisualizer) {
    v.enabled = true;
}

pub fn pv_set_glyph_radius(v: &mut PoleVisualizer, r: f32) {
    v.glyph_radius = r.clamp(0.001, 0.5);
}

pub fn pv_set_n_pole_color(v: &mut PoleVisualizer, r: f32, g: f32, b: f32) {
    v.n_pole_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

pub fn pv_set_e_pole_color(v: &mut PoleVisualizer, r: f32, g: f32, b: f32) {
    v.e_pole_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Returns the colour for a vertex with the given valence.
pub fn pv_color_for_valence(view: &PoleVisualizer, valence: u32) -> Option<[f32; 3]> {
    match valence {
        3 => Some(view.n_pole_color),
        val if val >= 5 => Some(view.e_pole_color),
        _ => None,
    }
}

pub fn pv_is_pole(valence: u32) -> bool {
    valence == 3 || valence >= 5
}

pub fn pv_to_json(v: &PoleVisualizer) -> String {
    format!(
        r#"{{"enabled":{},"glyph_radius":{:.4}}}"#,
        v.enabled, v.glyph_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* enabled=false, radius=0.01 */
        let v = new_pole_visualizer();
        assert!(!v.enabled);
        assert!((v.glyph_radius - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_pole_visualizer();
        pv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_glyph_radius() {
        /* valid radius */
        let mut v = new_pole_visualizer();
        pv_set_glyph_radius(&mut v, 0.05);
        assert!((v.glyph_radius - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_glyph_radius_clamp_low() {
        /* clamped below min */
        let mut v = new_pole_visualizer();
        pv_set_glyph_radius(&mut v, 0.0);
        assert!(v.glyph_radius >= 0.001);
    }

    #[test]
    fn test_is_pole_3() {
        /* 3-valence is pole */
        assert!(pv_is_pole(3));
    }

    #[test]
    fn test_is_pole_4_not() {
        /* 4-valence is not pole */
        assert!(!pv_is_pole(4));
    }

    #[test]
    fn test_color_n_pole() {
        /* n-pole returns n_pole_color */
        let v = new_pole_visualizer();
        let c = pv_color_for_valence(&v, 3).expect("should succeed");
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_regular_none() {
        /* valence 4 returns None */
        let v = new_pole_visualizer();
        assert!(pv_color_for_valence(&v, 4).is_none());
    }

    #[test]
    fn test_to_json() {
        /* JSON has glyph_radius */
        assert!(pv_to_json(&new_pole_visualizer()).contains("glyph_radius"));
    }
}
