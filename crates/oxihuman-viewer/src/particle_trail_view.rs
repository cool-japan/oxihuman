// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ParticleTrailView {
    pub enabled: bool,
    pub trail_length: usize,
    pub fade: bool,
}

pub fn new_particle_trail_view() -> ParticleTrailView {
    ParticleTrailView {
        enabled: false,
        trail_length: 16,
        fade: true,
    }
}

pub fn ptv_set_trail_length(v: &mut ParticleTrailView, n: usize) {
    v.trail_length = n.max(1);
}

pub fn ptv_enable(v: &mut ParticleTrailView) {
    v.enabled = true;
}

pub fn ptv_toggle_fade(v: &mut ParticleTrailView) {
    v.fade = !v.fade;
}

pub fn ptv_trail_alpha(v: &ParticleTrailView, age: usize) -> f32 {
    if !v.fade || v.trail_length == 0 {
        return 1.0;
    }
    let t = age as f32 / v.trail_length as f32;
    (1.0 - t).clamp(0.0, 1.0)
}

pub fn ptv_is_enabled(v: &ParticleTrailView) -> bool {
    v.enabled
}

pub fn ptv_to_json(v: &ParticleTrailView) -> String {
    format!(
        r#"{{"enabled":{},"trail_length":{},"fade":{}}}"#,
        v.enabled, v.trail_length, v.fade
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, length=16, fade=true */
        let v = new_particle_trail_view();
        assert!(!v.enabled);
        assert_eq!(v.trail_length, 16);
        assert!(v.fade);
    }

    #[test]
    fn test_set_trail_length() {
        /* valid length */
        let mut v = new_particle_trail_view();
        ptv_set_trail_length(&mut v, 32);
        assert_eq!(v.trail_length, 32);
    }

    #[test]
    fn test_set_trail_length_min() {
        /* minimum 1 */
        let mut v = new_particle_trail_view();
        ptv_set_trail_length(&mut v, 0);
        assert_eq!(v.trail_length, 1);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_particle_trail_view();
        ptv_enable(&mut v);
        assert!(ptv_is_enabled(&v));
    }

    #[test]
    fn test_toggle_fade() {
        /* toggle flips flag */
        let mut v = new_particle_trail_view();
        ptv_toggle_fade(&mut v);
        assert!(!v.fade);
    }

    #[test]
    fn test_trail_alpha_head() {
        /* age 0 (head) -> alpha 1 */
        let v = new_particle_trail_view();
        let a = ptv_trail_alpha(&v, 0);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_trail_alpha_tail() {
        /* age == trail_length -> alpha 0 */
        let v = new_particle_trail_view();
        let a = ptv_trail_alpha(&v, 16);
        assert!((a - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_trail_alpha_no_fade() {
        /* no fade -> always 1 */
        let mut v = new_particle_trail_view();
        ptv_toggle_fade(&mut v);
        let a = ptv_trail_alpha(&v, 10);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON has trail_length */
        let v = new_particle_trail_view();
        assert!(ptv_to_json(&v).contains("trail_length"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_particle_trail_view();
        let v2 = v.clone();
        assert_eq!(v.trail_length, v2.trail_length);
    }
}
