// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MorphDeltaView {
    pub scale: f32,
    pub threshold: f32,
    pub enabled: bool,
}

pub fn new_morph_delta_view() -> MorphDeltaView {
    MorphDeltaView {
        scale: 1.0,
        threshold: 0.001,
        enabled: false,
    }
}

pub fn mdlv_set_scale(v: &mut MorphDeltaView, s: f32) {
    v.scale = s.clamp(0.001, 100.0);
}

pub fn mdlv_enable(v: &mut MorphDeltaView) {
    v.enabled = true;
}

pub fn mdlv_delta_color(v: &MorphDeltaView, magnitude: f32) -> [f32; 3] {
    let scaled = (magnitude * v.scale).clamp(0.0, 1.0);
    if scaled < v.threshold {
        [0.2, 0.2, 0.2] /* dark grey: no delta */
    } else {
        let r = scaled;
        let g = 1.0 - scaled;
        let b = 0.5 * (1.0 - scaled);
        [r, g, b]
    }
}

pub fn mdlv_is_enabled(v: &MorphDeltaView) -> bool {
    v.enabled
}

pub fn mdlv_to_json(v: &MorphDeltaView) -> String {
    format!(
        r#"{{"scale":{:.4},"threshold":{:.6},"enabled":{}}}"#,
        v.scale, v.threshold, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* scale=1, disabled */
        let v = new_morph_delta_view();
        assert!((v.scale - 1.0).abs() < 1e-6);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_scale() {
        /* valid scale stored */
        let mut v = new_morph_delta_view();
        mdlv_set_scale(&mut v, 5.0);
        assert!((v.scale - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_scale_clamp_low() {
        /* below min clamped */
        let mut v = new_morph_delta_view();
        mdlv_set_scale(&mut v, 0.0);
        assert!(v.scale >= 0.001);
    }

    #[test]
    fn test_set_scale_clamp_high() {
        /* above max clamped */
        let mut v = new_morph_delta_view();
        mdlv_set_scale(&mut v, 10000.0);
        assert!(v.scale <= 100.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_morph_delta_view();
        mdlv_enable(&mut v);
        assert!(mdlv_is_enabled(&v));
    }

    #[test]
    fn test_delta_color_zero_magnitude() {
        /* zero magnitude => grey */
        let v = new_morph_delta_view();
        let c = mdlv_delta_color(&v, 0.0);
        assert!((c[0] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_delta_color_high_magnitude() {
        /* high magnitude => reddish */
        let v = new_morph_delta_view();
        let c = mdlv_delta_color(&v, 1.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has scale and threshold */
        let v = new_morph_delta_view();
        let s = mdlv_to_json(&v);
        assert!(s.contains("scale"));
        assert!(s.contains("threshold"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_morph_delta_view();
        let v2 = v.clone();
        assert_eq!(v.enabled, v2.enabled);
    }
}
