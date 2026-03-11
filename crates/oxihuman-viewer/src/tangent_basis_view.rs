// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Tangent/bitangent basis vectors per-vertex visualization.
#[derive(Debug, Clone)]
pub struct TangentBasisView {
    pub enabled: bool,
    /// Scale for tangent arrows.
    pub scale: f32,
    /// Show tangent (T) arrows.
    pub show_tangent: bool,
    /// Show bitangent (B) arrows.
    pub show_bitangent: bool,
    /// Show normal (N) arrows.
    pub show_normal: bool,
}

pub fn new_tangent_basis_view() -> TangentBasisView {
    TangentBasisView {
        enabled: false,
        scale: 0.03,
        show_tangent: true,
        show_bitangent: true,
        show_normal: false,
    }
}

pub fn tbv_enable(v: &mut TangentBasisView) {
    v.enabled = true;
}

pub fn tbv_set_scale(v: &mut TangentBasisView, s: f32) {
    v.scale = s.max(1e-4);
}

pub fn tbv_set_show_tangent(v: &mut TangentBasisView, show: bool) {
    v.show_tangent = show;
}

pub fn tbv_set_show_bitangent(v: &mut TangentBasisView, show: bool) {
    v.show_bitangent = show;
}

/// Returns the colour for the tangent (red).
pub fn tbv_tangent_color() -> [f32; 3] {
    [1.0, 0.0, 0.0]
}

/// Returns the colour for the bitangent (green).
pub fn tbv_bitangent_color() -> [f32; 3] {
    [0.0, 1.0, 0.0]
}

/// Returns the colour for the normal (blue).
pub fn tbv_normal_color() -> [f32; 3] {
    [0.0, 0.0, 1.0]
}

pub fn tbv_active_channel_count(v: &TangentBasisView) -> u32 {
    [v.show_tangent, v.show_bitangent, v.show_normal]
        .iter()
        .filter(|&&b| b)
        .count() as u32
}

pub fn tbv_to_json(v: &TangentBasisView) -> String {
    format!(
        r#"{{"enabled":{},"scale":{:.4},"show_tangent":{},"show_bitangent":{}}}"#,
        v.enabled, v.scale, v.show_tangent, v.show_bitangent
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* scale=0.03, T and B shown */
        let v = new_tangent_basis_view();
        assert!((v.scale - 0.03).abs() < 1e-6);
        assert!(v.show_tangent);
        assert!(v.show_bitangent);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_tangent_basis_view();
        tbv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_scale() {
        /* valid scale */
        let mut v = new_tangent_basis_view();
        tbv_set_scale(&mut v, 0.05);
        assert!((v.scale - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_scale_min() {
        /* min enforced */
        let mut v = new_tangent_basis_view();
        tbv_set_scale(&mut v, 0.0);
        assert!(v.scale > 0.0);
    }

    #[test]
    fn test_tangent_color_red() {
        /* tangent is red */
        let c = tbv_tangent_color();
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_bitangent_color_green() {
        /* bitangent is green */
        let c = tbv_bitangent_color();
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_normal_color_blue() {
        /* normal is blue */
        let c = tbv_normal_color();
        assert_eq!(c, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_active_channel_count_default() {
        /* T and B enabled = 2 */
        let v = new_tangent_basis_view();
        assert_eq!(tbv_active_channel_count(&v), 2);
    }

    #[test]
    fn test_to_json() {
        /* JSON has scale */
        assert!(tbv_to_json(&new_tangent_basis_view()).contains("scale"));
    }
}
