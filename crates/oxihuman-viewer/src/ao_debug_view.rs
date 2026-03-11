// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct AoDebugView {
    pub power: f32,
    pub enabled: bool,
    pub invert: bool,
}

pub fn new_ao_debug_view() -> AoDebugView {
    AoDebugView {
        power: 1.0,
        enabled: false,
        invert: false,
    }
}

pub fn adv_set_power(v: &mut AoDebugView, p: f32) {
    v.power = p.clamp(0.01, 10.0);
}

pub fn adv_enable(v: &mut AoDebugView) {
    v.enabled = true;
}

pub fn adv_ao_color(v: &AoDebugView, ao_value: f32) -> [f32; 3] {
    let a = ao_value.clamp(0.0, 1.0);
    let mut g = a.powf(v.power);
    if v.invert {
        g = 1.0 - g;
    }
    [g, g, g]
}

pub fn adv_is_enabled(v: &AoDebugView) -> bool {
    v.enabled
}

pub fn adv_to_json(v: &AoDebugView) -> String {
    format!(
        r#"{{"power":{:.4},"enabled":{},"invert":{}}}"#,
        v.power, v.enabled, v.invert
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* power=1, disabled, not inverted */
        let v = new_ao_debug_view();
        assert!((v.power - 1.0).abs() < 1e-6);
        assert!(!v.enabled);
        assert!(!v.invert);
    }

    #[test]
    fn test_set_power() {
        /* valid power stored */
        let mut v = new_ao_debug_view();
        adv_set_power(&mut v, 2.0);
        assert!((v.power - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_power_clamp_low() {
        /* below min clamped */
        let mut v = new_ao_debug_view();
        adv_set_power(&mut v, 0.0);
        assert!(v.power >= 0.01);
    }

    #[test]
    fn test_set_power_clamp_high() {
        /* above max clamped */
        let mut v = new_ao_debug_view();
        adv_set_power(&mut v, 100.0);
        assert!(v.power <= 10.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_ao_debug_view();
        adv_enable(&mut v);
        assert!(adv_is_enabled(&v));
    }

    #[test]
    fn test_ao_color_zero() {
        /* ao=0 => black */
        let v = new_ao_debug_view();
        let c = adv_ao_color(&v, 0.0);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_ao_color_one() {
        /* ao=1 => white */
        let v = new_ao_debug_view();
        let c = adv_ao_color(&v, 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ao_color_inverted() {
        /* inverted: ao=1 => black */
        let mut v = new_ao_debug_view();
        v.invert = true;
        let c = adv_ao_color(&v, 1.0);
        assert!(c[0] < 0.1);
    }

    #[test]
    fn test_to_json() {
        /* JSON has power and invert */
        let v = new_ao_debug_view();
        let s = adv_to_json(&v);
        assert!(s.contains("power"));
        assert!(s.contains("invert"));
    }
}
