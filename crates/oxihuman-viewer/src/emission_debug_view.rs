// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct EmissionDebugView {
    pub exposure_ev: f32,
    pub enabled: bool,
    pub show_zero: bool,
}

pub fn new_emission_debug_view() -> EmissionDebugView {
    EmissionDebugView {
        exposure_ev: 0.0,
        enabled: false,
        show_zero: false,
    }
}

pub fn edv_set_exposure(v: &mut EmissionDebugView, ev: f32) {
    v.exposure_ev = ev.clamp(-10.0, 10.0);
}

pub fn edv_enable(v: &mut EmissionDebugView) {
    v.enabled = true;
}

pub fn edv_emission_color(v: &EmissionDebugView, emission: [f32; 3]) -> [f32; 3] {
    let scale = 2_f32.powf(v.exposure_ev);
    /* simple Reinhard tonemapping */
    let r = (emission[0] * scale) / (1.0 + emission[0] * scale);
    let g = (emission[1] * scale) / (1.0 + emission[1] * scale);
    let b = (emission[2] * scale) / (1.0 + emission[2] * scale);
    [r, g, b]
}

pub fn edv_is_enabled(v: &EmissionDebugView) -> bool {
    v.enabled
}

pub fn edv_to_json(v: &EmissionDebugView) -> String {
    format!(
        r#"{{"exposure_ev":{:.4},"enabled":{},"show_zero":{}}}"#,
        v.exposure_ev, v.enabled, v.show_zero
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* ev=0, disabled */
        let v = new_emission_debug_view();
        assert_eq!(v.exposure_ev, 0.0);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_exposure() {
        /* valid ev stored */
        let mut v = new_emission_debug_view();
        edv_set_exposure(&mut v, 2.0);
        assert!((v.exposure_ev - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_exposure_clamp_high() {
        /* above max clamped */
        let mut v = new_emission_debug_view();
        edv_set_exposure(&mut v, 100.0);
        assert!(v.exposure_ev <= 10.0);
    }

    #[test]
    fn test_set_exposure_clamp_low() {
        /* below min clamped */
        let mut v = new_emission_debug_view();
        edv_set_exposure(&mut v, -100.0);
        assert!(v.exposure_ev >= -10.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_emission_debug_view();
        edv_enable(&mut v);
        assert!(edv_is_enabled(&v));
    }

    #[test]
    fn test_emission_color_zero() {
        /* zero emission => black */
        let v = new_emission_debug_view();
        let c = edv_emission_color(&v, [0.0, 0.0, 0.0]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_emission_color_range() {
        /* tonemapped components in [0,1) */
        let v = new_emission_debug_view();
        let c = edv_emission_color(&v, [1.0, 0.5, 2.0]);
        for ch in c {
            assert!((0.0..1.0).contains(&ch));
        }
    }

    #[test]
    fn test_emission_color_high_exposure() {
        /* high exposure brightens */
        let mut v = new_emission_debug_view();
        edv_set_exposure(&mut v, 4.0);
        let c_low = edv_emission_color(&new_emission_debug_view(), [0.1, 0.1, 0.1]);
        let c_high = edv_emission_color(&v, [0.1, 0.1, 0.1]);
        assert!(c_high[0] > c_low[0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has exposure_ev */
        let v = new_emission_debug_view();
        assert!(edv_to_json(&v).contains("exposure_ev"));
    }
}
