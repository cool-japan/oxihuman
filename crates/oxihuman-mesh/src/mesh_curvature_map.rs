// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CurvatureMap {
    pub mean: Vec<f32>,
    pub gaussian: Vec<f32>,
    pub vertex_count: usize,
}

pub fn new_curvature_map(n: usize) -> CurvatureMap {
    CurvatureMap {
        mean: vec![0.0; n],
        gaussian: vec![0.0; n],
        vertex_count: n,
    }
}

pub fn curv_set_mean(m: &mut CurvatureMap, i: usize, v: f32) {
    m.mean[i] = v;
}

pub fn curv_set_gaussian(m: &mut CurvatureMap, i: usize, v: f32) {
    m.gaussian[i] = v;
}

pub fn curv_get_mean(m: &CurvatureMap, i: usize) -> f32 {
    m.mean[i]
}

pub fn curv_get_gaussian(m: &CurvatureMap, i: usize) -> f32 {
    m.gaussian[i]
}

/// Blue = concave (k < 0), red = convex (k > 0).
pub fn curv_mean_to_color(k: f32) -> [f32; 3] {
    let t = k.clamp(-1.0, 1.0);
    if t < 0.0 {
        [0.0, 0.0, -t]
    } else {
        [t, 0.0, 0.0]
    }
}

pub fn curv_gaussian_to_color(k: f32) -> [f32; 3] {
    let t = k.clamp(-1.0, 1.0);
    if t < 0.0 {
        [0.0, -t, 0.0]
    } else {
        [t, t, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_curvature_map() {
        /* zeroed */
        let m = new_curvature_map(4);
        assert_eq!(m.vertex_count, 4);
        assert!((curv_get_mean(&m, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_set_get_mean() {
        /* round-trip mean */
        let mut m = new_curvature_map(3);
        curv_set_mean(&mut m, 1, 0.7);
        assert!((curv_get_mean(&m, 1) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_get_gaussian() {
        /* round-trip gaussian */
        let mut m = new_curvature_map(3);
        curv_set_gaussian(&mut m, 0, -0.3);
        assert!((curv_get_gaussian(&m, 0) + 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_mean_to_color_convex() {
        /* positive => red */
        let c = curv_mean_to_color(1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!(c[2] < 1e-6);
    }

    #[test]
    fn test_mean_to_color_concave() {
        /* negative => blue */
        let c = curv_mean_to_color(-1.0);
        assert!(c[0] < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gaussian_to_color() {
        /* positive gaussian => yellow-ish */
        let c = curv_gaussian_to_color(1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gaussian_to_color_negative() {
        /* negative gaussian => green */
        let c = curv_gaussian_to_color(-1.0);
        assert!(c[0] < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }
}
