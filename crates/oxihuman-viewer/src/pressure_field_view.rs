// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct PressureFieldView {
    pub enabled: bool,
    pub pressure_min: f32,
    pub pressure_max: f32,
    pub show_isobars: bool,
    pub isobar_count: usize,
}

pub fn new_pressure_field_view() -> PressureFieldView {
    PressureFieldView {
        enabled: false,
        pressure_min: 0.0,
        pressure_max: 1.0,
        show_isobars: false,
        isobar_count: 5,
    }
}

pub fn pfv_set_enabled(v: &mut PressureFieldView, e: bool) {
    v.enabled = e;
}

pub fn pfv_set_pressure_min(v: &mut PressureFieldView, m: f32) {
    v.pressure_min = m;
}

pub fn pfv_set_pressure_max(v: &mut PressureFieldView, m: f32) {
    v.pressure_max = m;
}

pub fn pfv_set_show_isobars(v: &mut PressureFieldView, s: bool) {
    v.show_isobars = s;
}

pub fn pfv_set_isobar_count(v: &mut PressureFieldView, n: usize) {
    v.isobar_count = n.max(1);
}

pub fn pfv_normalize(v: &PressureFieldView, pressure: f32) -> f32 {
    let range = v.pressure_max - v.pressure_min;
    if range.abs() < 1e-9 {
        return 0.0;
    }
    ((pressure - v.pressure_min) / range).clamp(0.0, 1.0)
}

pub fn pressure_field_view_to_json(v: &PressureFieldView) -> String {
    format!(
        r#"{{"enabled":{},"pressure_min":{:.4},"pressure_max":{:.4},"show_isobars":{},"isobar_count":{}}}"#,
        v.enabled, v.pressure_min, v.pressure_max, v.show_isobars, v.isobar_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, range [0,1] */
        let v = new_pressure_field_view();
        assert!(!v.enabled);
        assert_eq!(v.pressure_min, 0.0);
        assert!((v.pressure_max - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_enabled() {
        /* enable and disable */
        let mut v = new_pressure_field_view();
        pfv_set_enabled(&mut v, true);
        assert!(v.enabled);
        pfv_set_enabled(&mut v, false);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_pressure_min() {
        /* min stored */
        let mut v = new_pressure_field_view();
        pfv_set_pressure_min(&mut v, -1.0);
        assert!((v.pressure_min - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_set_pressure_max() {
        /* max stored */
        let mut v = new_pressure_field_view();
        pfv_set_pressure_max(&mut v, 5.0);
        assert!((v.pressure_max - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_show_isobars() {
        /* flag set */
        let mut v = new_pressure_field_view();
        pfv_set_show_isobars(&mut v, true);
        assert!(v.show_isobars);
    }

    #[test]
    fn test_set_isobar_count() {
        /* count stored */
        let mut v = new_pressure_field_view();
        pfv_set_isobar_count(&mut v, 10);
        assert_eq!(v.isobar_count, 10);
    }

    #[test]
    fn test_normalize_midpoint() {
        /* midpoint normalizes to 0.5 */
        let v = new_pressure_field_view();
        assert!((pfv_normalize(&v, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_clamp() {
        /* out of range clamped */
        let v = new_pressure_field_view();
        assert_eq!(pfv_normalize(&v, 2.0), 1.0);
        assert_eq!(pfv_normalize(&v, -1.0), 0.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has pressure_min */
        let v = new_pressure_field_view();
        assert!(pressure_field_view_to_json(&v).contains("pressure_min"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_pressure_field_view();
        let v2 = v.clone();
        assert_eq!(v.isobar_count, v2.isobar_count);
    }
}
