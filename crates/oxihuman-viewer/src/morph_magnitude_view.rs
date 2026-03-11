// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MorphMagnitudeView {
    pub enabled: bool,
    pub max_delta: f32,
    pub show_arrows: bool,
}

pub fn new_morph_magnitude_view() -> MorphMagnitudeView {
    MorphMagnitudeView {
        enabled: false,
        max_delta: 1.0,
        show_arrows: false,
    }
}

pub fn mmv_set_max_delta(v: &mut MorphMagnitudeView, d: f32) {
    v.max_delta = d.max(1e-6);
}

pub fn mmv_enable(v: &mut MorphMagnitudeView) {
    v.enabled = true;
}

pub fn mmv_toggle_arrows(v: &mut MorphMagnitudeView) {
    v.show_arrows = !v.show_arrows;
}

pub fn mmv_delta_color(v: &MorphMagnitudeView, delta_length: f32) -> [f32; 3] {
    let t = (delta_length / v.max_delta).clamp(0.0, 1.0);
    [t, 1.0 - t, 0.0]
}

pub fn mmv_is_enabled(v: &MorphMagnitudeView) -> bool {
    v.enabled
}

pub fn mmv_to_json(v: &MorphMagnitudeView) -> String {
    format!(
        r#"{{"enabled":{},"max_delta":{:.4},"show_arrows":{}}}"#,
        v.enabled, v.max_delta, v.show_arrows
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, max_delta=1 */
        let v = new_morph_magnitude_view();
        assert!(!v.enabled);
        assert!((v.max_delta - 1.0).abs() < 1e-6);
        assert!(!v.show_arrows);
    }

    #[test]
    fn test_set_max_delta() {
        /* valid delta */
        let mut v = new_morph_magnitude_view();
        mmv_set_max_delta(&mut v, 5.0);
        assert!((v.max_delta - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_delta_min() {
        /* minimum enforced */
        let mut v = new_morph_magnitude_view();
        mmv_set_max_delta(&mut v, 0.0);
        assert!(v.max_delta > 0.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_morph_magnitude_view();
        mmv_enable(&mut v);
        assert!(mmv_is_enabled(&v));
    }

    #[test]
    fn test_toggle_arrows() {
        /* toggle flips flag */
        let mut v = new_morph_magnitude_view();
        mmv_toggle_arrows(&mut v);
        assert!(v.show_arrows);
    }

    #[test]
    fn test_delta_color_zero() {
        /* zero delta -> green */
        let v = new_morph_magnitude_view();
        let c = mmv_delta_color(&v, 0.0);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_delta_color_max() {
        /* max delta -> red */
        let v = new_morph_magnitude_view();
        let c = mmv_delta_color(&v, 1.0);
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has max_delta */
        let v = new_morph_magnitude_view();
        assert!(mmv_to_json(&v).contains("max_delta"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_morph_magnitude_view();
        let v2 = v.clone();
        assert_eq!(v.max_delta, v2.max_delta);
    }
}
