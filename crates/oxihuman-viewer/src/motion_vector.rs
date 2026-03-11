// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Motion-vector buffer management for temporal effects (TAA, motion blur).

/// A per-pixel motion vector (screen-space displacement in UV space).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MotionVector {
    pub du: f32,
    pub dv: f32,
}

impl MotionVector {
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        (self.du * self.du + self.dv * self.dv).sqrt()
    }
}

/// Motion vector buffer.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MotionVectorBuffer {
    pub width: u32,
    pub height: u32,
    pub vectors: Vec<MotionVector>,
}

impl MotionVectorBuffer {
    #[allow(dead_code)]
    pub fn new(width: u32, height: u32) -> Self {
        let count = (width * height) as usize;
        MotionVectorBuffer {
            width,
            height,
            vectors: vec![MotionVector::default(); count],
        }
    }
}

#[allow(dead_code)]
pub fn mv_set(buf: &mut MotionVectorBuffer, x: u32, y: u32, du: f32, dv: f32) {
    if x < buf.width && y < buf.height {
        let idx = (y * buf.width + x) as usize;
        buf.vectors[idx] = MotionVector { du, dv };
    }
}

#[allow(dead_code)]
pub fn mv_get(buf: &MotionVectorBuffer, x: u32, y: u32) -> Option<MotionVector> {
    if x < buf.width && y < buf.height {
        Some(buf.vectors[(y * buf.width + x) as usize])
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn mv_clear(buf: &mut MotionVectorBuffer) {
    for v in &mut buf.vectors {
        *v = MotionVector::default();
    }
}

#[allow(dead_code)]
pub fn mv_max_magnitude(buf: &MotionVectorBuffer) -> f32 {
    buf.vectors
        .iter()
        .map(|v| v.magnitude())
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn mv_average_magnitude(buf: &MotionVectorBuffer) -> f32 {
    if buf.vectors.is_empty() {
        return 0.0;
    }
    buf.vectors.iter().map(|v| v.magnitude()).sum::<f32>() / buf.vectors.len() as f32
}

#[allow(dead_code)]
pub fn mv_pixel_count(buf: &MotionVectorBuffer) -> usize {
    buf.vectors.len()
}

#[allow(dead_code)]
pub fn mv_to_json(buf: &MotionVectorBuffer) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"max_mag\":{:.4}}}",
        buf.width,
        buf.height,
        mv_max_magnitude(buf)
    )
}

#[allow(dead_code)]
pub fn mv_scale(buf: &mut MotionVectorBuffer, scale: f32) {
    for v in &mut buf.vectors {
        v.du *= scale;
        v.dv *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_all_zero() {
        let b = MotionVectorBuffer::new(4, 4);
        assert!((mv_max_magnitude(&b)).abs() < 1e-5);
    }

    #[test]
    fn set_and_get() {
        let mut b = MotionVectorBuffer::new(8, 8);
        mv_set(&mut b, 2, 3, 0.1, 0.2);
        let v = mv_get(&b, 2, 3).unwrap();
        assert!((v.du - 0.1).abs() < 1e-5);
    }

    #[test]
    fn out_of_bounds_get_none() {
        let b = MotionVectorBuffer::new(4, 4);
        assert!(mv_get(&b, 10, 10).is_none());
    }

    #[test]
    fn clear_zeroes() {
        let mut b = MotionVectorBuffer::new(2, 2);
        mv_set(&mut b, 0, 0, 1.0, 1.0);
        mv_clear(&mut b);
        assert!((mv_max_magnitude(&b)).abs() < 1e-5);
    }

    #[test]
    fn magnitude_formula() {
        let v = MotionVector { du: 3.0, dv: 4.0 };
        assert!((v.magnitude() - 5.0).abs() < 1e-4);
    }

    #[test]
    fn pixel_count() {
        let b = MotionVectorBuffer::new(10, 10);
        assert_eq!(mv_pixel_count(&b), 100);
    }

    #[test]
    fn scale_doubles() {
        let mut b = MotionVectorBuffer::new(2, 2);
        mv_set(&mut b, 0, 0, 1.0, 0.0);
        mv_scale(&mut b, 2.0);
        let v = mv_get(&b, 0, 0).unwrap();
        assert!((v.du - 2.0).abs() < 1e-5);
    }

    #[test]
    fn average_magnitude_uniform() {
        let mut b = MotionVectorBuffer::new(1, 1);
        mv_set(&mut b, 0, 0, 1.0, 0.0);
        assert!((mv_average_magnitude(&b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_width() {
        assert!(mv_to_json(&MotionVectorBuffer::new(3, 3)).contains("width"));
    }

    #[test]
    fn out_of_bounds_set_ignored() {
        let mut b = MotionVectorBuffer::new(4, 4);
        mv_set(&mut b, 99, 99, 1.0, 1.0); // should not panic
        assert_eq!(mv_pixel_count(&b), 16);
    }
}
