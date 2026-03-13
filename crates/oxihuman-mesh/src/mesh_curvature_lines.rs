// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CurvatureLine {
    pub points: Vec<[f32; 3]>,
    pub k1: f32,
    pub k2: f32,
    pub is_max: bool,
}

pub fn new_curvature_line(k1: f32, k2: f32, is_max: bool) -> CurvatureLine {
    CurvatureLine {
        points: vec![],
        k1,
        k2,
        is_max,
    }
}

pub fn curvline_push(l: &mut CurvatureLine, p: [f32; 3]) {
    l.points.push(p);
}

pub fn curvline_length(l: &CurvatureLine) -> f32 {
    if l.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..l.points.len() {
        let a = l.points[i - 1];
        let b = l.points[i];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

pub fn curvline_anisotropy(l: &CurvatureLine) -> f32 {
    (l.k1 - l.k2).abs()
}

pub fn curvline_color(l: &CurvatureLine) -> [f32; 3] {
    if l.is_max {
        [1.0, 0.5, 0.0]
    } else {
        [0.0, 0.5, 1.0]
    }
}

pub fn curvline_point_count(l: &CurvatureLine) -> usize {
    l.points.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_curvature_line() {
        /* construction */
        let l = new_curvature_line(1.0, -1.0, true);
        assert!((l.k1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_push_and_count() {
        /* push adds points */
        let mut l = new_curvature_line(1.0, 0.5, false);
        curvline_push(&mut l, [0.0, 0.0, 0.0]);
        curvline_push(&mut l, [1.0, 0.0, 0.0]);
        assert_eq!(curvline_point_count(&l), 2);
    }

    #[test]
    fn test_length() {
        /* two unit points */
        let mut l = new_curvature_line(1.0, 0.5, true);
        curvline_push(&mut l, [0.0, 0.0, 0.0]);
        curvline_push(&mut l, [1.0, 0.0, 0.0]);
        assert!((curvline_length(&l) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_anisotropy() {
        /* k1-k2 abs */
        let l = new_curvature_line(3.0, 1.0, true);
        assert!((curvline_anisotropy(&l) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_max() {
        /* max principal => orange */
        let l = new_curvature_line(1.0, 0.5, true);
        let c = curvline_color(&l);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_min() {
        /* min principal => blue-ish */
        let l = new_curvature_line(1.0, 0.5, false);
        let c = curvline_color(&l);
        assert!(c[2] > 0.0);
        assert!(c[0] < 1e-6);
    }
}
