// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Dihedral angle computation for mesh edges.

#[allow(dead_code)]
pub fn da_compute(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let dot = (n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2]).clamp(-1.0, 1.0);
    dot.acos()
}

#[allow(dead_code)]
pub fn da_is_sharp(n1: [f32; 3], n2: [f32; 3], threshold_rad: f32) -> bool {
    da_compute(n1, n2) > threshold_rad
}

#[allow(dead_code)]
pub fn da_average(normals: &[[f32; 3]]) -> f32 {
    let n = normals.len();
    if n < 2 { return 0.0; }
    let pairs = n / 2;
    let mut sum = 0.0f32;
    for i in 0..pairs {
        sum += da_compute(normals[2 * i], normals[2 * i + 1]);
    }
    sum / pairs as f32
}

#[allow(dead_code)]
pub fn da_max(normals: &[[f32; 3]]) -> f32 {
    let n = normals.len();
    if n < 2 { return 0.0; }
    let pairs = n / 2;
    let mut mx = 0.0f32;
    for i in 0..pairs {
        let a = da_compute(normals[2 * i], normals[2 * i + 1]);
        if a > mx { mx = a; }
    }
    mx
}

#[allow(dead_code)]
pub fn da_min(normals: &[[f32; 3]]) -> f32 {
    let n = normals.len();
    if n < 2 { return 0.0; }
    let pairs = n / 2;
    let mut mn = f32::MAX;
    for i in 0..pairs {
        let a = da_compute(normals[2 * i], normals[2 * i + 1]);
        if a < mn { mn = a; }
    }
    if mn == f32::MAX { 0.0 } else { mn }
}

#[allow(dead_code)]
pub struct DihedralStatsCalc {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub count: usize,
}

#[allow(dead_code)]
pub fn da_stats_calc(normals: &[[f32; 3]]) -> DihedralStatsCalc {
    DihedralStatsCalc {
        min: da_min(normals),
        max: da_max(normals),
        avg: da_average(normals),
        count: normals.len() / 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_same_normal_zero_angle() {
        let n = [0.0f32, 0.0, 1.0];
        assert!(da_compute(n, n).abs() < 1e-4);
    }

    #[test]
    fn test_opposite_normals_pi() {
        let n1 = [0.0f32, 0.0, 1.0];
        let n2 = [0.0f32, 0.0, -1.0];
        assert!((da_compute(n1, n2) - PI).abs() < 1e-4);
    }

    #[test]
    fn test_sharp_detection_true() {
        let n1 = [0.0f32, 0.0, 1.0];
        let n2 = [0.0f32, 0.0, -1.0];
        assert!(da_is_sharp(n1, n2, PI / 4.0));
    }

    #[test]
    fn test_sharp_detection_false() {
        let n = [0.0f32, 0.0, 1.0];
        assert!(!da_is_sharp(n, n, PI / 4.0));
    }

    #[test]
    fn test_average_of_pairs() {
        let normals = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, 1.0]];
        assert!(da_average(&normals).abs() < 1e-4);
    }

    #[test]
    fn test_stats_count() {
        let normals = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let s = da_stats_calc(&normals);
        assert_eq!(s.count, 2);
    }

    #[test]
    fn test_max_min_empty() {
        let normals: Vec<[f32; 3]> = vec![];
        assert_eq!(da_max(&normals), 0.0);
        assert_eq!(da_min(&normals), 0.0);
    }

    #[test]
    fn test_perpendicular_normals_angle() {
        let n1 = [1.0f32, 0.0, 0.0];
        let n2 = [0.0f32, 1.0, 0.0];
        let angle = da_compute(n1, n2);
        assert!((angle - PI / 2.0).abs() < 1e-4);
    }
}
