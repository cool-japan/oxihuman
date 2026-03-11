// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ContactForceView {
    pub enabled: bool,
    pub force_scale: f32,
    pub max_force: f32,
}

pub fn new_contact_force_view() -> ContactForceView {
    ContactForceView {
        enabled: false,
        force_scale: 1.0,
        max_force: 100.0,
    }
}

pub fn cfv_set_force_scale(v: &mut ContactForceView, s: f32) {
    v.force_scale = s.max(0.0);
}

pub fn cfv_set_max_force(v: &mut ContactForceView, f: f32) {
    v.max_force = f.max(1e-6);
}

pub fn cfv_enable(v: &mut ContactForceView) {
    v.enabled = true;
}

pub fn cfv_force_color(v: &ContactForceView, magnitude: f32) -> [f32; 3] {
    let t = (magnitude / v.max_force).clamp(0.0, 1.0);
    [t, 1.0 - t, 0.0]
}

pub fn cfv_arrow_length(v: &ContactForceView, magnitude: f32) -> f32 {
    (magnitude / v.max_force).clamp(0.0, 1.0) * v.force_scale
}

pub fn cfv_is_enabled(v: &ContactForceView) -> bool {
    v.enabled
}

pub fn cfv_to_json(v: &ContactForceView) -> String {
    format!(
        r#"{{"enabled":{},"force_scale":{:.4},"max_force":{:.4}}}"#,
        v.enabled, v.force_scale, v.max_force
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, scale=1, max=100 */
        let v = new_contact_force_view();
        assert!(!v.enabled);
        assert!((v.force_scale - 1.0).abs() < 1e-6);
        assert!((v.max_force - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_force_scale() {
        /* valid scale */
        let mut v = new_contact_force_view();
        cfv_set_force_scale(&mut v, 3.0);
        assert!((v.force_scale - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_force_scale_min() {
        /* minimum 0 */
        let mut v = new_contact_force_view();
        cfv_set_force_scale(&mut v, -1.0);
        assert_eq!(v.force_scale, 0.0);
    }

    #[test]
    fn test_set_max_force() {
        /* valid max */
        let mut v = new_contact_force_view();
        cfv_set_max_force(&mut v, 50.0);
        assert!((v.max_force - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_contact_force_view();
        cfv_enable(&mut v);
        assert!(cfv_is_enabled(&v));
    }

    #[test]
    fn test_force_color_zero() {
        /* zero force -> green */
        let v = new_contact_force_view();
        let c = cfv_force_color(&v, 0.0);
        assert_eq!(c, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_force_color_max() {
        /* max force -> red */
        let v = new_contact_force_view();
        let c = cfv_force_color(&v, 100.0);
        assert_eq!(c, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_arrow_length() {
        /* proportional to magnitude */
        let v = new_contact_force_view();
        let len = cfv_arrow_length(&v, 50.0);
        assert!((len - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON has force_scale */
        let v = new_contact_force_view();
        assert!(cfv_to_json(&v).contains("force_scale"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_contact_force_view();
        let v2 = v.clone();
        assert_eq!(v.max_force, v2.max_force);
    }
}
