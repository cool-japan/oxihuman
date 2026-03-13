// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct UvStretchView {
    pub low_threshold: f32,
    pub high_threshold: f32,
    pub enabled: bool,
    pub mode: u32,
}

pub fn new_uv_stretch_view() -> UvStretchView {
    UvStretchView {
        low_threshold: 0.2,
        high_threshold: 0.8,
        enabled: false,
        mode: 0,
    }
}

pub fn usv_set_thresholds(v: &mut UvStretchView, low: f32, high: f32) {
    v.low_threshold = low.clamp(0.0, 1.0);
    v.high_threshold = high.clamp(0.0, 1.0).max(v.low_threshold);
}

pub fn usv_enable(v: &mut UvStretchView) {
    v.enabled = true;
}

pub fn usv_stretch_color(v: &UvStretchView, stretch: f32) -> [f32; 3] {
    if stretch < v.low_threshold {
        [0.0, 0.0, 1.0] /* blue: understretched */
    } else if stretch > v.high_threshold {
        [1.0, 0.0, 0.0] /* red: overstretched */
    } else {
        [0.0, 1.0, 0.0] /* green: good */
    }
}

pub fn usv_mode_name(v: &UvStretchView) -> &str {
    match v.mode {
        0 => "Area",
        1 => "Angle",
        2 => "Length",
        _ => "Unknown",
    }
}

pub fn usv_is_enabled(v: &UvStretchView) -> bool {
    v.enabled
}

pub fn usv_to_json(v: &UvStretchView) -> String {
    format!(
        r#"{{"low_threshold":{:.4},"high_threshold":{:.4},"enabled":{},"mode":{}}}"#,
        v.low_threshold, v.high_threshold, v.enabled, v.mode
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* default thresholds and mode */
        let v = new_uv_stretch_view();
        assert!(v.low_threshold < v.high_threshold);
        assert_eq!(v.mode, 0);
    }

    #[test]
    fn test_set_thresholds() {
        /* stored correctly */
        let mut v = new_uv_stretch_view();
        usv_set_thresholds(&mut v, 0.1, 0.9);
        assert!((v.low_threshold - 0.1).abs() < 1e-6);
        assert!((v.high_threshold - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_uv_stretch_view();
        usv_enable(&mut v);
        assert!(usv_is_enabled(&v));
    }

    #[test]
    fn test_stretch_color_low() {
        /* under low => blue */
        let v = new_uv_stretch_view();
        let c = usv_stretch_color(&v, 0.0);
        assert_eq!(c, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_stretch_color_high() {
        /* over high => red */
        let v = new_uv_stretch_view();
        let c = usv_stretch_color(&v, 1.0);
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_stretch_color_mid() {
        /* in range => green */
        let v = new_uv_stretch_view();
        let c = usv_stretch_color(&v, 0.5);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_mode_name_area() {
        /* mode 0 = Area */
        let v = new_uv_stretch_view();
        assert_eq!(usv_mode_name(&v), "Area");
    }

    #[test]
    fn test_mode_name_angle() {
        /* mode 1 = Angle */
        let mut v = new_uv_stretch_view();
        v.mode = 1;
        assert_eq!(usv_mode_name(&v), "Angle");
    }

    #[test]
    fn test_to_json() {
        /* JSON has mode field */
        let v = new_uv_stretch_view();
        assert!(usv_to_json(&v).contains("mode"));
    }
}
