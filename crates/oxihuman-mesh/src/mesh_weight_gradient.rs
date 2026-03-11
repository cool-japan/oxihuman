#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Assign vertex weights by gradient (distance from reference point or height).

#[allow(dead_code)]
pub fn weight_by_distance(verts: &[[f32; 3]], ref_pt: [f32; 3], max_dist: f32) -> Vec<f32> {
    verts
        .iter()
        .map(|&v| {
            let d = dist3(v, ref_pt);
            if max_dist < 1e-7 {
                0.0
            } else {
                (1.0 - (d / max_dist)).clamp(0.0, 1.0)
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn weight_by_height(verts: &[[f32; 3]], axis: u8) -> Vec<f32> {
    if verts.is_empty() {
        return vec![];
    }
    let vals: Vec<f32> = verts.iter().map(|v| if axis == 0 { v[0] } else if axis == 1 { v[1] } else { v[2] }).collect();
    let min = vals.iter().cloned().fold(f32::MAX, f32::min);
    let max = vals.iter().cloned().fold(f32::MIN, f32::max);
    let range = max - min;
    if range < 1e-7 {
        return vec![0.0; verts.len()];
    }
    vals.iter().map(|&h| (h - min) / range).collect()
}

#[allow(dead_code)]
pub fn invert_weights(weights: &[f32]) -> Vec<f32> {
    weights.iter().map(|&w| 1.0 - w.clamp(0.0, 1.0)).collect()
}

#[allow(dead_code)]
pub fn clamp_weights(weights: &[f32], lo: f32, hi: f32) -> Vec<f32> {
    weights.iter().map(|&w| w.clamp(lo, hi)).collect()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_by_distance_at_origin() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let w = weight_by_distance(&verts, [0.0, 0.0, 0.0], 2.0);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weight_by_distance_at_max() {
        let verts = vec![[2.0, 0.0, 0.0]];
        let w = weight_by_distance(&verts, [0.0, 0.0, 0.0], 2.0);
        assert!(w[0].abs() < 1e-5);
    }

    #[test]
    fn weight_by_distance_beyond_max_clamped() {
        let verts = vec![[5.0, 0.0, 0.0]];
        let w = weight_by_distance(&verts, [0.0, 0.0, 0.0], 2.0);
        assert!(w[0].abs() < 1e-5);
    }

    #[test]
    fn weight_by_height_min_zero_max_one() {
        let verts = vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]];
        let w = weight_by_height(&verts, 1);
        assert!(w[0].abs() < 1e-5);
        assert!((w[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weight_by_height_empty() {
        let w = weight_by_height(&[], 1);
        assert!(w.is_empty());
    }

    #[test]
    fn weight_by_height_uniform_returns_zero() {
        let verts = vec![[0.0, 5.0, 0.0], [1.0, 5.0, 0.0]];
        let w = weight_by_height(&verts, 1);
        for &wv in &w {
            assert!(wv.abs() < 1e-5);
        }
    }

    #[test]
    fn invert_weights_flips() {
        let w = vec![0.0, 0.5, 1.0];
        let inv = invert_weights(&w);
        assert!((inv[0] - 1.0).abs() < 1e-5);
        assert!((inv[1] - 0.5).abs() < 1e-5);
        assert!(inv[2].abs() < 1e-5);
    }

    #[test]
    fn clamp_weights_below() {
        let w = vec![-0.5, 0.5, 1.5];
        let c = clamp_weights(&w, 0.0, 1.0);
        assert!(c[0].abs() < 1e-5);
        assert!((c[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn clamp_weights_range() {
        let w = vec![0.0, 0.3, 0.8, 1.0];
        let c = clamp_weights(&w, 0.2, 0.7);
        assert!((c[0] - 0.2).abs() < 1e-5);
        assert!((c[3] - 0.7).abs() < 1e-5);
    }
}
