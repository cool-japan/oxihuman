// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct OverdrawView {
    pub max_overdraw: u32,
    pub enabled: bool,
    pub accumulate: bool,
}

pub fn new_overdraw_view() -> OverdrawView {
    OverdrawView {
        max_overdraw: 8,
        enabled: false,
        accumulate: true,
    }
}

pub fn odv_set_max(v: &mut OverdrawView, max: u32) {
    v.max_overdraw = max.max(1);
}

pub fn odv_enable(v: &mut OverdrawView) {
    v.enabled = true;
}

pub fn odv_overdraw_color(v: &OverdrawView, count: u32) -> [f32; 3] {
    let t = (count as f32 / v.max_overdraw as f32).clamp(0.0, 1.0);
    /* green -> yellow -> red gradient */
    let r = (t * 2.0).clamp(0.0, 1.0);
    let g = (2.0 - t * 2.0).clamp(0.0, 1.0);
    [r, g, 0.0]
}

pub fn odv_reset(v: &mut OverdrawView) {
    v.accumulate = false;
    v.accumulate = true;
}

pub fn odv_is_enabled(v: &OverdrawView) -> bool {
    v.enabled
}

pub fn odv_to_json(v: &OverdrawView) -> String {
    format!(
        r#"{{"max_overdraw":{},"enabled":{},"accumulate":{}}}"#,
        v.max_overdraw, v.enabled, v.accumulate
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new view disabled, accumulate on */
        let v = new_overdraw_view();
        assert!(!v.enabled);
        assert!(v.accumulate);
    }

    #[test]
    fn test_set_max() {
        /* max stored */
        let mut v = new_overdraw_view();
        odv_set_max(&mut v, 16);
        assert_eq!(v.max_overdraw, 16);
    }

    #[test]
    fn test_set_max_clamp_zero() {
        /* 0 clamped to 1 */
        let mut v = new_overdraw_view();
        odv_set_max(&mut v, 0);
        assert_eq!(v.max_overdraw, 1);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_overdraw_view();
        odv_enable(&mut v);
        assert!(odv_is_enabled(&v));
    }

    #[test]
    fn test_overdraw_color_zero() {
        /* zero overdraw => green */
        let v = new_overdraw_view();
        let c = odv_overdraw_color(&v, 0);
        assert!(c[1] >= c[0]);
    }

    #[test]
    fn test_overdraw_color_max() {
        /* max overdraw => red */
        let v = new_overdraw_view();
        let c = odv_overdraw_color(&v, v.max_overdraw);
        assert!(c[0] >= c[1]);
    }

    #[test]
    fn test_overdraw_color_clamped() {
        /* beyond max still clamped to 1 */
        let v = new_overdraw_view();
        let c = odv_overdraw_color(&v, 9999);
        assert!(c[0] <= 1.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has max_overdraw */
        let v = new_overdraw_view();
        assert!(odv_to_json(&v).contains("max_overdraw"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_overdraw_view();
        let v2 = v.clone();
        assert_eq!(v.max_overdraw, v2.max_overdraw);
    }
}
