// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HSV color wheel utility for the viewer UI.

use std::f32::consts::TAU;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HsvColor {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RgbColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorWheelState {
    pub selected: HsvColor,
    pub radius: f32,
    pub show_alpha: bool,
}

#[allow(dead_code)]
pub fn new_hsv(h: f32, s: f32, v: f32) -> HsvColor {
    HsvColor {
        hue: h % 360.0,
        saturation: s.clamp(0.0, 1.0),
        value: v.clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn hsv_to_rgb(hsv: &HsvColor) -> RgbColor {
    let h = hsv.hue / 60.0;
    let c = hsv.value * hsv.saturation;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = hsv.value - c;
    let (r, g, b) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    RgbColor { r: r + m, g: g + m, b: b + m }
}

#[allow(dead_code)]
pub fn rgb_to_hsv(rgb: &RgbColor) -> HsvColor {
    let max = rgb.r.max(rgb.g).max(rgb.b);
    let min = rgb.r.min(rgb.g).min(rgb.b);
    let delta = max - min;
    let hue = if delta.abs() < 1e-8 {
        0.0
    } else if (max - rgb.r).abs() < 1e-8 {
        60.0 * (((rgb.g - rgb.b) / delta) % 6.0)
    } else if (max - rgb.g).abs() < 1e-8 {
        60.0 * ((rgb.b - rgb.r) / delta + 2.0)
    } else {
        60.0 * ((rgb.r - rgb.g) / delta + 4.0)
    };
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    let saturation = if max.abs() < 1e-8 { 0.0 } else { delta / max };
    HsvColor { hue, saturation, value: max }
}

#[allow(dead_code)]
pub fn wheel_position_to_hue(x: f32, y: f32) -> f32 {
    let angle = y.atan2(x);
    let deg = angle * 360.0 / TAU;
    if deg < 0.0 { deg + 360.0 } else { deg }
}

#[allow(dead_code)]
pub fn new_color_wheel_state() -> ColorWheelState {
    ColorWheelState {
        selected: new_hsv(0.0, 1.0, 1.0),
        radius: 100.0,
        show_alpha: false,
    }
}

#[allow(dead_code)]
pub fn set_wheel_color(state: &mut ColorWheelState, hsv: HsvColor) {
    state.selected = hsv;
}

#[allow(dead_code)]
pub fn color_wheel_to_json(state: &ColorWheelState) -> String {
    let rgb = hsv_to_rgb(&state.selected);
    format!(
        r#"{{"h":{},"s":{},"v":{},"r":{},"g":{},"b":{}}}"#,
        state.selected.hue, state.selected.saturation, state.selected.value,
        rgb.r, rgb.g, rgb.b
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hsv() {
        let c = new_hsv(120.0, 0.5, 0.8);
        assert!((c.hue - 120.0).abs() < 1e-6);
    }

    #[test]
    fn test_hsv_red() {
        let rgb = hsv_to_rgb(&new_hsv(0.0, 1.0, 1.0));
        assert!((rgb.r - 1.0).abs() < 1e-4);
        assert!(rgb.g.abs() < 1e-4);
        assert!(rgb.b.abs() < 1e-4);
    }

    #[test]
    fn test_hsv_green() {
        let rgb = hsv_to_rgb(&new_hsv(120.0, 1.0, 1.0));
        assert!(rgb.r.abs() < 1e-4);
        assert!((rgb.g - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_hsv_blue() {
        let rgb = hsv_to_rgb(&new_hsv(240.0, 1.0, 1.0));
        assert!(rgb.r.abs() < 1e-4);
        assert!((rgb.b - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_rgb_to_hsv_roundtrip() {
        let original = new_hsv(90.0, 0.7, 0.9);
        let rgb = hsv_to_rgb(&original);
        let back = rgb_to_hsv(&rgb);
        assert!((back.hue - original.hue).abs() < 1.0);
        assert!((back.saturation - original.saturation).abs() < 0.01);
    }

    #[test]
    fn test_wheel_position() {
        let hue = wheel_position_to_hue(1.0, 0.0);
        assert!(hue.abs() < 1e-4);
    }

    #[test]
    fn test_new_wheel_state() {
        let s = new_color_wheel_state();
        assert!((s.radius - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_color() {
        let mut s = new_color_wheel_state();
        set_wheel_color(&mut s, new_hsv(200.0, 0.5, 0.8));
        assert!((s.selected.hue - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_color_wheel_state();
        let j = color_wheel_to_json(&s);
        assert!(j.contains("\"h\":"));
    }

    #[test]
    fn test_hsv_black() {
        let rgb = hsv_to_rgb(&new_hsv(0.0, 0.0, 0.0));
        assert!(rgb.r.abs() < 1e-4);
        assert!(rgb.g.abs() < 1e-4);
        assert!(rgb.b.abs() < 1e-4);
    }
}
