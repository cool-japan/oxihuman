// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Contour line extraction at a scalar threshold.

#[allow(dead_code)]
pub struct ContourLine {
    pub segments: Vec<[[f32; 3]; 2]>,
}

#[allow(dead_code)]
pub fn new_contour_line() -> ContourLine {
    ContourLine { segments: Vec::new() }
}

#[allow(dead_code)]
pub fn cl_add_segment(c: &mut ContourLine, a: [f32; 3], b: [f32; 3]) {
    c.segments.push([a, b]);
}

#[allow(dead_code)]
pub fn cl_segment_count(c: &ContourLine) -> usize {
    c.segments.len()
}

#[allow(dead_code)]
pub fn cl_total_length(c: &ContourLine) -> f32 {
    c.segments.iter().map(|seg| {
        let dx = seg[1][0] - seg[0][0];
        let dy = seg[1][1] - seg[0][1];
        let dz = seg[1][2] - seg[0][2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }).sum()
}

fn lerp_pt(a: [f32; 3], b: [f32; 3], va: f32, vb: f32, threshold: f32) -> [f32; 3] {
    let t = if (vb - va).abs() < 1e-10 { 0.5 } else { (threshold - va) / (vb - va) };
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

#[allow(dead_code)]
pub fn cl_extract(positions: &[[f32; 3]], indices: &[[u32; 3]], values: &[f32], threshold: f32) -> ContourLine {
    let mut c = new_contour_line();
    for tri in indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let v0 = values[i0];
        let v1 = values[i1];
        let v2 = values[i2];
        let a0 = v0 >= threshold;
        let a1 = v1 >= threshold;
        let a2 = v2 >= threshold;
        let crossings: Vec<[f32; 3]> = {
            let mut pts = Vec::new();
            if a0 != a1 { pts.push(lerp_pt(positions[i0], positions[i1], v0, v1, threshold)); }
            if a1 != a2 { pts.push(lerp_pt(positions[i1], positions[i2], v1, v2, threshold)); }
            if a2 != a0 { pts.push(lerp_pt(positions[i2], positions[i0], v2, v0, threshold)); }
            pts
        };
        if crossings.len() == 2 {
            cl_add_segment(&mut c, crossings[0], crossings[1]);
        }
    }
    c
}

#[allow(dead_code)]
pub fn cl_clear(c: &mut ContourLine) {
    c.segments.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let c = new_contour_line();
        assert_eq!(cl_segment_count(&c), 0);
    }

    #[test]
    fn test_add_segment() {
        let mut c = new_contour_line();
        cl_add_segment(&mut c, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(cl_segment_count(&c), 1);
    }

    #[test]
    fn test_total_length() {
        let mut c = new_contour_line();
        cl_add_segment(&mut c, [0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((cl_total_length(&c) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_clear() {
        let mut c = new_contour_line();
        cl_add_segment(&mut c, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        cl_clear(&mut c);
        assert_eq!(cl_segment_count(&c), 0);
    }

    #[test]
    fn test_extract_finds_crossing() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2]];
        let values = vec![0.0f32, 1.0, 0.0];
        let c = cl_extract(&positions, &indices, &values, 0.5);
        assert_eq!(cl_segment_count(&c), 1);
    }

    #[test]
    fn test_extract_no_crossing() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let indices = vec![[0u32, 1, 2]];
        let values = vec![1.0f32, 1.0, 1.0];
        let c = cl_extract(&positions, &indices, &values, 0.5);
        assert_eq!(cl_segment_count(&c), 0);
    }

    #[test]
    fn test_multiple_segments() {
        let mut c = new_contour_line();
        cl_add_segment(&mut c, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        cl_add_segment(&mut c, [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert_eq!(cl_segment_count(&c), 2);
    }

    #[test]
    fn test_total_length_two_segments() {
        let mut c = new_contour_line();
        cl_add_segment(&mut c, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        cl_add_segment(&mut c, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((cl_total_length(&c) - 2.0).abs() < 1e-5);
    }
}
