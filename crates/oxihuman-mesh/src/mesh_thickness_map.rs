// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct ThicknessMap {
    pub values: Vec<f32>,
    pub vertex_count: usize,
}

pub fn new_thickness_map(n: usize) -> ThicknessMap {
    ThicknessMap {
        values: vec![0.0; n],
        vertex_count: n,
    }
}

pub fn thickness_set(m: &mut ThicknessMap, i: usize, v: f32) {
    m.values[i] = v;
}

pub fn thickness_get(m: &ThicknessMap, i: usize) -> f32 {
    m.values[i]
}

pub fn thickness_mean(m: &ThicknessMap) -> f32 {
    if m.values.is_empty() {
        return 0.0;
    }
    m.values.iter().sum::<f32>() / m.values.len() as f32
}

pub fn thickness_max(m: &ThicknessMap) -> f32 {
    m.values.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

pub fn thickness_to_color(t: f32, max: f32) -> [f32; 3] {
    let v = if max > 0.0 {
        (t / max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    [v, 1.0 - v, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_thickness_map() {
        /* zeroed on construction */
        let m = new_thickness_map(4);
        assert_eq!(m.vertex_count, 4);
        assert!((thickness_get(&m, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_set_get() {
        /* round-trip */
        let mut m = new_thickness_map(3);
        thickness_set(&mut m, 2, 2.71);
        assert!((thickness_get(&m, 2) - 2.71).abs() < 1e-4);
    }

    #[test]
    fn test_mean() {
        /* mean of 0,1,2 = 1 */
        let mut m = new_thickness_map(3);
        thickness_set(&mut m, 0, 0.0);
        thickness_set(&mut m, 1, 1.0);
        thickness_set(&mut m, 2, 2.0);
        assert!((thickness_mean(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max() {
        /* max value */
        let mut m = new_thickness_map(3);
        thickness_set(&mut m, 0, 1.0);
        thickness_set(&mut m, 1, 5.0);
        thickness_set(&mut m, 2, 2.0);
        assert!((thickness_max(&m) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_color() {
        /* max maps to [1,0,0] */
        let c = thickness_to_color(1.0, 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_empty() {
        /* empty => 0 */
        let m = new_thickness_map(0);
        assert!((thickness_mean(&m)).abs() < 1e-6);
    }
}
