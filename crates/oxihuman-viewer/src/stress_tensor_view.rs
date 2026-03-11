// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct StressTensorView {
    pub enabled: bool,
    pub glyph_scale: f32,
    pub show_compression: bool,
    pub show_tension: bool,
}

pub fn new_stress_tensor_view() -> StressTensorView {
    StressTensorView {
        enabled: false,
        glyph_scale: 1.0,
        show_compression: true,
        show_tension: true,
    }
}

pub fn stv_set_glyph_scale(v: &mut StressTensorView, s: f32) {
    v.glyph_scale = s.max(0.0);
}

pub fn stv_enable(v: &mut StressTensorView) {
    v.enabled = true;
}

pub fn stv_toggle_compression(v: &mut StressTensorView) {
    v.show_compression = !v.show_compression;
}

pub fn stv_toggle_tension(v: &mut StressTensorView) {
    v.show_tension = !v.show_tension;
}

pub fn stv_principal_color(eigenvalue: f32) -> [f32; 3] {
    if eigenvalue > 0.0 {
        [1.0, 0.3, 0.3]
    } else {
        [0.3, 0.3, 1.0]
    }
}

pub fn stv_is_enabled(v: &StressTensorView) -> bool {
    v.enabled
}

pub fn stv_to_json(v: &StressTensorView) -> String {
    format!(
        r#"{{"enabled":{},"glyph_scale":{:.4},"show_compression":{},"show_tension":{}}}"#,
        v.enabled, v.glyph_scale, v.show_compression, v.show_tension
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, scale=1, both on */
        let v = new_stress_tensor_view();
        assert!(!v.enabled);
        assert!((v.glyph_scale - 1.0).abs() < 1e-6);
        assert!(v.show_compression);
        assert!(v.show_tension);
    }

    #[test]
    fn test_set_glyph_scale() {
        /* valid scale */
        let mut v = new_stress_tensor_view();
        stv_set_glyph_scale(&mut v, 2.0);
        assert!((v.glyph_scale - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_glyph_scale_min() {
        /* minimum 0 */
        let mut v = new_stress_tensor_view();
        stv_set_glyph_scale(&mut v, -1.0);
        assert_eq!(v.glyph_scale, 0.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_stress_tensor_view();
        stv_enable(&mut v);
        assert!(stv_is_enabled(&v));
    }

    #[test]
    fn test_toggle_compression() {
        /* toggle flips flag */
        let mut v = new_stress_tensor_view();
        stv_toggle_compression(&mut v);
        assert!(!v.show_compression);
    }

    #[test]
    fn test_toggle_tension() {
        /* toggle flips flag */
        let mut v = new_stress_tensor_view();
        stv_toggle_tension(&mut v);
        assert!(!v.show_tension);
    }

    #[test]
    fn test_principal_color_tension() {
        /* positive eigenvalue -> reddish */
        let c = stv_principal_color(0.5);
        assert!(c[0] > c[2]);
    }

    #[test]
    fn test_principal_color_compression() {
        /* negative eigenvalue -> bluish */
        let c = stv_principal_color(-0.5);
        assert!(c[2] > c[0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has glyph_scale */
        let v = new_stress_tensor_view();
        assert!(stv_to_json(&v).contains("glyph_scale"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_stress_tensor_view();
        let v2 = v.clone();
        assert_eq!(v.glyph_scale, v2.glyph_scale);
    }
}
