// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct RoughnessDebugView {
    pub min_roughness: f32,
    pub max_roughness: f32,
    pub enabled: bool,
}

pub fn new_roughness_debug_view() -> RoughnessDebugView {
    RoughnessDebugView {
        min_roughness: 0.0,
        max_roughness: 1.0,
        enabled: false,
    }
}

pub fn rdv_set_range(v: &mut RoughnessDebugView, min: f32, max: f32) {
    v.min_roughness = min.clamp(0.0, 1.0);
    v.max_roughness = max.clamp(0.0, 1.0).max(v.min_roughness);
}

pub fn rdv_enable(v: &mut RoughnessDebugView) {
    v.enabled = true;
}

pub fn rdv_roughness_color(v: &RoughnessDebugView, roughness: f32) -> [f32; 3] {
    let range = (v.max_roughness - v.min_roughness).max(1e-6);
    let t = ((roughness - v.min_roughness) / range).clamp(0.0, 1.0);
    /* smooth=blue, rough=white */
    let b = 1.0 - t * 0.5;
    [t, t, b]
}

pub fn rdv_is_enabled(v: &RoughnessDebugView) -> bool {
    v.enabled
}

pub fn rdv_to_json(v: &RoughnessDebugView) -> String {
    format!(
        r#"{{"min_roughness":{:.4},"max_roughness":{:.4},"enabled":{}}}"#,
        v.min_roughness, v.max_roughness, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* 0..1 range, disabled */
        let v = new_roughness_debug_view();
        assert_eq!(v.min_roughness, 0.0);
        assert_eq!(v.max_roughness, 1.0);
    }

    #[test]
    fn test_set_range() {
        /* range stored */
        let mut v = new_roughness_debug_view();
        rdv_set_range(&mut v, 0.1, 0.9);
        assert!((v.min_roughness - 0.1).abs() < 1e-6);
        assert!((v.max_roughness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_roughness_debug_view();
        rdv_enable(&mut v);
        assert!(rdv_is_enabled(&v));
    }

    #[test]
    fn test_roughness_color_smooth() {
        /* min roughness => dark */
        let v = new_roughness_debug_view();
        let c = rdv_roughness_color(&v, 0.0);
        assert!(c[2] > c[0]);
    }

    #[test]
    fn test_roughness_color_rough() {
        /* max roughness => light */
        let v = new_roughness_debug_view();
        let c = rdv_roughness_color(&v, 1.0);
        assert!(c[0] > 0.0);
    }

    #[test]
    fn test_roughness_color_range() {
        /* components in [0,1] */
        let v = new_roughness_debug_view();
        let c = rdv_roughness_color(&v, 0.5);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_to_json() {
        /* JSON has min_roughness */
        let v = new_roughness_debug_view();
        assert!(rdv_to_json(&v).contains("min_roughness"));
    }

    #[test]
    fn test_to_json_enabled() {
        /* new view shows false */
        let v = new_roughness_debug_view();
        assert!(rdv_to_json(&v).contains("false"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_roughness_debug_view();
        let v2 = v.clone();
        assert_eq!(v.enabled, v2.enabled);
    }
}
