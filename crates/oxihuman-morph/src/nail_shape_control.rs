// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fingernail shape morph (square, oval, pointed).

/// Nail shape preset.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NailShape {
    Square,
    Oval,
    Pointed,
    Squoval,
    Almond,
}

impl NailShape {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            NailShape::Square => "square",
            NailShape::Oval => "oval",
            NailShape::Pointed => "pointed",
            NailShape::Squoval => "squoval",
            NailShape::Almond => "almond",
        }
    }
}

/// Parameters for nail shape morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NailShapeParams {
    pub shape: NailShape,
    pub length: f32,
    pub curvature: f32,
    pub width_scale: f32,
}

impl Default for NailShapeParams {
    fn default() -> Self {
        NailShapeParams {
            shape: NailShape::Oval,
            length: 0.0,
            curvature: 0.0,
            width_scale: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_nail_shape_params() -> NailShapeParams {
    NailShapeParams::default()
}

#[allow(dead_code)]
pub fn nail_set_shape(p: &mut NailShapeParams, s: NailShape) {
    p.shape = s;
}

#[allow(dead_code)]
pub fn nail_set_length(p: &mut NailShapeParams, v: f32) {
    p.length = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nail_set_curvature(p: &mut NailShapeParams, v: f32) {
    p.curvature = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nail_set_width_scale(p: &mut NailShapeParams, v: f32) {
    p.width_scale = v.clamp(-0.5, 0.5);
}

#[allow(dead_code)]
pub fn nail_reset(p: &mut NailShapeParams) {
    *p = NailShapeParams::default();
}

#[allow(dead_code)]
pub fn nail_is_neutral(p: &NailShapeParams) -> bool {
    p.length.abs() < 1e-6 && p.curvature.abs() < 1e-6 && p.width_scale.abs() < 1e-6
}

#[allow(dead_code)]
pub fn nail_blend(a: &NailShapeParams, b: &NailShapeParams, t: f32) -> NailShapeParams {
    let t = t.clamp(0.0, 1.0);
    let shape = if t < 0.5 { a.shape } else { b.shape };
    NailShapeParams {
        shape,
        length: a.length + (b.length - a.length) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        width_scale: a.width_scale + (b.width_scale - a.width_scale) * t,
    }
}

/// Sharpness index: square=0, oval=0.5, pointed=1.
#[allow(dead_code)]
pub fn nail_sharpness_index(p: &NailShapeParams) -> f32 {
    match p.shape {
        NailShape::Square => 0.0,
        NailShape::Squoval => 0.25,
        NailShape::Oval => 0.5,
        NailShape::Almond => 0.75,
        NailShape::Pointed => 1.0,
    }
}

#[allow(dead_code)]
pub fn nail_to_json(p: &NailShapeParams) -> String {
    format!(
        r#"{{"shape":"{}","length":{:.4},"curvature":{:.4},"width_scale":{:.4}}}"#,
        p.shape.name(),
        p.length,
        p.curvature,
        p.width_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shape_is_oval() {
        assert_eq!(default_nail_shape_params().shape, NailShape::Oval);
    }

    #[test]
    fn default_is_neutral() {
        assert!(nail_is_neutral(&default_nail_shape_params()));
    }

    #[test]
    fn set_length_clamps() {
        let mut p = default_nail_shape_params();
        nail_set_length(&mut p, 5.0);
        assert!((p.length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_shape_square() {
        let mut p = default_nail_shape_params();
        nail_set_shape(&mut p, NailShape::Square);
        assert_eq!(p.shape, NailShape::Square);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_nail_shape_params();
        nail_set_length(&mut p, 0.7);
        nail_reset(&mut p);
        assert!(nail_is_neutral(&p));
    }

    #[test]
    fn sharpness_square_zero() {
        let mut p = default_nail_shape_params();
        nail_set_shape(&mut p, NailShape::Square);
        assert!(nail_sharpness_index(&p).abs() < 1e-6);
    }

    #[test]
    fn sharpness_pointed_one() {
        let mut p = default_nail_shape_params();
        nail_set_shape(&mut p, NailShape::Pointed);
        assert!((nail_sharpness_index(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint_length() {
        let a = default_nail_shape_params();
        let mut b = default_nail_shape_params();
        nail_set_length(&mut b, 1.0);
        let m = nail_blend(&a, &b, 0.5);
        assert!((m.length - 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_shape() {
        assert!(nail_to_json(&default_nail_shape_params()).contains("shape"));
    }
}
