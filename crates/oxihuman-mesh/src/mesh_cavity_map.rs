// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CavityMap {
    pub values: Vec<f32>,
    pub vertex_count: usize,
}

pub fn new_cavity_map(n: usize) -> CavityMap {
    CavityMap {
        values: vec![0.0; n],
        vertex_count: n,
    }
}

pub fn cavity_set(m: &mut CavityMap, i: usize, v: f32) {
    m.values[i] = v;
}

pub fn cavity_get(m: &CavityMap, i: usize) -> f32 {
    m.values[i]
}

pub fn cavity_is_cavity(v: f32) -> bool {
    v < -0.1
}

pub fn cavity_is_convex(v: f32) -> bool {
    v > 0.1
}

pub fn cavity_to_color(v: f32) -> [f32; 3] {
    let t = v.clamp(-1.0, 1.0);
    if t < 0.0 {
        [(-t), 0.0, 0.0]
    } else {
        [0.0, 0.0, t]
    }
}

pub fn cavity_mean(m: &CavityMap) -> f32 {
    if m.values.is_empty() {
        return 0.0;
    }
    m.values.iter().sum::<f32>() / m.values.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cavity_map() {
        /* zeroed on construction */
        let m = new_cavity_map(5);
        assert_eq!(m.vertex_count, 5);
    }

    #[test]
    fn test_cavity_detection() {
        /* negative is cavity */
        assert!(cavity_is_cavity(-0.5));
        assert!(!cavity_is_cavity(0.5));
    }

    #[test]
    fn test_convex_detection() {
        /* positive is convex */
        assert!(cavity_is_convex(0.5));
        assert!(!cavity_is_convex(-0.5));
    }

    #[test]
    fn test_to_color_cavity() {
        /* negative value -> red channel */
        let c = cavity_to_color(-1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!(c[2] < 1e-6);
    }

    #[test]
    fn test_to_color_convex() {
        /* positive value -> blue channel */
        let c = cavity_to_color(1.0);
        assert!(c[0] < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean() {
        /* mean */
        let mut m = new_cavity_map(2);
        cavity_set(&mut m, 0, -1.0);
        cavity_set(&mut m, 1, 1.0);
        assert!(cavity_mean(&m).abs() < 1e-6);
    }
}
