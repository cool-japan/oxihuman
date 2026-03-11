// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SkinTension — per-vertex skin tension map.

#![allow(dead_code)]

/// A map of per-vertex tension values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TensionMap {
    pub values: Vec<f32>,
}

/// Skin tension state referencing a `TensionMap`.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SkinTension {
    pub map: TensionMap,
}

/// Create an empty `TensionMap`.
#[allow(dead_code)]
pub fn new_tension_map(n_verts: usize) -> TensionMap {
    TensionMap { values: vec![0.0; n_verts] }
}

/// Compute tension as the absolute displacement from rest for each vertex.
#[allow(dead_code)]
pub fn compute_tension(rest: &[f32], deformed: &[f32]) -> TensionMap {
    let n = rest.len().min(deformed.len());
    let values = (0..n).map(|i| (deformed[i] - rest[i]).abs()).collect();
    TensionMap { values }
}

/// Return the tension at vertex `index`.
#[allow(dead_code)]
pub fn tension_at_vertex(map: &TensionMap, index: usize) -> f32 {
    map.values.get(index).copied().unwrap_or(0.0)
}

/// Return the maximum tension value.
#[allow(dead_code)]
pub fn max_tension(map: &TensionMap) -> f32 {
    map.values.iter().cloned().fold(0.0_f32, f32::max)
}

/// Return the minimum tension value.
#[allow(dead_code)]
pub fn min_tension(map: &TensionMap) -> f32 {
    if map.values.is_empty() {
        return 0.0;
    }
    map.values.iter().cloned().fold(f32::INFINITY, f32::min)
}

/// Return the average tension.
#[allow(dead_code)]
pub fn average_tension(map: &TensionMap) -> f32 {
    if map.values.is_empty() {
        return 0.0;
    }
    map.values.iter().sum::<f32>() / map.values.len() as f32
}

/// Smooth tension by averaging each vertex with its neighbours (simple 1-D kernel).
#[allow(dead_code)]
pub fn smooth_tension(map: &TensionMap) -> TensionMap {
    let n = map.values.len();
    if n == 0 {
        return TensionMap::default();
    }
    let mut out = vec![0.0_f32; n];
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let lo = if i == 0 { 0 } else { i - 1 };
        let hi = (i + 1).min(n - 1);
        out[i] = (map.values[lo] + map.values[i] + map.values[hi]) / 3.0;
    }
    TensionMap { values: out }
}

/// Map tension values to an RGBA colour (low = blue, high = red).
#[allow(dead_code)]
pub fn tension_to_color(map: &TensionMap) -> Vec<[u8; 4]> {
    let max_t = max_tension(map).max(f32::EPSILON);
    map.values
        .iter()
        .map(|&t| {
            let ratio = (t / max_t).clamp(0.0, 1.0);
            let r = (ratio * 255.0) as u8;
            let b = ((1.0 - ratio) * 255.0) as u8;
            [r, 0, b, 255]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tension_map_zeros() {
        let m = new_tension_map(5);
        assert_eq!(m.values.len(), 5);
        assert!(m.values.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_tension_no_deform() {
        let rest = vec![0.0, 1.0, 2.0];
        let m = compute_tension(&rest, &rest);
        assert!(m.values.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_tension_deformed() {
        let rest = vec![0.0, 0.0, 0.0];
        let def = vec![1.0, 2.0, 3.0];
        let m = compute_tension(&rest, &def);
        assert!((m.values[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_tension_at_vertex() {
        let m = TensionMap { values: vec![0.1, 0.5, 0.9] };
        assert!((tension_at_vertex(&m, 1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_max_tension() {
        let m = TensionMap { values: vec![0.1, 0.9, 0.5] };
        assert!((max_tension(&m) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_min_tension() {
        let m = TensionMap { values: vec![0.1, 0.9, 0.5] };
        assert!((min_tension(&m) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_average_tension() {
        let m = TensionMap { values: vec![0.0, 1.0] };
        assert!((average_tension(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_tension_reduces_extremes() {
        let m = TensionMap { values: vec![0.0, 1.0, 0.0] };
        let s = smooth_tension(&m);
        assert!(s.values[1] < 1.0);
    }

    #[test]
    fn test_tension_to_color_count() {
        let m = TensionMap { values: vec![0.0, 0.5, 1.0] };
        let colors = tension_to_color(&m);
        assert_eq!(colors.len(), 3);
        // Alpha should always be 255
        assert!(colors.iter().all(|c| c[3] == 255));
    }
}
