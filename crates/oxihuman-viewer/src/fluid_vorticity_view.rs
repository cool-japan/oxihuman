// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct FluidVorticityView {
    pub enabled: bool,
    pub max_vorticity: f32,
    pub show_direction: bool,
}

pub fn new_fluid_vorticity_view() -> FluidVorticityView {
    FluidVorticityView {
        enabled: false,
        max_vorticity: 1.0,
        show_direction: true,
    }
}

pub fn fvv_set_max_vorticity(v: &mut FluidVorticityView, m: f32) {
    v.max_vorticity = m.max(1e-6);
}

pub fn fvv_enable(v: &mut FluidVorticityView) {
    v.enabled = true;
}

pub fn fvv_toggle_direction(v: &mut FluidVorticityView) {
    v.show_direction = !v.show_direction;
}

pub fn fvv_vorticity_color(v: &FluidVorticityView, vorticity: f32) -> [f32; 3] {
    let t = (vorticity.abs() / v.max_vorticity).clamp(0.0, 1.0);
    if vorticity >= 0.0 {
        [t, 0.3 * t, 1.0 - t]
    } else {
        [1.0 - t, 0.3 * t, t]
    }
}

pub fn fvv_is_enabled(v: &FluidVorticityView) -> bool {
    v.enabled
}

pub fn fvv_to_json(v: &FluidVorticityView) -> String {
    format!(
        r#"{{"enabled":{},"max_vorticity":{:.4},"show_direction":{}}}"#,
        v.enabled, v.max_vorticity, v.show_direction
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled, max=1, show_direction=true */
        let v = new_fluid_vorticity_view();
        assert!(!v.enabled);
        assert!((v.max_vorticity - 1.0).abs() < 1e-6);
        assert!(v.show_direction);
    }

    #[test]
    fn test_set_max_vorticity() {
        /* valid max */
        let mut v = new_fluid_vorticity_view();
        fvv_set_max_vorticity(&mut v, 5.0);
        assert!((v.max_vorticity - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_vorticity_min() {
        /* minimum enforced */
        let mut v = new_fluid_vorticity_view();
        fvv_set_max_vorticity(&mut v, 0.0);
        assert!(v.max_vorticity > 0.0);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_fluid_vorticity_view();
        fvv_enable(&mut v);
        assert!(fvv_is_enabled(&v));
    }

    #[test]
    fn test_toggle_direction() {
        /* toggle flips flag */
        let mut v = new_fluid_vorticity_view();
        fvv_toggle_direction(&mut v);
        assert!(!v.show_direction);
    }

    #[test]
    fn test_vorticity_color_positive_range() {
        /* positive vorticity -> channels in [0,1] */
        let v = new_fluid_vorticity_view();
        let c = fvv_vorticity_color(&v, 0.5);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_vorticity_color_negative_range() {
        /* negative vorticity -> channels in [0,1] */
        let v = new_fluid_vorticity_view();
        let c = fvv_vorticity_color(&v, -0.5);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_vorticity_color_zero() {
        /* zero vorticity -> black */
        let v = new_fluid_vorticity_view();
        let c = fvv_vorticity_color(&v, 0.0);
        assert_eq!(c, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_to_json() {
        /* JSON has max_vorticity */
        let v = new_fluid_vorticity_view();
        assert!(fvv_to_json(&v).contains("max_vorticity"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_fluid_vorticity_view();
        let v2 = v.clone();
        assert_eq!(v.max_vorticity, v2.max_vorticity);
    }
}
