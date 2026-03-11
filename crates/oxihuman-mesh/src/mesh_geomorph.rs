// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Geomorphing — smooth interpolation between two LOD levels.

/// A pair of vertex positions for geomorphing (from LOD A to LOD B).
#[derive(Debug, Clone, Copy)]
pub struct GeomorphVertex {
    pub pos_a: [f32; 3],
    pub pos_b: [f32; 3],
}

/// A geomorph buffer holding per-vertex blend data.
#[derive(Debug, Default, Clone)]
pub struct GeomorphBuffer {
    pub vertices: Vec<GeomorphVertex>,
    pub blend: f32,
}

impl GeomorphBuffer {
    /// Creates a new geomorph buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a vertex pair.
    pub fn push(&mut self, v: GeomorphVertex) {
        self.vertices.push(v);
    }

    /// Sets the blend factor (0.0 = full LOD A, 1.0 = full LOD B).
    pub fn set_blend(&mut self, t: f32) {
        self.blend = t.clamp(0.0, 1.0);
    }

    /// Evaluates the blended position for vertex `idx`.
    pub fn evaluate(&self, idx: usize) -> Option<[f32; 3]> {
        self.vertices
            .get(idx)
            .map(|v| lerp_vec3(v.pos_a, v.pos_b, self.blend))
    }

    /// Returns the number of vertices in the buffer.
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Linearly interpolates between two 3D positions.
pub fn lerp_vec3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Computes the maximum positional error between LOD A and LOD B across all vertices.
pub fn max_geomorph_error(buf: &GeomorphBuffer) -> f32 {
    buf.vertices
        .iter()
        .map(|v| {
            let dx = v.pos_a[0] - v.pos_b[0];
            let dy = v.pos_a[1] - v.pos_b[1];
            let dz = v.pos_a[2] - v.pos_b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Builds a geomorph buffer from two separate vertex position arrays.
pub fn build_geomorph_buffer(pos_a: &[[f32; 3]], pos_b: &[[f32; 3]]) -> GeomorphBuffer {
    let mut buf = GeomorphBuffer::new();
    let n = pos_a.len().min(pos_b.len());
    for i in 0..n {
        buf.push(GeomorphVertex {
            pos_a: pos_a[i],
            pos_b: pos_b[i],
        });
    }
    buf
}

/// Returns all blended positions at the current blend factor.
pub fn evaluate_all(buf: &GeomorphBuffer) -> Vec<[f32; 3]> {
    (0..buf.len()).filter_map(|i| buf.evaluate(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_at_zero() {
        /* t=0 should return pos_a exactly */
        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, 5.0, 6.0];
        assert_eq!(lerp_vec3(a, b, 0.0), a);
    }

    #[test]
    fn test_lerp_at_one() {
        /* t=1 should return pos_b exactly */
        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, 5.0, 6.0];
        assert_eq!(lerp_vec3(a, b, 1.0), b);
    }

    #[test]
    fn test_lerp_midpoint() {
        /* t=0.5 should return midpoint */
        let r = lerp_vec3([0.0, 0.0, 0.0], [2.0, 2.0, 2.0], 0.5);
        assert!((r[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_buffer_push_and_len() {
        /* Push should increase length */
        let mut buf = GeomorphBuffer::new();
        buf.push(GeomorphVertex {
            pos_a: [0.0; 3],
            pos_b: [1.0; 3],
        });
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_set_blend_clamps() {
        /* Blend should be clamped to [0,1] */
        let mut buf = GeomorphBuffer::new();
        buf.set_blend(5.0);
        assert!((0.0..=1.0).contains(&buf.blend));
    }

    #[test]
    fn test_evaluate_out_of_bounds() {
        /* Evaluate on empty buffer should return None */
        let buf = GeomorphBuffer::new();
        assert!(buf.evaluate(0).is_none());
    }

    #[test]
    fn test_max_geomorph_error_same() {
        /* Same positions → zero error */
        let pos = vec![[1.0f32, 2.0, 3.0]];
        let buf = build_geomorph_buffer(&pos, &pos);
        assert_eq!(max_geomorph_error(&buf), 0.0);
    }

    #[test]
    fn test_build_geomorph_buffer_count() {
        /* Buffer should have min(a,b) vertices */
        let a = vec![[0.0f32; 3]; 4];
        let b = vec![[1.0f32; 3]; 6];
        let buf = build_geomorph_buffer(&a, &b);
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn test_evaluate_all_length() {
        /* evaluate_all should return one entry per vertex */
        let a = vec![[0.0f32; 3]; 3];
        let b = vec![[2.0f32; 3]; 3];
        let buf = build_geomorph_buffer(&a, &b);
        assert_eq!(evaluate_all(&buf).len(), 3);
    }

    #[test]
    fn test_is_empty_on_new() {
        /* New buffer should be empty */
        assert!(GeomorphBuffer::new().is_empty());
    }
}
