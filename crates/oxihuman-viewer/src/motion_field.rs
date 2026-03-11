// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Motion field — screen-space motion vector field for motion blur and TAA.

/// A 2-D motion vector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MotionVec2 {
    pub x: f32,
    pub y: f32,
}

/// Motion field stored as a flat buffer of motion vectors.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionField {
    pub width: usize,
    pub height: usize,
    pub vectors: Vec<MotionVec2>,
}

#[allow(dead_code)]
pub fn new_motion_field(width: usize, height: usize) -> MotionField {
    MotionField {
        width,
        height,
        vectors: vec![MotionVec2::default(); width * height],
    }
}

#[allow(dead_code)]
pub fn mf_set(field: &mut MotionField, x: usize, y: usize, mv: MotionVec2) {
    if x < field.width && y < field.height {
        let idx = y * field.width + x;
        field.vectors[idx] = mv;
    }
}

#[allow(dead_code)]
pub fn mf_get(field: &MotionField, x: usize, y: usize) -> MotionVec2 {
    if x < field.width && y < field.height {
        field.vectors[y * field.width + x]
    } else {
        MotionVec2::default()
    }
}

#[allow(dead_code)]
pub fn mf_clear(field: &mut MotionField) {
    for v in field.vectors.iter_mut() {
        *v = MotionVec2::default();
    }
}

#[allow(dead_code)]
pub fn mf_max_magnitude(field: &MotionField) -> f32 {
    field
        .vectors
        .iter()
        .map(|v| (v.x * v.x + v.y * v.y).sqrt())
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn mf_average_magnitude(field: &MotionField) -> f32 {
    if field.vectors.is_empty() {
        return 0.0;
    }
    let total: f32 = field
        .vectors
        .iter()
        .map(|v| (v.x * v.x + v.y * v.y).sqrt())
        .sum();
    total / field.vectors.len() as f32
}

#[allow(dead_code)]
pub fn mf_scale(field: &mut MotionField, factor: f32) {
    for v in field.vectors.iter_mut() {
        v.x *= factor;
        v.y *= factor;
    }
}

#[allow(dead_code)]
pub fn mf_pixel_count(field: &MotionField) -> usize {
    field.vectors.len()
}

#[allow(dead_code)]
pub fn mf_to_json(field: &MotionField) -> String {
    format!(
        r#"{{"width":{},"height":{},"max_mag":{:.4}}}"#,
        field.width,
        field.height,
        mf_max_magnitude(field)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_field_zero() {
        let f = new_motion_field(4, 4);
        assert!((mf_max_magnitude(&f) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn set_and_get() {
        let mut f = new_motion_field(4, 4);
        mf_set(&mut f, 1, 2, MotionVec2 { x: 0.5, y: -0.3 });
        let v = mf_get(&f, 1, 2);
        assert!((v.x - 0.5).abs() < 1e-6);
        assert!((v.y + 0.3).abs() < 1e-6);
    }

    #[test]
    fn get_out_of_bounds_returns_zero() {
        let f = new_motion_field(2, 2);
        let v = mf_get(&f, 10, 10);
        assert!((v.x - 0.0).abs() < 1e-6);
    }

    #[test]
    fn max_magnitude() {
        let mut f = new_motion_field(3, 3);
        mf_set(&mut f, 0, 0, MotionVec2 { x: 3.0, y: 4.0 });
        assert!((mf_max_magnitude(&f) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn clear_resets() {
        let mut f = new_motion_field(2, 2);
        mf_set(&mut f, 0, 0, MotionVec2 { x: 1.0, y: 0.0 });
        mf_clear(&mut f);
        assert!((mf_max_magnitude(&f) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn scale_doubles() {
        let mut f = new_motion_field(2, 2);
        mf_set(&mut f, 0, 0, MotionVec2 { x: 1.0, y: 0.0 });
        mf_scale(&mut f, 2.0);
        let v = mf_get(&f, 0, 0);
        assert!((v.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn pixel_count() {
        let f = new_motion_field(4, 8);
        assert_eq!(mf_pixel_count(&f), 32);
    }

    #[test]
    fn average_magnitude_zero() {
        let f = new_motion_field(2, 2);
        assert!((mf_average_magnitude(&f) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let f = new_motion_field(8, 6);
        let j = mf_to_json(&f);
        assert!(j.contains("width"));
        assert!(j.contains("max_mag"));
    }
}
