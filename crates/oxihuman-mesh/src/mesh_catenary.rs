// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

// --- new API (Wave 151B) ---

pub struct CatenaryParams {
    pub a: f32,
    pub num_segments: usize,
    pub x_range: [f32; 2],
}

pub fn new_catenary(a: f32, x_min: f32, x_max: f32, segments: usize) -> CatenaryParams {
    CatenaryParams {
        a,
        num_segments: segments,
        x_range: [x_min, x_max],
    }
}

pub fn catenary_y(a: f32, x: f32) -> f32 {
    a * (x / a).cosh()
}

pub fn catenary_arc_length(a: f32, x0: f32, x1: f32) -> f32 {
    a * (x1 / a).sinh() - a * (x0 / a).sinh()
}

pub fn catenary_to_polyline(p: &CatenaryParams) -> Vec<[f32; 2]> {
    let n = p.num_segments.max(1);
    let x0 = p.x_range[0];
    let x1 = p.x_range[1];
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let x = x0 + t * (x1 - x0);
            [x, catenary_y(p.a, x)]
        })
        .collect()
}

pub fn catenary_sag(p: &CatenaryParams) -> f32 {
    let y_mid = catenary_y(p.a, (p.x_range[0] + p.x_range[1]) * 0.5);
    let y_end = catenary_y(p.a, p.x_range[0]).max(catenary_y(p.a, p.x_range[1]));
    y_end - y_mid
}

// --- legacy API stubs (previously expected by lib.rs) ---

pub struct CatenaryPoint {
    pub x: f32,
    pub y: f32,
}

pub fn default_catenary_params() -> CatenaryParams {
    new_catenary(1.0, -1.0, 1.0, 32)
}

pub fn euler_number() -> f32 {
    std::f32::consts::E
}

pub fn lowest_point(p: &CatenaryParams) -> [f32; 2] {
    let x = (p.x_range[0] + p.x_range[1]) * 0.5;
    [x, catenary_y(p.a, x)]
}

pub fn point_count(p: &CatenaryParams) -> usize {
    p.num_segments + 1
}

pub fn sample_catenary(p: &CatenaryParams, t: f32) -> CatenaryPoint {
    let x = p.x_range[0] + t * (p.x_range[1] - p.x_range[0]);
    CatenaryPoint {
        x,
        y: catenary_y(p.a, x),
    }
}

pub fn to_positions(p: &CatenaryParams) -> Vec<[f32; 3]> {
    catenary_to_polyline(p)
        .iter()
        .map(|&[x, y]| [x, y, 0.0])
        .collect()
}

pub fn catenary_tube_vertex_count(p: &CatenaryParams, tube_segments: u32) -> usize {
    (p.num_segments + 1) * tube_segments as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_catenary() {
        /* basic construction */
        let p = new_catenary(1.0, -1.0, 1.0, 10);
        assert_eq!(p.num_segments, 10);
        assert!((p.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_catenary_y_at_zero() {
        /* y(0) = a for any a */
        let y = catenary_y(2.0, 0.0);
        assert!((y - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_catenary_y_symmetric() {
        /* y is even */
        let y_pos = catenary_y(1.0, 1.0);
        let y_neg = catenary_y(1.0, -1.0);
        assert!((y_pos - y_neg).abs() < 1e-5);
    }

    #[test]
    fn test_catenary_arc_length_positive() {
        /* arc length from -1 to 1 is positive */
        let len = catenary_arc_length(1.0, -1.0, 1.0);
        assert!(len > 0.0);
    }

    #[test]
    fn test_catenary_to_polyline_count() {
        /* polyline has segments+1 points */
        let p = new_catenary(1.0, -2.0, 2.0, 8);
        let pts = catenary_to_polyline(&p);
        assert_eq!(pts.len(), 9);
    }

    #[test]
    fn test_catenary_sag_positive() {
        /* sag is non-negative for symmetric range */
        let p = new_catenary(1.0, -2.0, 2.0, 20);
        let sag = catenary_sag(&p);
        assert!(sag >= 0.0);
    }
}
