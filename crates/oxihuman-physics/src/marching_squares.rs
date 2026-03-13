// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Marching squares — 2D contour extraction from a scalar field.

/// A contour segment (pair of 2D points).
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub a: [f64; 2],
    pub b: [f64; 2],
}

impl Segment {
    pub fn new(a: [f64; 2], b: [f64; 2]) -> Self {
        Segment { a, b }
    }

    /// Length of the segment.
    pub fn length(&self) -> f64 {
        let dx = self.b[0] - self.a[0];
        let dy = self.b[1] - self.a[1];
        (dx * dx + dy * dy).sqrt()
    }
}

/// Linearly interpolate edge crossing position.
fn interp_edge(v0: f64, v1: f64, iso: f64, t: f64) -> f64 {
    if (v1 - v0).abs() < 1e-12 {
        return t + 0.5;
    }
    t + (iso - v0) / (v1 - v0)
}

/// Extract contour segments at iso-level `iso` from a 2D scalar field.
///
/// The field is `nx * ny` values in row-major order (y outer, x inner).
/// Cell size is `dx` x `dy`.
pub fn marching_squares(
    field: &[f64],
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
    iso: f64,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    if nx < 2 || ny < 2 {
        return segments;
    }
    for cy in 0..(ny - 1) {
        for cx in 0..(nx - 1) {
            /* corner values */
            let v00 = field[cy * nx + cx];
            let v10 = field[cy * nx + cx + 1];
            let v01 = field[(cy + 1) * nx + cx];
            let v11 = field[(cy + 1) * nx + cx + 1];
            /* build case index */
            let case = ((v00 >= iso) as u8)
                | (((v10 >= iso) as u8) << 1)
                | (((v01 >= iso) as u8) << 2)
                | (((v11 >= iso) as u8) << 3);
            if case == 0 || case == 15 {
                continue; /* all inside or all outside */
            }
            let x0 = cx as f64 * dx;
            let y0 = cy as f64 * dy;
            /* midpoints of edges */
            let ex_bot = [interp_edge(v00, v10, iso, x0 / dx) * dx, y0]; /* bottom edge */
            let ex_top = [interp_edge(v01, v11, iso, x0 / dx) * dx, y0 + dy]; /* top edge */
            let ey_left = [x0, interp_edge(v00, v01, iso, y0 / dy) * dy]; /* left edge */
            let ey_right = [x0 + dx, interp_edge(v10, v11, iso, y0 / dy) * dy]; /* right edge */
            let seg = match case {
                1 | 14 => Some(Segment::new(ex_bot, ey_left)),
                2 | 13 => Some(Segment::new(ex_bot, ey_right)),
                3 | 12 => Some(Segment::new(ey_left, ey_right)),
                4 | 11 => Some(Segment::new(ex_top, ey_left)),
                6 | 9 => Some(Segment::new(ex_bot, ex_top)),
                7 | 8 => Some(Segment::new(ex_top, ey_right)),
                /* saddle cases 5 and 10: two segments, emit first */
                5 | 10 => Some(Segment::new(ex_bot, ey_left)),
                _ => None,
            };
            if let Some(s) = seg {
                segments.push(s);
            }
        }
    }
    segments
}

/// Total length of all contour segments.
pub fn total_contour_length(segments: &[Segment]) -> f64 {
    segments.iter().map(|s| s.length()).sum()
}

/// Count segments that cross the iso-level (all of them, by definition).
pub fn count_segments(segments: &[Segment]) -> usize {
    segments.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_2x2_field(v00: f64, v10: f64, v01: f64, v11: f64) -> Vec<f64> {
        vec![v00, v10, v01, v11]
    }

    #[test]
    fn test_all_below_iso() {
        let field = make_2x2_field(0.0, 0.0, 0.0, 0.0);
        let segs = marching_squares(&field, 2, 2, 1.0, 1.0, 0.5);
        assert!(segs.is_empty() /* all below iso: no segments */);
    }

    #[test]
    fn test_all_above_iso() {
        let field = make_2x2_field(1.0, 1.0, 1.0, 1.0);
        let segs = marching_squares(&field, 2, 2, 1.0, 1.0, 0.5);
        assert!(segs.is_empty() /* all above iso: no segments */);
    }

    #[test]
    fn test_one_corner_inside() {
        let field = make_2x2_field(1.0, 0.0, 0.0, 0.0);
        let segs = marching_squares(&field, 2, 2, 1.0, 1.0, 0.5);
        assert_eq!(segs.len(), 1 /* one corner inside: one segment */);
    }

    #[test]
    fn test_segment_length_positive() {
        let s = Segment::new([0.0, 0.0], [3.0, 4.0]);
        assert!((s.length() - 5.0).abs() < 1e-10 /* 3-4-5 triangle */);
    }

    #[test]
    fn test_total_contour_length() {
        let segs = vec![
            Segment::new([0.0, 0.0], [1.0, 0.0]),
            Segment::new([0.0, 0.0], [0.0, 1.0]),
        ];
        let total = total_contour_length(&segs);
        assert!((total - 2.0).abs() < 1e-10 /* sum of lengths */);
    }

    #[test]
    fn test_count_segments() {
        let segs = vec![
            Segment::new([0.0, 0.0], [1.0, 0.0]),
            Segment::new([0.0, 0.0], [0.0, 1.0]),
        ];
        assert_eq!(count_segments(&segs), 2);
    }

    #[test]
    fn test_4x4_circular_blob() {
        /* create a circular blob in a 4x4 grid */
        let mut field = vec![0.0f64; 16];
        for y in 0..4 {
            for x in 0..4 {
                let dx = x as f64 - 1.5;
                let dy = y as f64 - 1.5;
                field[y * 4 + x] = 1.0 - (dx * dx + dy * dy) / 4.0;
            }
        }
        let segs = marching_squares(&field, 4, 4, 1.0, 1.0, 0.5);
        assert!(!segs.is_empty() /* circular blob produces contour segments */);
    }

    #[test]
    fn test_small_grid_returns_empty() {
        let field = vec![0.5f64; 1];
        let segs = marching_squares(&field, 1, 1, 1.0, 1.0, 0.5);
        assert!(segs.is_empty() /* grid smaller than 2x2 returns empty */);
    }

    #[test]
    fn test_half_field() {
        /* left column < iso, right column > iso: horizontal boundary */
        let field = vec![0.0, 1.0, 0.0, 1.0];
        let segs = marching_squares(&field, 2, 2, 1.0, 1.0, 0.5);
        assert!(!segs.is_empty() /* vertical split produces segments */);
    }
}
