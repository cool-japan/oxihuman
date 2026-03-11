// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Skin subsurface scattering parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SkinSubsurface {
    pub scatter_radius: f32,
    pub scatter_r: f32,
    pub scatter_g: f32,
    pub scatter_b: f32,
}

/// Create a new skin subsurface with default parameters.
#[allow(dead_code)]
pub fn new_skin_subsurface() -> SkinSubsurface {
    default_subsurface()
}

/// Set the scatter radius.
#[allow(dead_code)]
pub fn set_scatter_radius(sss: &mut SkinSubsurface, radius: f32) {
    sss.scatter_radius = radius.max(0.0);
}

/// Get the scatter radius.
#[allow(dead_code)]
pub fn scatter_radius(sss: &SkinSubsurface) -> f32 {
    sss.scatter_radius
}

/// Set the scatter color.
#[allow(dead_code)]
pub fn set_scatter_color(sss: &mut SkinSubsurface, r: f32, g: f32, b: f32) {
    sss.scatter_r = r.clamp(0.0, 1.0);
    sss.scatter_g = g.clamp(0.0, 1.0);
    sss.scatter_b = b.clamp(0.0, 1.0);
}

/// Get the scatter color as (r, g, b).
#[allow(dead_code)]
pub fn scatter_color(sss: &SkinSubsurface) -> (f32, f32, f32) {
    (sss.scatter_r, sss.scatter_g, sss.scatter_b)
}

/// Convert subsurface parameters to a parameter array [radius, r, g, b].
#[allow(dead_code)]
pub fn subsurface_to_params(sss: &SkinSubsurface) -> [f32; 4] {
    [sss.scatter_radius, sss.scatter_r, sss.scatter_g, sss.scatter_b]
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn subsurface_to_json(sss: &SkinSubsurface) -> String {
    format!(
        "{{\"radius\":{:.4},\"color\":[{:.4},{:.4},{:.4}]}}",
        sss.scatter_radius, sss.scatter_r, sss.scatter_g, sss.scatter_b
    )
}

/// Return default subsurface scattering parameters (warm skin tone).
#[allow(dead_code)]
pub fn default_subsurface() -> SkinSubsurface {
    SkinSubsurface {
        scatter_radius: 0.012,
        scatter_r: 0.8,
        scatter_g: 0.3,
        scatter_b: 0.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let s = default_subsurface();
        assert!(s.scatter_radius > 0.0);
    }

    #[test]
    fn new_equals_default() {
        let a = new_skin_subsurface();
        let b = default_subsurface();
        assert!((a.scatter_radius - b.scatter_radius).abs() < 1e-9);
    }

    #[test]
    fn set_radius() {
        let mut s = new_skin_subsurface();
        set_scatter_radius(&mut s, 0.05);
        assert!((scatter_radius(&s) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn radius_non_negative() {
        let mut s = new_skin_subsurface();
        set_scatter_radius(&mut s, -1.0);
        assert!(scatter_radius(&s) >= 0.0);
    }

    #[test]
    fn set_color() {
        let mut s = new_skin_subsurface();
        set_scatter_color(&mut s, 0.5, 0.6, 0.7);
        let (r, g, b) = scatter_color(&s);
        assert!((r - 0.5).abs() < 1e-6);
        assert!((g - 0.6).abs() < 1e-6);
        assert!((b - 0.7).abs() < 1e-6);
    }

    #[test]
    fn color_clamped() {
        let mut s = new_skin_subsurface();
        set_scatter_color(&mut s, 1.5, -0.1, 0.5);
        let (r, g, b) = scatter_color(&s);
        assert!((r - 1.0).abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!((b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_params() {
        let s = default_subsurface();
        let p = subsurface_to_params(&s);
        assert_eq!(p.len(), 4);
        assert!((p[0] - s.scatter_radius).abs() < 1e-9);
    }

    #[test]
    fn to_json() {
        let s = default_subsurface();
        let j = subsurface_to_json(&s);
        assert!(j.contains("radius"));
    }

    #[test]
    fn zero_radius() {
        let mut s = new_skin_subsurface();
        set_scatter_radius(&mut s, 0.0);
        assert!(scatter_radius(&s).abs() < 1e-9);
    }

    #[test]
    fn color_at_extremes() {
        let mut s = new_skin_subsurface();
        set_scatter_color(&mut s, 0.0, 0.0, 0.0);
        let (r, g, b) = scatter_color(&s);
        assert!(r.abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!(b.abs() < 1e-6);
    }
}
