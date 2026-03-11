// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct ColorManagementView {
    pub input_gamma: f32,
    pub output_gamma: f32,
    pub exposure: f32,
}

pub fn new_color_management_view() -> ColorManagementView {
    ColorManagementView {
        input_gamma: 2.2,
        output_gamma: 2.2,
        exposure: 0.0,
    }
}

pub fn cm_set_exposure(v: &mut ColorManagementView, ev: f32) {
    v.exposure = ev.clamp(-10.0, 10.0);
}

pub fn cm_linearize(v: &ColorManagementView, srgb: f32) -> f32 {
    srgb.clamp(0.0, 1.0).powf(v.input_gamma)
}

pub fn cm_encode(v: &ColorManagementView, linear: f32) -> f32 {
    let exposed = linear * 2.0f32.powf(v.exposure);
    exposed.clamp(0.0, 1.0).powf(1.0 / v.output_gamma)
}

pub fn cm_is_linear_workflow(v: &ColorManagementView) -> bool {
    (v.input_gamma - 1.0).abs() < 1e-3 && (v.output_gamma - 1.0).abs() < 1e-3
}

pub fn cm_blend(a: &ColorManagementView, b: &ColorManagementView, t: f32) -> ColorManagementView {
    let t = t.clamp(0.0, 1.0);
    ColorManagementView {
        input_gamma: a.input_gamma + (b.input_gamma - a.input_gamma) * t,
        output_gamma: a.output_gamma + (b.output_gamma - a.output_gamma) * t,
        exposure: a.exposure + (b.exposure - a.exposure) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default gamma */
        let v = new_color_management_view();
        assert!((v.input_gamma - 2.2).abs() < 1e-5);
    }

    #[test]
    fn test_linearize_range() {
        /* output in [0, 1] */
        let v = new_color_management_view();
        let l = cm_linearize(&v, 0.5);
        assert!((0.0..=1.0).contains(&l));
    }

    #[test]
    fn test_encode_range() {
        /* encoded value in [0, 1] */
        let v = new_color_management_view();
        let e = cm_encode(&v, 0.5);
        assert!((0.0..=1.0).contains(&e));
    }

    #[test]
    fn test_not_linear_workflow_by_default() {
        /* gamma 2.2 is not linear */
        let v = new_color_management_view();
        assert!(!cm_is_linear_workflow(&v));
    }

    #[test]
    fn test_blend() {
        /* midpoint exposure */
        let a = ColorManagementView {
            input_gamma: 1.0,
            output_gamma: 1.0,
            exposure: -2.0,
        };
        let b = ColorManagementView {
            input_gamma: 1.0,
            output_gamma: 1.0,
            exposure: 2.0,
        };
        let c = cm_blend(&a, &b, 0.5);
        assert!(c.exposure.abs() < 1e-5);
    }
}
