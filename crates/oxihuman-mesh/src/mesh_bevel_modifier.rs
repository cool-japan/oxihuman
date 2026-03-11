// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Parameters for bevel modifier.
pub struct BevelParams {
    pub amount: f32,
    pub segments: usize,
    pub vertex_only: bool,
}

pub fn new_bevel_params(amount: f32, segments: usize) -> BevelParams {
    BevelParams {
        amount: amount.max(0.0),
        segments: segments.max(1),
        vertex_only: false,
    }
}

pub fn bevel_edge_offset(start: [f32; 3], end: [f32; 3], amount: f32) -> ([f32; 3], [f32; 3]) {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let dz = end[2] - start[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
    let t = (amount / len).clamp(0.0, 0.5);
    let a = [start[0] + dx * t, start[1] + dy * t, start[2] + dz * t];
    let b = [end[0] - dx * t, end[1] - dy * t, end[2] - dz * t];
    (a, b)
}

pub fn bevel_segment_point(start: [f32; 3], end: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        start[0] + (end[0] - start[0]) * t,
        start[1] + (end[1] - start[1]) * t,
        start[2] + (end[2] - start[2]) * t,
    ]
}

pub fn bevel_new_vertex_count(edge_count: usize, params: &BevelParams) -> usize {
    edge_count * 2 * params.segments
}

pub fn bevel_new_face_count(edge_count: usize, params: &BevelParams) -> usize {
    edge_count * params.segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bevel_params() {
        /* basic construction and clamping */
        let p = new_bevel_params(0.1, 3);
        assert!((p.amount - 0.1).abs() < 1e-6);
        assert_eq!(p.segments, 3);
    }

    #[test]
    fn test_bevel_params_min_segments() {
        let p = new_bevel_params(0.5, 0);
        assert_eq!(p.segments, 1);
    }

    #[test]
    fn test_bevel_edge_offset_midpoint() {
        /* with amount = half length the two offset points should meet */
        let s = [0.0, 0.0, 0.0];
        let e = [2.0, 0.0, 0.0];
        let (a, b) = bevel_edge_offset(s, e, 1.0);
        assert!((a[0] - b[0]).abs() < 1e-5);
    }

    #[test]
    fn test_bevel_segment_point_endpoints() {
        let s = [0.0, 0.0, 0.0];
        let e = [4.0, 0.0, 0.0];
        let p0 = bevel_segment_point(s, e, 0.0);
        let p1 = bevel_segment_point(s, e, 1.0);
        assert!((p0[0] - 0.0).abs() < 1e-6);
        assert!((p1[0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_bevel_new_vertex_count() {
        let p = new_bevel_params(0.1, 2);
        assert_eq!(bevel_new_vertex_count(5, &p), 20);
    }

    #[test]
    fn test_bevel_new_face_count() {
        let p = new_bevel_params(0.1, 3);
        assert_eq!(bevel_new_face_count(4, &p), 12);
    }
}
