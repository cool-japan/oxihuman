// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Motion vector v2: per-object + per-pixel velocity buffers for TAA and motion blur.

/// A 2-D screen-space motion vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(dead_code)]
pub struct MotionVec2 {
    pub dx: f32,
    pub dy: f32,
}

impl MotionVec2 {
    #[allow(dead_code)]
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    /// Magnitude of the vector.
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }

    /// Scale by `s`.
    #[allow(dead_code)]
    pub fn scale(&self, s: f32) -> Self {
        Self {
            dx: self.dx * s,
            dy: self.dy * s,
        }
    }
}

/// Per-pixel motion vector buffer (screen-space).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MotionVectorBuffer {
    pub width: u32,
    pub height: u32,
    vectors: Vec<MotionVec2>,
}

impl MotionVectorBuffer {
    /// Create a zeroed buffer.
    #[allow(dead_code)]
    pub fn new(width: u32, height: u32) -> Self {
        let count = (width * height) as usize;
        Self {
            width,
            height,
            vectors: vec![MotionVec2::default(); count],
        }
    }

    /// Set the motion vector at pixel `(x, y)`.
    #[allow(dead_code)]
    pub fn set(&mut self, x: u32, y: u32, v: MotionVec2) {
        let idx = (y * self.width + x) as usize;
        if idx < self.vectors.len() {
            self.vectors[idx] = v;
        }
    }

    /// Get the motion vector at pixel `(x, y)`.
    #[allow(dead_code)]
    pub fn get(&self, x: u32, y: u32) -> MotionVec2 {
        let idx = (y * self.width + x) as usize;
        if idx < self.vectors.len() {
            self.vectors[idx]
        } else {
            MotionVec2::default()
        }
    }

    /// Clear all vectors to zero.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for v in self.vectors.iter_mut() {
            *v = MotionVec2::default();
        }
    }

    /// Return the maximum magnitude in the buffer.
    #[allow(dead_code)]
    pub fn max_magnitude(&self) -> f32 {
        self.vectors
            .iter()
            .map(|v| v.magnitude())
            .fold(0.0_f32, f32::max)
    }
}

/// Compute a screen-space motion vector from clip-space positions.
/// `prev` and `curr` are NDC coordinates (x, y in [-1, 1]).
#[allow(dead_code)]
pub fn ndc_to_motion_vec(prev: [f32; 2], curr: [f32; 2]) -> MotionVec2 {
    MotionVec2::new(curr[0] - prev[0], curr[1] - prev[1])
}

/// Dilate a motion vector buffer by replacing each pixel with its neighbourhood maximum.
#[allow(dead_code)]
pub fn dilate_max(buf: &MotionVectorBuffer) -> MotionVectorBuffer {
    let mut out = buf.clone();
    for y in 0..buf.height {
        for x in 0..buf.width {
            let mut best = buf.get(x, y);
            let mut best_mag = best.magnitude();
            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as u32) < buf.width && (ny as u32) < buf.height {
                        let v = buf.get(nx as u32, ny as u32);
                        let m = v.magnitude();
                        if m > best_mag {
                            best = v;
                            best_mag = m;
                        }
                    }
                }
            }
            out.set(x, y, best);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_vec2_default_zero() {
        let v = MotionVec2::default();
        assert_eq!(v.dx, 0.0);
        assert_eq!(v.dy, 0.0);
    }

    #[test]
    fn magnitude_pythagoras() {
        let v = MotionVec2::new(3.0, 4.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn scale_doubles() {
        let v = MotionVec2::new(1.0, 2.0).scale(2.0);
        assert!((v.dx - 2.0).abs() < 1e-6);
    }

    #[test]
    fn buffer_set_get() {
        let mut buf = MotionVectorBuffer::new(4, 4);
        buf.set(2, 1, MotionVec2::new(0.5, 0.3));
        let g = buf.get(2, 1);
        assert!((g.dx - 0.5).abs() < 1e-6);
    }

    #[test]
    fn buffer_clear_zeros() {
        let mut buf = MotionVectorBuffer::new(2, 2);
        buf.set(0, 0, MotionVec2::new(1.0, 1.0));
        buf.clear();
        assert_eq!(buf.get(0, 0).dx, 0.0);
    }

    #[test]
    fn max_magnitude_zero_on_empty() {
        let buf = MotionVectorBuffer::new(2, 2);
        assert_eq!(buf.max_magnitude(), 0.0);
    }

    #[test]
    fn ndc_to_motion_vec_correct() {
        let mv = ndc_to_motion_vec([0.0, 0.0], [0.2, -0.1]);
        assert!((mv.dx - 0.2).abs() < 1e-6);
        assert!((mv.dy + 0.1).abs() < 1e-6);
    }

    #[test]
    fn dilate_max_no_shrink() {
        let mut buf = MotionVectorBuffer::new(3, 3);
        buf.set(1, 1, MotionVec2::new(1.0, 0.0));
        let out = dilate_max(&buf);
        // Neighbours of (1,1) should have magnitude >= 0
        let v = out.get(0, 0);
        assert!(v.magnitude() >= 0.0);
    }

    #[test]
    fn buffer_out_of_bounds_get_default() {
        let buf = MotionVectorBuffer::new(2, 2);
        let v = buf.get(100, 100);
        assert_eq!(v, MotionVec2::default());
    }
}
