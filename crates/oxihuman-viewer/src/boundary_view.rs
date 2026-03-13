// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct BoundaryView {
    pub enabled: bool,
    pub color: [f32; 3],
    pub pulse: bool,
}

pub fn new_boundary_view() -> BoundaryView {
    BoundaryView {
        enabled: false,
        color: [1.0, 0.8, 0.0],
        pulse: false,
    }
}

pub fn bov_set_color(v: &mut BoundaryView, r: f32, g: f32, b: f32) {
    v.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

pub fn bov_enable(v: &mut BoundaryView) {
    v.enabled = true;
}

pub fn bov_toggle_pulse(v: &mut BoundaryView) {
    v.pulse = !v.pulse;
}

pub fn bov_boundary_color(v: &BoundaryView, time: f32) -> [f32; 3] {
    if v.pulse {
        let alpha = (time * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        [v.color[0] * alpha, v.color[1] * alpha, v.color[2] * alpha]
    } else {
        v.color
    }
}

pub fn bov_is_enabled(v: &BoundaryView) -> bool {
    v.enabled
}

pub fn bov_to_json(v: &BoundaryView) -> String {
    format!(
        r#"{{"enabled":{},"color":[{:.4},{:.4},{:.4}],"pulse":{}}}"#,
        v.enabled, v.color[0], v.color[1], v.color[2], v.pulse
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, yellow, no pulse */
        let v = new_boundary_view();
        assert!(!v.enabled);
        assert!(!v.pulse);
        assert!((v.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_color() {
        /* color stored */
        let mut v = new_boundary_view();
        bov_set_color(&mut v, 0.5, 0.6, 0.7);
        assert!((v.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_clamp() {
        /* clamp to [0,1] */
        let mut v = new_boundary_view();
        bov_set_color(&mut v, 5.0, -1.0, 0.5);
        assert_eq!(v.color[0], 1.0);
        assert_eq!(v.color[1], 0.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_boundary_view();
        bov_enable(&mut v);
        assert!(bov_is_enabled(&v));
    }

    #[test]
    fn test_toggle_pulse() {
        /* toggle flips flag */
        let mut v = new_boundary_view();
        bov_toggle_pulse(&mut v);
        assert!(v.pulse);
        bov_toggle_pulse(&mut v);
        assert!(!v.pulse);
    }

    #[test]
    fn test_boundary_color_no_pulse() {
        /* without pulse returns stored color */
        let v = new_boundary_view();
        let c = bov_boundary_color(&v, 0.5);
        assert_eq!(c, v.color);
    }

    #[test]
    fn test_boundary_color_pulse_range() {
        /* with pulse, channels in [0,1] */
        let mut v = new_boundary_view();
        bov_toggle_pulse(&mut v);
        let c = bov_boundary_color(&v, 0.25);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_to_json() {
        /* JSON has pulse key */
        let v = new_boundary_view();
        assert!(bov_to_json(&v).contains("pulse"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_boundary_view();
        let v2 = v.clone();
        assert_eq!(v.color, v2.color);
    }
}
