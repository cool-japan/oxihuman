// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MetalnessDebugView {
    pub threshold: f32,
    pub enabled: bool,
    pub show_transition: bool,
}

pub fn new_metalness_debug_view() -> MetalnessDebugView {
    MetalnessDebugView {
        threshold: 0.5,
        enabled: false,
        show_transition: false,
    }
}

pub fn mnv_set_threshold(v: &mut MetalnessDebugView, t: f32) {
    v.threshold = t.clamp(0.0, 1.0);
}

pub fn mnv_enable(v: &mut MetalnessDebugView) {
    v.enabled = true;
}

pub fn mnv_metalness_color(v: &MetalnessDebugView, metalness: f32) -> [f32; 3] {
    let m = metalness.clamp(0.0, 1.0);
    if v.show_transition {
        /* smooth gradient */
        [m, m * 0.5, 1.0 - m]
    } else if m >= v.threshold {
        [0.8, 0.8, 0.8] /* metallic: silver */
    } else {
        [0.4, 0.2, 0.1] /* dielectric: brownish */
    }
}

pub fn mnv_is_enabled(v: &MetalnessDebugView) -> bool {
    v.enabled
}

pub fn mnv_to_json(v: &MetalnessDebugView) -> String {
    format!(
        r#"{{"threshold":{:.4},"enabled":{},"show_transition":{}}}"#,
        v.threshold, v.enabled, v.show_transition
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* threshold=0.5, disabled */
        let v = new_metalness_debug_view();
        assert!((v.threshold - 0.5).abs() < 1e-6);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_threshold() {
        /* valid threshold stored */
        let mut v = new_metalness_debug_view();
        mnv_set_threshold(&mut v, 0.8);
        assert!((v.threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_threshold_clamp() {
        /* clamp above 1 */
        let mut v = new_metalness_debug_view();
        mnv_set_threshold(&mut v, 5.0);
        assert_eq!(v.threshold, 1.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_metalness_debug_view();
        mnv_enable(&mut v);
        assert!(mnv_is_enabled(&v));
    }

    #[test]
    fn test_metalness_color_metallic() {
        /* high metalness without transition => silver */
        let v = new_metalness_debug_view();
        let c = mnv_metalness_color(&v, 1.0);
        assert!((c[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_metalness_color_dielectric() {
        /* low metalness => brownish */
        let v = new_metalness_debug_view();
        let c = mnv_metalness_color(&v, 0.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_metalness_color_transition() {
        /* with transition gradient */
        let mut v = new_metalness_debug_view();
        v.show_transition = true;
        let c = mnv_metalness_color(&v, 0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON has threshold */
        let v = new_metalness_debug_view();
        assert!(mnv_to_json(&v).contains("threshold"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_metalness_debug_view();
        let v2 = v.clone();
        assert_eq!(v.threshold, v2.threshold);
    }
}
